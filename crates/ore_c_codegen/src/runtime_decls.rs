/// Generate C extern declarations for all ore_runtime functions.
pub fn runtime_declarations() -> Vec<String> {
    let mut decls = Vec::new();

    // Helper macros as closures
    let mut d = |s: &str| decls.push(format!("extern {};", s));

    // Print primitives
    d("void ore_print_int(int64_t)");
    d("void ore_print_bool(int8_t)");
    d("void ore_print_float(double)");
    d("void ore_print_int_no_newline(int64_t)");
    d("void ore_print_float_no_newline(double)");
    d("void ore_print_bool_no_newline(int8_t)");

    // Stderr
    d("void ore_eprint_str(void*)");
    d("void ore_eprint_int(int64_t)");
    d("void ore_eprint_float(double)");
    d("void ore_eprint_bool(int8_t)");

    // Runtime errors
    d("void ore_div_by_zero(void)");

    // Strings — creation, RC, conversion
    d("void* ore_str_new(const char*, int32_t)");
    d("void* ore_str_concat(void*, void*)");
    d("void ore_str_print(void*)");
    d("void ore_str_print_no_newline(void*)");
    d("void ore_str_retain(void*)");
    d("void ore_str_release(void*)");
    d("void* ore_int_to_str(int64_t)");
    d("void* ore_bool_to_str(int8_t)");
    d("void* ore_float_to_str(double)");
    d("void* ore_dynamic_to_str(int64_t, int8_t)");

    // String methods
    d("int64_t ore_str_len(void*)");
    d("int8_t ore_str_eq(void*, void*)");
    d("int64_t ore_str_cmp(void*, void*)");
    d("int8_t ore_str_contains(void*, void*)");
    d("void* ore_str_trim(void*)");
    d("void* ore_str_trim_start(void*)");
    d("void* ore_str_trim_end(void*)");
    d("void* ore_str_lines(void*)");
    d("void* ore_str_split(void*, void*)");
    d("void* ore_str_split_whitespace(void*)");
    d("int64_t ore_str_to_int(void*)");
    d("double ore_str_to_float(void*)");
    d("void* ore_str_replace(void*, void*, void*)");
    d("int8_t ore_str_starts_with(void*, void*)");
    d("int8_t ore_str_ends_with(void*, void*)");
    d("void* ore_str_to_upper(void*)");
    d("void* ore_str_to_lower(void*)");
    d("void* ore_str_substr(void*, int64_t, int64_t)");
    d("void* ore_str_chars(void*)");
    d("void* ore_str_repeat(void*, int64_t)");
    d("void* ore_str_pad_left(void*, int64_t, void*)");
    d("void* ore_str_pad_right(void*, int64_t, void*)");
    d("int64_t ore_str_count(void*, void*)");
    d("void* ore_str_strip_prefix(void*, void*)");
    d("void* ore_str_strip_suffix(void*, void*)");
    d("int64_t ore_str_index_of(void*, void*)");
    d("void* ore_str_slice(void*, int64_t, int64_t)");
    d("void* ore_str_reverse(void*)");
    d("void* ore_str_char_at(void*, int64_t)");
    d("void* ore_str_capitalize(void*)");
    d("int64_t ore_ord(void*)");
    d("void* ore_chr(int64_t)");

    // Assert
    d("void ore_assert(int8_t, void*, int64_t)");
    d("void ore_assert_eq_int(int64_t, int64_t, void*, int64_t)");
    d("void ore_assert_eq_float(double, double, void*, int64_t)");
    d("void ore_assert_eq_str(void*, void*, void*, int64_t)");
    d("void ore_assert_ne_int(int64_t, int64_t, void*, int64_t)");
    d("void ore_assert_ne_str(void*, void*, void*, int64_t)");
    d("void ore_assert_fail(void*)");
    d("void ore_assert_set_test_mode(int8_t)");
    d("int8_t ore_assert_check_and_reset(void)");

    // Lists — basic operations
    d("void* ore_list_new(void)");
    d("void ore_list_push(void*, int64_t)");
    d("int64_t ore_list_pop(void*)");
    d("void ore_list_clear(void*)");
    d("void ore_list_insert(void*, int64_t, int64_t)");
    d("int64_t ore_list_remove_at(void*, int64_t)");
    d("int64_t ore_list_get(void*, int64_t)");
    d("void ore_list_set(void*, int64_t, int64_t)");
    d("int64_t ore_list_get_or(void*, int64_t, int64_t)");
    d("int64_t ore_list_len(void*)");

    // Lists — print
    d("void ore_list_print(void*)");
    d("void ore_list_print_typed(void*, int64_t)");
    d("void ore_list_print_str(void*)");
    d("void ore_list_print_float(void*)");
    d("void ore_list_print_bool(void*)");

    // Lists — higher-order methods (fn_ptr, env_ptr)
    d("void* ore_list_map(void*, void*, void*)");
    d("void* ore_list_filter(void*, void*, void*)");
    d("void ore_list_each(void*, void*, void*)");
    d("int64_t ore_list_find_index(void*, void*, void*)");
    d("int64_t ore_list_find(void*, void*, void*, int64_t)");
    d("int64_t ore_list_fold(void*, int64_t, void*, void*)");
    d("int64_t ore_list_reduce1(void*, void*, void*)");
    d("void* ore_list_scan(void*, int64_t, void*, void*)");
    d("int8_t ore_list_any(void*, void*, void*)");
    d("int8_t ore_list_all(void*, void*, void*)");
    d("void* ore_list_flat_map(void*, void*, void*)");
    d("int64_t ore_list_count(void*, void*, void*)");
    d("void* ore_list_count_by(void*, void*, void*)");
    d("void* ore_list_count_by_int(void*, void*, void*)");
    d("void* ore_list_group_by(void*, void*, void*)");
    d("void* ore_list_sort_by(void*, void*, void*)");
    d("void* ore_list_sort_by_key(void*, void*, void*)");
    d("void* ore_list_sort_by_key_str(void*, void*, void*)");
    d("int64_t ore_list_min_by(void*, void*, void*)");
    d("int64_t ore_list_max_by(void*, void*, void*)");
    d("void* ore_list_partition(void*, void*, void*)");
    d("void* ore_list_take_while(void*, void*, void*)");
    d("void* ore_list_drop_while(void*, void*, void*)");
    d("void* ore_list_tap(void*, void*, void*)");
    d("void* ore_list_map_with_index(void*, void*, void*)");
    d("void ore_list_each_with_index(void*, void*, void*)");
    d("void* ore_list_unique_by(void*, void*, void*)");
    d("void* ore_list_to_map(void*, void*, void*)");
    d("void* ore_list_par_map(void*, void*, void*)");
    d("void ore_list_par_each(void*, void*, void*)");
    d("void* ore_list_zip_with(void*, void*, void*, void*)");

    // Lists — non-higher-order methods
    d("void* ore_list_sort(void*)");
    d("void* ore_list_sort_str(void*)");
    d("void* ore_list_sort_float(void*)");
    d("void ore_list_reverse(void*)");
    d("void* ore_list_reverse_new(void*)");
    d("int8_t ore_list_contains(void*, int64_t)");
    d("int8_t ore_list_contains_str(void*, void*)");
    d("void* ore_list_concat(void*, void*)");
    d("void* ore_list_zip(void*, void*)");
    d("void* ore_list_enumerate(void*)");
    d("void* ore_list_join(void*, void*)");
    d("void* ore_list_join_str(void*, void*)");
    d("void* ore_list_join_float(void*, void*)");
    d("void* ore_list_take(void*, int64_t)");
    d("void* ore_list_skip(void*, int64_t)");
    d("void* ore_list_step(void*, int64_t)");
    d("void* ore_list_slice(void*, int64_t, int64_t)");
    d("void* ore_list_window(void*, int64_t)");
    d("void* ore_list_chunks(void*, int64_t)");
    d("void* ore_list_flatten(void*)");
    d("int64_t ore_list_index_of(void*, int64_t)");
    d("int64_t ore_list_index_of_str(void*, void*)");
    d("void* ore_list_unique(void*)");
    d("void* ore_list_unique_str(void*)");
    d("void* ore_list_dedup(void*)");
    d("void* ore_list_intersperse(void*, int64_t)");
    d("void* ore_list_frequencies(void*, int8_t)");

    // Lists — aggregation
    d("int64_t ore_list_sum(void*)");
    d("int64_t ore_list_product(void*)");
    d("int64_t ore_list_min(void*)");
    d("int64_t ore_list_max(void*)");
    d("double ore_list_sum_float(void*)");
    d("double ore_list_product_float(void*)");
    d("double ore_list_min_float(void*)");
    d("double ore_list_max_float(void*)");
    d("void* ore_list_min_str(void*)");
    d("void* ore_list_max_str(void*)");
    d("double ore_list_average(void*)");
    d("double ore_list_average_float(void*)");

    // Lists — range/repeat
    d("void* ore_range(int64_t, int64_t)");
    d("void* ore_range_step(int64_t, int64_t, int64_t)");
    d("void* ore_list_repeat(int64_t, int64_t)");

    // Maps
    d("void* ore_map_new(void)");
    d("void ore_map_set(void*, void*, int64_t)");
    d("void ore_map_set_typed(void*, void*, int64_t, int8_t)");
    d("int64_t ore_map_get(void*, void*)");
    d("int64_t ore_map_get_or(void*, void*, int64_t)");
    d("int8_t ore_map_contains(void*, void*)");
    d("int64_t ore_map_len(void*)");
    d("int64_t ore_map_remove(void*, void*)");
    d("void* ore_map_keys(void*)");
    d("void* ore_map_values(void*)");
    d("void* ore_map_entries(void*)");
    d("void ore_map_print(void*)");
    d("void ore_map_print_str(void*)");
    d("void* ore_map_merge(void*, void*)");
    d("void ore_map_clear(void*)");
    d("void ore_map_each(void*, void*, void*)");
    d("void* ore_map_map_values(void*, void*, void*)");
    d("void* ore_map_filter(void*, void*, void*)");

    // Concurrency
    d("void ore_spawn(void*)");
    d("void ore_spawn_with_arg(void*, int64_t)");
    d("void ore_spawn_with_2args(void*, int64_t, int64_t)");
    d("void ore_spawn_with_3args(void*, int64_t, int64_t, int64_t)");
    d("void ore_thread_join_all(void)");
    d("void ore_sleep(int64_t)");
    d("void* ore_channel_new(void)");
    d("void ore_channel_send(void*, int64_t)");
    d("int64_t ore_channel_recv(void*)");

    // Math
    d("int64_t ore_int_pow(int64_t, int64_t)");
    d("double ore_math_sqrt(double)");
    d("double ore_math_sin(double)");
    d("double ore_math_cos(double)");
    d("double ore_math_tan(double)");
    d("double ore_math_log(double)");
    d("double ore_math_log10(double)");
    d("double ore_math_exp(double)");
    d("double ore_math_pow(double, double)");
    d("double ore_math_abs(double)");
    d("double ore_math_floor(double)");
    d("double ore_math_ceil(double)");
    d("double ore_math_round(double)");
    d("double ore_math_pi(void)");
    d("double ore_math_e(void)");
    d("double ore_math_atan2(double, double)");
    d("double ore_float_round_to(double, int64_t)");
    d("void* ore_float_format(double, int64_t)");

    // I/O
    d("void* ore_readln(void)");
    d("void* ore_file_read(void*)");
    d("void* ore_file_read_lines(void*)");
    d("int8_t ore_file_write(void*, void*)");
    d("int8_t ore_file_exists(void*)");
    d("int8_t ore_file_append(void*, void*)");

    // Process / environment
    d("void* ore_args(void)");
    d("void ore_exit(int64_t)");
    d("void* ore_exec(void*)");
    d("void* ore_type_of(int8_t)");
    d("void* ore_env_get(void*)");
    d("void ore_env_set(void*, void*)");

    // JSON
    d("void* ore_json_parse(void*)");
    d("void* ore_json_stringify(void*)");

    // Time
    d("int64_t ore_time_now(void)");
    d("int64_t ore_time_ms(void)");

    // Random
    d("int64_t ore_rand_int(int64_t, int64_t)");

    decls
}
