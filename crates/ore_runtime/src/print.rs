use crate::*;
use std::io::Write;

// ── Print primitives (with newline) ──

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
    let _ = writeln!(handle, "{}", format_float(f));
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
    let _ = write!(handle, "{}", format_float(f));
}

#[no_mangle]
pub extern "C" fn ore_print_bool_no_newline(b: i8) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(handle, "{}", if b != 0 { "true" } else { "false" });
}

// ── String printing ──

fn print_ore_str(handle: &mut impl std::io::Write, s: *mut OreStr) {
    if s.is_null() {
        let _ = writeln!(handle);
    } else {
        unsafe { let _ = writeln!(handle, "{}", (*s).as_str()); }
    }
}

#[no_mangle]
pub extern "C" fn ore_str_print(s: *mut OreStr) {
    print_ore_str(&mut std::io::stdout().lock(), s);
}

// ── Stderr printing ──

#[no_mangle]
pub extern "C" fn ore_eprint_str(s: *mut OreStr) {
    print_ore_str(&mut std::io::stderr().lock(), s);
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

// ── format_float helper ──

pub fn format_float(f: f64) -> String {
    if f == f.floor() && !f.is_infinite() && !f.is_nan() {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}

// ── List printing ──

#[no_mangle]
pub extern "C" fn ore_list_print(list: *mut OreList) {
    ore_list_print_typed(list, 0); // 0 = Int
}

#[no_mangle]
pub extern "C" fn ore_list_print_str(list: *mut OreList) {
    ore_list_print_typed(list, 3); // 3 = Str
}

#[no_mangle]
pub extern "C" fn ore_list_print_float(list: *mut OreList) {
    ore_list_print_typed(list, 1); // 1 = Float
}

#[no_mangle]
pub extern "C" fn ore_list_print_bool(list: *mut OreList) {
    ore_list_print_typed(list, 2); // 2 = Bool
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
