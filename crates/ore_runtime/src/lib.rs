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

#[no_mangle]
pub extern "C" fn ore_int_to_str(n: i64) -> *mut OreStr {
    let s = n.to_string();
    ore_str_new(s.as_ptr(), s.len() as u32)
}

#[no_mangle]
pub extern "C" fn ore_bool_to_str(b: i8) -> *mut OreStr {
    let s = if b != 0 { "true" } else { "false" };
    ore_str_new(s.as_ptr(), s.len() as u32)
}

// ── String utilities ──

#[no_mangle]
pub extern "C" fn ore_float_to_str(f: f64) -> *mut OreStr {
    let s = if f == f.floor() && !f.is_infinite() && !f.is_nan() {
        format!("{:.1}", f)
    } else {
        f.to_string()
    };
    ore_str_new(s.as_ptr(), s.len() as u32)
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
                ore_str_new(s.as_ptr(), s.len() as u32)
            }
        }
        _ => {
            let s = format!("<dynamic:{}>", kind);
            ore_str_new(s.as_ptr(), s.len() as u32)
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
    if s.is_null() { return ore_str_new(std::ptr::null(), 0); }
    let trimmed = unsafe { (*s).as_str().trim() };
    ore_str_new(trimmed.as_ptr(), trimmed.len() as u32)
}

#[no_mangle]
pub extern "C" fn ore_str_split(s: *mut OreStr, delim: *mut OreStr) -> *mut OreList {
    let result = ore_list_new();
    if s.is_null() { return result; }
    let str_val = unsafe { (*s).as_str() };
    let delim_str = if delim.is_null() { " " } else { unsafe { (*delim).as_str() } };
    for part in str_val.split(delim_str) {
        let part_str = ore_str_new(part.as_ptr(), part.len() as u32);
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
        return ore_str_new(std::ptr::null(), 0);
    }
    let result = unsafe { (*s).as_str().replace((*from).as_str(), (*to).as_str()) };
    ore_str_new(result.as_ptr(), result.len() as u32)
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
    if s.is_null() { return ore_str_new(std::ptr::null(), 0); }
    let upper = unsafe { (*s).as_str().to_uppercase() };
    ore_str_new(upper.as_ptr(), upper.len() as u32)
}

#[no_mangle]
pub extern "C" fn ore_str_to_lower(s: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return ore_str_new(std::ptr::null(), 0); }
    let lower = unsafe { (*s).as_str().to_lowercase() };
    ore_str_new(lower.as_ptr(), lower.len() as u32)
}

#[no_mangle]
pub extern "C" fn ore_str_reverse(s: *mut OreStr) -> *mut OreStr {
    if s.is_null() { return ore_str_new(std::ptr::null(), 0); }
    let reversed: String = unsafe { (*s).as_str().chars().rev().collect() };
    ore_str_new(reversed.as_ptr(), reversed.len() as u32)
}

#[no_mangle]
pub extern "C" fn ore_list_reverse_new(list: *mut OreList) -> *mut OreList {
    let result = ore_list_new();
    if list.is_null() { return result; }
    let l = unsafe { &*list };
    for i in (0..l.len).rev() {
        let val = unsafe { *l.data.offset(i as isize) };
        ore_list_push(result, val);
    }
    result
}

#[no_mangle]
pub extern "C" fn ore_str_substr(s: *mut OreStr, start: i64, len: i64) -> *mut OreStr {
    if s.is_null() { return ore_str_new(std::ptr::null(), 0); }
    let str_val = unsafe { (*s).as_str() };
    let start = start.max(0) as usize;
    let len = len.max(0) as usize;
    if start >= str_val.len() {
        return ore_str_new(std::ptr::null(), 0);
    }
    let end = (start + len).min(str_val.len());
    let sub = &str_val[start..end];
    ore_str_new(sub.as_ptr(), sub.len() as u32)
}

#[no_mangle]
pub extern "C" fn ore_str_repeat(s: *mut OreStr, n: i64) -> *mut OreStr {
    if s.is_null() || n <= 0 { return ore_str_new(std::ptr::null(), 0); }
    let str_val = unsafe { (*s).as_str() };
    let repeated = str_val.repeat(n as usize);
    ore_str_new(repeated.as_ptr(), repeated.len() as u32)
}

