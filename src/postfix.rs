use chrono::{DateTime, Datelike, FixedOffset, NaiveDate};

use log::warn;
use pest::Parser;
use pest_derive::Parser;

use crate::model::{FluentBitJson, LogOrString};

#[derive(Parser)]
#[grammar_inline = r#"
postfix_log = { SOI ~ timestamp ~ " " ~ host ~ " " ~ process_message ~ EOI}

timestamp = { month ~ " "+ ~ day ~ " "+ ~ hour ~ ":" ~ minute ~ ":" ~ second }
month = { "Jan" | "Feb" | "Mar" | "Apr" | "May" | "Jun" | "Jul" | "Aug" | "Sep" | "Oct" | "Nov" | "Dec" }
day = { ASCII_DIGIT{1,2} }
hour = { ASCII_DIGIT{1,2} }
minute = { ASCII_DIGIT{1,2} }
second = { ASCII_DIGIT{1,2} }

host = { not_space+ }
pid = { ASCII_DIGIT+ }

process_message = { process_smtpd | process_postfix_script | process_anvil | process_master | process_main | process_other }

process_smtpd = { "postfix/smtpd" ~ "[" ~ pid ~ "]: " ~ log_level ~ message_smtpd }
message_smtpd = { smtpd_connect | smtpd_disconnect | smtpd_lost_connection | smtpd_auth_failed | smtpd_mail_open_stream | message_other }
smtpd_connect = { "connect from " ~ hostname_ip}
smtpd_disconnect = { "disconnect from " ~ hostname_ip ~ ANY* }
smtpd_lost_connection = {smtpd_lost_connection_msg ~ " from " ~ hostname_ip ~ ANY* }
smtpd_lost_connection_msg = {"lost connection after " ~ not_space+ }
smtpd_auth_failed = { hostname_ip ~ ": SASL " ~ not_space+ ~ " authentication failed: " ~ ANY*}
smtpd_mail_open_stream = { queue_id ~ ": client=" ~ hostname_ip ~ (", " ~ key_value_pair*)? }

process_postfix_script = { "postfix/postfix-script" ~ "[" ~ pid ~ "]: "~ log_level ~ message_postfix_script }
message_postfix_script = { postfix_script_starting_postfix | postfix_script_group_writable | message_other }
postfix_script_starting_postfix = { "starting the Postfix mail system" }
postfix_script_group_writable = { "group or other writable:" ~ ANY* }

process_anvil = { "postfix/anvil" ~ "[" ~ pid ~ "]: "~ message_anvil }
message_anvil = { anvil_rate | anvil_count | anvil_cache | message_other }
anvil_rate = { "statistics: max " ~ anvil_metric_type ~ " rate " ~ ASCII_DIGIT+ ~ "/" ~ ASCII_DIGIT+ ~ "s for (" ~ anvil_protocol ~ ":" ~ ip ~") at "~ timestamp ~ ANY* }
anvil_count = { "statistics: max " ~ anvil_metric_type ~ " count " ~ ASCII_DIGIT+ ~ " for (" ~ anvil_protocol ~ ":" ~ ip ~") at "~ timestamp ~ ANY*}
anvil_cache = { "statistics: max cache size " ~ ASCII_DIGIT+ ~ " at " ~ timestamp }
anvil_metric_type = { "connection" | "message" | "recipient" | "newtls" | "auth" }
anvil_protocol = { (!(":") ~ ANY)+}

process_master = { "postfix/master" ~ "[" ~ pid ~ "]: " ~ log_level ~ message_master }
message_master = { master_daemon_started | message_other }
master_daemon_started = { "daemon started -- " ~ ANY* }

process_main = { "postfix" ~ "[" ~ pid ~ "]: "~ log_level ~ message_main }
message_main = { message_other }

process_other = { "postfix/" ~ process_name ~ "[" ~ pid ~ "]: "~ log_level ~ message_other }
process_name = { not_bracket+ }
message_other = { ANY* }

// log level
log_level = { log_level_warning? }
log_level_warning = { "warning: " }

