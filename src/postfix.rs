use chrono::{DateTime, Datelike, FixedOffset, NaiveDate};

use log::warn;
use pest::Parser;
use pest_derive::Parser;

use crate::model::{FluentBitJson, LogOrString};

#[derive(Parser)]
#[grammar_inline = r#"
postfix_log = { SOI ~ timestamp ~ " " ~ host ~ " " ~ "postfix/" ~ process_message ~ EOI}

timestamp = { month ~ " " ~ day ~ " " ~ hour ~ ":" ~ minute ~ ":" ~ second }
month = { "Jan" | "Feb" | "Mar" | "Apr" | "May" | "Jun" | "Jul" | "Aug" | "Sep" | "Oct" | "Nov" | "Dec" }
day = { ASCII_DIGIT{2} }
hour = { ASCII_DIGIT{2} }
minute = { ASCII_DIGIT{2} }
second = { ASCII_DIGIT{2} }

host = { not_space+ }
pid = { ASCII_DIGIT+ }

not_space = _{!" " ~ ANY}

process_message = { process_smtpd | process_other }

process_smtpd = { "smtpd" ~ "[" ~ pid ~ "]: " ~ message_smtpd }
message_smtpd = { ANY* }

process_other = { process_name ~ "[" ~ pid ~ "]: " ~ message_other }
process_name = { ASCII_ALPHA+ }
message_other = { ANY* }

"#]
struct PostfixLogParser;

pub fn convert_postfix_logs(json: &mut FluentBitJson, event_date: &DateTime<FixedOffset>) {
    let log = match json.log.as_ref() {
        Some(LogOrString::String(_)) => match json.log.take() {
            Some(LogOrString::String(log)) => Some(log),
            _ => unreachable!(),
        },
        _ => None,
    };

    match log {
        None => {
            convert_log_missing(json);
        }
        Some(log) => match PostfixLogParser::parse(Rule::postfix_log, &log) {
            Err(err) => convert_parse_error(json, err, log),
            Ok(ast) => convert_parsed_logs(json, ast, event_date, &log),
        },
    }
}

fn convert_log_missing(json: &mut FluentBitJson) {
    json.message = Some("fluent-ecs postfix parser failed: no log line passed.".to_string());

    let event = json.event();
    event.module = Some("postfix".to_string());
    event.severity = Some(300);
    event.outcome = Some("failure".to_string());
    event.kind = Some("pipeline_error".to_string());
}

fn convert_parse_error(json: &mut FluentBitJson, err: pest::error::Error<Rule>, log: String) {
    let err = format!("fluent-ecs postfix parser failed:{}", err.to_string());
    warn!("parsing failed: {}", err);
    json.message = Some(err);

    let event = json.event();
    event.module = Some("postfix".to_string());
    event.severity = Some(300);
    event.outcome = Some("failure".to_string());
    event.kind = Some("pipeline_error".to_string());
    event.original = Some(log);
}

fn convert_parsed_logs(
    json: &mut FluentBitJson,
    pairs: pest::iterators::Pairs<'_, Rule>,
    event_date: &DateTime<FixedOffset>,
    log: &String,
) {
    // event basics
    {
        let event = json.event();
        event.module = Some("postfix".to_string());
        event.original = Some(log.to_owned());
    }

    for pair in pairs {
        match pair.as_rule() {
            Rule::postfix_log => {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::timestamp => {
                            json.timestamp = convert_date(pair.into_inner(), event_date)
                        }
                        Rule::process_message => {
                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::process_smtpd => convert_smtpd(json, pair.into_inner()),
                                    Rule::process_other => convert_other(json, pair.into_inner()),
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn convert_date(
    pairs: pest::iterators::Pairs<'_, Rule>,
    event_date: &DateTime<FixedOffset>,
) -> Option<DateTime<FixedOffset>> {
    let mut month: u32 = 0;
    let mut day: u32 = 0;

    let mut hour: u32 = 1000;
    let mut minute: u32 = 1000;
    let mut second: u32 = 1000;

    for pair in pairs {
        match pair.as_rule() {
            Rule::month => {
                month = match pair.as_str() {
                    "Jan" => 1,
                    "Feb" => 2,
                    "Mar" => 3,
                    "Apr" => 4,
                    "May" => 5,
                    "Jun" => 6,
                    "Jul" => 7,
                    "Aug" => 8,
                    "Sep" => 9,
                    "Oct" => 10,
                    "Nov" => 11,
                    "Dec" => 12,
                    _ => 0,
                }
            }
            Rule::day => day = pair.as_str().parse().unwrap_or(0),
            Rule::hour => hour = pair.as_str().parse().unwrap_or(1000),
            Rule::minute => minute = pair.as_str().parse().unwrap_or(1000),
            Rule::second => second = pair.as_str().parse().unwrap_or(1000),
            _ => {}
        }
    }

    let year: i32 = match (month, event_date.month()) {
        // If the postfix month is december and the fluent-bit event month is january
        // than it's probably new year right now and the event actually still happened last year.
        (12, 1) => event_date.year() - 1,
        // if not the year should be tha same as the year in that the event arrived at fluent bit.
        _ => event_date.year(),
    };

    // log::debug!(
    //     "postfix date parts: year: {}, month: {}, day: {}, hour: {}, minute: {}, second: {}",
    //     year, month, day, hour, minute, second
    // );

    Some(
        NaiveDate::from_ymd_opt(year, month, day)?
            .and_hms_opt(hour, minute, second)?
            .and_utc()
            .fixed_offset(),
    )
}

fn convert_smtpd(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    json.process().name = Some("smtpd".to_string());

    for pair in pairs {
        match pair.as_rule() {
            Rule::pid => convert_pid(json, pair.as_str()),
            _ => {}
        }
    }
}

fn convert_other(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::pid => convert_pid(json, pair.as_str()),
            Rule::process_other => json.process().name = Some(pair.as_str().to_string()),
            _ => {}
        }
    }
}

fn convert_pid(json: &mut FluentBitJson, pid: &str) {
    match pid.parse::<u32>() {
        Ok(pid) => json.process().pid = Some(pid),
        Err(_) => {}
    }
}
