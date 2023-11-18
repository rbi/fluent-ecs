use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use crate::model::FluentBitJson;

pub fn convert_etcd_logs(json: &mut FluentBitJson) {
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

    // msg
    if let Some(Value::String(msg)) = json.other.remove("msg") {
        json.message = Some(msg);
    }

    // service
    json.service().type_val = Some("etcd".to_string());

    // event
    let took = json.other.remove("took");
    {
        let event = json.event();
        event.kind = Some("event".to_string());
        event.module = Some("etcd".to_string());
        event.category = vec!["database".to_string()];
        match &level {
            Some(level) => {
                event.severity = convert_severity(&level);
            }
            _ => {}
        };
        if let Some(Value::String(duration)) = took {
            event.duration = convert_duration(duration);
        }
    }

    // log
    let caller = convert_caller(json);
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
    }

    // Remove non-ecs mappings if they exist
    json.move_key_to_misc("hash");
    json.move_key_to_misc("compact-index");
    json.move_key_to_misc("compact-revision");
    json.move_key_to_misc("expected-duration");
    json.move_key_to_misc("prefix");
    json.move_key_to_misc("request");
    json.move_key_to_misc("response");
    json.move_key_to_misc("revision");
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

fn convert_duration(duration: String) -> Option<u64> {
    let (number, factor) = if let Some(number) = duration.strip_suffix("ms") {
        Some((number, 1000.0 * 1000.0))
    } else if let Some(number) = duration.strip_suffix("s") {
        Some((number, 1000.0 * 1000.0 * 1000.0))
    } else {
        None
    }?;

    let duration = number.parse::<f64>().ok()?;
    Some((duration * factor) as u64)
    // if number.contains(".") {
    //     let (int, fract) = number.split_once(".")?;
    //     if let (Ok(int), Ok(fract)) = (int.parse::<u64>(), fract.parse::<u64>()) {
    //         let factor_fract =
    //     }
    //     None
    // } else {
    //     match number.parse::<u64>() {
    //         Ok(n) => Some(n * factor),
    //         Err(_) => None,
    //     }
    // }
}

enum TsParseResult {
    Ok(DateTime<FixedOffset>),
    Err(String),
    None,
}
