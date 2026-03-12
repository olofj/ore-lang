// All extern "C" functions in this crate take raw pointers from LLVM-generated code.
// They cannot be marked `unsafe` since they're called from C FFI.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::io::Write;
use std::sync::Mutex;

// ── Print primitives ──

#[no_mangle]
pub extern "C" fn ore_print_int(n: i64) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{}", n);
}

#[no_mangle]
pub extern "C" fn ore_print_bool(b: i8) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    if b != 0 {
        let _ = writeln!(handle, "true");
    } else {
        let _ = writeln!(handle, "false");
    }
}

#[no_mangle]
pub extern "C" fn ore_print_float(f: f64) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    // Print without trailing zeros but always show at least one decimal
    if f == f.floor() {
        let _ = writeln!(handle, "{:.1}", f);
    } else {
        let _ = writeln!(handle, "{}", f);
    }
}

// ── No-newline print primitives (for list display) ──

#[no_mangle]
pub extern "C" fn ore_str_print_no_newline(s: *mut OreStr) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    unsafe {
        let _ = write!(handle, "{}", (*s).as_str());
    }
    let _ = handle.flush();
}

#[no_mangle]
pub extern "C" fn ore_print_int_no_newline(n: i64) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(handle, "{}", n);
}

#[no_mangle]
pub extern "C" fn ore_print_float_no_newline(f: f64) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    if f == f.floor() {
        let _ = write!(handle, "{:.1}", f);
    } else {
        let _ = write!(handle, "{}", f);
    }
}

#[no_mangle]
pub extern "C" fn ore_print_bool_no_newline(b: i8) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(handle, "{}", if b != 0 { "true" } else { "false" });
}

// ── RC Strings ──
//
// OreStr layout: [refcount: u32][len: u32][data: u8...]
// Allocated on the heap, reference-counted.

#[repr(C)]
pub struct OreStr {
    pub refcount: std::sync::atomic::AtomicU32,
    pub len: u32,
    // Followed by `len` bytes of UTF-8 data
}

impl OreStr {
    fn data_ptr(&self) -> *const u8 {
        unsafe { (self as *const OreStr).add(1) as *const u8 }
    }