// building bocks
queue_id = { "NOQUEUE" | (!(":" | " ") ~ ASCII_ALPHANUMERIC)+}
key_value_pair = { key ~ "=" ~ value ~ ","? ~ " "? }
key = { (ASCII_ALPHA_LOWER | "_" | "-" )+ }
value = { (!("," | " ") ~ ANY)+ }
not_space = _{!" " ~ ANY}
not_bracket = _{!("[" | "]" | "(" | ")" ) ~ ANY}
hostname_ip = { hostname ~ "[" ~ ip ~ "]"}
hostname = { not_bracket+ }
ip = { not_bracket+ }
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
    json.message = Some(log);

    let err = format!("fluent-ecs postfix parser failed:{}", err.to_string());
    warn!("parsing failed: {}", err);
    json.error().message = Some(err);

    let event = json.event();
    event.module = Some("postfix".to_string());
    event.severity = Some(300);
    event.outcome = Some("failure".to_string());
    event.kind = Some("pipeline_error".to_string());

    json.service().type_val = Some("postfix".to_string());
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

    // service basics
    json.service().type_val = Some("postfix".to_string());

    let mut host = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::postfix_log => {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::timestamp => {
                            json.timestamp = convert_date(pair.into_inner(), event_date)
                        }
                        Rule::host => host = Some(pair.as_str()),
                        Rule::process_message => {
                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::process_smtpd => {
                                        convert_smtpd(json, pair.into_inner(), host)
                                    }
                                    Rule::process_postfix_script => {
                                        convert_postfix_script(json, pair.into_inner())
                                    }
                                    Rule::process_anvil => {
                                        convert_anvil(json, pair.into_inner(), event_date)
                                    }
                                    Rule::process_master => convert_master(json, pair.into_inner()),
                                    Rule::process_main => convert_main(json, pair.into_inner()),
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

fn convert_smtpd(
    json: &mut FluentBitJson,
    pairs: pest::iterators::Pairs<'_, Rule>,
    host: Option<&str>,
) {
    json.process().name = Some("smtpd".to_string());

    json.event().category.push("email".to_string());

    for pair in pairs {
        match pair.as_rule() {
            Rule::pid => convert_pid(json, pair.as_str()),
            Rule::log_level => convert_log_level(json, pair.into_inner()),
            Rule::message_smtpd => {
                json.message = Some(pair.as_str().to_string());
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::smtpd_connect => {
                            let event = json.event();
                            event.category.push("network".to_string());
                            event.type_val.push("connection".to_string());
                            event.type_val.push("start".to_string());
                            event.outcome = Some("success".to_string());

                            let network = json.network();
                            network.protocol = Some("smtp".to_string());
                            network.transport = Some("tcp".to_string());

                            convert_source(json, pair.into_inner());
                        }
                        Rule::smtpd_disconnect => {
                            let event = json.event();
                            event.category.push("network".to_string());
                            event.type_val.push("connection".to_string());
                            event.type_val.push("end".to_string());
                            event.outcome = Some("success".to_string());

                            let network = json.network();
                            network.protocol = Some("smtp".to_string());
                            network.transport = Some("tcp".to_string());

                            convert_source(json, pair.into_inner());
                        }
                        Rule::smtpd_lost_connection => {
                            let event = json.event();
                            event.category.push("network".to_string());
                            event.type_val.push("connection".to_string());
                            event.type_val.push("protocol".to_string());
                            event.outcome = Some("failure".to_string());
                            event.severity = Some(300);

                            let network = json.network();
                            network.protocol = Some("smtp".to_string());
                            network.transport = Some("tcp".to_string());

                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::smtpd_lost_connection_msg => {
                                        json.error().message = Some(pair.as_str().to_string())
                                    }

                                    Rule::hostname_ip => {
                                        convert_hostname_ip_to_source(json, pair.into_inner())
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Rule::smtpd_auth_failed => {
                            let event = json.event();
                            event.category.push("authentication".to_string());
                            event.type_val.push("protocol".to_string());
                            event.outcome = Some("failure".to_string());
                            event.severity = Some(300);

                            convert_source(json, pair.into_inner());
                        }
                        Rule::smtpd_mail_open_stream => {
                            let event = json.event();
                            event.type_val.push("connection".to_string());
                            event.outcome = Some("success".to_string());

                            let network = json.network();
                            network.protocol = Some("smtp".to_string());
                            network.transport = Some("tcp".to_string());

                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::queue_id => convert_queue_id(json, pair.as_str(), host),
                                    Rule::hostname_ip => {
                                        convert_hostname_ip_to_source(json, pair.into_inner())
                                    }
                                    Rule::key_value_pair => {
                                        let mut key = None;
                                        let mut value = None;
                                        for pair in pair.into_inner() {
                                            match pair.as_rule() {
                                                Rule::key => key = Some(pair.as_str()),
                                                Rule::value => value = Some(pair.as_str()),
                                                _ => {}
                                            }
                                        }
                                        if let (Some("sasl_username"), Some(value)) = (key, value) {
                                            json.event()
                                                .category
                                                .push("authentication".to_string());
                                            json.user().name = Some(value.to_string());
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
            _ => {}
        }
    }
}

fn convert_postfix_script(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    json.process().name = Some("postfix-script".to_string());

    json.event().category.push("email".to_string());

    for pair in pairs {
        match pair.as_rule() {
            Rule::pid => convert_pid(json, pair.as_str()),
            Rule::log_level => convert_log_level(json, pair.into_inner()),
            Rule::message_postfix_script => {
                json.message = Some(pair.as_str().to_string());
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::postfix_script_starting_postfix => {
                            let event = json.event();
                            event.category.push("process".to_string());
                            event.type_val.push("start".to_string());
                            event.outcome = Some("unknown".to_string());
                        }
                        Rule::postfix_script_group_writable => {
                            let event = json.event();
                            event.category.push("file".to_string());
                            event.type_val.push("info".to_string());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn convert_anvil(
    json: &mut FluentBitJson,
    pairs: pest::iterators::Pairs<'_, Rule>,
    event_date: &DateTime<FixedOffset>,
) {
    json.process().name = Some("anvil".to_string());

    json.event().kind = Some("metric".to_string());
    json.event().category.push("email".to_string());
    json.event().severity = Some(100);

    for pair in pairs {
        match pair.as_rule() {
            Rule::pid => convert_pid(json, pair.as_str()),
            Rule::message_anvil => {
                json.message = Some(pair.as_str().to_string());
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::anvil_rate | Rule::anvil_count | Rule::anvil_cache => {
                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::anvil_metric_type => {
                                        let event = json.event();
                                        event.category.push("network".to_string());
                                        if pair.as_str() == "connection" {
                                            event.type_val.push("connection".to_string());
                                        }
                                    }
                                    Rule::anvil_protocol => {
                                        json.network().protocol = Some(pair.as_str().to_string())
                                    }
                                    Rule::ip => json.source().ip = Some(pair.as_str().to_string()),
                                    Rule::timestamp => {
                                        json.event().end =
                                            convert_date(pair.into_inner(), event_date)
                                    }
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

fn convert_master(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    json.process().name = Some("master".to_string());

    json.event().category.push("email".to_string());

    for pair in pairs {
        match pair.as_rule() {
            Rule::pid => convert_pid(json, pair.as_str()),
            Rule::log_level => convert_log_level(json, pair.into_inner()),
            Rule::message_master => {
                json.message = Some(pair.as_str().to_string());
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::master_daemon_started => {
                            let event = json.event();
                            event.category.push("process".to_string());
                            event.type_val.push("start".to_string());
                            event.outcome = Some("success".to_string());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn convert_main(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    json.event().category.push("email".to_string());

    for pair in pairs {
        match pair.as_rule() {
            Rule::pid => convert_pid(json, pair.as_str()),
            Rule::log_level => convert_log_level(json, pair.into_inner()),
            Rule::message_main => json.message = Some(pair.as_str().to_string()),
            _ => {}
        }
    }
}

fn convert_other(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::pid => convert_pid(json, pair.as_str()),
            Rule::log_level => convert_log_level(json, pair.into_inner()),
            Rule::process_other => json.process().name = Some(pair.as_str().to_string()),
            Rule::message_other => json.message = Some(pair.as_str().to_string()),
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

fn convert_log_level(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    json.event().severity = Some(200);
    for pair in pairs {
        match pair.as_rule() {
            Rule::log_level_warning => {
                json.event().severity = Some(300);
                json.log().level = Some("warning".to_string());
            }
            _ => {}
        }
    }
}

fn convert_source(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::hostname_ip => convert_hostname_ip_to_source(json, pair.into_inner()),
            _ => {}
        }
    }
}

fn convert_hostname_ip_to_source(
    json: &mut FluentBitJson,
    pairs: pest::iterators::Pairs<'_, Rule>,
) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::hostname => match pair.as_str() {
                "unknown" => {}
                hostname => json.source().domain = Some(hostname.to_string()),
            },
            Rule::ip => json.source().ip = Some(pair.as_str().to_string()),
            _ => {}
        }
    }
}

fn convert_queue_id(json: &mut FluentBitJson, queue_id: &str, host: Option<&str>) {
    if queue_id == "NOQUEUE" {
        return;
    }
    match host {
        Some(host) => json.transaction().id = Some(format!("{}.{}", host, queue_id)),
        None => json.transaction().id = Some(queue_id.to_string()),
    }
}