#[no_mangle]
pub extern "C" fn ore_str_chars(s: *mut OreStr) -> *mut OreList {
    let list = ore_list_new();
    if s.is_null() { return list; }
    let str_val = unsafe { (*s).as_str() };
    for ch in str_val.chars() {
        let mut buf = [0u8; 4];
        let ch_str = ch.encode_utf8(&mut buf);
        let ore_ch = ore_str_new(ch_str.as_ptr(), ch_str.len() as u32);
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
    if s.is_null() { return ore_str_new(std::ptr::null(), 0); }
    let str_val = unsafe { (*s).as_str() };
    let len = str_val.len() as i64;
    let s_idx = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
    let e_idx = if end < 0 { (len + end).max(0) as usize } else { (end as usize).min(len as usize) };
    if s_idx >= e_idx {
        return ore_str_new(std::ptr::null(), 0);
    }
    let slice = &str_val[s_idx..e_idx];
    ore_str_new(slice.as_ptr(), slice.len() as u32)
}

#[no_mangle]
pub extern "C" fn ore_list_slice(list: *mut OreList, start: i64, end: i64) -> *mut OreList {
    let result = ore_list_new();
    if list.is_null() { return result; }
    let l = unsafe { &*list };
    let len = l.len;
    let s_idx = if start < 0 { (len + start).max(0) } else { start.min(len) };
    let e_idx = if end < 0 { (len + end).max(0) } else { end.min(len) };
    for i in s_idx..e_idx {
        let val = unsafe { *l.data.offset(i as isize) };
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
        let ore_word = ore_str_new(word.as_ptr(), word.len() as u32);
        ore_list_push(list, ore_word as i64);
    }
    list
}

#[no_mangle]
pub extern "C" fn ore_list_min(list: *mut OreList) -> i64 {
    if list.is_null() { return 0; }
    let l = unsafe { &*list };
    if l.len == 0 { return 0; }
    let mut min = unsafe { *l.data };
    for i in 1..l.len {
        let val = unsafe { *l.data.offset(i as isize) };
        if val < min { min = val; }
    }
    min
}

#[no_mangle]
pub extern "C" fn ore_list_max(list: *mut OreList) -> i64 {
    if list.is_null() { return 0; }
    let l = unsafe { &*list };
    if l.len == 0 { return 0; }
    let mut max = unsafe { *l.data };
    for i in 1..l.len {
        let val = unsafe { *l.data.offset(i as isize) };
        if val > max { max = val; }
    }
    max
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
    for i in 0..l.len {
        let val = unsafe { *l.data.offset(i as isize) };
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
    for i in 0..l.len {
        let inner = unsafe { *l.data.offset(i as isize) } as *mut OreList;
        if !inner.is_null() {
            let inner_l = unsafe { &*inner };
            for j in 0..inner_l.len {
                let val = unsafe { *inner_l.data.offset(j as isize) };
                ore_list_push(result, val);
            }
        }
    }
    result
}

#[no_mangle]
pub extern "C" fn ore_list_index_of(list: *mut OreList, value: i64) -> i64 {
    if list.is_null() { return -1; }
    let l = unsafe { &*list };
    for i in 0..l.len {
        let val = unsafe { *l.data.offset(i as isize) };
        if val == value { return i; }
    }
    -1
}

#[no_mangle]
pub extern "C" fn ore_list_unique(list: *mut OreList) -> *mut OreList {
    let result = ore_list_new();
    if list.is_null() { return result; }
    let l = unsafe { &*list };
    let mut seen = std::collections::HashSet::new();
    for i in 0..l.len {
        let val = unsafe { *l.data.offset(i as isize) };
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
    ore_str_new(line.as_ptr(), line.len() as u32)
}

#[no_mangle]
pub extern "C" fn ore_file_read(path: *mut OreStr) -> *mut OreStr {
    if path.is_null() { return std::ptr::null_mut(); }
    let path_str = unsafe { (*path).as_str() };
    match std::fs::read_to_string(path_str) {
        Ok(contents) => ore_str_new(contents.as_ptr(), contents.len() as u32),
        Err(e) => {
            eprintln!("error reading file '{}': {}", path_str, e);
            ore_str_new(std::ptr::null(), 0)
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
pub extern "C" fn ore_list_len(list: *mut OreList) -> i64 {
    unsafe { (*list).len }
}

#[no_mangle]
pub extern "C" fn ore_list_print(list: *mut OreList) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    unsafe {
        let list = &*list;
        let _ = write!(handle, "[");
        for i in 0..list.len as usize {
            if i > 0 {
                let _ = write!(handle, ", ");
            }
            let _ = write!(handle, "{}", *list.data.add(i));
        }
        let _ = writeln!(handle, "]");
    }
}

#[no_mangle]
pub extern "C" fn ore_list_sort(list: *mut OreList) {
    unsafe {
        let list = &mut *list;
        let slice = std::slice::from_raw_parts_mut(list.data, list.len as usize);
        slice.sort();
    }
}

#[no_mangle]
pub extern "C" fn ore_list_sort_by(
    list: *mut OreList,
    cmp: extern "C" fn(i64, i64, *mut u8) -> i64,
    env_ptr: *mut u8,
) {
    unsafe {
        let list = &mut *list;
        let slice = std::slice::from_raw_parts_mut(list.data, list.len as usize);
        slice.sort_by(|a, b| {
            let result = cmp(*a, *b, env_ptr);
            if result < 0 { std::cmp::Ordering::Less }
            else if result > 0 { std::cmp::Ordering::Greater }
            else { std::cmp::Ordering::Equal }
        });
    }
}

#[no_mangle]
pub extern "C" fn ore_list_reverse(list: *mut OreList) {
    unsafe {
        let list = &mut *list;
        let slice = std::slice::from_raw_parts_mut(list.data, list.len as usize);
        slice.reverse();
    }
}

#[no_mangle]
pub extern "C" fn ore_list_concat(a: *mut OreList, b: *mut OreList) -> *mut OreList {
    unsafe {
        let a = &*a;
        let b = &*b;
        let result = ore_list_new();
        for i in 0..a.len as usize {
            ore_list_push(result, *a.data.add(i));
        }
        for i in 0..b.len as usize {
            ore_list_push(result, *b.data.add(i));
        }
        result
    }
}

#[no_mangle]
pub extern "C" fn ore_list_contains(list: *mut OreList, value: i64) -> i8 {
    unsafe {
        let list = &*list;
        for i in 0..list.len as usize {
            if *list.data.add(i) == value {
                return 1;
            }
        }
        0
    }
}

#[no_mangle]
pub extern "C" fn ore_list_print_str(list: *mut OreList) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    unsafe {
        let list = &*list;
        let _ = write!(handle, "[");
        for i in 0..list.len as usize {
            if i > 0 {
                let _ = write!(handle, ", ");
            }
            let s = *list.data.add(i) as *mut OreStr;
            if !s.is_null() {
                let _ = write!(handle, "{}", (*s).as_str());
            }
        }
        let _ = writeln!(handle, "]");
    }
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
        for i in 0..list.len as usize {
            if i > 0 {
                let _ = write!(handle, ", ");
            }
            let val = *list.data.add(i);
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
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    unsafe {
        let list = &*list;
        let _ = write!(handle, "[");
        for i in 0..list.len as usize {
            if i > 0 { let _ = write!(handle, ", "); }
            let val = *list.data.add(i);
            let _ = write!(handle, "{}", format_float(f64::from_bits(val as u64)));
        }
        let _ = writeln!(handle, "]");
    }
}

#[no_mangle]
pub extern "C" fn ore_list_print_bool(list: *mut OreList) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    unsafe {
        let list = &*list;
        let _ = write!(handle, "[");
        for i in 0..list.len as usize {
            if i > 0 { let _ = write!(handle, ", "); }
            let val = *list.data.add(i);
            let _ = write!(handle, "{}", if val != 0 { "true" } else { "false" });
        }
        let _ = writeln!(handle, "]");
    }
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
        for i in 0..src.len as usize {
            let val = *src.data.add(i);
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
        for i in 0..src.len as usize {
            let val = *src.data.add(i);
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
        for i in 0..src.len as usize {
            let val = *src.data.add(i);
            call_closure(func, env, val);
        }
    }
}

/// Parallel map: applies func to each element in parallel using threads
#[no_mangle]
pub extern "C" fn ore_list_par_map(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreList {
    unsafe {
        let src = &*list;
        let len = src.len as usize;
        if len == 0 {
            return ore_list_new();
        }
        // Collect elements
        let elements: Vec<i64> = (0..len).map(|i| *src.data.add(i)).collect();
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
        let len = src.len as usize;
        let elements: Vec<i64> = (0..len).map(|i| *src.data.add(i)).collect();
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
        if index >= 0 && (index as usize) < list.len as usize {
            *list.data.add(index as usize) = value;
        }
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
        for i in 0..src.len as usize {
            acc = call_closure2(func, env, acc, *src.data.add(i));
        }
        acc
    }
}

/// Find first element matching predicate. Returns the element, or the default value if not found.
#[no_mangle]
pub extern "C" fn ore_list_find(list: *mut OreList, func: *const u8, env: *mut u8, default: i64) -> i64 {
    unsafe {
        let src = &*list;
        for i in 0..src.len as usize {
            let val = *src.data.add(i);
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
        for i in 0..src.len as usize {
            parts.push(format!("{}", *src.data.add(i)));
        }
        let joined = parts.join(sep_str);
        ore_str_new(joined.as_ptr(), joined.len() as u32)
    }
}

/// Join list elements where elements are OreStr pointers.
#[no_mangle]
pub extern "C" fn ore_list_join_str(list: *mut OreList, sep: *mut OreStr) -> *mut OreStr {
    unsafe {
        let src = &*list;
        let sep_str = (*sep).as_str();
        let mut parts: Vec<&str> = Vec::new();
        for i in 0..src.len as usize {
            let s = *src.data.add(i) as *mut OreStr;
            if !s.is_null() {
                parts.push((*s).as_str());
            }
        }
        let joined = parts.join(sep_str);
        ore_str_new(joined.as_ptr(), joined.len() as u32)
    }
}

/// Take first n elements from a list.
#[no_mangle]
pub extern "C" fn ore_list_take(list: *mut OreList, n: i64) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        let count = (n.max(0) as usize).min(src.len as usize);
        for i in 0..count {
            ore_list_push(result, *src.data.add(i));
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
        for i in start..src.len as usize {
            ore_list_push(result, *src.data.add(i));
        }
        result
    }
}

/// Sum all i64 elements in a list.
#[no_mangle]
pub extern "C" fn ore_list_sum(list: *mut OreList) -> i64 {
    unsafe {
        let src = &*list;
        let mut total: i64 = 0;
        for i in 0..src.len as usize {
            total += *src.data.add(i);
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
        for i in 0..src.len as usize {
            let val = *src.data.add(i);
            let sub_list = call_closure(func, env, val) as *mut OreList;
            if !sub_list.is_null() {
                let sub = &*sub_list;
                for j in 0..sub.len as usize {
                    ore_list_push(result, *sub.data.add(j));
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
        for i in 0..src.len as usize {
            let val = *src.data.add(i);
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
        for i in 0..src.len as usize {
            let val = *src.data.add(i);
            if call_closure(func, env, val) == 0 {
                return 0;
            }
        }
        1
    }
}

/// Zip two lists into a list of [a, b] pairs (as nested lists).
#[no_mangle]
pub extern "C" fn ore_list_zip(a: *mut OreList, b: *mut OreList) -> *mut OreList {
    unsafe {
        let a = &*a;
        let b = &*b;
        let result = ore_list_new();
        let min_len = a.len.min(b.len) as usize;
        for i in 0..min_len {
            let pair = ore_list_new();
            ore_list_push(pair, *a.data.add(i));
            ore_list_push(pair, *b.data.add(i));
            ore_list_push(result, pair as i64);
        }
        result
    }
}

/// Enumerate: returns list of [index, value] pairs.
#[no_mangle]
pub extern "C" fn ore_list_enumerate(list: *mut OreList) -> *mut OreList {
    unsafe {
        let src = &*list;
        let result = ore_list_new();
        for i in 0..src.len as usize {
            let pair = ore_list_new();
            ore_list_push(pair, i as i64);
            ore_list_push(pair, *src.data.add(i));
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

/// countBy: apply function to each element, count occurrences of each result (as string key).
/// Returns a new OreMap with string keys and int counts.
#[no_mangle]
pub extern "C" fn ore_list_count_by(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreMap {
    unsafe {
        let src = &*list;
        let result = ore_map_new();
        for i in 0..src.len as usize {
            let val = *src.data.add(i);
            let key_val = call_closure(func, env, val);
            // key_val is a *mut OreStr cast to i64
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

/// group_by: apply function to each element, group elements by result (as string key).
/// Returns a new OreMap with string keys and list values.
#[no_mangle]
pub extern "C" fn ore_list_group_by(list: *mut OreList, func: *const u8, env: *mut u8) -> *mut OreMap {
    unsafe {
        let src = &*list;
        let result = ore_map_new();
        for i in 0..src.len as usize {
            let val = *src.data.add(i);
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

/// Get a value by key. Returns the value, or 0 if not found.
#[no_mangle]
pub extern "C" fn ore_map_get(map: *mut OreMap, key: *mut OreStr) -> i64 {
    unsafe {
        let map = &*map;
        let key_str = (*key).as_str();
        *map.inner.get(key_str).unwrap_or(&0)
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
            let s = ore_str_new(key.as_ptr(), key.len() as u32);
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

/// Print a map: {key1: value1, key2: value2, ...} (assumes int values)
#[no_mangle]
pub extern "C" fn ore_map_print(map: *mut OreMap) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    unsafe {
        let map = &*map;
        let _ = write!(handle, "{{");
        let mut first = true;
        // Sort keys for deterministic output
        let mut keys: Vec<&String> = map.inner.keys().collect();
        keys.sort();
        for key in keys {
            if !first {
                let _ = write!(handle, ", ");
            }
            first = false;
            let val = map.inner[key];
            let _ = write!(handle, "{}: {}", key, val);
        }
        let _ = writeln!(handle, "}}");
    }
}

/// Print a map with string values
#[no_mangle]
pub extern "C" fn ore_map_print_str(map: *mut OreMap) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    unsafe {
        let map = &*map;
        let _ = write!(handle, "{{");
        let mut first = true;
        let mut keys: Vec<&String> = map.inner.keys().collect();
        keys.sort();
        for key in keys {
            if !first {
                let _ = write!(handle, ", ");
            }
            first = false;
            let val = map.inner[key] as *mut OreStr;
            if !val.is_null() {
                let _ = write!(handle, "{}: {}", key, (*val).as_str());
            }
        }
        let _ = writeln!(handle, "}}");
    }
}

/// Merge other map into a copy of self, returning new map
#[no_mangle]
pub extern "C" fn ore_map_merge(a: *mut OreMap, b: *mut OreMap) -> *mut OreMap {
    let result = ore_map_new();
    unsafe {
        // Copy all entries from a
        for (k, v) in &(*a).inner {
            let key_str = ore_str_new(k.as_ptr(), k.len() as u32);
            ore_map_set(result, key_str, *v);
        }
        // Copy all entries from b (overwriting duplicates)
        for (k, v) in &(*b).inner {
            let key_str = ore_str_new(k.as_ptr(), k.len() as u32);
            ore_map_set(result, key_str, *v);
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
    let ore_str = unsafe { &*s };
    let text = ore_str.as_str();
    text.trim().parse::<i64>().unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ore_str_parse_float(s: *mut OreStr) -> f64 {
    let ore_str = unsafe { &*s };
    let text = ore_str.as_str();
    text.trim().parse::<f64>().unwrap_or(0.0)
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

#[no_mangle]
pub extern "C" fn ore_assert_eq_int(left: i64, right: i64, msg: *const u8, line: i64) {
    if left != right {
        let message = unsafe { std::ffi::CStr::from_ptr(msg as *const i8) };
        let msg_str = message.to_str().unwrap_or("(invalid utf8)");
        let full_msg = format!("assertion failed at line {}: {} (left: {}, right: {})", line, msg_str, left, right);
        if ASSERT_TEST_MODE.load(Ordering::SeqCst) {
            eprintln!("    {}", full_msg);
            ASSERT_FAILED.store(true, Ordering::SeqCst);
        } else {
            eprintln!("{}", full_msg);
            std::process::exit(1);
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_assert_eq_float(left: f64, right: f64, msg: *const u8, line: i64) {
    if (left - right).abs() > f64::EPSILON {
        let message = unsafe { std::ffi::CStr::from_ptr(msg as *const i8) };
        let msg_str = message.to_str().unwrap_or("(invalid utf8)");
        let full_msg = format!("assertion failed at line {}: {} (left: {}, right: {})", line, msg_str, left, right);
        if ASSERT_TEST_MODE.load(Ordering::SeqCst) {
            eprintln!("    {}", full_msg);
            ASSERT_FAILED.store(true, Ordering::SeqCst);
        } else {
            eprintln!("{}", full_msg);
            std::process::exit(1);
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_assert_eq_str(left: *mut OreStr, right: *mut OreStr, msg: *const u8, line: i64) {
    let l = unsafe { &*left }.as_str();
    let r = unsafe { &*right }.as_str();
    if l != r {
        let message = unsafe { std::ffi::CStr::from_ptr(msg as *const i8) };
        let msg_str = message.to_str().unwrap_or("(invalid utf8)");
        let full_msg = format!("assertion failed at line {}: {} (left: \"{}\", right: \"{}\")", line, msg_str, l, r);
        if ASSERT_TEST_MODE.load(Ordering::SeqCst) {
            eprintln!("    {}", full_msg);
            ASSERT_FAILED.store(true, Ordering::SeqCst);
        } else {
            eprintln!("{}", full_msg);
            std::process::exit(1);
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_assert_ne_int(left: i64, right: i64, msg: *const u8, line: i64) {
    if left == right {
        let message = unsafe { std::ffi::CStr::from_ptr(msg as *const i8) };
        let msg_str = message.to_str().unwrap_or("(invalid utf8)");
        let full_msg = format!("assertion failed at line {}: {} (both values: {})", line, msg_str, left);
        if ASSERT_TEST_MODE.load(Ordering::SeqCst) {
            eprintln!("    {}", full_msg);
            ASSERT_FAILED.store(true, Ordering::SeqCst);
        } else {
            eprintln!("{}", full_msg);
            std::process::exit(1);
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_assert_ne_str(left: *mut OreStr, right: *mut OreStr, msg: *const u8, line: i64) {
    let l = unsafe { &*left }.as_str();
    let r = unsafe { &*right }.as_str();
    if l == r {
        let message = unsafe { std::ffi::CStr::from_ptr(msg as *const i8) };
        let msg_str = message.to_str().unwrap_or("(invalid utf8)");
        let full_msg = format!("assertion failed at line {}: {} (both values: \"{}\")", line, msg_str, l);
        if ASSERT_TEST_MODE.load(Ordering::SeqCst) {
            eprintln!("    {}", full_msg);
            ASSERT_FAILED.store(true, Ordering::SeqCst);
        } else {
            eprintln!("{}", full_msg);
            std::process::exit(1);
        }
    }
}

#[no_mangle]
pub extern "C" fn ore_assert(cond: i8, msg: *const u8, line: i64) {
    if cond == 0 {
        let message = unsafe { std::ffi::CStr::from_ptr(msg as *const i8) };
        let msg_str = message.to_str().unwrap_or("(invalid utf8)");
        let full_msg = format!("assertion failed at line {}: {}", line, msg_str);
        if ASSERT_TEST_MODE.load(Ordering::SeqCst) {
            eprintln!("    {}", full_msg);
            ASSERT_FAILED.store(true, Ordering::SeqCst);
        } else {
            eprintln!("{}", full_msg);
            std::process::exit(1);
        }
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
    ore_str_new(name.as_ptr(), name.len() as u32)
}

// ── Environment ──────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn ore_env_get(key: *mut OreStr) -> *mut OreStr {
    let key_str = unsafe { (*key).as_str() };
    match std::env::var(key_str) {
        Ok(val) => ore_str_new(val.as_ptr(), val.len() as u32),
        Err(_) => ore_str_new(std::ptr::null(), 0),
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
    static RNG_STATE: Cell<u64> = Cell::new(0);
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
            let ore_s = ore_str_new(s.as_ptr(), s.len() as u32);
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
            for i in 0..list.len {
                let elem = unsafe { *list.data.offset(i as isize) };
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
    ore_str_new(json.as_ptr(), json.len() as u32)
}
