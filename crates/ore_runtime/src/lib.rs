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
        if index < 0 || index >= list.len {
            eprintln!("index out of bounds: {} (len {})", index, list.len);
            std::process::exit(1);
        }
        *list.data.add(index as usize)
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

// ── Maps ──

/// OreMap: A string-keyed map storing i64 values (which can be pointers to strings, lists, etc.)
/// Internally uses a Rust HashMap wrapped in a Box.
pub struct OreMap {
    inner: std::collections::HashMap<String, i64>,
}

#[no_mangle]
pub extern "C" fn ore_map_new() -> *mut OreMap {
    Box::into_raw(Box::new(OreMap {
        inner: std::collections::HashMap::new(),
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

// ── Concurrency ──

static THREADS: Mutex<Vec<std::thread::JoinHandle<()>>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "C" fn ore_spawn(func: extern "C" fn()) {
    let handle = std::thread::spawn(move || func());
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
