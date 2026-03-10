use std::io::Write;

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
