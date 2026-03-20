use crate::*;
use crate::kinds::*;
use std::collections::HashSet;

// ── Lists ──
//
// OreList: heap-allocated growable array of i64 values.
// Layout: { len: i64, cap: i64, data: *mut i64 }

/// Normalize a possibly-negative index and bounds-check it.
/// Returns the valid usize index, or exits with an error.
fn checked_index(index: i64, len: i64) -> usize {
    let idx = if index < 0 { len + index } else { index };
    if idx < 0 || idx >= len {
        eprintln!("index out of bounds: {} (len {})", index, len);
        std::process::exit(1);
    }
    idx as usize
}

#[repr(C)]
pub struct OreList {
    pub len: i64,
    pub cap: i64,
    pub data: *mut i64,
    /// Per-element kind tags (parallel to `data`).  NULL means "all KIND_INT".
    pub kinds: *mut i8,
}

impl OreList {
    /// Return the list contents as a slice.
    /// # Safety
    /// The caller must ensure `self` is a valid, initialized OreList.
    pub unsafe fn as_slice(&self) -> &[i64] {
        if self.data.is_null() || self.len == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.data, self.len as usize)
        }
    }

    /// Return the list contents as a mutable slice.
    /// # Safety
    /// The caller must ensure `self` is a valid, initialized OreList.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [i64] {
        if self.data.is_null() || self.len == 0 {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(self.data, self.len as usize)
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_list_new() -> *mut OreList {
    unsafe {
        let layout = std::alloc::Layout::new::<OreList>();
        let list = std::alloc::alloc_zeroed(layout) as *mut OreList;
        (*list).len = 0;
        (*list).cap = 0;
        (*list).data = std::ptr::null_mut();
        (*list).kinds = std::ptr::null_mut();
        list
    }
}

/// Grow both `data` and `kinds` arrays when capacity is exceeded.
unsafe fn list_grow(list: &mut OreList) {
    let new_cap = if list.cap == 0 { 4 } else { list.cap * 2 };
    let new_layout = std::alloc::Layout::array::<i64>(new_cap as usize).unwrap();
    let new_data = if list.data.is_null() {
        std::alloc::alloc(new_layout) as *mut i64
    } else {
        let old_layout = std::alloc::Layout::array::<i64>(list.cap as usize).unwrap();
        std::alloc::realloc(list.data as *mut u8, old_layout, new_layout.size()) as *mut i64
    };
    list.data = new_data;
    // Grow kinds array in parallel if it exists
    if !list.kinds.is_null() {
        let new_k_layout = std::alloc::Layout::array::<i8>(new_cap as usize).unwrap();
        let old_k_layout = std::alloc::Layout::array::<i8>(list.cap as usize).unwrap();
        list.kinds = std::alloc::realloc(list.kinds as *mut u8, old_k_layout, new_k_layout.size()) as *mut i8;
        // Zero-fill new slots (KIND_INT = 0)
        std::ptr::write_bytes(list.kinds.add(list.cap as usize), 0, (new_cap - list.cap) as usize);
    }
    list.cap = new_cap;
}

/// Ensure the `kinds` array is allocated, backfilling existing elements with KIND_INT.
unsafe fn list_ensure_kinds(list: &mut OreList) {
    if list.kinds.is_null() && list.cap > 0 {
        let k_layout = std::alloc::Layout::array::<i8>(list.cap as usize).unwrap();
        list.kinds = std::alloc::alloc_zeroed(k_layout) as *mut i8; // zeroed = KIND_INT
    }
}

#[no_mangle]
pub extern "C" fn ore_list_push(list: *mut OreList, value: i64) {
    unsafe {
        let list = &mut *list;
        if list.len >= list.cap { list_grow(list); }
        *list.data.add(list.len as usize) = value;
        // If kinds array exists, record KIND_INT for the new element
        if !list.kinds.is_null() {
            *list.kinds.add(list.len as usize) = KIND_INT;
        }
        list.len += 1;
    }
}

/// Push a value with an explicit kind tag.  Allocates the kinds array on
/// first non-INT push so that homogeneous int-lists pay no overhead.
#[no_mangle]
pub extern "C" fn ore_list_push_typed(list: *mut OreList, value: i64, kind: i8) {
    unsafe {
        let list = &mut *list;
        if list.len >= list.cap { list_grow(list); }
        *list.data.add(list.len as usize) = value;
        if kind != KIND_INT || !list.kinds.is_null() {
            list_ensure_kinds(list);
            *list.kinds.add(list.len as usize) = kind;
        }
        list.len += 1;
    }
}

/// Return the kind tag for the element at `index`, or KIND_INT when no
/// per-element tags have been recorded.
#[no_mangle]
pub extern "C" fn ore_list_get_kind(list: *mut OreList, index: i64) -> i8 {
    unsafe {
        let list = &*list;
        if list.kinds.is_null() { return KIND_INT; }
        let idx = checked_index(index, list.len);
        *list.kinds.add(idx)
    }
}

/// Remove and return the last element of a list.
#[no_mangle]
pub extern "C" fn ore_list_pop(list: *mut OreList) -> i64 {
    unsafe {
        let list = &mut *list;
        if list.len == 0 { return 0; }
        list.len -= 1;
        *list.data.add(list.len as usize)
    }
}

/// Clear all elements from a list (sets length to 0 but keeps capacity).
#[no_mangle]
pub extern "C" fn ore_list_clear(list: *mut OreList) {
    unsafe {
        (*list).len = 0;
        // kinds array is kept allocated but length governs valid range
    }
}

/// Insert a value at the given index, shifting elements right.
#[no_mangle]
pub extern "C" fn ore_list_insert(list: *mut OreList, index: i64, value: i64) {
    unsafe {
        let list_ref = &mut *list;
        let idx = if index < 0 { (list_ref.len + index).max(0) } else { index.min(list_ref.len) } as usize;
        // Ensure capacity
        ore_list_push(list, 0); // grow if needed
        let list_ref = &mut *list;
        let len = list_ref.len as usize;
        // Shift elements right
        if idx < len - 1 {
            std::ptr::copy(list_ref.data.add(idx), list_ref.data.add(idx + 1), len - 1 - idx);
            if !list_ref.kinds.is_null() {
                std::ptr::copy(list_ref.kinds.add(idx), list_ref.kinds.add(idx + 1), len - 1 - idx);
            }
        }
        *list_ref.data.add(idx) = value;
        if !list_ref.kinds.is_null() {
            *list_ref.kinds.add(idx) = KIND_INT;
        }
    }
}

/// Remove element at the given index, shifting elements left. Returns the removed element.
#[no_mangle]
pub extern "C" fn ore_list_remove_at(list: *mut OreList, index: i64) -> i64 {
    unsafe {
        let list_ref = &mut *list;
        let idx = checked_index(index, list_ref.len);
        let removed = *list_ref.data.add(idx);
        let len = list_ref.len as usize;
        if idx < len - 1 {
            std::ptr::copy(list_ref.data.add(idx + 1), list_ref.data.add(idx), len - 1 - idx);
            if !list_ref.kinds.is_null() {
                std::ptr::copy(list_ref.kinds.add(idx + 1), list_ref.kinds.add(idx), len - 1 - idx);
            }
        }
        list_ref.len -= 1;
        removed
    }
}

#[no_mangle]
pub extern "C" fn ore_list_get(list: *mut OreList, index: i64) -> i64 {
    unsafe {
        let list = &*list;
        let idx = checked_index(index, list.len);
        *list.data.add(idx)
    }
}

#[no_mangle]
pub extern "C" fn ore_list_get_or(list: *mut OreList, index: i64, default: i64) -> i64 {
    unsafe {
        let list = &*list;
        let idx = if index < 0 { list.len + index } else { index };
        if idx < 0 || idx >= list.len { default } else { *list.data.add(idx as usize) }
    }
}

#[no_mangle]
pub extern "C" fn ore_list_len(list: *mut OreList) -> i64 {
    unsafe { (*list).len }
}

unsafe fn list_copy_and_sort(list: *mut OreList, cmp: impl FnMut(&i64, &i64) -> std::cmp::Ordering) -> *mut OreList {
    let src = &*list;
    let new_list = ore_list_new();
    for &val in src.as_slice() {
        ore_list_push(new_list, val);
    }
    let new = &mut *new_list;
    new.as_mut_slice().sort_by(cmp);
    new_list
}

#[no_mangle]
pub extern "C" fn ore_list_sort(list: *mut OreList) -> *mut OreList {
    unsafe { list_copy_and_sort(list, |a, b| a.cmp(b)) }
}

/// Sort a list of strings lexicographically.
#[no_mangle]
pub extern "C" fn ore_list_sort_str(list: *mut OreList) -> *mut OreList {
    unsafe { list_copy_and_sort(list, |a, b| {
        let sa = &*(*a as *mut OreStr);
        let sb = &*(*b as *mut OreStr);
        sa.as_str().cmp(sb.as_str())
    })}
}

/// Sort a list of floats.
#[no_mangle]
pub extern "C" fn ore_list_sort_float(list: *mut OreList) -> *mut OreList {
    unsafe { list_copy_and_sort(list, |a, b| {
        let fa = f64::from_bits(*a as u64);
        let fb = f64::from_bits(*b as u64);
        fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
    })}
}

/// Remove consecutive duplicate elements.
#[no_mangle]
pub extern "C" fn ore_list_dedup(list: *mut OreList) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        let mut prev: Option<i64> = None;
        for &val in src.as_slice() {
            if prev != Some(val) {
                ore_list_push(result, val);
                prev = Some(val);
            }
        }
        result
    }
}

