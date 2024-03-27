use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use crate::model::FluentBitJson;

pub fn convert_kubernetes_dashboard_metrics_scraper(json: &mut FluentBitJson) {
    let level = json.other.remove("level").and_then(|level| match level {
        Value::String(level) => Some(level),
        _ => None,
    });

    // @timestamp
    match convert_ts(json) {
        TsParseResult::Ok(ts) => json.timestamp = Some(ts),
        TsParseResult::Err(ts) => json.misc.push(format!("time:{}", ts)),
        TsParseResult::None => {}
    }

    // msg
    match json.other.remove("msg") {
        Some(Value::String(message)) => json.message = Some(message),
        Some(message) => json.message = Some(message.to_string()),
        _ => {}
    }

    // event
    let event = json.event();
    event.module = Some("kubernetes_dashboard".to_string());
    match &level {
        Some(level) => {
            event.severity = convert_severity(&level);
        }
        _ => {}
    };

    // log
    let log = json.log();
    match level {
        Some(level) => log.level = Some(level),
        _ => {}
    };
}

fn convert_severity(level: &str) -> Option<u32> {
    match level {
        "info" => Some(200),
        "warning" => Some(300),
        "error" => Some(400),
        "fatal" => Some(500),
        _ => None,
    }
}


fn convert_ts(json: &mut FluentBitJson) -> TsParseResult {
    match json.other.remove("time") {
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