#[cfg(test)]
mod ore_str_tests {
    use crate::*;

    /// Helper: create an OreStr from a Rust string and return the raw pointer.
    fn mk(s: &str) -> *mut OreStr {
        str_to_ore(s)
    }

    /// Helper: read back the Rust &str from an OreStr pointer.
    unsafe fn rd(p: *mut OreStr) -> &'static str {
        (*p).as_str()
    }

    #[test]
    fn ore_str_new_empty() {
        let s = ore_str_new(std::ptr::null(), 0);
        assert!(!s.is_null());
        unsafe {
            assert_eq!((*s).len, 0);
            assert_eq!((*s).as_str(), "");
        }
    }

    #[test]
    fn ore_str_new_hello() {
        let data = b"hello";
        let s = ore_str_new(data.as_ptr(), 5);
        unsafe {
            assert_eq!((*s).len, 5);
            assert_eq!((*s).as_str(), "hello");
        }
    }

    #[test]
    fn str_to_ore_roundtrip() {
        let s = mk("world");
        unsafe { assert_eq!(rd(s), "world"); }
    }

    #[test]
    fn empty_ore_str_is_empty() {
        let s = empty_ore_str();
        unsafe { assert_eq!(rd(s), ""); }
    }

    #[test]
    fn retain_release_basic() {
        let s = mk("refcount");
        ore_str_retain(s);
        unsafe {
            assert_eq!(
                (*s).refcount.load(std::sync::atomic::Ordering::Relaxed),
                2
            );
        }
        ore_str_release(s);
        unsafe {
            assert_eq!(
                (*s).refcount.load(std::sync::atomic::Ordering::Relaxed),
                1
            );
        }
    }

    #[test]
    fn retain_release_null_safe() {
        ore_str_retain(std::ptr::null_mut());
        ore_str_release(std::ptr::null_mut());
    }

    #[test]
    fn concat_basic() {
        let a = mk("foo");
        let b = mk("bar");
        let c = ore_str_concat(a, b);
        unsafe { assert_eq!(rd(c), "foobar"); }
    }

    #[test]
    fn concat_with_null() {
        let a = mk("hi");
        let c = ore_str_concat(a, std::ptr::null_mut());
        unsafe { assert_eq!(rd(c), "hi"); }

        let d = ore_str_concat(std::ptr::null_mut(), a);
        unsafe { assert_eq!(rd(d), "hi"); }
    }

    #[test]
    fn concat_both_empty() {
        let a = empty_ore_str();
        let b = empty_ore_str();
        let c = ore_str_concat(a, b);
        unsafe { assert_eq!(rd(c), ""); }
    }
}

#[cfg(test)]
mod string_ops_tests {
    use crate::*;
    fn mk(s: &str) -> *mut OreStr { str_to_ore(s) }
    unsafe fn rd(p: *mut OreStr) -> &'static str { (*p).as_str() }

    #[test]
    fn str_len() {
        assert_eq!(ore_str_len(mk("hello")), 5);
        assert_eq!(ore_str_len(mk("")), 0);
        assert_eq!(ore_str_len(std::ptr::null_mut()), 0);
    }

    #[test]
    fn str_eq() {
        assert_eq!(ore_str_eq(mk("a"), mk("a")), 1);
        assert_eq!(ore_str_eq(mk("a"), mk("b")), 0);
        assert_eq!(ore_str_eq(std::ptr::null_mut(), std::ptr::null_mut()), 1);
        assert_eq!(ore_str_eq(mk("a"), std::ptr::null_mut()), 0);
    }

    #[test]
    fn str_cmp_ordering() {
        assert_eq!(ore_str_cmp(mk("a"), mk("b")), -1);
        assert_eq!(ore_str_cmp(mk("b"), mk("a")), 1);
        assert_eq!(ore_str_cmp(mk("x"), mk("x")), 0);
        assert_eq!(ore_str_cmp(std::ptr::null_mut(), mk("a")), -1);
        assert_eq!(ore_str_cmp(mk("a"), std::ptr::null_mut()), 1);
        assert_eq!(ore_str_cmp(std::ptr::null_mut(), std::ptr::null_mut()), 0);
    }

    #[test]
    fn str_contains() {
        assert_eq!(ore_str_contains(mk("hello world"), mk("world")), 1);
        assert_eq!(ore_str_contains(mk("hello"), mk("xyz")), 0);
        assert_eq!(ore_str_contains(std::ptr::null_mut(), mk("x")), 0);
    }

    #[test]
    fn str_trim() {
        unsafe {
            assert_eq!(rd(ore_str_trim(mk("  hi  "))), "hi");
            assert_eq!(rd(ore_str_trim(mk("no_space"))), "no_space");
            assert_eq!(rd(ore_str_trim(std::ptr::null_mut())), "");
        }
    }

    #[test]
    fn str_trim_start_end() {
        unsafe {
            assert_eq!(rd(ore_str_trim_start(mk("  hi  "))), "hi  ");
            assert_eq!(rd(ore_str_trim_end(mk("  hi  "))), "  hi");
        }
    }

    #[test]
    fn str_capitalize() {
        unsafe {
            assert_eq!(rd(ore_str_capitalize(mk("hello"))), "Hello");
            assert_eq!(rd(ore_str_capitalize(mk(""))), "");
            assert_eq!(rd(ore_str_capitalize(mk("A"))), "A");
            assert_eq!(rd(ore_str_capitalize(std::ptr::null_mut())), "");
        }
    }

    #[test]
    fn str_char_at() {
        unsafe {
            assert_eq!(rd(ore_str_char_at(mk("abc"), 0)), "a");
            assert_eq!(rd(ore_str_char_at(mk("abc"), 2)), "c");
            assert_eq!(rd(ore_str_char_at(mk("abc"), -1)), "c");
            assert_eq!(rd(ore_str_char_at(mk("abc"), 99)), "");
        }
    }

    #[test]
    fn str_ord_chr() {
        assert_eq!(ore_ord(mk("A")), 65);
        assert_eq!(ore_ord(mk("")), 0);
        unsafe { assert_eq!(rd(ore_chr(65)), "A"); }
        unsafe { assert_eq!(rd(ore_chr(0x1F600)), "\u{1F600}"); }
    }

    #[test]
    fn str_lines() {
        let lines = ore_str_lines(mk("a\nb\nc"));
        unsafe {
            assert_eq!(ore_list_len(lines), 3);
            assert_eq!(rd(ore_list_get(lines, 0) as *mut OreStr), "a");
            assert_eq!(rd(ore_list_get(lines, 1) as *mut OreStr), "b");
            assert_eq!(rd(ore_list_get(lines, 2) as *mut OreStr), "c");
        }
    }

    #[test]
    fn str_split() {
        let parts = ore_str_split(mk("a,b,c"), mk(","));
        unsafe {
            assert_eq!(ore_list_len(parts), 3);
            assert_eq!(rd(ore_list_get(parts, 0) as *mut OreStr), "a");
            assert_eq!(rd(ore_list_get(parts, 2) as *mut OreStr), "c");
        }
    }