#[no_mangle]
pub extern "C" fn ore_list_sort_by(
    list: *mut OreList,
    cmp: extern "C" fn(i64, i64, *mut u8) -> i64,
    env_ptr: *mut u8,
) -> *mut OreList {
    unsafe { list_copy_and_sort(list, |a, b| {
        let result = cmp(*a, *b, env_ptr);
        if result < 0 { std::cmp::Ordering::Less }
        else if result > 0 { std::cmp::Ordering::Greater }
        else { std::cmp::Ordering::Equal }
    })}
}

unsafe fn list_extremum_by(
    list: *mut OreList,
    key_fn: extern "C" fn(i64, *mut u8) -> i64,
    env_ptr: *mut u8,
    is_better: fn(i64, i64) -> bool,
) -> i64 {
    let slice = (&*list).as_slice();
    if slice.is_empty() { return 0; }
    let mut best_elem = slice[0];
    let mut best_key = key_fn(best_elem, env_ptr);
    for &elem in &slice[1..] {
        let key = key_fn(elem, env_ptr);
        if is_better(key, best_key) {
            best_key = key;
            best_elem = elem;
        }
    }
    best_elem
}

/// Find the element with the minimum key value.
#[no_mangle]
pub extern "C" fn ore_list_min_by(
    list: *mut OreList,
    key_fn: extern "C" fn(i64, *mut u8) -> i64,
    env_ptr: *mut u8,
) -> i64 {
    unsafe { list_extremum_by(list, key_fn, env_ptr, |a, b| a < b) }
}

