use std::os::raw::c_char;
use std::slice;

mod kubernetes;
mod model;

#[no_mangle]
pub extern "C" fn fluent_ecs_filter(
    _tag: *const c_char,
    _tag_len: u32,
    _time_sec: u32,
    _time_nsec: u32,
    record: *const c_char,
    record_len: u32,
) -> *const u8 {
    let slice_record: &[u8] =
        unsafe { slice::from_raw_parts(record as *const u8, record_len as usize) };

    fluent_ecs_filter_rust(slice_record).as_ptr()
}

pub fn fluent_ecs_filter_rust(record: &[u8]) -> String {
    let mut json: model::FluentBitJson = serde_json::from_slice(record).unwrap();

    kubernetes::convert_kubernetes_metadata(&mut json);

    // https://www.elastic.co/guide/en/ecs/current/ecs-ecs.html
    // ecs.version: 8.11

    match serde_json::to_string(&json) {
        Ok(res) => res,
        Err(err) => format!("{{\"fluent-ecs-error\": \"{}\"}}", err),
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
    #[case(
        "examples/kubernetes-StatefulSet-in.json",
        "examples/kubernetes-StatefulSet-out.json"
    )]
    #[case(
        "examples/kubernetes-Deployment-in.json",
        "examples/kubernetes-Deployment-out.json"
    )]
    fn conversion_test(#[case] input: &str, #[case] output: &str) -> Result<(), String> {
        let input = fs::read(input)
            .map_err(|err| format!("Input file could not be read: {}", err.to_string()))?;

        let actual_string = fluent_ecs_filter_rust(&input);
        let actual: Value = serde_json::from_str(&actual_string).map_err(|err| {
            format!(
                "Module under test did not return valid JSON error:\n{}\n\n value:\n{}",
                err.to_string(),
                actual_string
            )
        })?;

        println!("Output compared against '{}':\n{}\n", output, actual_string);

        let expected_file =
            fs::read(output).map_err(|err| format!("Output file could not be read: {}", err))?;
        let expected: Value = serde_json::from_slice(&expected_file)
            .map_err(|err| format!("Output file is not valid JSON: {}", err))?;

        assert_json_eq!(actual, expected);

        Ok(())
    }
}