    #[test]
    fn str_split_whitespace() {
        let parts = ore_str_split_whitespace(mk("  one   two  three  "));
        unsafe {
            assert_eq!(ore_list_len(parts), 3);
            assert_eq!(rd(ore_list_get(parts, 0) as *mut OreStr), "one");
        }
    }

    #[test]
    fn str_to_int() {
        assert_eq!(ore_str_to_int(mk("42")), 42);
        assert_eq!(ore_str_to_int(mk("-7")), -7);
        assert_eq!(ore_str_to_int(mk("bad")), 0);
        assert_eq!(ore_str_to_int(mk("  99  ")), 99);
    }

    #[test]
    fn str_to_float() {
        assert!((ore_str_to_float(mk("3.14")) - 3.14).abs() < 1e-10);
        assert_eq!(ore_str_to_float(mk("bad")), 0.0);
    }

    #[test]
    fn str_replace() {
        unsafe {
            assert_eq!(rd(ore_str_replace(mk("hello world"), mk("world"), mk("rust"))), "hello rust");
            assert_eq!(rd(ore_str_replace(mk("aaa"), mk("a"), mk("bb"))), "bbbbbb");
        }
    }

    #[test]
    fn str_starts_ends_with() {
        assert_eq!(ore_str_starts_with(mk("hello"), mk("hel")), 1);
        assert_eq!(ore_str_starts_with(mk("hello"), mk("xyz")), 0);
        assert_eq!(ore_str_ends_with(mk("hello"), mk("llo")), 1);
        assert_eq!(ore_str_ends_with(mk("hello"), mk("xyz")), 0);
    }

    #[test]
    fn str_to_upper_lower() {
        unsafe {
            assert_eq!(rd(ore_str_to_upper(mk("hello"))), "HELLO");
            assert_eq!(rd(ore_str_to_lower(mk("HELLO"))), "hello");
        }
    }

    #[test]
    fn str_reverse() {
        unsafe {
            assert_eq!(rd(ore_str_reverse(mk("abc"))), "cba");
            assert_eq!(rd(ore_str_reverse(mk(""))), "");
        }
    }

    #[test]
    fn str_substr() {
        unsafe {
            assert_eq!(rd(ore_str_substr(mk("hello world"), 6, 5)), "world");
            assert_eq!(rd(ore_str_substr(mk("abc"), 0, 100)), "abc");
            assert_eq!(rd(ore_str_substr(mk("abc"), 99, 1)), "");
        }
    }

    #[test]
    fn str_repeat() {
        unsafe {
            assert_eq!(rd(ore_str_repeat(mk("ab"), 3)), "ababab");
            assert_eq!(rd(ore_str_repeat(mk("x"), 0)), "");
            assert_eq!(rd(ore_str_repeat(std::ptr::null_mut(), 5)), "");
        }
    }

    #[test]
    fn str_pad_left_right() {
        unsafe {
            assert_eq!(rd(ore_str_pad_left(mk("hi"), 5, mk("0"))), "000hi");
            assert_eq!(rd(ore_str_pad_right(mk("hi"), 5, mk("0"))), "hi000");
            // Already wider than target
            assert_eq!(rd(ore_str_pad_left(mk("hello"), 3, mk("0"))), "hello");
        }
    }

    #[test]
    fn str_chars() {
        let list = ore_str_chars(mk("abc"));
        unsafe {
            assert_eq!(ore_list_len(list), 3);
            assert_eq!(rd(ore_list_get(list, 0) as *mut OreStr), "a");
            assert_eq!(rd(ore_list_get(list, 1) as *mut OreStr), "b");
            assert_eq!(rd(ore_list_get(list, 2) as *mut OreStr), "c");
        }
    }

    #[test]
    fn str_index_of() {
        assert_eq!(ore_str_index_of(mk("hello"), mk("ll")), 2);
        assert_eq!(ore_str_index_of(mk("hello"), mk("xyz")), -1);
        assert_eq!(ore_str_index_of(std::ptr::null_mut(), mk("x")), -1);
    }

    #[test]
    fn str_slice() {
        unsafe {
            assert_eq!(rd(ore_str_slice(mk("hello world"), 0, 5)), "hello");
            assert_eq!(rd(ore_str_slice(mk("hello"), -3, 5)), "llo");
            assert_eq!(rd(ore_str_slice(mk("abc"), 2, 1)), ""); // start >= end
        }
    }

    #[test]
    fn str_count() {
        assert_eq!(ore_str_count(mk("banana"), mk("an")), 2);
        assert_eq!(ore_str_count(mk("hello"), mk("xyz")), 0);
        assert_eq!(ore_str_count(mk("aaa"), mk("")), 0);
    }

    #[test]
    fn str_strip_prefix_suffix() {
        unsafe {
            assert_eq!(rd(ore_str_strip_prefix(mk("hello"), mk("hel"))), "lo");
            assert_eq!(rd(ore_str_strip_prefix(mk("hello"), mk("xyz"))), "hello");
            assert_eq!(rd(ore_str_strip_suffix(mk("hello"), mk("llo"))), "he");
            assert_eq!(rd(ore_str_strip_suffix(mk("hello"), mk("xyz"))), "hello");
        }
    }
}

#[cfg(test)]
mod list_tests {
    use crate::*;
    fn mk_str(s: &str) -> *mut OreStr { str_to_ore(s) }
    unsafe fn rd(p: *mut OreStr) -> &'static str { (*p).as_str() }

    /// Build a list from a slice of i64.
    fn mk_list(vals: &[i64]) -> *mut OreList {
        let list = ore_list_new();
        for &v in vals {
            ore_list_push(list, v);
        }
        list
    }

    /// Build a list of OreStr pointers from string slices.
    fn mk_str_list(vals: &[&str]) -> *mut OreList {
        let list = ore_list_new();
        for &s in vals {
            let ore_s = str_to_ore(s);
            ore_list_push(list, ore_s as i64);
        }
        list
    }

    /// Read list back as a Vec<i64>.
    unsafe fn to_vec(list: *mut OreList) -> Vec<i64> {
        (&*list).as_slice().to_vec()
    }

    #[test]
    fn new_list_is_empty() {
        let list = ore_list_new();
        assert_eq!(ore_list_len(list), 0);
    }

    #[test]
    fn push_and_get() {
        let list = ore_list_new();
        ore_list_push(list, 10);
        ore_list_push(list, 20);
        ore_list_push(list, 30);
        assert_eq!(ore_list_len(list), 3);
        assert_eq!(ore_list_get(list, 0), 10);
        assert_eq!(ore_list_get(list, 1), 20);
        assert_eq!(ore_list_get(list, 2), 30);
    }

    #[test]
    fn get_negative_index() {
        let list = mk_list(&[10, 20, 30]);
        assert_eq!(ore_list_get(list, -1), 30);
        assert_eq!(ore_list_get(list, -2), 20);
        assert_eq!(ore_list_get(list, -3), 10);
    }

    #[test]
    fn get_or_default() {
        let list = mk_list(&[10, 20]);
        assert_eq!(ore_list_get_or(list, 0, -1), 10);
        assert_eq!(ore_list_get_or(list, 99, -1), -1);
        assert_eq!(ore_list_get_or(list, -1, -1), 20);
        assert_eq!(ore_list_get_or(list, -99, -1), -1);
    }

