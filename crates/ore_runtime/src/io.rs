use crate::*;

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

// ── Process ──

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

#[no_mangle]
pub extern "C" fn ore_exit(code: i64) {
    std::process::exit(code as i32);
}

// ── Environment ──

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
