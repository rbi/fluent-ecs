use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use crate::model::ErrorOrString;
use crate::model::EventOrString;
use crate::model::FluentBitJson;

pub fn convert_metallb_logs(json: &mut FluentBitJson) {
    let event_or_op = extract_op(json);
    let level = json.other.remove("level").and_then(|level| match level {
        Value::String(level) => Some(level),
        _ => None,
    });

    // @timestamp
    match convert_ts(json) {
        TsParseResult::Ok(ts) => json.timestamp = Some(ts),
        TsParseResult::Err(ts) => json.misc.push(format!("ts:{}", ts)),
        TsParseResult::None => {}
    }

    // error and msg
    let action = json.other.remove("action");
    match (json.other.remove("msg"), json.error.as_ref()) {
        (Some(Value::String(msg)), Some(ErrorOrString::String(error))) => {
            json.message = Some(error.to_string());
            json.error().message = Some(error.to_string());
            json.misc.push(format!("msg:{}", msg));
        }
        (Some(Value::String(msg)), _) => {
            json.message = Some(msg);
        }
        (_, Some(ErrorOrString::String(error))) => {
            json.message = Some(error.to_string());
            json.error().message = Some(error.to_string());
        }
        _ => {
            if let Some(ev) = event_or_op.as_ref() {
                json.message = Some(ev.to_string());
            } else if let Some(action) = action {
                json.message = Some(action.to_string());
            }
        }
    }

    // service
    json.service().type_val = Some("metallb".to_string());

    // event
    {
        let event = json.event();
        event.kind = Some("event".to_string());
        event.module = Some("metallb".to_string());
        match event_or_op {
            Some(ev) => {
                event.category = convert_category(&ev);
                event.type_val = convert_type(&ev);
                event.outcome = convert_outcome(&ev, &level);
                event.action = convert_action(&ev);
            }
            _ => {
                event.category = vec!["network".to_string()];
            }
        };
        match &level {
            Some(level) => {
                event.severity = convert_severity(&level);
            }
            _ => {}
        };
    }

    // log
    let caller = convert_caller(json);
    let logger = json.other.remove("logger");
    {
        let log = json.log();
        match level {
            Some(level) => log.level = Some(level),
            _ => {}
        };
        match caller {
            Some((file, line)) => {
                let orign_file = log.origin().file();
                orign_file.name = Some(file);
                orign_file.line = Some(line);
            }
            _ => {}
        }
        if let Some(Value::String(logger)) = logger {
            log.logger = Some(logger);
        }
    }

    // network
    if let Some(Value::String(protocol)) = json.other.remove("protocol") {
        json.network().protocol = Some(protocol);
    }

    // Remove non-ecs mappings if they exist
    json.move_key_to_misc("ips");
    json.move_key_to_misc("interface");
    json.move_key_to_misc("ips");
    json.move_key_to_misc("pool");
    json.move_key_to_misc("controller");
    json.move_key_to_misc("name");
    json.move_key_to_misc("gvk");
    json.move_key_to_misc("IPAdvertisement");
    json.move_key_to_misc("service");
    json.move_key_to_misc("localIfs");
    json.move_key_to_misc("reason");
    json.move_key_to_misc("expected");
    json.move_key_to_misc("joined");
}

fn convert_action(ev: &str) -> Option<String> {
    match ev {
        "force service reload" => None,
        _ => Some(ev.to_string()),
    }
}

fn extract_op(json: &mut FluentBitJson) -> Option<String> {
    json.event
        .as_ref()
        .and_then(|ev| match ev {
            EventOrString::String(s) => Some(s.to_string()),
            _ => None,
        })
        .or_else(|| match json.other.remove("op") {
            Some(Value::String(op)) => Some(op),
            _ => None,
        })
}

fn convert_category(event: &str) -> Vec<String> {
    match event {
        "sessionUp" | "sessionDown" => vec!["network".to_string(), "session".to_string()],
        _ => vec!["network".to_string()],
    }
}

fn convert_type(event: &str) -> Vec<String> {
    match event {
        "serviceAnnounced" | "sessionUp" => vec!["start".to_string()],
        "serviceWithdrawn" | "sessionDown" => vec!["end".to_string()],
        "createARPResponder" | "createNDPResponder" => vec!["creation".to_string()],
        _ => Vec::new(),
    }
}

fn convert_outcome(ev: &str, level: &Option<String>) -> Option<String> {
    match level {
        Some(level) => match (level.as_str(), ev) {
            (
                "info",
                "serviceAnnounced" | "serviceWithdrawn" | "serviceDeleted" | "peerAdded"
                | "peerRemoved" | "sessionUp" | "createARPResponder" | "createNDPResponder",
            ) => Some("success".to_string()),
            (
                "error",
                "updateServiceStatus"
                | "connect"
                | "sendUpdate"
                | "getInterfaces"
                | "getAddresses"
                | "reload-validate"
                | "reload"
                | "listenAndServe",
            ) => Some("failure".to_string()),
            _ => None,
        },
        None => None,
    }
}

fn convert_severity(level: &str) -> Option<u32> {
    match level {
        "debug" => Some(100),
        "info" => Some(200),
        "warn" => Some(300),
        "error" => Some(400),
        _ => None,
    }
}

fn convert_caller(json: &mut FluentBitJson) -> Option<(String, u32)> {
    match json.other.remove("caller") {
        Some(Value::String(caller)) => {
            let (file, line) = caller.split_once(":")?;
            let line_nr = line.parse::<u32>().ok()?;
            Some((file.to_string(), line_nr))
        }
        _ => None,
    }
}

fn convert_ts(json: &mut FluentBitJson) -> TsParseResult {
    match json.other.remove("ts") {
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
