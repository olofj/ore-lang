use crate::*;

// ── Primitive to string ──

#[no_mangle]
pub extern "C" fn ore_int_to_str(n: i64) -> *mut OreStr {
    let s = n.to_string();
    str_to_ore(s)
}

#[no_mangle]
pub extern "C" fn ore_float_to_str(f: f64) -> *mut OreStr {
    str_to_ore(crate::print::format_float(f))
}

#[no_mangle]
pub extern "C" fn ore_bool_to_str(b: i8) -> *mut OreStr {
    let s = if b != 0 { "true" } else { "false" };
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

// ── type_of ──

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
        7 | 8 => "Result",
        9 => "List",
        10 => "Map",
        11 => "Channel",
        _ => "Unknown",
    };
    str_to_ore(name)
}

// ── JSON support ──

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
