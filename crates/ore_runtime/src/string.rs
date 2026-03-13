use crate::*;

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

// ── String parsing ──

#[no_mangle]
pub extern "C" fn ore_str_parse_float(s: *mut OreStr) -> f64 {
    ore_str_to_float(s)
}