    #[test]
    fn pop() {
        let list = mk_list(&[1, 2, 3]);
        assert_eq!(ore_list_pop(list), 3);
        assert_eq!(ore_list_len(list), 2);
        assert_eq!(ore_list_pop(list), 2);
        assert_eq!(ore_list_pop(list), 1);
        assert_eq!(ore_list_pop(list), 0); // empty list returns 0
    }

    #[test]
    fn clear() {
        let list = mk_list(&[1, 2, 3]);
        ore_list_clear(list);
        assert_eq!(ore_list_len(list), 0);
    }

    #[test]
    fn insert() {
        let list = mk_list(&[1, 3]);
        ore_list_insert(list, 1, 2);
        unsafe { assert_eq!(to_vec(list), vec![1, 2, 3]); }
    }

    #[test]
    fn insert_at_beginning() {
        let list = mk_list(&[2, 3]);
        ore_list_insert(list, 0, 1);
        unsafe { assert_eq!(to_vec(list), vec![1, 2, 3]); }
    }

    #[test]
    fn insert_at_end() {
        let list = mk_list(&[1, 2]);
        ore_list_insert(list, 2, 3);
        unsafe { assert_eq!(to_vec(list), vec![1, 2, 3]); }
    }

    #[test]
    fn set() {
        let list = mk_list(&[1, 2, 3]);
        ore_list_set(list, 1, 99);
        assert_eq!(ore_list_get(list, 1), 99);
    }

    #[test]
    fn sort() {
        let list = mk_list(&[3, 1, 2]);
        let sorted = ore_list_sort(list);
        unsafe { assert_eq!(to_vec(sorted), vec![1, 2, 3]); }
        // Original unchanged
        unsafe { assert_eq!(to_vec(list), vec![3, 1, 2]); }
    }

    #[test]
    fn sort_str() {
        let list = mk_str_list(&["cherry", "apple", "banana"]);
        let sorted = ore_list_sort_str(list);
        unsafe {
            assert_eq!(rd(ore_list_get(sorted, 0) as *mut OreStr), "apple");
            assert_eq!(rd(ore_list_get(sorted, 1) as *mut OreStr), "banana");
            assert_eq!(rd(ore_list_get(sorted, 2) as *mut OreStr), "cherry");
        }
    }

    #[test]
    fn sort_float() {
        let list = ore_list_new();
        ore_list_push(list, 3.0_f64.to_bits() as i64);
        ore_list_push(list, 1.0_f64.to_bits() as i64);
        ore_list_push(list, 2.0_f64.to_bits() as i64);
        let sorted = ore_list_sort_float(list);
        unsafe {
            let slice = (&*sorted).as_slice();
            assert_eq!(f64::from_bits(slice[0] as u64), 1.0);
            assert_eq!(f64::from_bits(slice[1] as u64), 2.0);
            assert_eq!(f64::from_bits(slice[2] as u64), 3.0);
        }
    }

    #[test]
    fn dedup() {
        let list = mk_list(&[1, 1, 2, 2, 3, 3, 3]);
        let deduped = ore_list_dedup(list);
        unsafe { assert_eq!(to_vec(deduped), vec![1, 2, 3]); }
    }

    #[test]
    fn dedup_non_consecutive() {
        let list = mk_list(&[1, 2, 1, 2]);
        let deduped = ore_list_dedup(list);
        unsafe { assert_eq!(to_vec(deduped), vec![1, 2, 1, 2]); }
    }

    #[test]
    fn reverse() {
        let list = mk_list(&[1, 2, 3]);
        ore_list_reverse(list);
        unsafe { assert_eq!(to_vec(list), vec![3, 2, 1]); }
    }

    #[test]
    fn reverse_new() {
        let list = mk_list(&[1, 2, 3]);
        let rev = ore_list_reverse_new(list);
        unsafe {
            assert_eq!(to_vec(rev), vec![3, 2, 1]);
            assert_eq!(to_vec(list), vec![1, 2, 3]); // original unchanged
        }
    }

    #[test]
    fn concat() {
        let a = mk_list(&[1, 2]);
        let b = mk_list(&[3, 4]);
        let c = ore_list_concat(a, b);
        unsafe { assert_eq!(to_vec(c), vec![1, 2, 3, 4]); }
    }

    #[test]
    fn contains() {
        let list = mk_list(&[10, 20, 30]);
        assert_eq!(ore_list_contains(list, 20), 1);
        assert_eq!(ore_list_contains(list, 99), 0);
    }

    #[test]
    fn contains_str() {
        let list = mk_str_list(&["apple", "banana"]);
        assert_eq!(ore_list_contains_str(list, mk_str("banana")), 1);
        assert_eq!(ore_list_contains_str(list, mk_str("cherry")), 0);
    }

    #[test]
    fn index_of() {
        let list = mk_list(&[10, 20, 30]);
        assert_eq!(ore_list_index_of(list, 20), 1);
        assert_eq!(ore_list_index_of(list, 99), -1);
        assert_eq!(ore_list_index_of(std::ptr::null_mut(), 1), -1);
    }

    #[test]
    fn index_of_str() {
        let list = mk_str_list(&["a", "b", "c"]);
        assert_eq!(ore_list_index_of_str(list, mk_str("b")), 1);
        assert_eq!(ore_list_index_of_str(list, mk_str("z")), -1);
    }

    #[test]
    fn unique() {
        let list = mk_list(&[1, 2, 1, 3, 2]);
        let u = ore_list_unique(list);
        unsafe { assert_eq!(to_vec(u), vec![1, 2, 3]); }
    }

    #[test]
    fn unique_str() {
        let list = mk_str_list(&["a", "b", "a", "c"]);
        let u = ore_list_unique_str(list);
        unsafe {
            assert_eq!(ore_list_len(u), 3);
            assert_eq!(rd(ore_list_get(u, 0) as *mut OreStr), "a");
            assert_eq!(rd(ore_list_get(u, 1) as *mut OreStr), "b");
            assert_eq!(rd(ore_list_get(u, 2) as *mut OreStr), "c");
        }
    }

    #[test]
    fn min_max_int() {
        let list = mk_list(&[5, 1, 9, 3]);
        assert_eq!(ore_list_min(list), 1);
        assert_eq!(ore_list_max(list), 9);
    }

    #[test]
    fn min_max_empty() {
        let list = mk_list(&[]);
        assert_eq!(ore_list_min(list), 0);
        assert_eq!(ore_list_max(list), 0);
    }

    #[test]
    fn min_max_str() {
        let list = mk_str_list(&["banana", "apple", "cherry"]);
        unsafe {
            assert_eq!(rd(ore_list_min_str(list)), "apple");
            assert_eq!(rd(ore_list_max_str(list)), "cherry");
        }
    }

