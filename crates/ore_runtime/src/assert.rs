use crate::*;
use std::sync::atomic::{AtomicBool, Ordering};

// ── Fatal errors ──

#[no_mangle]
pub extern "C" fn ore_assert_fail(msg: *mut OreStr) {
    let s = if msg.is_null() {
        "assertion failed".to_string()
    } else {
        unsafe { (*msg).as_str().to_string() }
    };
    eprintln!("{}", s);
    std::process::exit(1);
}

/// Runtime error for division by zero.
#[no_mangle]
pub extern "C" fn ore_div_by_zero() {
    eprintln!("runtime error: division by zero");
    std::process::exit(1);
}

// ── Test mode support ──

/// When true, ore_assert sets a flag instead of calling process::exit (used by `ore test`)
static ASSERT_TEST_MODE: AtomicBool = AtomicBool::new(false);
/// Set to true when an assertion fails in test mode
static ASSERT_FAILED: AtomicBool = AtomicBool::new(false);

/// Enable/disable test mode for assertions
pub fn ore_assert_set_test_mode(enabled: bool) {
    ASSERT_TEST_MODE.store(enabled, Ordering::SeqCst);
}

/// Check and reset the assertion failure flag
pub fn ore_assert_check_and_reset() -> bool {
    ASSERT_FAILED.swap(false, Ordering::SeqCst)
}

// ── Assert helper ──

fn assert_fail_msg(msg: *const u8, line: i64, detail: &str) {
    let message = unsafe { std::ffi::CStr::from_ptr(msg as *const i8) };
    let msg_str = message.to_str().unwrap_or("(invalid utf8)");
    let full_msg = if detail.is_empty() {
        format!("assertion failed at line {}: {}", line, msg_str)
    } else {
        format!("assertion failed at line {}: {} ({})", line, msg_str, detail)
    };
    if ASSERT_TEST_MODE.load(Ordering::SeqCst) {
        eprintln!("    {}", full_msg);
        ASSERT_FAILED.store(true, Ordering::SeqCst);
    } else {
        eprintln!("{}", full_msg);
        std::process::exit(1);
    }
}

// ── Assert functions ──

#[no_mangle]
pub extern "C" fn ore_assert(cond: i8, msg: *const u8, line: i64) {
    if cond == 0 {
        assert_fail_msg(msg, line, "");
    }
}

#[no_mangle]
pub extern "C" fn ore_assert_eq_int(left: i64, right: i64, msg: *const u8, line: i64) {
    if left != right {
        assert_fail_msg(msg, line, &format!("left: {}, right: {}", left, right));
    }
}

#[no_mangle]
pub extern "C" fn ore_assert_eq_float(left: f64, right: f64, msg: *const u8, line: i64) {
    if (left - right).abs() > f64::EPSILON {
        assert_fail_msg(msg, line, &format!("left: {}, right: {}", left, right));
    }
}

#[no_mangle]
pub extern "C" fn ore_assert_eq_str(left: *mut OreStr, right: *mut OreStr, msg: *const u8, line: i64) {
    let l = unsafe { &*left }.as_str();
    let r = unsafe { &*right }.as_str();
    if l != r {
        assert_fail_msg(msg, line, &format!("left: \"{}\", right: \"{}\"", l, r));
    }
}

#[no_mangle]
pub extern "C" fn ore_assert_ne_int(left: i64, right: i64, msg: *const u8, line: i64) {
    if left == right {
        assert_fail_msg(msg, line, &format!("both values: {}", left));
    }
}

#[no_mangle]
pub extern "C" fn ore_assert_ne_str(left: *mut OreStr, right: *mut OreStr, msg: *const u8, line: i64) {
    let l = unsafe { &*left }.as_str();
    let r = unsafe { &*right }.as_str();
    if l == r {
        assert_fail_msg(msg, line, &format!("both values: \"{}\"", l));
    }
}
