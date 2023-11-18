use std::os::raw::c_char;
use std::slice;

use chrono::{DateTime, Duration, FixedOffset};
use serde_json::Value;

mod kubernetes;
mod metallb;
mod model;

#[no_mangle]
pub extern "C" fn fluent_ecs_filter(
    _tag: *const c_char,
    _tag_len: u32,
    time_sec: u32,
    time_nsec: u32,
    record: *const c_char,
    record_len: u32,
) -> *const u8 {
    let slice_record: &[u8] =
        unsafe { slice::from_raw_parts(record as *const u8, record_len as usize) };

    let time = DateTime::from_timestamp(time_sec.into(), time_nsec)
        .expect("Time passed from fluent-bit could not be parsed.")
        .fixed_offset();
    let mut res = fluent_ecs_filter_rust(slice_record, time)
        .as_bytes()
        .to_vec();
    res.push(0);
    res.as_ptr()
}

// https://www.elastic.co/guide/en/ecs/current/ecs-ecs.html
// ecs.version: 8.11

pub fn fluent_ecs_filter_rust(record: &[u8], time: DateTime<FixedOffset>) -> String {
    let mut json: model::FluentBitJson = serde_json::from_slice(record).unwrap();

    let parser = json
        .kubernetes
        .as_ref()
        .and_then(|k| k.annotations.get("fluent-ecs.bieniek-it.de/parser"))
        .and_then(|val| match val {
            Value::String(string_val) => Some(string_val),
            _ => None,
        });

    if let Some(parser) = parser {
        match parser.as_str() {
            "metallb" => metallb::convert_metallb_logs(&mut json),
            _ => {}
        }
    }

    kubernetes::convert_kubernetes_metadata(&mut json);

    set_basic_data(&mut json, time);

    match serde_json::to_string(&json) {
        Ok(res) => res,
        Err(err) => format!("{{\"fluent-ecs-error\": \"{}\"}}", err),
    }
}

fn set_basic_data(json: &mut model::FluentBitJson, time: DateTime<FixedOffset>) {
    let event = json.event();

    if let None = event.kind {
        event.kind.get_or_insert("event".to_string());
    }
    if let Some(ts) = json.timestamp {
        if ts - time != Duration::zero() {
            json.event().created = Some(time);
        }
    } else {
        json.timestamp = Some(time);
    }
}

#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use rstest::*;
    use serde_json::Value;
    use std::fs;

    use super::*;

    #[rstest]
    #[case("kubernetes-StatefulSet")]
    #[case("kubernetes-Deployment")]
    #[case("metallb-speaker-service_announced")]
    #[case("metallb-speaker-partial_join")]
    #[case("metallb-controller-PoolReconciler")]
    #[case("metallb-controller-cert_rotation")]
    fn conversion_test(#[case] test_case: &str) -> Result<(), String> {
        let some_time = DateTime::parse_from_rfc3339("2023-11-16T13:27:38.555+01:00")
            .map_err(|err| err.to_string())?;
        let input = fs::read(format!("examples/{}-in.json", test_case))
            .map_err(|err| format!("Input file could not be read: {}", err.to_string()))?;

        let actual_string = fluent_ecs_filter_rust(&input, some_time);
        let actual: Value = serde_json::from_str(&actual_string).map_err(|err| {
            format!(
                "Module under test did not return valid JSON error:\n{}\n\n value:\n{}",
                err.to_string(),
                actual_string
            )
        })?;

        let output = format!("examples/{}-out.json", test_case);
        println!("Output compared against '{}':\n{}\n", output, actual_string);

        let expected_file =
            fs::read(output).map_err(|err| format!("Output file could not be read: {}", err))?;
        let expected: Value = serde_json::from_slice(&expected_file)
            .map_err(|err| format!("Output file is not valid JSON: {}", err))?;

        assert_json_eq!(actual, expected);

        Ok(())
    }
}