    #[test]
    fn min_max_float() {
        let list = ore_list_new();
        ore_list_push(list, 3.5_f64.to_bits() as i64);
        ore_list_push(list, 1.2_f64.to_bits() as i64);
        ore_list_push(list, 7.8_f64.to_bits() as i64);
        assert!((ore_list_min_float(list) - 1.2).abs() < 1e-10);
        assert!((ore_list_max_float(list) - 7.8).abs() < 1e-10);
    }

    #[test]
    fn sum_product() {
        let list = mk_list(&[1, 2, 3, 4]);
        assert_eq!(ore_list_sum(list), 10);
        assert_eq!(ore_list_product(list), 24);
    }

    #[test]
    fn sum_product_float() {
        let list = ore_list_new();
        ore_list_push(list, 1.0_f64.to_bits() as i64);
        ore_list_push(list, 2.0_f64.to_bits() as i64);
        ore_list_push(list, 3.0_f64.to_bits() as i64);
        assert!((ore_list_sum_float(list) - 6.0).abs() < 1e-10);
        assert!((ore_list_product_float(list) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn average() {
        let list = mk_list(&[2, 4, 6]);
        assert!((ore_list_average(list) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn average_empty() {
        let list = mk_list(&[]);
        assert_eq!(ore_list_average(list), 0.0);
    }

    #[test]
    fn average_float() {
        let list = ore_list_new();
        ore_list_push(list, 1.0_f64.to_bits() as i64);
        ore_list_push(list, 2.0_f64.to_bits() as i64);
        ore_list_push(list, 3.0_f64.to_bits() as i64);
        assert!((ore_list_average_float(list) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn flatten() {
        let inner1 = mk_list(&[1, 2]);
        let inner2 = mk_list(&[3, 4]);
        let outer = ore_list_new();
        ore_list_push(outer, inner1 as i64);
        ore_list_push(outer, inner2 as i64);
        let flat = ore_list_flatten(outer);
        unsafe { assert_eq!(to_vec(flat), vec![1, 2, 3, 4]); }
    }

    #[test]
    fn flatten_empty() {
        let flat = ore_list_flatten(std::ptr::null_mut());
        assert_eq!(ore_list_len(flat), 0);
    }

    #[test]
    fn take_and_skip() {
        let list = mk_list(&[1, 2, 3, 4, 5]);
        let taken = ore_list_take(list, 3);
        unsafe { assert_eq!(to_vec(taken), vec![1, 2, 3]); }
        let skipped = ore_list_skip(list, 3);
        unsafe { assert_eq!(to_vec(skipped), vec![4, 5]); }
    }

    #[test]
    fn take_more_than_len() {
        let list = mk_list(&[1, 2]);
        let taken = ore_list_take(list, 100);
        unsafe { assert_eq!(to_vec(taken), vec![1, 2]); }
    }

    #[test]
    fn skip_more_than_len() {
        let list = mk_list(&[1, 2]);
        let skipped = ore_list_skip(list, 100);
        unsafe { assert_eq!(to_vec(skipped), Vec::<i64>::new()); }
    }

    #[test]
    fn slice() {
        let list = mk_list(&[10, 20, 30, 40, 50]);
        let sliced = ore_list_slice(list, 1, 4);
        unsafe { assert_eq!(to_vec(sliced), vec![20, 30, 40]); }
    }

    #[test]
    fn slice_negative() {
        let list = mk_list(&[10, 20, 30, 40, 50]);
        let sliced = ore_list_slice(list, -3, -1);
        unsafe { assert_eq!(to_vec(sliced), vec![30, 40]); }
    }

    #[test]
    fn repeat() {
        let list = ore_list_repeat(42, 3);
        unsafe { assert_eq!(to_vec(list), vec![42, 42, 42]); }
    }

    #[test]
    fn step() {
        let list = mk_list(&[0, 1, 2, 3, 4, 5, 6]);
        let stepped = ore_list_step(list, 2);
        unsafe { assert_eq!(to_vec(stepped), vec![0, 2, 4, 6]); }
    }

    #[test]
    fn step_zero_returns_empty() {
        let list = mk_list(&[1, 2, 3]);
        let stepped = ore_list_step(list, 0);
        assert_eq!(ore_list_len(stepped), 0);
    }

    #[test]
    fn window() {
        let list = mk_list(&[1, 2, 3, 4]);
        let windows = ore_list_window(list, 2);
        assert_eq!(ore_list_len(windows), 3);
        unsafe {
            let w0 = ore_list_get(windows, 0) as *mut OreList;
            assert_eq!(to_vec(w0), vec![1, 2]);
            let w2 = ore_list_get(windows, 2) as *mut OreList;
            assert_eq!(to_vec(w2), vec![3, 4]);
        }
    }

    #[test]
    fn window_larger_than_list() {
        let list = mk_list(&[1, 2]);
        let windows = ore_list_window(list, 5);
        assert_eq!(ore_list_len(windows), 0);
    }

    #[test]
    fn chunks() {
        let list = mk_list(&[1, 2, 3, 4, 5]);
        let chunked = ore_list_chunks(list, 2);
        assert_eq!(ore_list_len(chunked), 3);
        unsafe {
            let c0 = ore_list_get(chunked, 0) as *mut OreList;
            assert_eq!(to_vec(c0), vec![1, 2]);
            let c2 = ore_list_get(chunked, 2) as *mut OreList;
            assert_eq!(to_vec(c2), vec![5]);
        }
    }

    #[test]
    fn enumerate() {
        let list = mk_list(&[10, 20, 30]);
        let enumerated = ore_list_enumerate(list);
        assert_eq!(ore_list_len(enumerated), 3);
        let pair0 = ore_list_get(enumerated, 0) as *mut OreList;
        assert_eq!(ore_list_get(pair0, 0), 0);
        assert_eq!(ore_list_get(pair0, 1), 10);
        let pair2 = ore_list_get(enumerated, 2) as *mut OreList;
        assert_eq!(ore_list_get(pair2, 0), 2);
        assert_eq!(ore_list_get(pair2, 1), 30);
    }

    #[test]
    fn zip() {
        let a = mk_list(&[1, 2, 3]);
        let b = mk_list(&[10, 20, 30]);
        let zipped = ore_list_zip(a, b);
        assert_eq!(ore_list_len(zipped), 3);
        let pair = ore_list_get(zipped, 1) as *mut OreList;
        assert_eq!(ore_list_get(pair, 0), 2);
        assert_eq!(ore_list_get(pair, 1), 20);
    }

    #[test]
    fn zip_unequal_length() {
        let a = mk_list(&[1, 2, 3]);
        let b = mk_list(&[10, 20]);
        let zipped = ore_list_zip(a, b);
        assert_eq!(ore_list_len(zipped), 2); // min length
    }

    #[test]
    fn intersperse() {
        let list = mk_list(&[1, 2, 3]);
        let result = ore_list_intersperse(list, 0);
        unsafe { assert_eq!(to_vec(result), vec![1, 0, 2, 0, 3]); }
    }

    #[test]
    fn intersperse_single() {
        let list = mk_list(&[42]);
        let result = ore_list_intersperse(list, 0);
        unsafe { assert_eq!(to_vec(result), vec![42]); }
    }

    // Test closure-based operations using simple C-compatible functions
    extern "C" fn double(x: i64) -> i64 { x * 2 }
    extern "C" fn is_even(x: i64) -> i64 { if x % 2 == 0 { 1 } else { 0 } }
    extern "C" fn add(a: i64, b: i64) -> i64 { a + b }

    #[test]
    fn map() {
        let list = mk_list(&[1, 2, 3]);
        let mapped = ore_list_map(list, double as *const u8, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(mapped), vec![2, 4, 6]); }
    }

    #[test]
    fn filter() {
        let list = mk_list(&[1, 2, 3, 4, 5, 6]);
        let filtered = ore_list_filter(list, is_even as *const u8, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(filtered), vec![2, 4, 6]); }
    }

    #[test]
    fn fold() {
        let list = mk_list(&[1, 2, 3, 4]);
        let result = ore_list_fold(list, 0, add as *const u8, std::ptr::null_mut());
        assert_eq!(result, 10);
    }

    #[test]
    fn reduce1() {
        let list = mk_list(&[1, 2, 3, 4]);
        let result = ore_list_reduce1(list, add as *const u8, std::ptr::null_mut());
        assert_eq!(result, 10);
    }

    #[test]
    fn reduce1_empty() {
        let list = mk_list(&[]);
        let result = ore_list_reduce1(list, add as *const u8, std::ptr::null_mut());
        assert_eq!(result, 0);
    }

    #[test]
    fn find() {
        let list = mk_list(&[1, 3, 4, 5]);
        let found = ore_list_find(list, is_even as *const u8, std::ptr::null_mut(), -1);
        assert_eq!(found, 4);
    }

    #[test]
    fn find_not_found() {
        let list = mk_list(&[1, 3, 5]);
        let found = ore_list_find(list, is_even as *const u8, std::ptr::null_mut(), -1);
        assert_eq!(found, -1);
    }

    #[test]
    fn find_index() {
        let list = mk_list(&[1, 3, 4, 5]);
        let idx = ore_list_find_index(list, is_even as *const u8, std::ptr::null_mut());
        assert_eq!(idx, 2);
    }

    #[test]
    fn find_index_not_found() {
        let list = mk_list(&[1, 3, 5]);
        let idx = ore_list_find_index(list, is_even as *const u8, std::ptr::null_mut());
        assert_eq!(idx, -1);
    }

    #[test]
    fn any() {
        let list = mk_list(&[1, 3, 4, 5]);
        assert_eq!(ore_list_any(list, is_even as *const u8, std::ptr::null_mut()), 1);
        let list2 = mk_list(&[1, 3, 5]);
        assert_eq!(ore_list_any(list2, is_even as *const u8, std::ptr::null_mut()), 0);
    }

    #[test]
    fn all() {
        let list = mk_list(&[2, 4, 6]);
        assert_eq!(ore_list_all(list, is_even as *const u8, std::ptr::null_mut()), 1);
        let list2 = mk_list(&[2, 3, 6]);
        assert_eq!(ore_list_all(list2, is_even as *const u8, std::ptr::null_mut()), 0);
    }

    #[test]
    fn scan() {
        let list = mk_list(&[1, 2, 3]);
        let scanned = ore_list_scan(list, 0, add as *const u8, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(scanned), vec![0, 1, 3, 6]); }
    }

    extern "C" fn is_positive(x: i64) -> i64 { if x > 0 { 1 } else { 0 } }

    #[test]
    fn take_while() {
        let list = mk_list(&[1, 2, 3, -1, 4]);
        let taken = ore_list_take_while(list, is_positive as *const u8, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(taken), vec![1, 2, 3]); }
    }

    #[test]
    fn drop_while() {
        let list = mk_list(&[1, 2, 3, -1, 4]);
        let dropped = ore_list_drop_while(list, is_positive as *const u8, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(dropped), vec![-1, 4]); }
    }

    #[test]
    fn partition() {
        let list = mk_list(&[1, 2, 3, 4, 5]);
        let parts = ore_list_partition(list, is_even as *const u8, std::ptr::null_mut());
        unsafe {
            let evens = ore_list_get(parts, 0) as *mut OreList;
            let odds = ore_list_get(parts, 1) as *mut OreList;
            assert_eq!(to_vec(evens), vec![2, 4]);
            assert_eq!(to_vec(odds), vec![1, 3, 5]);
        }
    }

    extern "C" fn mk_pair(x: i64) -> i64 {
        let list = ore_list_new();
        ore_list_push(list, x);
        ore_list_push(list, x * 10);
        list as i64
    }

    #[test]
    fn flat_map() {
        let list = mk_list(&[1, 2, 3]);
        let result = ore_list_flat_map(list, mk_pair as *const u8, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(result), vec![1, 10, 2, 20, 3, 30]); }
    }

    extern "C" fn cmp_fn(a: i64, b: i64, _env: *mut u8) -> i64 {
        if a < b { -1 } else if a > b { 1 } else { 0 }
    }

    #[test]
    fn sort_by() {
        let list = mk_list(&[3, 1, 2]);
        let sorted = ore_list_sort_by(list, cmp_fn, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(sorted), vec![1, 2, 3]); }
    }

    extern "C" fn negate_key(x: i64, _env: *mut u8) -> i64 { -x }

    #[test]
    fn sort_by_key() {
        let list = mk_list(&[1, 2, 3]);
        let sorted = ore_list_sort_by_key(list, negate_key, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(sorted), vec![3, 2, 1]); }
    }

    extern "C" fn identity_key(x: i64, _env: *mut u8) -> i64 { x }

    #[test]
    fn min_by_max_by() {
        let list = mk_list(&[5, 1, 9, 3]);
        let min = ore_list_min_by(list, identity_key, std::ptr::null_mut());
        let max = ore_list_max_by(list, identity_key, std::ptr::null_mut());
        assert_eq!(min, 1);
        assert_eq!(max, 9);
    }

    extern "C" fn count_pred(x: i64, _env: *mut u8) -> i8 {
        if x > 2 { 1 } else { 0 }
    }

    #[test]
    fn count() {
        let list = mk_list(&[1, 2, 3, 4, 5]);
        assert_eq!(ore_list_count(list, count_pred, std::ptr::null_mut()), 3);
    }

    #[test]
    fn count_null() {
        assert_eq!(ore_list_count(std::ptr::null_mut(), count_pred, std::ptr::null_mut()), 0);
    }

    extern "C" fn zip_add(a: i64, b: i64) -> i64 { a + b }

    #[test]
    fn zip_with() {
        let a = mk_list(&[1, 2, 3]);
        let b = mk_list(&[10, 20, 30]);
        let result = ore_list_zip_with(a, b, zip_add as *const u8, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(result), vec![11, 22, 33]); }
    }