/// Find the element with the maximum key value.
#[no_mangle]
pub extern "C" fn ore_list_max_by(
    list: *mut OreList,
    key_fn: extern "C" fn(i64, *mut u8) -> i64,
    env_ptr: *mut u8,
) -> i64 {
    unsafe { list_extremum_by(list, key_fn, env_ptr, |a, b| a > b) }
}

/// Sort list in-place by a key function that returns an i64.
/// The key function is called once per element to compute a sort key.
#[no_mangle]
pub extern "C" fn ore_list_sort_by_key(
    list: *mut OreList,
    key_fn: extern "C" fn(i64, *mut u8) -> i64,
    env_ptr: *mut u8,
) -> *mut OreList {
    unsafe {
        let src = &*list;
        let new_list = ore_list_new();
        let mut keyed: Vec<(i64, i64)> = Vec::with_capacity(src.len as usize);
        for &elem in src.as_slice() {
            keyed.push((key_fn(elem, env_ptr), elem));
        }
        keyed.sort_by_key(|&(k, _)| k);
        for (_, elem) in keyed {
            ore_list_push(new_list, elem);
        }
        new_list
    }
}

/// Sort list in-place by a string key function.
#[no_mangle]
pub extern "C" fn ore_list_sort_by_key_str(
    list: *mut OreList,
    key_fn: extern "C" fn(i64, *mut u8) -> *mut OreStr,
    env_ptr: *mut u8,
) -> *mut OreList {
    unsafe {
        let src = &*list;
        let new_list = ore_list_new();
        let mut keyed: Vec<(String, i64)> = Vec::with_capacity(src.len as usize);
        for &elem in src.as_slice() {
            let key_str = key_fn(elem, env_ptr);
            let s = if !key_str.is_null() { (*key_str).as_str().to_string() } else { String::new() };
            keyed.push((s, elem));
        }
        keyed.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, elem) in keyed {
            ore_list_push(new_list, elem);
        }
        new_list
    }
}


#[no_mangle]
pub extern "C" fn ore_list_reverse(list: *mut OreList) {
    unsafe {
        let list = &mut *list;
        list.as_mut_slice().reverse();
    }
}

#[no_mangle]
pub extern "C" fn ore_list_reverse_new(list: *mut OreList) -> *mut OreList {
    let result = ore_list_new();
    if list.is_null() { return result; }
    let l = unsafe { &*list };
    for &val in unsafe { l.as_slice() }.iter().rev() {
        ore_list_push(result, val);
    }
    result
}

#[no_mangle]
pub extern "C" fn ore_list_concat(a: *mut OreList, b: *mut OreList) -> *mut OreList {
    unsafe {
        let a = &*a;
        let b = &*b;
        let result = ore_list_new();
        for &val in a.as_slice() {
            ore_list_push(result, val);
        }
        for &val in b.as_slice() {
            ore_list_push(result, val);
        }
        result
    }
}

#[no_mangle]
pub extern "C" fn ore_list_contains(list: *mut OreList, value: i64) -> i8 {
    unsafe {
        let list = &*list;
        for &val in list.as_slice() {
            if val == value {
                return 1;
            }
        }
        0
    }
}

/// Contains for string lists — compares by string value
#[no_mangle]
pub extern "C" fn ore_list_contains_str(list: *mut OreList, value: *mut OreStr) -> i8 {
    unsafe {
        let list = &*list;
        let target = (*value).as_str();
        for &val in list.as_slice() {
            let ptr = val as *mut OreStr;
            if !ptr.is_null() && (*ptr).as_str() == target {
                return 1;
            }
        }
        0
    }
}

/// Call a closure: if env is null, call as fn(i64)->i64; otherwise fn(env, i64)->i64.
pub(crate) unsafe fn call_closure(func: *const u8, env: *mut u8, arg: i64) -> i64 {
    if env.is_null() {
        let f: extern "C" fn(i64) -> i64 = std::mem::transmute(func);
        f(arg)
    } else {
        let f: extern "C" fn(*mut u8, i64) -> i64 = std::mem::transmute(func);
        f(env, arg)
    }
}

/// Reduce a list with a 2-arg closure: fn(acc, elem) -> acc
/// call_closure2 dispatches based on whether env is null.
pub(crate) unsafe fn call_closure2(func: *const u8, env: *mut u8, a: i64, b: i64) -> i64 {
    if env.is_null() {
        let f: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(func);
        f(a, b)
    } else {
        let f: extern "C" fn(*mut u8, i64, i64) -> i64 = std::mem::transmute(func);
        f(env, a, b)
    }
}

#[no_mangle]
pub extern "C" fn ore_list_map(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        for &val in src.as_slice() {
            ore_list_push(result, call_closure(func, env, val));
        }
        result
    }
}

#[no_mangle]
pub extern "C" fn ore_list_filter(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        for &val in src.as_slice() {
            if call_closure(func, env, val) != 0 {
                ore_list_push(result, val);
            }
        }
        result
    }
}

#[no_mangle]
pub extern "C" fn ore_list_each(list: *mut OreList, func: *const u8, env: *mut u8) {
    unsafe {
        let src = &*list;
        for &val in src.as_slice() {
            call_closure(func, env, val);
        }
    }
}

