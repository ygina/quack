use crate::*;

#[no_mangle]
pub extern "C" fn quack_global_config_set_max_power_sum_threshold(threshold: usize) {
    global_config_set_max_power_sum_threshold(threshold);
}

#[no_mangle]
pub extern "C" fn quack_new(threshold: usize) -> *mut PowerSumQuackU32 {
    let quack = PowerSumQuackU32::new(threshold);
    Box::into_raw(Box::new(quack))
}

#[no_mangle]
pub extern "C" fn quack_threshold(quack: *const PowerSumQuackU32) -> usize {
    debug_assert!(!quack.is_null());
    unsafe { (*quack).threshold() }
}

#[no_mangle]
pub extern "C" fn quack_count(quack: *const PowerSumQuackU32) -> u32 {
    debug_assert!(!quack.is_null());
    unsafe { (*quack).count() }
}

#[no_mangle]
pub extern "C" fn quack_last_value(quack: *const PowerSumQuackU32) -> u32 {
    debug_assert!(!quack.is_null());
    let result = unsafe { (*quack).last_value() };
    result.unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn quack_insert(quack: *mut PowerSumQuackU32, value: u32) {
    debug_assert!(!quack.is_null());
    unsafe { (*quack).insert(value) };
}

#[no_mangle]
pub extern "C" fn quack_remove(quack: *mut PowerSumQuackU32, value: u32) {
    debug_assert!(!quack.is_null());
    unsafe { (*quack).remove(value) };
}

#[no_mangle]
pub extern "C" fn quack_decode_with_log(
    quack: *const PowerSumQuackU32,
    log: *const u32,
    len: usize,
    out_buffer: *mut u32,
    out_buffer_size: usize,
) -> usize {
    debug_assert!(!quack.is_null());
    debug_assert!(!log.is_null());
    debug_assert!(!out_buffer.is_null());
    let log_slice = unsafe { std::slice::from_raw_parts(log, len) };
    let result = unsafe { (*quack).decode_with_log(log_slice) };
    assert!(result.len() <= out_buffer_size);
    unsafe {
        std::ptr::copy_nonoverlapping(result.as_ptr(), out_buffer, result.len());
    }
    result.len()
}

#[no_mangle]
pub extern "C" fn quack_sub(
    lhs: *mut PowerSumQuackU32,
    rhs: *mut PowerSumQuackU32,
) -> *mut PowerSumQuackU32 {
    debug_assert!(!lhs.is_null());
    debug_assert!(!rhs.is_null());
    let lhs_ref = unsafe { Box::from_raw(lhs) };
    let rhs_ref = unsafe { Box::from_raw(rhs) };
    let result = lhs_ref.sub(*rhs_ref);
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn quack_free(quack: *mut PowerSumQuackU32) {
    debug_assert!(!quack.is_null());
    unsafe { drop(Box::from_raw(quack)) };
}