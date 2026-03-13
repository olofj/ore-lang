// All extern "C" functions in this crate take raw pointers from LLVM-generated code.
// They cannot be marked `unsafe` since they're called from C FFI.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod assert;
pub mod concurrency;
pub mod convert;
pub mod io;
pub mod kinds;
pub mod list;
pub mod map;
pub mod math;
pub mod print;
pub mod range;
pub mod string;

// Re-export all public items so `ore_runtime::func_name` paths keep working
// and so submodules can use `use crate::*;` to access cross-module items.
pub use assert::*;
pub use concurrency::*;
pub use convert::*;
pub use io::*;
pub use list::*;
pub use map::*;
pub use math::*;
pub use print::*;
pub use range::*;
pub use string::*;

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
pub(crate) fn str_to_ore(s: impl AsRef<str>) -> *mut OreStr {
    let s = s.as_ref();
    ore_str_new(s.as_ptr(), s.len() as u32)
}

/// Create an empty `OreStr`.
pub(crate) fn empty_ore_str() -> *mut OreStr {
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