/// find_index: returns index of first element matching predicate, or -1
#[no_mangle]
pub extern "C" fn ore_list_find_index(list: *mut OreList, func: *const u8, env: *mut u8) -> i64 {
    unsafe {
        for (i, &val) in (&*list).as_slice().iter().enumerate() {
            if call_closure(func, env, val) != 0 {
                return i as i64;
            }
        }
        -1
    }
}

/// fold: left fold with initial accumulator value
#[no_mangle]
pub extern "C" fn ore_list_fold(list: *mut OreList, init: i64, func: *const u8, env: *mut u8) -> i64 {
    unsafe {
        let src = &*list;
        let mut acc = init;
        for &val in src.as_slice() {
            acc = call_closure2(func, env, acc, val);
        }
        acc
    }
}

fn list_extremum_int(list: *mut OreList, is_better: fn(i64, i64) -> bool) -> i64 {
    if list.is_null() { return 0; }
    let slice = unsafe { (&*list).as_slice() };
    if slice.is_empty() { return 0; }
    let mut best = slice[0];
    for &val in &slice[1..] {
        if is_better(val, best) { best = val; }
    }
    best
}

#[no_mangle]
pub extern "C" fn ore_list_min(list: *mut OreList) -> i64 {
    list_extremum_int(list, |a, b| a < b)
}

#[no_mangle]
pub extern "C" fn ore_list_max(list: *mut OreList) -> i64 {
    list_extremum_int(list, |a, b| a > b)
}

unsafe fn list_extremum_str(list: *mut OreList, is_better: fn(&str, &str) -> bool) -> *mut OreStr {
    let slice = (&*list).as_slice();
    if slice.is_empty() { return empty_ore_str(); }
    let mut best_val = slice[0];
    let mut best_str = (&*(best_val as *mut OreStr)).as_str();
    for &val in &slice[1..] {
        let s = (&*(val as *mut OreStr)).as_str();
        if is_better(s, best_str) {
            best_val = val;
            best_str = s;
        }
    }
    best_val as *mut OreStr
}

/// min for string lists (lexicographic comparison)
#[no_mangle]
pub extern "C" fn ore_list_min_str(list: *mut OreList) -> *mut OreStr {
    unsafe { list_extremum_str(list, |a, b| a < b) }
}

/// max for string lists (lexicographic comparison)
#[no_mangle]
pub extern "C" fn ore_list_max_str(list: *mut OreList) -> *mut OreStr {
    unsafe { list_extremum_str(list, |a, b| a > b) }
}

unsafe fn list_extremum_float(list: *mut OreList, is_better: fn(f64, f64) -> bool) -> f64 {
    let slice = (&*list).as_slice();
    if slice.is_empty() { return 0.0; }
    let mut best = f64::from_bits(slice[0] as u64);
    for &val in &slice[1..] {
        let v = f64::from_bits(val as u64);
        if is_better(v, best) { best = v; }
    }
    best
}

#[no_mangle]
pub extern "C" fn ore_list_min_float(list: *mut OreList) -> f64 {
    unsafe { list_extremum_float(list, |a, b| a < b) }
}

#[no_mangle]
pub extern "C" fn ore_list_max_float(list: *mut OreList) -> f64 {
    unsafe { list_extremum_float(list, |a, b| a > b) }
}

unsafe fn int_list_fold(list: *mut OreList, init: i64, op: fn(i64, i64) -> i64) -> i64 {
    let mut acc = init;
    for &val in (&*list).as_slice() { acc = op(acc, val); }
    acc
}

unsafe fn float_list_fold(list: *mut OreList, init: f64, op: fn(f64, f64) -> f64) -> f64 {
    let mut acc = init;
    for &val in (&*list).as_slice() { acc = op(acc, f64::from_bits(val as u64)); }
    acc
}

/// Sum all i64 elements in a list.
#[no_mangle]
pub extern "C" fn ore_list_sum(list: *mut OreList) -> i64 {
    unsafe { int_list_fold(list, 0, |a, b| a + b) }
}

/// Average of integers, returned as float.
#[no_mangle]
pub extern "C" fn ore_list_average(list: *mut OreList) -> f64 {
    unsafe {
        let len = (&*list).len;
        if len == 0 { return 0.0; }
        int_list_fold(list, 0, |a, b| a + b) as f64 / len as f64
    }
}

/// Average of floats.
#[no_mangle]
pub extern "C" fn ore_list_average_float(list: *mut OreList) -> f64 {
    unsafe {
        let len = (&*list).len;
        if len == 0 { return 0.0; }
        float_list_fold(list, 0.0, |a, b| a + b) / len as f64
    }
}

#[no_mangle]
pub extern "C" fn ore_list_sum_float(list: *mut OreList) -> f64 {
    unsafe { float_list_fold(list, 0.0, |a, b| a + b) }
}

#[no_mangle]
pub extern "C" fn ore_list_product_float(list: *mut OreList) -> f64 {
    unsafe { float_list_fold(list, 1.0, |a, b| a * b) }
}

/// Product of all i64 elements in a list.
#[no_mangle]
pub extern "C" fn ore_list_product(list: *mut OreList) -> i64 {
    unsafe { int_list_fold(list, 1, |a, b| a * b) }
}

#[no_mangle]
pub extern "C" fn ore_list_count(
    list: *mut OreList,
    pred: extern "C" fn(i64, *mut u8) -> i8,
    env_ptr: *mut u8,
) -> i64 {
    if list.is_null() { return 0; }
    let l = unsafe { &*list };
    let mut count = 0i64;
    for &val in unsafe { l.as_slice() } {
        if pred(val, env_ptr) != 0 {
            count += 1;
        }
    }
    count
}

