use crate::*;
use std::collections::HashMap;
use std::io::Write;

// ── Maps ──

/// OreMap: A string-keyed map storing i64 values (which can be pointers to strings, lists, etc.)
/// Internally uses a Rust HashMap wrapped in a Box.
pub struct OreMap {
    pub(crate) inner: HashMap<String, i64>,
    /// Value kind tags for each key (0=Int, 1=Float, 2=Bool, 3=Str, 9=List, 10=Map)
    pub(crate) kinds: HashMap<String, i8>,
}

#[no_mangle]
pub extern "C" fn ore_map_new() -> *mut OreMap {
    Box::into_raw(Box::new(OreMap {
        inner: HashMap::new(),
        kinds: HashMap::new(),
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
pub fn ore_map_set_with_kind(map: *mut OreMap, key: &str, value: i64, kind: i8) {
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

pub fn print_map_with(map: *mut OreMap, fmt_val: impl Fn(i64) -> String) {
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
            ore_map_set(result, key_str, new_val);
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
