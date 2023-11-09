use std::os::raw::c_char;
use std::slice;

#[no_mangle]
pub extern "C" fn fluent_ecs_filter(
    _tag: *const c_char,
    _tag_len: u32,
    _time_sec: u32,
    _time_nsec: u32,
    record: *const c_char,
    record_len: u32,
) -> *const u8 {
    // let slice_record: &[u8] = unsafe { slice::from_raw_parts(record as *const u8, record_len as usize) };

    format!("{{\"fluent.ecs\": {} }}", record_len).as_ptr()
}