#[no_mangle]
pub extern "C" fn ore_list_flatten(list: *mut OreList) -> *mut OreList {
    let result = ore_list_new();
    if list.is_null() { return result; }
    let l = unsafe { &*list };
    for &val in unsafe { l.as_slice() } {
        let inner = val as *mut OreList;
        if !inner.is_null() {
            let inner_l = unsafe { &*inner };
            for &inner_val in unsafe { inner_l.as_slice() } {
                ore_list_push(result, inner_val);
            }
        }
    }
    result
}

#[no_mangle]
pub extern "C" fn ore_list_index_of(list: *mut OreList, value: i64) -> i64 {
    if list.is_null() { return -1; }
    let l = unsafe { &*list };
    for (i, &val) in unsafe { l.as_slice() }.iter().enumerate() {
        if val == value { return i as i64; }
    }
    -1
}

/// index_of for string lists — compares by string value
#[no_mangle]
pub extern "C" fn ore_list_index_of_str(list: *mut OreList, value: *mut OreStr) -> i64 {
    if list.is_null() || value.is_null() { return -1; }
    let l = unsafe { &*list };
    let target = unsafe { (*value).as_str() };
    for (i, &val) in unsafe { l.as_slice() }.iter().enumerate() {
        let ptr = val as *mut OreStr;
        if !ptr.is_null() && unsafe { (*ptr).as_str() } == target {
            return i as i64;
        }
    }
    -1
}

#[no_mangle]
pub extern "C" fn ore_list_unique(list: *mut OreList) -> *mut OreList {
    let result = ore_list_new();
    if list.is_null() { return result; }
    let l = unsafe { &*list };
    let mut seen = HashSet::new();
    for &val in unsafe { l.as_slice() } {
        if seen.insert(val) {
            ore_list_push(result, val);
        }
    }
    result
}

/// unique_by: deduplicate using a key function that returns a string key
#[no_mangle]
pub extern "C" fn ore_list_unique_by(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        let mut seen = HashSet::new();
        for &val in src.as_slice() {
            let key_val = call_closure(func, env, val);
            // Key is expected to be a string pointer
            let key_str = &*(key_val as *mut OreStr);
            let key = key_str.as_str().to_string();
            if seen.insert(key) {
                ore_list_push(result, val);
            }
        }
        result
    }
}

/// unique for string lists (compares by string value, not pointer)
#[no_mangle]
pub extern "C" fn ore_list_unique_str(list: *mut OreList) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        let mut seen = HashSet::new();
        for &val in src.as_slice() {
            let s = &*(val as *mut OreStr);
            let key = s.as_str().to_string();
            if seen.insert(key) {
                ore_list_push(result, val);
            }
        }
        result
    }
}

