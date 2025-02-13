use crate::*;

#[no_mangle]
pub extern "C" fn quack_global_config_set_max_power_sum_threshold(threshold: usize) {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn quack_new(threshold: usize) -> *mut PowerSumQuackU32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn quack_threshold(quack: *const PowerSumQuackU32) -> usize {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn quack_count(quack: *const PowerSumQuackU32) -> u32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn quack_last_value(quack: *const PowerSumQuackU32) -> u32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn quack_insert(quack: *mut PowerSumQuackU32, value: u32) {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn quack_remove(quack: *mut PowerSumQuackU32, value: u32) {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn quack_decode_with_log(
    quack: *const PowerSumQuackU32,
    log: *const u32,
    len: usize,
    out_buffer: *mut u32,
    out_buffer_size: usize,
) -> usize {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn quack_sub(
    lhs: *mut PowerSumQuackU32,
    rhs: *mut PowerSumQuackU32,
) -> *mut PowerSumQuackU32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn quack_free(quack: *mut PowerSumQuackU32) {
    unimplemented!()
}