    fn data_ptr_mut(&mut self) -> *mut u8 {
        unsafe { (self as *mut OreStr).add(1) as *mut u8 }
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            let slice = std::slice::from_raw_parts(self.data_ptr(), self.len as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_str_new(ptr: *const u8, len: u32) -> *mut OreStr {
    unsafe {
        let total = std::mem::size_of::<OreStr>() + len as usize;
        let layout = std::alloc::Layout::from_size_align(total, std::mem::align_of::<OreStr>()).unwrap();
        let mem = std::alloc::alloc(layout) as *mut OreStr;
        (*mem).refcount = std::sync::atomic::AtomicU32::new(1);
        (*mem).len = len;
        if len > 0 && !ptr.is_null() {
            std::ptr::copy_nonoverlapping(ptr, (*mem).data_ptr_mut(), len as usize);
        }
        mem
    }
}

/// Convert a Rust string to a heap-allocated `OreStr`.
fn str_to_ore(s: impl AsRef<str>) -> *mut OreStr {
    let s = s.as_ref();
    ore_str_new(s.as_ptr(), s.len() as u32)
}

/// Create an empty `OreStr`.
fn empty_ore_str() -> *mut OreStr {
    ore_str_new(std::ptr::null(), 0)
}

#[no_mangle]
pub extern "C" fn ore_str_retain(s: *mut OreStr) {
    if s.is_null() { return; }
    unsafe {
        (*s).refcount.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[no_mangle]
pub extern "C" fn ore_str_release(s: *mut OreStr) {
    if s.is_null() { return; }
    unsafe {
        let old = (*s).refcount.fetch_sub(1, std::sync::atomic::Ordering::Release);
        if old == 1 {
            std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
            let total = std::mem::size_of::<OreStr>() + (*s).len as usize;
            let layout = std::alloc::Layout::from_size_align(total, std::mem::align_of::<OreStr>()).unwrap();
            std::alloc::dealloc(s as *mut u8, layout);
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_str_concat(a: *mut OreStr, b: *mut OreStr) -> *mut OreStr {
    unsafe {
        let a_len = if a.is_null() { 0 } else { (*a).len };
        let b_len = if b.is_null() { 0 } else { (*b).len };
        let new_len = a_len + b_len;
        let result = ore_str_new(std::ptr::null(), new_len);
        if a_len > 0 {
            std::ptr::copy_nonoverlapping((*a).data_ptr(), (*result).data_ptr_mut(), a_len as usize);
        }
        if b_len > 0 {
            std::ptr::copy_nonoverlapping(
                (*b).data_ptr(),
                (*result).data_ptr_mut().add(a_len as usize),
                b_len as usize,
            );
        }
        result
    }
}

#[no_mangle]
pub extern "C" fn ore_str_print(s: *mut OreStr) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    if s.is_null() {
        let _ = writeln!(handle);
    } else {
        unsafe {
            let _ = writeln!(handle, "{}", (*s).as_str());
        }
    }
}

// ── Stderr printing ──

#[no_mangle]
pub extern "C" fn ore_eprint_str(s: *mut OreStr) {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    if s.is_null() {
        let _ = writeln!(handle);
    } else {
        unsafe {
            let _ = writeln!(handle, "{}", (*s).as_str());
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_eprint_int(n: i64) {
    eprintln!("{}", n);
}

#[no_mangle]
pub extern "C" fn ore_eprint_float(f: f64) {
    let s = format!("{}", f);
    if s.contains('.') {
        eprintln!("{}", s);
    } else {
        eprintln!("{}.0", s);
    }
}

#[no_mangle]
pub extern "C" fn ore_eprint_bool(b: i8) {
    eprintln!("{}", if b != 0 { "true" } else { "false" });
}

#[no_mangle]
pub extern "C" fn ore_int_to_str(n: i64) -> *mut OreStr {
    let s = n.to_string();
    str_to_ore(s)
}

#[no_mangle]
pub extern "C" fn ore_bool_to_str(b: i8) -> *mut OreStr {
    let s = if b != 0 { "true" } else { "false" };
    str_to_ore(s)
}

// ── String utilities ──

#[no_mangle]
pub extern "C" fn ore_float_to_str(f: f64) -> *mut OreStr {
    let s = if f == f.floor() && !f.is_infinite() && !f.is_nan() {
        format!("{:.1}", f)
    } else {
        f.to_string()
    };
    str_to_ore(s)
}

/// Dynamic to_str: converts a payload i64 to string based on a runtime kind tag.
/// Kind tags: 0=Int, 1=Float, 2=Bool, 3=Str, 9=List, 10=Map
#[no_mangle]
pub extern "C" fn ore_dynamic_to_str(payload: i64, kind: i8) -> *mut OreStr {
    match kind {
        0 => ore_int_to_str(payload),
        1 => ore_float_to_str(f64::from_bits(payload as u64)),
        2 => ore_bool_to_str(payload as i8),
        3 => {
            // payload is a pointer to OreStr — retain and return it
            let ptr = payload as *mut OreStr;
            if !ptr.is_null() {
                ore_str_retain(ptr);
                ptr
            } else {
                let s = "None";
                str_to_ore(s)
            }
        }
        _ => {
            let s = format!("<dynamic:{}>", kind);
            str_to_ore(s)
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_str_len(s: *mut OreStr) -> i64 {
    if s.is_null() { return 0; }
    unsafe { (*s).len as i64 }
}

#[no_mangle]
pub extern "C" fn ore_str_eq(a: *mut OreStr, b: *mut OreStr) -> i8 {
    unsafe {
        if a.is_null() && b.is_null() { return 1; }
        if a.is_null() || b.is_null() { return 0; }
        if (*a).as_str() == (*b).as_str() { 1 } else { 0 }
    }
}

/// Compare two strings lexicographically. Returns -1, 0, or 1.
#[no_mangle]
pub extern "C" fn ore_str_cmp(a: *mut OreStr, b: *mut OreStr) -> i64 {
    unsafe {
        if a.is_null() && b.is_null() { return 0; }
        if a.is_null() { return -1; }
        if b.is_null() { return 1; }
        match (*a).as_str().cmp((*b).as_str()) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
}

// ── String methods ──

#[no_mangle]
pub extern "C" fn ore_str_contains(haystack: *mut OreStr, needle: *mut OreStr) -> i8 {
    unsafe {
        if haystack.is_null() || needle.is_null() { return 0; }
        if (*haystack).as_str().contains((*needle).as_str()) { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn ore_str_trim(s: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let trimmed = unsafe { (*s).as_str().trim() };
    str_to_ore(trimmed)
}

/// Capitalize first letter of a string.
#[no_mangle]
pub extern "C" fn ore_str_capitalize(s: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let str_val = unsafe { (*s).as_str() };
    if str_val.is_empty() { return empty_ore_str(); }
    let mut result = String::with_capacity(str_val.len());
    let mut chars = str_val.chars();
    if let Some(first) = chars.next() {
        result.extend(first.to_uppercase());
        result.push_str(chars.as_str());
    }
    str_to_ore(result)
}

/// Get a single character at an index. Returns empty string if out of bounds.
#[no_mangle]
pub extern "C" fn ore_str_char_at(s: *mut OreStr, idx: i64) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let str_val = unsafe { (*s).as_str() };
    let i = if idx < 0 {
        (str_val.len() as i64 + idx) as usize
    } else {
        idx as usize
    };
    if i < str_val.len() {
        let ch = &str_val[i..i+1];
        str_to_ore(ch)
    } else {
        empty_ore_str()
    }
}

/// Get the ASCII/Unicode codepoint of the first character of a string.
#[no_mangle]
pub extern "C" fn ore_ord(s: *mut OreStr) -> i64 {
    if s.is_null() { return 0; }
    let str_val = unsafe { (*s).as_str() };
    str_val.chars().next().map_or(0, |c| c as i64)
}

/// Create a single-character string from a codepoint.
#[no_mangle]
pub extern "C" fn ore_chr(code: i64) -> *mut OreStr {
    if let Some(c) = char::from_u32(code as u32) {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        str_to_ore(s)
    } else {
        empty_ore_str()
    }
}

/// Split a string by newlines, returning a list of lines.
#[no_mangle]
pub extern "C" fn ore_str_lines(s: *mut OreStr) -> *mut OreList {
    let result = ore_list_new();
    if s.is_null() { return result; }
    let str_val = unsafe { (*s).as_str() };
    for line in str_val.lines() {
        let line_str = str_to_ore(line);
        ore_list_push(result, line_str as i64);
    }
    result
}

#[no_mangle]
pub extern "C" fn ore_str_trim_start(s: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let trimmed = unsafe { (*s).as_str().trim_start() };
    str_to_ore(trimmed)
}

#[no_mangle]
pub extern "C" fn ore_str_trim_end(s: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let trimmed = unsafe { (*s).as_str().trim_end() };
    str_to_ore(trimmed)
}

#[no_mangle]
pub extern "C" fn ore_str_split(s: *mut OreStr, delim: *mut OreStr) -> *mut OreList {
    let result = ore_list_new();
    if s.is_null() { return result; }
    let str_val = unsafe { (*s).as_str() };
    let delim_str = if delim.is_null() { " " } else { unsafe { (*delim).as_str() } };
    for part in str_val.split(delim_str) {
        let part_str = str_to_ore(part);
        ore_list_push(result, part_str as i64);
    }
    result
}

#[no_mangle]
pub extern "C" fn ore_str_to_int(s: *mut OreStr) -> i64 {
    if s.is_null() { return 0; }
    let str_val = unsafe { (*s).as_str().trim() };
    str_val.parse::<i64>().unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ore_str_to_float(s: *mut OreStr) -> f64 {
    if s.is_null() { return 0.0; }
    let str_val = unsafe { (*s).as_str().trim() };
    str_val.parse::<f64>().unwrap_or(0.0)
}

// ── Additional String Methods ──

#[no_mangle]
pub extern "C" fn ore_str_replace(s: *mut OreStr, from: *mut OreStr, to: *mut OreStr) -> *mut OreStr {
    if s.is_null() || from.is_null() || to.is_null() {
        return empty_ore_str();
    }
    let result = unsafe { (*s).as_str().replace((*from).as_str(), (*to).as_str()) };
    str_to_ore(result)
}

#[no_mangle]
pub extern "C" fn ore_str_starts_with(s: *mut OreStr, prefix: *mut OreStr) -> i8 {
    unsafe {
        if s.is_null() || prefix.is_null() { return 0; }
        if (*s).as_str().starts_with((*prefix).as_str()) { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn ore_str_ends_with(s: *mut OreStr, suffix: *mut OreStr) -> i8 {
    unsafe {
        if s.is_null() || suffix.is_null() { return 0; }
        if (*s).as_str().ends_with((*suffix).as_str()) { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn ore_str_to_upper(s: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let upper = unsafe { (*s).as_str().to_uppercase() };
    str_to_ore(upper)
}

#[no_mangle]
pub extern "C" fn ore_str_to_lower(s: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let lower = unsafe { (*s).as_str().to_lowercase() };
    str_to_ore(lower)
}

#[no_mangle]
pub extern "C" fn ore_str_reverse(s: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let reversed: String = unsafe { (*s).as_str().chars().rev().collect() };
    str_to_ore(reversed)
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
pub extern "C" fn ore_str_substr(s: *mut OreStr, start: i64, len: i64) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let str_val = unsafe { (*s).as_str() };
    let start = start.max(0) as usize;
    let len = len.max(0) as usize;
    if start >= str_val.len() {
        return empty_ore_str();
    }
    let end = (start + len).min(str_val.len());
    let sub = &str_val[start..end];
    str_to_ore(sub)
}

#[no_mangle]
pub extern "C" fn ore_str_repeat(s: *mut OreStr, n: i64) -> *mut OreStr {
    if s.is_null() || n <= 0 { return empty_ore_str(); }
    let str_val = unsafe { (*s).as_str() };
    let repeated = str_val.repeat(n as usize);
    str_to_ore(repeated)
}

fn pad_str(s: *mut OreStr, width: i64, pad_char: *mut OreStr, left: bool) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let str_val = unsafe { (*s).as_str() };
    let pad = if pad_char.is_null() { " " } else { unsafe { (*pad_char).as_str() } };
    let pad_ch = pad.chars().next().unwrap_or(' ');
    let width = width.max(0) as usize;
    if str_val.len() >= width {
        return str_to_ore(str_val);
    }
    let padding: String = std::iter::repeat_n(pad_ch, width - str_val.len()).collect();
    if left {
        str_to_ore(format!("{}{}", padding, str_val))
    } else {
        str_to_ore(format!("{}{}", str_val, padding))
    }
}

#[no_mangle]
pub extern "C" fn ore_str_pad_left(s: *mut OreStr, width: i64, pad_char: *mut OreStr) -> *mut OreStr {
    pad_str(s, width, pad_char, true)
}

#[no_mangle]
pub extern "C" fn ore_str_pad_right(s: *mut OreStr, width: i64, pad_char: *mut OreStr) -> *mut OreStr {
    pad_str(s, width, pad_char, false)
}

#[no_mangle]
pub extern "C" fn ore_str_chars(s: *mut OreStr) -> *mut OreList {
    let list = ore_list_new();
    if s.is_null() { return list; }
    let str_val = unsafe { (*s).as_str() };
    for ch in str_val.chars() {
        let mut buf = [0u8; 4];
        let ch_str = ch.encode_utf8(&mut buf);
        let ore_ch = str_to_ore(ch_str);
        ore_list_push(list, ore_ch as i64);
    }
    list
}

#[no_mangle]
pub extern "C" fn ore_str_index_of(s: *mut OreStr, needle: *mut OreStr) -> i64 {
    if s.is_null() || needle.is_null() { return -1; }
    let haystack = unsafe { (*s).as_str() };
    let needle_str = unsafe { (*needle).as_str() };
    match haystack.find(needle_str) {
        Some(pos) => pos as i64,
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn ore_str_slice(s: *mut OreStr, start: i64, end: i64) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    let str_val = unsafe { (*s).as_str() };
    let len = str_val.len() as i64;
    let s_idx = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
    let e_idx = if end < 0 { (len + end).max(0) as usize } else { (end as usize).min(len as usize) };
    if s_idx >= e_idx {
        return empty_ore_str();
    }
    let slice = &str_val[s_idx..e_idx];
    str_to_ore(slice)
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

/// Count occurrences of a substring.
#[no_mangle]
pub extern "C" fn ore_str_count(s: *mut OreStr, needle: *mut OreStr) -> i64 {
    if s.is_null() || needle.is_null() { return 0; }
    let haystack = unsafe { (*s).as_str() };
    let needle_str = unsafe { (*needle).as_str() };
    if needle_str.is_empty() { return 0; }
    haystack.matches(needle_str).count() as i64
}

/// Strip a prefix from a string. Returns original if prefix not found.
#[no_mangle]
pub extern "C" fn ore_str_strip_prefix(s: *mut OreStr, prefix: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    if prefix.is_null() { return s; }
    let str_val = unsafe { (*s).as_str() };
    let prefix_str = unsafe { (*prefix).as_str() };
    match str_val.strip_prefix(prefix_str) {
        Some(rest) => str_to_ore(rest),
        None => str_to_ore(str_val),
    }
}

/// Strip a suffix from a string. Returns original if suffix not found.
#[no_mangle]
pub extern "C" fn ore_str_strip_suffix(s: *mut OreStr, suffix: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return empty_ore_str(); }
    if suffix.is_null() { return s; }
    let str_val = unsafe { (*s).as_str() };
    let suffix_str = unsafe { (*suffix).as_str() };
    match str_val.strip_suffix(suffix_str) {
        Some(rest) => str_to_ore(rest),
        None => str_to_ore(str_val),
    }
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

#[no_mangle]
pub extern "C" fn ore_str_split_whitespace(s: *mut OreStr) -> *mut OreList {
    let list = ore_list_new();
    if s.is_null() { return list; }
    let str_val = unsafe { (*s).as_str() };
    for word in str_val.split_whitespace() {
        let ore_word = str_to_ore(word);
        ore_list_push(list, ore_word as i64);
    }
    list
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
    let mut seen = std::collections::HashSet::new();
    for &val in unsafe { l.as_slice() } {
        if seen.insert(val) {
            ore_list_push(result, val);
        }
    }
    result
}

// ── Assert ──

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

// ── I/O ──

#[no_mangle]
pub extern "C" fn ore_readln() -> *mut OreStr {
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    // Strip trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    str_to_ore(line)
}

#[no_mangle]
pub extern "C" fn ore_file_read(path: *mut OreStr) -> *mut OreStr {
    if path.is_null() { return std::ptr::null_mut(); }
    let path_str = unsafe { (*path).as_str() };
    match std::fs::read_to_string(path_str) {
        Ok(contents) => str_to_ore(contents),
        Err(e) => {
            eprintln!("error reading file '{}': {}", path_str, e);
            empty_ore_str()
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_file_read_lines(path: *mut OreStr) -> *mut OreList {
    if path.is_null() { return ore_list_new(); }
    let path_str = unsafe { (*path).as_str() };
    match std::fs::read_to_string(path_str) {
        Ok(contents) => {
            let list = ore_list_new();
            for line in contents.lines() {
                let s = str_to_ore(line);
                ore_list_push(list, s as i64);
            }
            list
        }
        Err(e) => {
            eprintln!("error reading file '{}': {}", path_str, e);
            ore_list_new()
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_file_write(path: *mut OreStr, content: *mut OreStr) -> i8 {
    if path.is_null() { return 0; }
    let path_str = unsafe { (*path).as_str() };
    let content_str = if content.is_null() { "" } else { unsafe { (*content).as_str() } };
    match std::fs::write(path_str, content_str) {
        Ok(()) => 1,
        Err(e) => {
            eprintln!("error writing file '{}': {}", path_str, e);
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_file_exists(path: *mut OreStr) -> i8 {
    if path.is_null() { return 0; }
    let path_str = unsafe { (*path).as_str() };
    if std::path::Path::new(path_str).exists() { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn ore_file_append(path: *mut OreStr, content: *mut OreStr) -> i8 {
    if path.is_null() { return 0; }
    let path_str = unsafe { (*path).as_str() };
    let content_str = if content.is_null() { "" } else { unsafe { (*content).as_str() } };
    use std::io::Write;
    match std::fs::OpenOptions::new().append(true).create(true).open(path_str) {
        Ok(mut f) => {
            match f.write_all(content_str.as_bytes()) {
                Ok(()) => 1,
                Err(e) => { eprintln!("error appending to '{}': {}", path_str, e); 0 }
            }
        }
        Err(e) => { eprintln!("error opening '{}': {}", path_str, e); 0 }
    }
}

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

/// Create a list by repeating a value N times.
#[no_mangle]
pub extern "C" fn ore_list_repeat(value: i64, count: i64) -> *mut OreList {
    let list = ore_list_new();
    for _ in 0..count {
        ore_list_push(list, value);
    }
    list
}

// ── Lists ──
//
// OreList: heap-allocated growable array of i64 values.
// Layout: { len: i64, cap: i64, data: *mut i64 }

#[repr(C)]
pub struct OreList {
    pub len: i64,
    pub cap: i64,
    pub data: *mut i64,
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
        list
    }
}

#[no_mangle]
pub extern "C" fn ore_list_push(list: *mut OreList, value: i64) {
    unsafe {
        let list = &mut *list;
        if list.len >= list.cap {
            let new_cap = if list.cap == 0 { 4 } else { list.cap * 2 };
            let new_layout = std::alloc::Layout::array::<i64>(new_cap as usize).unwrap();
            let new_data = if list.data.is_null() {
                std::alloc::alloc(new_layout) as *mut i64
            } else {
                let old_layout = std::alloc::Layout::array::<i64>(list.cap as usize).unwrap();
                std::alloc::realloc(list.data as *mut u8, old_layout, new_layout.size()) as *mut i64
            };
            list.data = new_data;
            list.cap = new_cap;
        }
        *list.data.add(list.len as usize) = value;
        list.len += 1;
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
        }
        *list_ref.data.add(idx) = value;
    }
}

/// Remove element at the given index, shifting elements left. Returns the removed element.
#[no_mangle]
pub extern "C" fn ore_list_remove_at(list: *mut OreList, index: i64) -> i64 {
    unsafe {
        let list_ref = &mut *list;
        let idx = if index < 0 { list_ref.len + index } else { index };
        if idx < 0 || idx >= list_ref.len {
            eprintln!("index out of bounds: {} (len {})", index, list_ref.len);
            std::process::exit(1);
        }
        let idx = idx as usize;
        let removed = *list_ref.data.add(idx);
        let len = list_ref.len as usize;
        if idx < len - 1 {
            std::ptr::copy(list_ref.data.add(idx + 1), list_ref.data.add(idx), len - 1 - idx);
        }
        list_ref.len -= 1;
        removed
    }
}

#[no_mangle]
pub extern "C" fn ore_list_get(list: *mut OreList, index: i64) -> i64 {
    unsafe {
        let list = &*list;
        let idx = if index < 0 { list.len + index } else { index };
        if idx < 0 || idx >= list.len {
            eprintln!("index out of bounds: {} (len {})", index, list.len);
            std::process::exit(1);
        }
        *list.data.add(idx as usize)
    }
}

#[no_mangle]
pub extern "C" fn ore_list_get_or(list: *mut OreList, index: i64, default: i64) -> i64 {
    unsafe {
        let list = &*list;
        let idx = if index < 0 { list.len + index } else { index };
        if idx < 0 || idx >= list.len {
            default
        } else {
            *list.data.add(idx as usize)
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_list_len(list: *mut OreList) -> i64 {
    unsafe { (*list).len }
}

#[no_mangle]
pub extern "C" fn ore_list_print(list: *mut OreList) {
    ore_list_print_typed(list, 0); // 0 = Int
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

#[no_mangle]
pub extern "C" fn ore_list_print_str(list: *mut OreList) {
    ore_list_print_typed(list, 3); // 3 = Str
}

/// Print a list with typed elements.
/// kind: 0=Int, 1=Float, 2=Bool, 3=Str
#[no_mangle]
pub extern "C" fn ore_list_print_typed(list: *mut OreList, kind: i64) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    unsafe {
        let list = &*list;
        let _ = write!(handle, "[");
        for (i, &val) in list.as_slice().iter().enumerate() {
            if i > 0 {
                let _ = write!(handle, ", ");
            }
            match kind {
                0 => { let _ = write!(handle, "{}", val); }
                1 => { let _ = write!(handle, "{}", format_float(f64::from_bits(val as u64))); }
                2 => { let _ = write!(handle, "{}", if val != 0 { "true" } else { "false" }); }
                3 => {
                    let s = val as *mut OreStr;
                    if !s.is_null() {
                        let _ = write!(handle, "{}", (*s).as_str());
                    }
                }
                _ => { let _ = write!(handle, "{}", val); }
            }
        }
        let _ = writeln!(handle, "]");
    }
}

fn format_float(f: f64) -> String {
    if f == f.floor() {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}

#[no_mangle]
pub extern "C" fn ore_list_print_float(list: *mut OreList) {
    ore_list_print_typed(list, 1); // 1 = Float
}

#[no_mangle]
pub extern "C" fn ore_list_print_bool(list: *mut OreList) {
    ore_list_print_typed(list, 2); // 2 = Bool
}

/// Call a closure: if env is null, call as fn(i64)->i64; otherwise fn(env, i64)->i64.
unsafe fn call_closure(func: *const u8, env: *mut u8, arg: i64) -> i64 {
    if env.is_null() {
        let f: extern "C" fn(i64) -> i64 = std::mem::transmute(func);
        f(arg)
    } else {
        let f: extern "C" fn(*mut u8, i64) -> i64 = std::mem::transmute(func);
        f(env, arg)
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

/// unique_by: deduplicate using a key function that returns a string key
#[no_mangle]
pub extern "C" fn ore_list_unique_by(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        let mut seen = std::collections::HashSet::new();
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
        let mut seen = std::collections::HashSet::new();
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
        let idx = if index < 0 { list.len + index } else { index };
        if idx < 0 || idx >= list.len {
            eprintln!("index out of bounds: {} (len {})", index, list.len);
            std::process::exit(1);
        }
        *list.data.add(idx as usize) = value;
    }
}

/// Reduce a list with a 2-arg closure: fn(acc, elem) -> acc
/// call_closure2 dispatches based on whether env is null.
unsafe fn call_closure2(func: *const u8, env: *mut u8, a: i64, b: i64) -> i64 {
    if env.is_null() {
        let f: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(func);
        f(a, b)
    } else {
        let f: extern "C" fn(*mut u8, i64, i64) -> i64 = std::mem::transmute(func);
        f(env, a, b)
    }
}

#[no_mangle]
pub extern "C" fn ore_list_reduce(list: *mut OreList, init: i64, func: *const u8, env: *mut u8) -> i64 {
    unsafe {
        let src = &*list;
        let mut acc = init;
        for &val in src.as_slice() {
            acc = call_closure2(func, env, acc, val);
        }
        acc
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
#[no_mangle]
pub extern "C" fn ore_list_join(list: *mut OreList, sep: *mut OreStr) -> *mut OreStr {
    unsafe {
        let src = &*list;
        let sep_str = (*sep).as_str();
        let mut parts: Vec<String> = Vec::new();
        for &val in src.as_slice() {
            parts.push(format!("{}", val));
        }
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
            let f = f64::from_bits(bits as u64);
            let s = if f == f.floor() && !f.is_infinite() && !f.is_nan() {
                format!("{:.1}", f)
            } else {
                f.to_string()
            };
            parts.push(s);
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

/// Sum all i64 elements in a list.
#[no_mangle]
pub extern "C" fn ore_list_sum(list: *mut OreList) -> i64 {
    unsafe {
        let src = &*list;
        let mut total: i64 = 0;
        for &val in src.as_slice() {
            total += val;
        }
        total
    }
}

/// Average of integers, returned as float.
#[no_mangle]
pub extern "C" fn ore_list_average(list: *mut OreList) -> f64 {
    unsafe {
        let src = &*list;
        if src.len == 0 { return 0.0; }
        let mut total: i64 = 0;
        for &val in src.as_slice() {
            total += val;
        }
        total as f64 / src.len as f64
    }
}

/// Average of floats.
#[no_mangle]
pub extern "C" fn ore_list_average_float(list: *mut OreList) -> f64 {
    unsafe {
        let src = &*list;
        if src.len == 0 { return 0.0; }
        let mut total: f64 = 0.0;
        for &val in src.as_slice() {
            total += f64::from_bits(val as u64);
        }
        total / src.len as f64
    }
}

#[no_mangle]
pub extern "C" fn ore_list_sum_float(list: *mut OreList) -> f64 {
    unsafe {
        let src = &*list;
        let mut total: f64 = 0.0;
        for &val in src.as_slice() {
            total += f64::from_bits(val as u64);
        }
        total
    }
}

#[no_mangle]
pub extern "C" fn ore_list_product_float(list: *mut OreList) -> f64 {
    unsafe {
        let src = &*list;
        let mut total: f64 = 1.0;
        for &val in src.as_slice() {
            total *= f64::from_bits(val as u64);
        }
        total
    }
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

/// Product of all i64 elements in a list.
#[no_mangle]
pub extern "C" fn ore_list_product(list: *mut OreList) -> i64 {
    unsafe {
        let src = &*list;
        let mut total: i64 = 1;
        for &val in src.as_slice() {
            total *= val;
        }
        total
    }
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

// ── Maps ──

/// OreMap: A string-keyed map storing i64 values (which can be pointers to strings, lists, etc.)
/// Internally uses a Rust HashMap wrapped in a Box.
pub struct OreMap {
    inner: std::collections::HashMap<String, i64>,
    /// Value kind tags for each key (0=Int, 1=Float, 2=Bool, 3=Str, 9=List, 10=Map)
    kinds: std::collections::HashMap<String, i8>,
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
            map.kinds.insert(key, 0); // 0 = Int
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
            map.kinds.insert(key, 0); // 0 = Int
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
            map.kinds.insert(key, 9); // 9 = List
        }
        result
    }
}

#[no_mangle]
pub extern "C" fn ore_map_new() -> *mut OreMap {
    Box::into_raw(Box::new(OreMap {
        inner: std::collections::HashMap::new(),
        kinds: std::collections::HashMap::new(),
    }))
}

/// Set a key-value pair. Key is an OreStr pointer, value is i64.
#[no_mangle]
pub extern "C" fn ore_map_set(map: *mut OreMap, key: *mut OreStr, value: i64) {
    unsafe {
        let map = &mut *map;
        let key_str = (*key).as_str().to_string();
        map.inner.insert(key_str, value);
    }
}

/// Set a key-value pair with an explicit kind tag.
fn ore_map_set_with_kind(map: *mut OreMap, key: &str, value: i64, kind: i8) {
    unsafe {
        let map = &mut *map;
        map.kinds.insert(key.to_string(), kind);
        map.inner.insert(key.to_string(), value);
    }
}

/// Set a key-value pair with a kind tag (exported for codegen).
#[no_mangle]
pub extern "C" fn ore_map_set_typed(map: *mut OreMap, key: *mut OreStr, value: i64, kind: i8) {
    unsafe {
        let map = &mut *map;
        let key_str = (*key).as_str().to_string();
        map.kinds.insert(key_str.clone(), kind);
        map.inner.insert(key_str, value);
    }
}

/// Get a value by key. Returns the value, or 0 if not found.
#[no_mangle]
pub extern "C" fn ore_map_get(map: *mut OreMap, key: *mut OreStr) -> i64 {
    unsafe {
        let map = &*map;
        let key_str = (*key).as_str();
        *map.inner.get(key_str).unwrap_or(&0)
    }
}

/// Get a value by key, or return the default if not found.
#[no_mangle]
pub extern "C" fn ore_map_get_or(map: *mut OreMap, key: *mut OreStr, default: i64) -> i64 {
    unsafe {
        let map = &*map;
        let key_str = (*key).as_str();
        *map.inner.get(key_str).unwrap_or(&default)
    }
}

/// Check if a key exists. Returns 1 if yes, 0 if no.
#[no_mangle]
pub extern "C" fn ore_map_contains(map: *mut OreMap, key: *mut OreStr) -> i8 {
    unsafe {
        let map = &*map;
        let key_str = (*key).as_str();
        if map.inner.contains_key(key_str) { 1 } else { 0 }
    }
}

/// Return the number of entries.
#[no_mangle]
pub extern "C" fn ore_map_len(map: *mut OreMap) -> i64 {
    unsafe { (*map).inner.len() as i64 }
}

/// Remove a key. Returns the removed value, or 0 if not found.
#[no_mangle]
pub extern "C" fn ore_map_remove(map: *mut OreMap, key: *mut OreStr) -> i64 {
    unsafe {
        let map = &mut *map;
        let key_str = (*key).as_str();
        map.inner.remove(key_str).unwrap_or(0)
    }
}

/// Return the keys as an OreList of OreStr pointers (each element is an i64 that is really *mut OreStr).
#[no_mangle]
pub extern "C" fn ore_map_keys(map: *mut OreMap) -> *mut OreList {
    unsafe {
        let map = &*map;
        let list = ore_list_new();
        for key in map.inner.keys() {
            let s = str_to_ore(key);
            ore_list_push(list, s as i64);
        }
        list
    }
}

/// Return the values as an OreList of i64.
#[no_mangle]
pub extern "C" fn ore_map_values(map: *mut OreMap) -> *mut OreList {
    unsafe {
        let map = &*map;
        let list = ore_list_new();
        for &val in map.inner.values() {
            ore_list_push(list, val);
        }
        list
    }
}

fn print_map_with(map: *mut OreMap, fmt_val: impl Fn(i64) -> String) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    unsafe {
        let map = &*map;
        let _ = write!(handle, "{{");
        let mut keys: Vec<&String> = map.inner.keys().collect();
        keys.sort();
        for (i, key) in keys.iter().enumerate() {
            if i > 0 { let _ = write!(handle, ", "); }
            let _ = write!(handle, "{}: {}", key, fmt_val(map.inner[*key]));
        }
        let _ = writeln!(handle, "}}");
    }
}

/// Print a map: {key1: value1, key2: value2, ...} (assumes int values)
#[no_mangle]
pub extern "C" fn ore_map_print(map: *mut OreMap) {
    print_map_with(map, |v| v.to_string());
}

/// Print a map with string values
#[no_mangle]
pub extern "C" fn ore_map_print_str(map: *mut OreMap) {
    print_map_with(map, |v| {
        let s = v as *mut OreStr;
        if s.is_null() { String::new() } else { unsafe { (*s).as_str().to_string() } }
    });
}

/// Merge other map into a copy of self, returning new map
#[no_mangle]
pub extern "C" fn ore_map_merge(a: *mut OreMap, b: *mut OreMap) -> *mut OreMap {
    let result = ore_map_new();
    unsafe {
        let result_map = &mut *result;
        // Copy all entries from a
        for (k, v) in &(*a).inner {
            result_map.inner.insert(k.clone(), *v);
        }
        for (k, v) in &(*a).kinds {
            result_map.kinds.insert(k.clone(), *v);
        }
        // Copy all entries from b (overwriting duplicates)
        for (k, v) in &(*b).inner {
            result_map.inner.insert(k.clone(), *v);
        }
        for (k, v) in &(*b).kinds {
            result_map.kinds.insert(k.clone(), *v);
        }
    }
    result
}

/// Clear all entries from map
#[no_mangle]
pub extern "C" fn ore_map_clear(map: *mut OreMap) {
    unsafe {
        (*map).inner.clear();
    }
}

/// Iterate a map calling lambda(key_ptr_as_i64, value_i64) for each entry.
#[no_mangle]
pub extern "C" fn ore_map_each(
    map: *mut OreMap,
    func: *const u8,
    env_ptr: *mut u8,
) {
    unsafe {
        let map = &*map;
        let mut keys: Vec<&String> = map.inner.keys().collect();
        keys.sort();
        for key in keys {
            let val = map.inner[key];
            let key_str = str_to_ore(key);
            call_closure2(func, env_ptr, key_str as i64, val);
        }
    }
}

/// Map over values: apply lambda(key, value) -> new_value for each entry, return new map.
#[no_mangle]
pub extern "C" fn ore_map_map_values(
    map: *mut OreMap,
    func: *const u8,
    env_ptr: *mut u8,
) -> *mut OreMap {
    let result = ore_map_new();
    unsafe {
        let map = &*map;
        for (key, &val) in &map.inner {
            let key_str = str_to_ore(key);
            let new_val = call_closure2(func, env_ptr, key_str as i64, val);
            let key_str2 = str_to_ore(key);
            ore_map_set(result, key_str2, new_val);
        }
    }
    result
}

/// Filter a map: keep entries where lambda(key, value) returns nonzero.
#[no_mangle]
pub extern "C" fn ore_map_filter(
    map: *mut OreMap,
    func: *const u8,
    env_ptr: *mut u8,
) -> *mut OreMap {
    let result = ore_map_new();
    unsafe {
        let map = &*map;
        let result_map = &mut *result;
        for (key, &val) in &map.inner {
            let key_str = str_to_ore(key);
            if call_closure2(func, env_ptr, key_str as i64, val) != 0 {
                result_map.inner.insert(key.clone(), val);
                if let Some(&kind) = map.kinds.get(key) {
                    result_map.kinds.insert(key.clone(), kind);
                }
            }
        }
    }
    result
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
                0 => format!("{}", val), // Int
                1 => {
                    let f = f64::from_bits(val as u64);
                    if f == f.floor() { format!("{:.1}", f) } else { format!("{}", f) }
                }
                2 => if val != 0 { "true".to_string() } else { "false".to_string() }, // Bool
                3 => { // Str
                    let s = &*(val as *mut OreStr);
                    s.as_str().to_string()
                }
                _ => format!("{}", val),
            };
            let count = map.inner.get(&key).copied().unwrap_or(0);
            map.inner.insert(key.clone(), count + 1);
            map.kinds.insert(key, 0); // 0 = Int
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

/// Return map entries as a list of [key, value] pairs (each pair is an OreList).
#[no_mangle]
pub extern "C" fn ore_map_entries(map: *mut OreMap) -> *mut OreList {
    unsafe {
        let map = &*map;
        let result = ore_list_new();
        let mut keys: Vec<&String> = map.inner.keys().collect();
        keys.sort();
        for key in keys {
            let val = map.inner[key];
            let pair = ore_list_new();
            let key_str = str_to_ore(key);
            ore_list_push(pair, key_str as i64);
            ore_list_push(pair, val);
            ore_list_push(result, pair as i64);
        }
        result
    }
}

// ── Math functions ──

#[no_mangle]
pub extern "C" fn ore_math_sqrt(x: f64) -> f64 { x.sqrt() }
#[no_mangle]
pub extern "C" fn ore_math_sin(x: f64) -> f64 { x.sin() }
#[no_mangle]
pub extern "C" fn ore_math_cos(x: f64) -> f64 { x.cos() }
#[no_mangle]
pub extern "C" fn ore_math_tan(x: f64) -> f64 { x.tan() }
#[no_mangle]
pub extern "C" fn ore_math_log(x: f64) -> f64 { x.ln() }
#[no_mangle]
pub extern "C" fn ore_math_log10(x: f64) -> f64 { x.log10() }
#[no_mangle]
pub extern "C" fn ore_math_exp(x: f64) -> f64 { x.exp() }
#[no_mangle]
pub extern "C" fn ore_math_pow(base: f64, exp: f64) -> f64 { base.powf(exp) }
#[no_mangle]
pub extern "C" fn ore_math_abs(x: f64) -> f64 { x.abs() }
#[no_mangle]
pub extern "C" fn ore_math_floor(x: f64) -> f64 { x.floor() }
#[no_mangle]
pub extern "C" fn ore_math_ceil(x: f64) -> f64 { x.ceil() }
#[no_mangle]
pub extern "C" fn ore_math_round(x: f64) -> f64 { x.round() }
#[no_mangle]
pub extern "C" fn ore_math_pi() -> f64 { std::f64::consts::PI }
#[no_mangle]
pub extern "C" fn ore_math_e() -> f64 { std::f64::consts::E }
#[no_mangle]
pub extern "C" fn ore_math_atan2(y: f64, x: f64) -> f64 { y.atan2(x) }
#[no_mangle]
pub extern "C" fn ore_float_round_to(x: f64, decimals: i64) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (x * factor).round() / factor
}
#[no_mangle]
pub extern "C" fn ore_float_format(x: f64, decimals: i64) -> *mut OreStr {
    let s = format!("{:.prec$}", x, prec = decimals as usize);
    str_to_ore(s)
}

// ── Concurrency ──

static THREADS: Mutex<Vec<std::thread::JoinHandle<()>>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "C" fn ore_spawn(func: extern "C" fn()) {
    let handle = std::thread::spawn(move || func());
    THREADS.lock().unwrap().push(handle);
}

/// Spawn a function that takes a single i64 argument (used for passing channels, etc.)
#[no_mangle]
pub extern "C" fn ore_spawn_with_arg(func: extern "C" fn(i64), arg: i64) {
    let handle = std::thread::spawn(move || func(arg));
    THREADS.lock().unwrap().push(handle);
}

#[no_mangle]
pub extern "C" fn ore_spawn_with_2args(func: extern "C" fn(i64, i64), a: i64, b: i64) {
    let handle = std::thread::spawn(move || func(a, b));
    THREADS.lock().unwrap().push(handle);
}

#[no_mangle]
pub extern "C" fn ore_spawn_with_3args(func: extern "C" fn(i64, i64, i64), a: i64, b: i64, c: i64) {
    let handle = std::thread::spawn(move || func(a, b, c));
    THREADS.lock().unwrap().push(handle);
}

#[no_mangle]
pub extern "C" fn ore_thread_join_all() {
    let mut threads = THREADS.lock().unwrap();
    for handle in threads.drain(..) {
        handle.join().unwrap();
    }
}

#[no_mangle]
pub extern "C" fn ore_sleep(ms: i64) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

// ── Channels ──

use std::sync::mpsc;

pub struct OreChannel {
    sender: mpsc::Sender<i64>,
    receiver: Mutex<mpsc::Receiver<i64>>,
}

#[no_mangle]
pub extern "C" fn ore_channel_new() -> *mut OreChannel {
    let (tx, rx) = mpsc::channel();
    let ch = Box::new(OreChannel {
        sender: tx,
        receiver: Mutex::new(rx),
    });
    Box::into_raw(ch)
}

#[no_mangle]
pub extern "C" fn ore_channel_send(ch: *mut OreChannel, val: i64) {
    let ch = unsafe { &*ch };
    ch.sender.send(val).unwrap();
}

#[no_mangle]
pub extern "C" fn ore_channel_recv(ch: *mut OreChannel) -> i64 {
    let ch = unsafe { &*ch };
    ch.receiver.lock().unwrap().recv().unwrap()
}

// ── Int math ──

#[no_mangle]
pub extern "C" fn ore_int_pow(base: i64, exp: i64) -> i64 {
    if exp < 0 {
        return 0; // integer pow with negative exponent → 0
    }
    (base as i128).pow(exp as u32) as i64
}

// ── String parsing ──

#[no_mangle]
pub extern "C" fn ore_str_parse_int(s: *mut OreStr) -> i64 {
    ore_str_to_int(s)
}

#[no_mangle]
pub extern "C" fn ore_str_parse_float(s: *mut OreStr) -> f64 {
    ore_str_to_float(s)
}

// ── Assert ──

use std::sync::atomic::{AtomicBool, Ordering};

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

#[no_mangle]
pub extern "C" fn ore_assert(cond: i8, msg: *const u8, line: i64) {
    if cond == 0 {
        assert_fail_msg(msg, line, "");
    }
}

// ── Time functions ────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn ore_time_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ore_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ── Process ──────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn ore_exit(code: i64) {
    std::process::exit(code as i32);
}

/// Return command-line arguments as a list of strings.
#[no_mangle]
pub extern "C" fn ore_args() -> *mut OreList {
    let list = ore_list_new();
    for arg in std::env::args() {
        let s = str_to_ore(arg);
        ore_list_push(list, s as i64);
    }
    list
}

/// Execute a shell command and return its stdout as a string.
#[no_mangle]
pub extern "C" fn ore_exec(cmd: *mut OreStr) -> *mut OreStr {
    if cmd.is_null() { return empty_ore_str(); }
    let cmd_str = unsafe { (*cmd).as_str() };
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd_str)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim_end_matches('\n');
            str_to_ore(trimmed)
        }
        Err(e) => {
            eprintln!("exec error: {}", e);
            empty_ore_str()
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_type_of(kind: i8) -> *mut OreStr {
    let name = match kind {
        0 => "Int",
        1 => "Float",
        2 => "Bool",
        3 => "Str",
        4 => "Record",
        5 => "Enum",
        6 => "Option",
        7 => "Result",
        8 => "Result",
        9 => "List",
        10 => "Map",
        11 => "Channel",
        _ => "Unknown",
    };
    str_to_ore(name)
}

// ── Environment ──────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn ore_env_get(key: *mut OreStr) -> *mut OreStr {
    let key_str = unsafe { (*key).as_str() };
    match std::env::var(key_str) {
        Ok(val) => str_to_ore(val),
        Err(_) => empty_ore_str(),
    }
}

#[no_mangle]
pub extern "C" fn ore_env_set(key: *mut OreStr, value: *mut OreStr) {
    let key_str = unsafe { (*key).as_str() };
    let val_str = unsafe { (*value).as_str() };
    unsafe { std::env::set_var(key_str, val_str) };
}

// ── Random ───────────────────────────────────────────────────

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

// ── JSON support ──────────────────────────────────────────────

fn json_value_to_ore(val: &serde_json::Value) -> (i64, i8) {
    match val {
        serde_json::Value::Null => (0, 0),
        serde_json::Value::Bool(b) => (if *b { 1 } else { 0 }, 2),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                (i, 0)
            } else if let Some(f) = n.as_f64() {
                (f.to_bits() as i64, 1)
            } else {
                (0, 0)
            }
        }
        serde_json::Value::String(s) => {
            let ore_s = str_to_ore(s);
            (ore_s as i64, 3)
        }
        serde_json::Value::Array(arr) => {
            let list = ore_list_new();
            for item in arr {
                let (v, _kind) = json_value_to_ore(item);
                ore_list_push(list, v);
            }
            (list as i64, 9)
        }
        serde_json::Value::Object(obj) => {
            let map = ore_map_new();
            for (k, v) in obj {
                let (val, kind) = json_value_to_ore(v);
                ore_map_set_with_kind(map, k, val, kind);
            }
            (map as i64, 10)
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_json_parse(s: *mut OreStr) -> *mut OreMap {
    let json_str = unsafe { (*s).as_str() };
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(serde_json::Value::Object(obj)) => {
            let map = ore_map_new();
            for (k, v) in &obj {
                let (val, kind) = json_value_to_ore(v);
                ore_map_set_with_kind(map, k, val, kind);
            }
            map
        }
        _ => ore_map_new(),
    }
}

fn ore_value_to_json(val: i64, kind: i8) -> serde_json::Value {
    match kind {
        0 => serde_json::Value::Number(serde_json::Number::from(val)),
        1 => {
            let f = f64::from_bits(val as u64);
            serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        2 => serde_json::Value::Bool(val != 0),
        3 => {
            let s = unsafe { &*(val as *mut OreStr) };
            serde_json::Value::String(s.as_str().to_string())
        }
        9 => {
            let list = unsafe { &*(val as *mut OreList) };
            let mut arr = Vec::new();
            for &elem in unsafe { list.as_slice() } {
                arr.push(serde_json::Value::Number(serde_json::Number::from(elem)));
            }
            serde_json::Value::Array(arr)
        }
        10 => {
            let map = unsafe { &*(val as *mut OreMap) };
            let mut obj = serde_json::Map::new();
            for (k, v) in &map.inner {
                let k_kind = map.kinds.get(k).copied().unwrap_or(0);
                obj.insert(k.clone(), ore_value_to_json(*v, k_kind));
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}

#[no_mangle]
pub extern "C" fn ore_json_stringify(map: *mut OreMap) -> *mut OreStr {
    let map = unsafe { &*map };
    let mut obj = serde_json::Map::new();
    for (k, v) in &map.inner {
        let kind = map.kinds.get(k).copied().unwrap_or(0);
        obj.insert(k.clone(), ore_value_to_json(*v, kind));
    }
    let json = serde_json::Value::Object(obj).to_string();
    str_to_ore(json)
}