/// tap: run a side-effect on the list and return it unchanged. Useful for debugging pipelines.
#[no_mangle]
pub extern "C" fn ore_list_tap(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreList {
    unsafe {
        let src = &*list;
        for &val in src.as_slice() {
            call_closure(func, env, val);
        }
    }
    list
}

/// map_with_index: call f(index, element) for each element, collect results
#[no_mangle]
pub extern "C" fn ore_list_map_with_index(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreList {
    unsafe {
        let result = ore_list_new();
        for (i, &val) in (&*list).as_slice().iter().enumerate() {
            ore_list_push(result, call_closure2(func, env, i as i64, val));
        }
        result
    }
}

/// each_with_index: call f(index, element) for each element
#[no_mangle]
pub extern "C" fn ore_list_each_with_index(list: *mut OreList, func: *const u8, env: *mut u8) {
    unsafe {
        for (i, &val) in (&*list).as_slice().iter().enumerate() {
            call_closure2(func, env, i as i64, val);
        }
    }
}

/// Parallel map: applies func to each element in parallel using threads
#[no_mangle]
pub extern "C" fn ore_list_par_map(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreList {
    unsafe {
        let src = &*list;
        if src.len == 0 {
            return ore_list_new();
        }
        let elements: Vec<i64> = src.as_slice().to_vec();
        // Func and env are both Send because they're raw pointers to immutable data
        let func_usize = func as usize;
        let env_usize = env as usize;
        let handles: Vec<_> = elements.into_iter().map(|val| {
            std::thread::spawn(move || {
                let f = func_usize as *const u8;
                let e = env_usize as *mut u8;
                call_closure(f, e, val)
            })
        }).collect();
        let result = ore_list_new();
        for h in handles {
            ore_list_push(result, h.join().unwrap());
        }
        result
    }
}

/// Parallel each: applies func to each element in parallel (no return values)
#[no_mangle]
pub extern "C" fn ore_list_par_each(list: *mut OreList, func: *const u8, env: *mut u8) {
    unsafe {
        let src = &*list;
        let elements: Vec<i64> = src.as_slice().to_vec();
        let func_usize = func as usize;
        let env_usize = env as usize;
        let handles: Vec<_> = elements.into_iter().map(|val| {
            std::thread::spawn(move || {
                let f = func_usize as *const u8;
                let e = env_usize as *mut u8;
                call_closure(f, e, val);
            })
        }).collect();
        for h in handles {
            h.join().unwrap();
        }
    }
}

/// Set a list element by index
#[no_mangle]
pub extern "C" fn ore_list_set(list: *mut OreList, index: i64, value: i64) {
    unsafe {
        let list = &mut *list;
        let idx = checked_index(index, list.len);
        *list.data.add(idx) = value;
    }
}

/// Set a value at the given index with an explicit kind tag.
#[no_mangle]
pub extern "C" fn ore_list_set_typed(list: *mut OreList, index: i64, value: i64, kind: i8) {
    unsafe {
        let list = &mut *list;
        let idx = checked_index(index, list.len);
        *list.data.add(idx) = value;
        if kind != KIND_INT || !list.kinds.is_null() {
            list_ensure_kinds(list);
            *list.kinds.add(idx) = kind;
        }
    }
}

/// Reduce without init — uses first element as initial, starts from index 1.
#[no_mangle]
pub extern "C" fn ore_list_reduce1(list: *mut OreList, func: *const u8, env: *mut u8) -> i64 {
    unsafe {
        let slice = (&*list).as_slice();
        if slice.is_empty() { return 0; }
        let mut acc = slice[0];
        for &val in &slice[1..] {
            acc = call_closure2(func, env, acc, val);
        }
        acc
    }
}

/// Find first element matching predicate. Returns the element, or the default value if not found.
#[no_mangle]
pub extern "C" fn ore_list_find(list: *mut OreList, func: *const u8, env: *mut u8, default: i64) -> i64 {
    unsafe {
        let src = &*list;
        for &val in src.as_slice() {
            if call_closure(func, env, val) != 0 {
                return val;
            }
        }
        default
    }
}

/// Join list elements as strings with a separator.
/// Auto-detects whether elements are OreStr pointers or integers.
/// If a value looks like a valid heap pointer (> 0x1000), it is treated as
/// an OreStr pointer and dereferenced; otherwise it is formatted as an integer.
#[no_mangle]
pub extern "C" fn ore_list_join(list: *mut OreList, sep: *mut OreStr) -> *mut OreStr {
    unsafe {
        let src = &*list;
        let sep_str = (*sep).as_str();
        let mut parts: Vec<String> = Vec::new();

        // Determine strategy from the first element: if it looks like a
        // valid OreStr pointer, join all elements as strings.
        let use_str = src.as_slice().first().map_or(false, |&v| {
            v > 0x1000 && (v as usize) % std::mem::align_of::<u32>() == 0
        });

        if use_str {
            for &val in src.as_slice() {
                let s = val as *mut OreStr;
                if !s.is_null() {
                    parts.push((*s).as_str().to_string());
                }
            }
        } else {
            for &val in src.as_slice() {
                parts.push(format!("{}", val));
            }
        }
        let joined = parts.join(sep_str);
        str_to_ore(joined)
    }
}

/// Join list elements where elements are i64 integers.
#[no_mangle]
pub extern "C" fn ore_list_join_int(list: *mut OreList, sep: *mut OreStr) -> *mut OreStr {
    unsafe {
        let src = &*list;
        let sep_str = (*sep).as_str();
        let parts: Vec<String> = src.as_slice().iter().map(|&v| format!("{}", v)).collect();
        let joined = parts.join(sep_str);
        str_to_ore(joined)
    }
}

/// Join list elements where elements are OreStr pointers.
#[no_mangle]
pub extern "C" fn ore_list_join_str(list: *mut OreList, sep: *mut OreStr) -> *mut OreStr {
    unsafe {
        let src = &*list;
        let sep_str = (*sep).as_str();
        let mut parts: Vec<&str> = Vec::new();
        for &val in src.as_slice() {
            let s = val as *mut OreStr;
            if !s.is_null() {
                parts.push((*s).as_str());
            }
        }
        let joined = parts.join(sep_str);
        str_to_ore(joined)
    }
}

/// Join list elements where elements are f64 stored as i64 bit patterns.
#[no_mangle]
pub extern "C" fn ore_list_join_float(list: *mut OreList, sep: *mut OreStr) -> *mut OreStr {
    unsafe {
        let src = &*list;
        let sep_str = (*sep).as_str();
        let mut parts: Vec<String> = Vec::new();
        for &bits in src.as_slice() {
            parts.push(format_float(f64::from_bits(bits as u64)));
        }
        let joined = parts.join(sep_str);
        str_to_ore(joined)
    }
}

/// Take first n elements from a list.
#[no_mangle]
pub extern "C" fn ore_list_take(list: *mut OreList, n: i64) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        let count = (n.max(0) as usize).min(src.len as usize);
        for &val in &src.as_slice()[..count] {
            ore_list_push(result, val);
        }
        result
    }
}

/// Skip first n elements from a list.
#[no_mangle]
pub extern "C" fn ore_list_skip(list: *mut OreList, n: i64) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        let start = (n.max(0) as usize).min(src.len as usize);
        for &val in &src.as_slice()[start..] {
            ore_list_push(result, val);
        }
        result
    }
}

/// Scan (cumulative reduce): returns list of all intermediate accumulator values.
/// scan(list, init, fn(acc, elem) -> acc) -> [init, fn(init,e0), fn(fn(init,e0),e1), ...]
#[no_mangle]
pub extern "C" fn ore_list_scan(
    list: *mut OreList,
    init: i64,
    func: *const u8,
    env: *mut u8,
) -> *mut OreList {
    let result = ore_list_new();
    unsafe {
        let src = &*list;
        let mut acc = init;
        ore_list_push(result, acc);
        for &elem in src.as_slice() {
            acc = call_closure2(func, env, acc, elem);
            ore_list_push(result, acc);
        }
    }
    result
}

