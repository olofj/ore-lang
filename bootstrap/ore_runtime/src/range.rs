use crate::*;

// ── Range ──

/// Create a list of integers from start (inclusive) to end (exclusive).
#[no_mangle]
pub extern "C" fn ore_range(start: i64, end: i64) -> *mut OreList {
    let list = ore_list_new();
    let mut i = start;
    while i < end {
        ore_list_push(list, i);
        i += 1;
    }
    list
}

/// Create a list of integers from start to end with a step.
#[no_mangle]
pub extern "C" fn ore_range_step(start: i64, end: i64, step: i64) -> *mut OreList {
    let list = ore_list_new();
    if step == 0 { return list; }
    let mut i = start;
    if step > 0 {
        while i < end {
            ore_list_push(list, i);
            i += step;
        }
    } else {
        while i > end {
            ore_list_push(list, i);
            i += step;
        }
    }
    list
}

// ── Time functions ──

fn unix_epoch_duration() -> std::time::Duration {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn ore_time_now() -> i64 {
    unix_epoch_duration().as_secs() as i64
}

#[no_mangle]
pub extern "C" fn ore_time_ms() -> i64 {
    unix_epoch_duration().as_millis() as i64
}

// ── Random ──

use std::cell::Cell;

thread_local! {
    static RNG_STATE: Cell<u64> = const { Cell::new(0) };
}

fn next_rand() -> u64 {
    RNG_STATE.with(|state| {
        let mut s = state.get();
        if s == 0 {
            // Seed from time and address
            s = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(12345);
        }
        // xorshift64
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        state.set(s);
        s
    })
}

#[no_mangle]
pub extern "C" fn ore_rand_int(low: i64, high: i64) -> i64 {
    let range = (high - low + 1) as u64;
    if range == 0 { return low; }
    low + (next_rand() % range) as i64
}