    extern "C" fn idx_mul(i: i64, v: i64) -> i64 { i * v }

    #[test]
    fn map_with_index() {
        let list = mk_list(&[10, 20, 30]);
        let result = ore_list_map_with_index(list, idx_mul as *const u8, std::ptr::null_mut());
        unsafe { assert_eq!(to_vec(result), vec![0, 20, 60]); }
    }

    #[test]
    fn join_int_list() {
        let list = mk_list(&[1, 2, 3]);
        let sep = mk_str(", ");
        let result = ore_list_join(list, sep);
        unsafe { assert_eq!(rd(result), "1, 2, 3"); }
    }

    #[test]
    fn join_str_list() {
        let list = mk_str_list(&["hello", "world"]);
        let sep = mk_str(" ");
        let result = ore_list_join_str(list, sep);
        unsafe { assert_eq!(rd(result), "hello world"); }
    }

    #[test]
    fn join_float_list() {
        let list = ore_list_new();
        ore_list_push(list, 1.0_f64.to_bits() as i64);
        ore_list_push(list, 2.5_f64.to_bits() as i64);
        let sep = mk_str(", ");
        let result = ore_list_join_float(list, sep);
        unsafe { assert_eq!(rd(result), "1.0, 2.5"); }
    }

    #[test]
    fn frequencies_int() {
        use crate::kinds::KIND_INT;
        let list = mk_list(&[1, 2, 1, 3, 2, 1]);
        let map = ore_list_frequencies(list, KIND_INT);
        assert_eq!(ore_map_get(map, mk_str("1")), 3);
        assert_eq!(ore_map_get(map, mk_str("2")), 2);
        assert_eq!(ore_map_get(map, mk_str("3")), 1);
    }