/// Take elements while predicate is true.
#[no_mangle]
pub extern "C" fn ore_list_take_while(
    list: *mut OreList,
    func: *const u8,
    env_ptr: *mut u8,
) -> *mut OreList {
    let result = ore_list_new();
    unsafe {
        let src = &*list;
        for &elem in src.as_slice() {
            if call_closure(func, env_ptr, elem) == 0 {
                break;
            }
            ore_list_push(result, elem);
        }
    }
    result
}

/// Drop elements while predicate is true, return the rest.
#[no_mangle]
pub extern "C" fn ore_list_drop_while(
    list: *mut OreList,
    func: *const u8,
    env_ptr: *mut u8,
) -> *mut OreList {
    let result = ore_list_new();
    unsafe {
        let src = &*list;
        let mut dropping = true;
        for &elem in src.as_slice() {
            if dropping && call_closure(func, env_ptr, elem) != 0 {
                continue;
            }
            dropping = false;
            ore_list_push(result, elem);
        }
    }
    result
}

/// Partition list into two lists: [matching, not_matching].
/// Returns a list containing two inner lists.
#[no_mangle]
pub extern "C" fn ore_list_partition(
    list: *mut OreList,
    func: *const u8,
    env_ptr: *mut u8,
) -> *mut OreList {
    let matching = ore_list_new();
    let not_matching = ore_list_new();
    unsafe {
        let src = &*list;
        for &elem in src.as_slice() {
            let result = call_closure(func, env_ptr, elem);
            if result != 0 {
                ore_list_push(matching, elem);
            } else {
                ore_list_push(not_matching, elem);
            }
        }
    }
    let result = ore_list_new();
    ore_list_push(result, matching as i64);
    ore_list_push(result, not_matching as i64);
    result
}

/// Sliding windows of size n. Returns list of lists.
#[no_mangle]
pub extern "C" fn ore_list_window(list: *mut OreList, n: i64) -> *mut OreList {
    let result = ore_list_new();
    if n <= 0 { return result; }
    let n = n as usize;
    unsafe {
        let src = &*list;
        let slice = src.as_slice();
        if slice.len() < n { return result; }
        for w in slice.windows(n) {
            let window = ore_list_new();
            for &val in w {
                ore_list_push(window, val);
            }
            ore_list_push(result, window as i64);
        }
    }
    result
}

/// Split list into chunks of size n. Returns list of lists.
#[no_mangle]
pub extern "C" fn ore_list_chunks(list: *mut OreList, n: i64) -> *mut OreList {
    let result = ore_list_new();
    if n <= 0 { return result; }
    let n = n as usize;
    unsafe {
        let src = &*list;
        for ch in src.as_slice().chunks(n) {
            let chunk = ore_list_new();
            for &val in ch {
                ore_list_push(chunk, val);
            }
            ore_list_push(result, chunk as i64);
        }
    }
    result
}

/// Flat map: applies func to each element (must return a list), concatenates results.
#[no_mangle]
pub extern "C" fn ore_list_flat_map(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        for &val in src.as_slice() {
            let sub_list = call_closure(func, env, val) as *mut OreList;
            if !sub_list.is_null() {
                let sub = &*sub_list;
                for &sub_val in sub.as_slice() {
                    ore_list_push(result, sub_val);
                }
            }
        }
        result
    }
}

/// Returns 1 (true) if any element satisfies the predicate, 0 otherwise.
#[no_mangle]
pub extern "C" fn ore_list_any(list: *mut OreList, func: *const u8, env: *mut u8) -> i8 {
    unsafe {
        let src = &*list;
        for &val in src.as_slice() {
            if call_closure(func, env, val) != 0 {
                return 1;
            }
        }
        0
    }
}

/// Returns 1 (true) if all elements satisfy the predicate, 0 otherwise.
#[no_mangle]
pub extern "C" fn ore_list_all(list: *mut OreList, func: *const u8, env: *mut u8) -> i8 {
    unsafe {
        let src = &*list;
        for &val in src.as_slice() {
            if call_closure(func, env, val) == 0 {
                return 0;
            }
        }
        1
    }
}

/// Zip two lists with a combiner function: zip_with(other, fn(a,b)->c).
#[no_mangle]
pub extern "C" fn ore_list_zip_with(
    a: *mut OreList,
    b: *mut OreList,
    func: *const u8,
    env: *mut u8,
) -> *mut OreList {
    unsafe {
        let a_ref = &*a;
        let b_ref = &*b;
        let result = ore_list_new();
        let a_slice = a_ref.as_slice();
        let b_slice = b_ref.as_slice();
        for (&av, &bv) in a_slice.iter().zip(b_slice.iter()) {
            let combined = call_closure2(func, env, av, bv);
            ore_list_push(result, combined);
        }
        result
    }
}

/// Zip two lists into a list of [a, b] pairs (as nested lists).
#[no_mangle]
pub extern "C" fn ore_list_zip(a: *mut OreList, b: *mut OreList) -> *mut OreList {
    unsafe {
        let a = &*a;
        let b = &*b;
        let result = ore_list_new();
        for (&av, &bv) in a.as_slice().iter().zip(b.as_slice().iter()) {
            let pair = ore_list_new();
            ore_list_push(pair, av);
            ore_list_push(pair, bv);
            ore_list_push(result, pair as i64);
        }
        result
    }
}

