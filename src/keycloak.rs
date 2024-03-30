use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use log::warn;
use pest::Parser;
use pest_derive::Parser;

use crate::model::FluentBitJson;

#[derive(Parser)]
#[grammar_inline = r#"
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }

event_log = { SOI ~ key_value_pair ~ ("," ~ key_value_pair)* ~ EOI }
key_value_pair = { key ~ "=" ~ value }

key = { (ASCII_ALPHA | "_" | "-" )+ }
value = ${ "\"" ~ value_inner ~ "\"" }
value_inner = @{ value_char * }
value_char = {
    !("\"" | "\\") ~ ANY
    | "\\" ~ ("\"" | "\\" | "n" | "r" | "t")
}
"#]
struct KeycloakEventParser;

pub fn convert_keycloak_logs(json: &mut FluentBitJson) {
    let level = json.other.remove("level").and_then(|level| match level {
        Value::String(level) => Some(level),
        _ => None,
    });

    // @timestamp
    match convert_ts(json) {
        TsParseResult::Ok(ts) => json.timestamp = Some(ts),
        TsParseResult::Err(ts) => json.misc.push(format!("ts:{}", ts)),
        TsParseResult::None => {}
    };

    // service
    json.service().type_val = Some("keycloak".to_string());
    let sequence = json.other.remove("sequence");

    // event
    let event = json.event();
    event.module = Some("keycloak".to_string());
    event.module = Some("keycloak".to_string());
    event.category = vec!["iam".to_string()];
    match &level {
        Some(level) => {
            event.severity = convert_severity(&level);
        }
        _ => {}
    };
    match sequence {
        Some(Value::Number(sequence)) => {
            event.sequence = sequence.as_u64();
        }
        Some(sequence) => {
            json.misc.push(format!("sequence:{}", sequence.to_string()));
        }
        _ => {}
    }

    // log
    let logger_name = json.other.remove("loggerName");
    let log = json.log();
    match level {
        Some(level) => log.level = Some(level),
        _ => {}
    };
    match logger_name {
        Some(Value::String(logger)) => {
            log.logger = Some(logger);
        }
        Some(logger) => {
            json.misc.push(format!("logger:{}", logger.to_string()));
        }
        _ => {}
    }

    // process
    match json.other.remove("processName") {
        Some(Value::String(process_name)) => {
            json.process().name = Some(process_name);
        }
        Some(process_name) => {
            json.misc
                .push(format!("processName:{}", process_name.to_string()));
        }
        _ => {}
    }
    match json.other.remove("processId") {
        Some(Value::Number(process_id)) => match process_id.as_u64() {
            Some(val_64) => match val_64.try_into() {
                Ok(val_32) => {
                    json.process().pid = Some(val_32);
                }
                _ => {
                    json.misc.push(format!("processId:{}", val_64));
                }
            },
            _ => {
                json.misc
                    .push(format!("processId:{}", process_id.to_string()));
            }
        },
        Some(process_id) => {
            json.misc.push(format!("process_id:{}", process_id));
        }
        _ => {}
    }
    match json.other.remove("threadName") {
        Some(Value::String(thread_name)) => {
            json.process().thread().name = Some(thread_name);
        }
        Some(thread_name) => {
            json.misc
                .push(format!("threadName:{}", thread_name.to_string()));
        }
        _ => {}
    }
    match json.other.remove("threadId") {
        Some(Value::Number(thread_id)) => match thread_id.as_u64() {
            Some(val_64) => match val_64.try_into() {
                Ok(val_32) => {
                    json.process().thread().id = Some(val_32);
                }
                _ => {
                    json.misc.push(format!("threadId:{}", val_64));
                }
            },
            _ => {
                json.misc
                    .push(format!("threadId:{}", thread_id.to_string()));
            }
        },
        Some(process_id) => {
            json.misc.push(format!("threadId:{}", process_id));
        }
        _ => {}
    }

    // host
    match json.other.remove("hostName") {
        Some(Value::String(host_name)) => json.host().hostname = Some(host_name),
        Some(host_name) => {
            json.misc
                .push(format!("hostName:{}", host_name.to_string()));
        }
        _ => {}
    }

    // Remove non-ecs mappings if they exist
    json.move_key_to_misc("mdc");
    json.move_key_to_misc("ndc");
    json.move_key_to_misc("loggerClassName");

    // parse events
    if json
        .log()
        .logger
        .as_ref()
        .is_some_and(|s| s == "org.keycloak.events")
    {
        parse_event_log(json);
    }
}

fn parse_event_log(json: &mut FluentBitJson) {
    let message = json.message.clone();

    match message {
        Some(message) => match KeycloakEventParser::parse(Rule::event_log, &message) {
            Ok(pairs) => {
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::event_log => parse_event_log_rule(json, pair.into_inner()),
                        _ => {}
                    }
                }
            }
            Err(err) => warn!("parsing Keycloak event log failed: {}", err),
        },
        None => {}
    }
}

fn parse_event_log_rule(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::key_value_pair => {
                parse_event_key_value_par(json, pair.into_inner());
            }
            _ => {}
        }
    }
}

fn parse_event_key_value_par(json: &mut FluentBitJson, pairs: pest::iterators::Pairs<'_, Rule>) {
    let mut key: Option<&str> = None;
    let mut value: Option<&str> = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::key => key = Some(pair.as_str()),
            Rule::value => {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::value_inner => value = Some(pair.as_str()),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if let (Some(key), Some(value)) = (key, value) {
        let value = unmask(value);
        match key {
            "type" => convert_event_type(json, value),
            "ipAddress" => json.source().ip = Some(value),
            "error" => json.error().message = Some(value),
            "username" => json.user().name = Some(value),
            "userId" => {
                if value != "null" {
                    json.user().id = Some(value);
                }
            }
            _ => {}
        }
    }
}

fn convert_event_type(json: &mut FluentBitJson, value: String) {
    let event = json.event();

    match value.as_str() {
        "LOGIN_ERROR" => {
            event.category.push("authentication".to_string());
            event.type_val.push("denied".to_string());
            event.outcome = Some("success".to_string());
        },
        "LOGIN" => {
            event.category.push("authentication".to_string());
            event.type_val.push("allowed".to_string());
            event.outcome = Some("success".to_string());
        },
        "CODE_TO_TOKEN" => {
            event.category.push("authentication".to_string());
        }
        _ => {}
    }
    
    event.action = Some(value);
}

fn unmask(string: &str) -> String {
    string
        .replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\\n", "\n")
        .replace("\\\r", "\r")
        .replace("\\\t", "\t")
}

fn convert_severity(level: &str) -> Option<u32> {
    match level {
        "TRACE" => Some(50),
        "DEBUG" => Some(100),
        "INFO" => Some(200),
        "WARN" => Some(300),
        "ERROR" => Some(400),
        "FATAL" => Some(500),
        _ => None,
    }
}

fn convert_ts(json: &mut FluentBitJson) -> TsParseResult {
    match json.other.remove("timestamp") {
        Some(Value::String(ts)) => match DateTime::parse_from_rfc3339(&ts) {
            Ok(date) => TsParseResult::Ok(date),
            Err(_) => TsParseResult::Err(ts),
        },
        _ => TsParseResult::None,
    }
}

enum TsParseResult {
    Ok(DateTime<FixedOffset>),
    Err(String),
    None,
}