    #[test]
    fn frequencies_str() {
        use crate::kinds::KIND_STR;
        let list = mk_str_list(&["a", "b", "a"]);
        let map = ore_list_frequencies(list, KIND_STR);
        assert_eq!(ore_map_get(map, mk_str("a")), 2);
        assert_eq!(ore_map_get(map, mk_str("b")), 1);
    }
}

#[cfg(test)]
mod map_tests {
    use crate::*;
    fn mk(s: &str) -> *mut OreStr { str_to_ore(s) }
    unsafe fn rd(p: *mut OreStr) -> &'static str { (*p).as_str() }

    #[test]
    fn new_map_empty() {
        let map = ore_map_new();
        assert_eq!(ore_map_len(map), 0);
    }

    #[test]
    fn set_and_get() {
        let map = ore_map_new();
        ore_map_set(map, mk("key"), 42);
        assert_eq!(ore_map_get(map, mk("key")), 42);
        assert_eq!(ore_map_len(map), 1);
    }

    #[test]
    fn get_missing_returns_zero() {
        let map = ore_map_new();
        assert_eq!(ore_map_get(map, mk("missing")), 0);
    }

    #[test]
    fn get_or_default() {
        let map = ore_map_new();
        ore_map_set(map, mk("a"), 10);
        assert_eq!(ore_map_get_or(map, mk("a"), -1), 10);
        assert_eq!(ore_map_get_or(map, mk("b"), -1), -1);
    }

    #[test]
    fn contains() {
        let map = ore_map_new();
        ore_map_set(map, mk("x"), 1);
        assert_eq!(ore_map_contains(map, mk("x")), 1);
        assert_eq!(ore_map_contains(map, mk("y")), 0);
    }

    #[test]
    fn remove() {
        let map = ore_map_new();
        ore_map_set(map, mk("a"), 42);
        assert_eq!(ore_map_remove(map, mk("a")), 42);
        assert_eq!(ore_map_len(map), 0);
        assert_eq!(ore_map_remove(map, mk("a")), 0);
    }

    #[test]
    fn keys_sorted() {
        let map = ore_map_new();
        ore_map_set(map, mk("c"), 3);
        ore_map_set(map, mk("a"), 1);
        ore_map_set(map, mk("b"), 2);
        let keys = ore_map_keys(map);
        assert_eq!(ore_list_len(keys), 3);
        unsafe {
            assert_eq!(rd(ore_list_get(keys, 0) as *mut OreStr), "a");
            assert_eq!(rd(ore_list_get(keys, 1) as *mut OreStr), "b");
            assert_eq!(rd(ore_list_get(keys, 2) as *mut OreStr), "c");
        }
    }

    #[test]
    fn values_sorted_by_key() {
        let map = ore_map_new();
        ore_map_set(map, mk("b"), 2);
        ore_map_set(map, mk("a"), 1);
        ore_map_set(map, mk("c"), 3);
        let vals = ore_map_values(map);
        assert_eq!(ore_list_len(vals), 3);
        assert_eq!(ore_list_get(vals, 0), 1);
        assert_eq!(ore_list_get(vals, 1), 2);
        assert_eq!(ore_list_get(vals, 2), 3);
    }

    #[test]
    fn set_typed() {
        use crate::kinds::KIND_STR;
        let map = ore_map_new();
        let val = str_to_ore("hello");
        ore_map_set_typed(map, mk("greeting"), val as i64, KIND_STR);
        let got = ore_map_get(map, mk("greeting"));
        unsafe { assert_eq!(rd(got as *mut OreStr), "hello"); }
    }

    #[test]
    fn merge() {
        let a = ore_map_new();
        ore_map_set(a, mk("x"), 1);
        ore_map_set(a, mk("y"), 2);

        let b = ore_map_new();
        ore_map_set(b, mk("y"), 20);
        ore_map_set(b, mk("z"), 30);

        let merged = ore_map_merge(a, b);
        assert_eq!(ore_map_get(merged, mk("x")), 1);
        assert_eq!(ore_map_get(merged, mk("y")), 20); // b overwrites
        assert_eq!(ore_map_get(merged, mk("z")), 30);
        assert_eq!(ore_map_len(merged), 3);
    }

    #[test]
    fn clear() {
        let map = ore_map_new();
        ore_map_set(map, mk("a"), 1);
        ore_map_set(map, mk("b"), 2);
        ore_map_clear(map);
        assert_eq!(ore_map_len(map), 0);
    }

    #[test]
    fn entries() {
        let map = ore_map_new();
        ore_map_set(map, mk("a"), 1);
        ore_map_set(map, mk("b"), 2);
        let entries = ore_map_entries(map);
        assert_eq!(ore_list_len(entries), 2);
        unsafe {
            let pair0 = ore_list_get(entries, 0) as *mut OreList;
            assert_eq!(rd(ore_list_get(pair0, 0) as *mut OreStr), "a");
            assert_eq!(ore_list_get(pair0, 1), 1);
        }
    }

    extern "C" fn double_val(key: i64, val: i64) -> i64 {
        let _ = key;
        val * 2
    }

    #[test]
    fn map_values() {
        let map = ore_map_new();
        ore_map_set(map, mk("a"), 5);
        ore_map_set(map, mk("b"), 10);
        let mapped = ore_map_map_values(map, double_val as *const u8, std::ptr::null_mut());
        assert_eq!(ore_map_get(mapped, mk("a")), 10);
        assert_eq!(ore_map_get(mapped, mk("b")), 20);
    }

    extern "C" fn val_gt_5(_key: i64, val: i64) -> i64 {
        if val > 5 { 1 } else { 0 }
    }