/// Enumerate: returns list of [index, value] pairs.
#[no_mangle]
pub extern "C" fn ore_list_enumerate(list: *mut OreList) -> *mut OreList {
    unsafe {
        let result = ore_list_new();
        for (i, &val) in (&*list).as_slice().iter().enumerate() {
            let pair = ore_list_new();
            ore_list_push(pair, i as i64);
            ore_list_push(pair, val);
            ore_list_push(result, pair as i64);
        }
        result
    }
}

/// Take every nth element from a list.
#[no_mangle]
pub extern "C" fn ore_list_step(list: *mut OreList, n: i64) -> *mut OreList {
    let result = ore_list_new();
    if n <= 0 { return result; }
    unsafe {
        let src = &*list;
        for &val in src.as_slice().iter().step_by(n as usize) {
            ore_list_push(result, val);
        }
    }
    result
}

#[no_mangle]
pub extern "C" fn ore_list_slice(list: *mut OreList, start: i64, end: i64) -> *mut OreList {
    let result = ore_list_new();
    if list.is_null() { return result; }
    let l = unsafe { &*list };
    let len = l.len;
    let s_idx = if start < 0 { (len + start).max(0) } else { start.min(len) };
    let e_idx = if end < 0 { (len + end).max(0) } else { end.min(len) };
    let slice = unsafe { l.as_slice() };
    for &val in &slice[s_idx as usize..e_idx as usize] {
        ore_list_push(result, val);
    }
    result
}

/// Create a list by repeating a value N times.
#[no_mangle]
pub extern "C" fn ore_list_repeat(value: i64, count: i64) -> *mut OreList {
    let list = ore_list_new();
    for _ in 0..count {
        ore_list_push(list, value);
    }
    list
}

/// countBy: apply function to each element, count occurrences of each result (string key).
/// Returns a new OreMap with string keys and int counts.
#[no_mangle]
pub extern "C" fn ore_list_count_by(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreMap {
    unsafe {
        let src = &*list;
        let result = ore_map_new();
        for &val in src.as_slice() {
            let key_val = call_closure(func, env, val);
            let key_str = &*(key_val as *mut OreStr);
            let key = key_str.as_str().to_string();
            let map = &mut *result;
            let count = map.inner.entry(key.clone()).or_insert(0);
            *count += 1;
            map.kinds.insert(key, KIND_INT);
        }
        result
    }
}

/// countBy variant for lambdas that return int/bool values (not string pointers).
/// Converts the i64 return value to a string key.
#[no_mangle]
pub extern "C" fn ore_list_count_by_int(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreMap {
    unsafe {
        let src = &*list;
        let result = ore_map_new();
        for &val in src.as_slice() {
            let key_val = call_closure(func, env, val);
            let key = key_val.to_string();
            let map = &mut *result;
            let count = map.inner.entry(key.clone()).or_insert(0);
            *count += 1;
            map.kinds.insert(key, KIND_INT);
        }
        result
    }
}

/// group_by: apply function to each element, group elements by result (as string key).
/// Returns a new OreMap with string keys and list values.
#[no_mangle]
pub extern "C" fn ore_list_group_by(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreMap {
    unsafe {
        let src = &*list;
        let result = ore_map_new();
        for &val in src.as_slice() {
            let key_val = call_closure(func, env, val);
            let key_str = &*(key_val as *mut OreStr);
            let key = key_str.as_str().to_string();
            let map = &mut *result;
            let list_ptr = map.inner.entry(key.clone()).or_insert_with(|| {
                ore_list_new() as i64
            });
            ore_list_push(*list_ptr as *mut OreList, val);
            map.kinds.insert(key, KIND_LIST);
        }
        result
    }
}

/// Convert a list to a map using a key function. Each element becomes a value,
/// keyed by the result of calling the key function on it.
#[no_mangle]
pub extern "C" fn ore_list_to_map(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreMap {
    unsafe {
        let src = &*list;
        let result = ore_map_new();
        let map = &mut *result;
        for &val in src.as_slice() {
            let key_val = call_closure(func, env, val);
            let key_str = &*(key_val as *mut OreStr);
            let key = key_str.as_str().to_string();
            map.inner.insert(key.clone(), val);
            // We don't know the value kind here, store as 0 (Int) by default
        }
        result
    }
}

/// Count occurrences of each element, returning a Map[Str, Int].
/// Elements are converted to their string representation for grouping.
#[no_mangle]
pub extern "C" fn ore_list_frequencies(list: *mut OreList, elem_kind: i8) -> *mut OreMap {
    unsafe {
        let src = &*list;
        let result = ore_map_new();
        let map = &mut *result;
        for &val in src.as_slice() {
            let key = match elem_kind {
                KIND_INT => format!("{}", val),
                KIND_FLOAT => format_float(f64::from_bits(val as u64)),
                KIND_BOOL => if val != 0 { "true".to_string() } else { "false".to_string() },
                KIND_STR => {
                    let s = &*(val as *mut OreStr);
                    s.as_str().to_string()
                }
                _ => format!("{}", val),
            };
            let count = map.inner.get(&key).copied().unwrap_or(0);
            map.inner.insert(key.clone(), count + 1);
            map.kinds.insert(key, KIND_INT);
        }
        result
    }
}

/// Insert an element between each pair of elements.
#[no_mangle]
pub extern "C" fn ore_list_intersperse(list: *mut OreList, sep: i64) -> *mut OreList {
    unsafe {
        let result = ore_list_new();
        for (i, &val) in (&*list).as_slice().iter().enumerate() {
            if i > 0 {
                ore_list_push(result, sep);
            }
            ore_list_push(result, val);
        }
        result
    }
}
