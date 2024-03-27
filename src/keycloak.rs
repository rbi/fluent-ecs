use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use crate::model::FluentBitJson;

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