    #[test]
    fn filter() {
        let map = ore_map_new();
        ore_map_set(map, mk("a"), 3);
        ore_map_set(map, mk("b"), 10);
        ore_map_set(map, mk("c"), 7);
        let filtered = ore_map_filter(map, val_gt_5 as *const u8, std::ptr::null_mut());
        assert_eq!(ore_map_len(filtered), 2);
        assert_eq!(ore_map_contains(filtered, mk("a")), 0);
        assert_eq!(ore_map_contains(filtered, mk("b")), 1);
        assert_eq!(ore_map_contains(filtered, mk("c")), 1);
    }

    extern "C" fn each_collector(key: i64, val: i64) -> i64 {
        let _ = key;
        let _ = val;
        0
    }

    #[test]
    fn each_runs_without_panic() {
        let map = ore_map_new();
        ore_map_set(map, mk("a"), 1);
        ore_map_set(map, mk("b"), 2);
        ore_map_each(map, each_collector as *const u8, std::ptr::null_mut());
    }
}

#[cfg(test)]
mod math_tests {
    use crate::math::*;

    #[test]
    fn sqrt() {
        assert!((ore_math_sqrt(4.0) - 2.0).abs() < 1e-10);
        assert!((ore_math_sqrt(0.0)).abs() < 1e-10);
    }

    #[test]
    fn trig() {
        assert!((ore_math_sin(0.0)).abs() < 1e-10);
        assert!((ore_math_cos(0.0) - 1.0).abs() < 1e-10);
        assert!((ore_math_tan(0.0)).abs() < 1e-10);
    }

    #[test]
    fn log_exp() {
        assert!((ore_math_log(1.0)).abs() < 1e-10);
        assert!((ore_math_exp(0.0) - 1.0).abs() < 1e-10);
        assert!((ore_math_log10(100.0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn pow() {
        assert!((ore_math_pow(2.0, 3.0) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn abs() {
        assert!((ore_math_abs(-5.0) - 5.0).abs() < 1e-10);
        assert!((ore_math_abs(5.0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn floor_ceil_round() {
        assert_eq!(ore_math_floor(2.7), 2.0);
        assert_eq!(ore_math_ceil(2.3), 3.0);
        assert_eq!(ore_math_round(2.5), 3.0);
        assert_eq!(ore_math_round(2.4), 2.0);
    }

    #[test]
    fn constants() {
        assert!((ore_math_pi() - std::f64::consts::PI).abs() < 1e-15);
        assert!((ore_math_e() - std::f64::consts::E).abs() < 1e-15);
    }

    #[test]
    fn atan2() {
        assert!((ore_math_atan2(1.0, 1.0) - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    }

    #[test]
    fn float_round_to() {
        assert!((ore_float_round_to(3.14159, 2) - 3.14).abs() < 1e-10);
        assert!((ore_float_round_to(3.14159, 0) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn float_format() {
        use crate::*;
        let s = ore_float_format(3.14159, 2);
        unsafe { assert_eq!((*s).as_str(), "3.14"); }
    }

    #[test]
    fn int_pow() {
        assert_eq!(ore_int_pow(2, 10), 1024);
        assert_eq!(ore_int_pow(3, 0), 1);
        assert_eq!(ore_int_pow(5, -1), 0); // negative exponent
    }
}

#[cfg(test)]
mod convert_tests {
    use crate::*;
    use crate::kinds::*;

    fn mk(s: &str) -> *mut OreStr { str_to_ore(s) }
    unsafe fn rd(p: *mut OreStr) -> &'static str { (*p).as_str() }

    #[test]
    fn int_to_str() {
        unsafe {
            assert_eq!(rd(ore_int_to_str(42)), "42");
            assert_eq!(rd(ore_int_to_str(-7)), "-7");
            assert_eq!(rd(ore_int_to_str(0)), "0");
        }
    }

    #[test]
    fn float_to_str() {
        unsafe {
            assert_eq!(rd(ore_float_to_str(3.14)), "3.14");
            assert_eq!(rd(ore_float_to_str(2.0)), "2.0");
        }
    }

    #[test]
    fn bool_to_str() {
        unsafe {
            assert_eq!(rd(ore_bool_to_str(1)), "true");
            assert_eq!(rd(ore_bool_to_str(0)), "false");
        }
    }

    #[test]
    fn dynamic_to_str() {
        unsafe {
            assert_eq!(rd(ore_dynamic_to_str(42, KIND_INT)), "42");
            assert_eq!(rd(ore_dynamic_to_str(1, KIND_BOOL)), "true");
            assert_eq!(rd(ore_dynamic_to_str(0, KIND_BOOL)), "false");
        }
    }

    #[test]
    fn dynamic_to_str_float() {
        let bits = 2.5_f64.to_bits() as i64;
        unsafe {
            assert_eq!(rd(ore_dynamic_to_str(bits, KIND_FLOAT)), "2.5");
        }
    }

    #[test]
    fn dynamic_to_str_string() {
        let s = mk("hello");
        unsafe {
            assert_eq!(rd(ore_dynamic_to_str(s as i64, KIND_STR)), "hello");
        }
    }

    #[test]
    fn dynamic_to_str_unknown() {
        unsafe {
            assert_eq!(rd(ore_dynamic_to_str(0, 99)), "<dynamic:99>");
        }
    }

    #[test]
    fn type_of() {
        unsafe {
            assert_eq!(rd(ore_type_of(KIND_INT)), "Int");
            assert_eq!(rd(ore_type_of(KIND_FLOAT)), "Float");
            assert_eq!(rd(ore_type_of(KIND_BOOL)), "Bool");
            assert_eq!(rd(ore_type_of(KIND_STR)), "Str");
            assert_eq!(rd(ore_type_of(KIND_LIST)), "List");
            assert_eq!(rd(ore_type_of(KIND_MAP)), "Map");
            assert_eq!(rd(ore_type_of(KIND_CHANNEL)), "Channel");
            assert_eq!(rd(ore_type_of(KIND_RECORD)), "Record");
            assert_eq!(rd(ore_type_of(KIND_ENUM)), "Enum");
            assert_eq!(rd(ore_type_of(KIND_OPTION)), "Option");
            assert_eq!(rd(ore_type_of(KIND_RESULT)), "Result");
            assert_eq!(rd(ore_type_of(127)), "Unknown");
        }
    }

    #[test]
    fn json_parse_basic() {
        let json = mk(r#"{"name":"test","count":42,"active":true}"#);
        let map = ore_json_parse(json);
        unsafe {
            assert_eq!(ore_map_len(map), 3);
            let name = ore_map_get(map, mk("name")) as *mut OreStr;
            assert_eq!(rd(name), "test");
            assert_eq!(ore_map_get(map, mk("count")), 42);
        }
    }

    #[test]
    fn json_parse_invalid() {
        let json = mk("not json");
        let map = ore_json_parse(json);
        assert_eq!(ore_map_len(map), 0);
    }

    #[test]
    fn json_stringify() {
        let map = ore_map_new();
        ore_map_set_with_kind(map, "x", 10, KIND_INT);
        let s = ore_json_stringify(map);
        unsafe {
            let json_str = rd(s);
            assert!(json_str.contains("\"x\":10"));
        }
    }

    #[test]
    fn json_roundtrip() {
        let map = ore_map_new();
        ore_map_set_with_kind(map, "num", 99, KIND_INT);
        ore_map_set_with_kind(map, "flag", 1, KIND_BOOL);
        let json = ore_json_stringify(map);
        let parsed = ore_json_parse(json);
        assert_eq!(ore_map_get(parsed, mk("num")), 99);
    }
}

#[cfg(test)]
mod range_tests {
    use crate::*;
    unsafe fn to_vec(list: *mut OreList) -> Vec<i64> {
        (&*list).as_slice().to_vec()
    }

    #[test]
    fn basic_range() {
        let list = ore_range(0, 5);
        unsafe { assert_eq!(to_vec(list), vec![0, 1, 2, 3, 4]); }
    }

    #[test]
    fn empty_range() {
        let list = ore_range(5, 5);
        assert_eq!(ore_list_len(list), 0);
    }

    #[test]
    fn inverted_range() {
        let list = ore_range(5, 0);
        assert_eq!(ore_list_len(list), 0);
    }

    #[test]
    fn range_step_positive() {
        let list = ore_range_step(0, 10, 3);
        unsafe { assert_eq!(to_vec(list), vec![0, 3, 6, 9]); }
    }

    #[test]
    fn range_step_negative() {
        let list = ore_range_step(10, 0, -3);
        unsafe { assert_eq!(to_vec(list), vec![10, 7, 4, 1]); }
    }

    #[test]
    fn range_step_zero() {
        let list = ore_range_step(0, 10, 0);
        assert_eq!(ore_list_len(list), 0);
    }

    #[test]
    fn time_now_positive() {
        assert!(ore_time_now() > 0);
    }

    #[test]
    fn time_ms_gt_sec() {
        let ms = ore_time_ms();
        let s = ore_time_now();
        assert!(ms > s);
    }

    #[test]
    fn rand_int_in_range() {
        for _ in 0..100 {
            let val = ore_rand_int(1, 6);
            assert!(val >= 1 && val <= 6);
        }
    }

    #[test]
    fn rand_int_same_bounds() {
        assert_eq!(ore_rand_int(5, 5), 5);
    }
}

#[cfg(test)]
mod assert_tests {
    use crate::assert::*;

    fn mk_cstr(s: &str) -> Vec<u8> {
        let mut v: Vec<u8> = s.bytes().collect();
        v.push(0);
        v
    }

    #[test]
    fn test_mode_flag() {
        ore_assert_set_test_mode(1);
        // Should not exit — just set the flag
        let msg = mk_cstr("test assert");
        ore_assert(0, msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 1);
        // After reset, should be clear
        assert_eq!(ore_assert_check_and_reset(), 0);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_pass_no_flag() {
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("should pass");
        ore_assert(1, msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 0);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_eq_int_pass() {
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("eq test");
        ore_assert_eq_int(5, 5, msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 0);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_eq_int_fail() {
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("eq test");
        ore_assert_eq_int(5, 6, msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 1);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_eq_float_pass() {
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("float eq");
        ore_assert_eq_float(1.0, 1.0, msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 0);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_eq_float_fail() {
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("float eq");
        ore_assert_eq_float(1.0, 2.0, msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 1);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_eq_str_pass() {
        use crate::str_to_ore;
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("str eq");
        ore_assert_eq_str(str_to_ore("hello"), str_to_ore("hello"), msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 0);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_eq_str_fail() {
        use crate::str_to_ore;
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("str eq");
        ore_assert_eq_str(str_to_ore("hello"), str_to_ore("world"), msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 1);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_ne_int_pass() {
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("ne test");
        ore_assert_ne_int(5, 6, msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 0);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_ne_int_fail() {
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("ne test");
        ore_assert_ne_int(5, 5, msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 1);
        ore_assert_set_test_mode(0);
    }

    #[test]
    fn assert_ne_str_pass() {
        use crate::str_to_ore;
        ore_assert_set_test_mode(1);
        let msg = mk_cstr("ne str");
        ore_assert_ne_str(str_to_ore("a"), str_to_ore("b"), msg.as_ptr(), 1);
        assert_eq!(ore_assert_check_and_reset(), 0);
        ore_assert_set_test_mode(0);
    }
}

#[cfg(test)]
mod print_tests {
    use crate::print::format_float;

    #[test]
    fn format_whole_number() {
        assert_eq!(format_float(2.0), "2.0");
        assert_eq!(format_float(0.0), "0.0");
        assert_eq!(format_float(-1.0), "-1.0");
    }

    #[test]
    fn format_fractional() {
        assert_eq!(format_float(3.14), "3.14");
        assert_eq!(format_float(-2.5), "-2.5");
    }

    #[test]
    fn format_special() {
        assert_eq!(format_float(f64::INFINITY), "inf");
        assert_eq!(format_float(f64::NEG_INFINITY), "-inf");
        assert_eq!(format_float(f64::NAN), "NaN");
    }
}

#[cfg(test)]
mod concurrency_tests {
    use crate::concurrency::*;

    #[test]
    fn channel_send_recv() {
        let ch = ore_channel_new();
        assert!(!ch.is_null());
        // Send and receive in the same thread (channel is unbounded)
        ore_channel_send(ch, 42);
        assert_eq!(ore_channel_recv(ch), 42);
    }

    #[test]
    fn channel_multiple_values() {
        let ch = ore_channel_new();
        ore_channel_send(ch, 1);
        ore_channel_send(ch, 2);
        ore_channel_send(ch, 3);
        assert_eq!(ore_channel_recv(ch), 1);
        assert_eq!(ore_channel_recv(ch), 2);
        assert_eq!(ore_channel_recv(ch), 3);
    }

    #[test]
    fn spawn_and_join() {
        extern "C" fn noop() {}
        ore_spawn(noop);
        ore_thread_join_all();
    }

    #[test]
    fn spawn_with_channel() {
        let ch = ore_channel_new();
        extern "C" fn send_val(ch_ptr: i64) {
            ore_channel_send(ch_ptr as *mut crate::concurrency::OreChannel, 99);
        }
        ore_spawn_with_arg(send_val, ch as i64);
        let val = ore_channel_recv(ch);
        assert_eq!(val, 99);
        ore_thread_join_all();
    }
}

#[cfg(test)]
mod kinds_tests {
    use crate::kinds::*;

    #[test]
    fn kind_constants_unique() {
        let kinds = [
            KIND_INT, KIND_FLOAT, KIND_BOOL, KIND_STR, KIND_VOID,
            KIND_RECORD, KIND_ENUM, KIND_OPTION, KIND_RESULT,
            KIND_LIST, KIND_MAP, KIND_CHANNEL,
        ];
        for i in 0..kinds.len() {
            for j in (i + 1)..kinds.len() {
                assert_ne!(kinds[i], kinds[j], "kinds at {} and {} collide", i, j);
            }
        }
    }

    #[test]
    fn kind_values() {
        assert_eq!(KIND_INT, 0);
        assert_eq!(KIND_FLOAT, 1);
        assert_eq!(KIND_BOOL, 2);
        assert_eq!(KIND_STR, 3);
    }
}
