use super::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    /// Declare all ore_runtime extern functions in the LLVM module.
    /// These are resolved at JIT time via `map_runtime_functions` in ore_cli,
    /// or at link time via libore_runtime.a for AOT builds.
    pub(crate) fn declare_runtime_functions(&mut self) {
        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();
        let f64_type = self.context.f64_type();
        let void_type = self.context.void_type();
        let ptr_type = self.ptr_type();

        let ext = Some(inkwell::module::Linkage::External);

        // Print primitives
        self.module.add_function("ore_print_int", void_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_print_bool", void_type.fn_type(&[i8_type.into()], false), ext);
        self.module.add_function("ore_print_float", void_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_print_int_no_newline", void_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_print_float_no_newline", void_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_print_bool_no_newline", void_type.fn_type(&[i8_type.into()], false), ext);

        // Stderr
        self.module.add_function("ore_eprint_str", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_eprint_int", void_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_eprint_float", void_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_eprint_bool", void_type.fn_type(&[i8_type.into()], false), ext);

        // Runtime errors
        self.module.add_function("ore_div_by_zero", void_type.fn_type(&[], false), ext);

        // Strings — creation, RC, conversion
        self.module.add_function("ore_str_new", ptr_type.fn_type(&[ptr_type.into(), i32_type.into()], false), ext);
        self.module.add_function("ore_str_concat", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_print", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_print_no_newline", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_retain", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_release", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_int_to_str", ptr_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_bool_to_str", ptr_type.fn_type(&[i8_type.into()], false), ext);
        self.module.add_function("ore_float_to_str", ptr_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_dynamic_to_str", ptr_type.fn_type(&[i64_type.into(), i8_type.into()], false), ext);

        // String methods
        self.module.add_function("ore_str_len", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_eq", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_cmp", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_contains", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_trim", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_trim_start", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_trim_end", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_lines", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_split", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_split_whitespace", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_to_int", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_to_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_replace", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_starts_with", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_ends_with", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_to_upper", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_to_lower", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_substr", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_str_chars", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_repeat", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_str_pad_left", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_pad_right", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_count", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_strip_prefix", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_strip_suffix", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_index_of", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_slice", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_str_reverse", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_char_at", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_str_capitalize", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_ord", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_chr", ptr_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_str_parse_int", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_parse_float", f64_type.fn_type(&[ptr_type.into()], false), ext);

        // Assert
        self.module.add_function("ore_assert", void_type.fn_type(&[i8_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_assert_eq_int", void_type.fn_type(&[i64_type.into(), i64_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_assert_eq_float", void_type.fn_type(&[f64_type.into(), f64_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_assert_eq_str", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_assert_ne_int", void_type.fn_type(&[i64_type.into(), i64_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_assert_ne_str", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_assert_fail", void_type.fn_type(&[ptr_type.into()], false), ext);

        // Lists — basic operations
        self.module.add_function("ore_list_new", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_list_push", void_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_pop", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_clear", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_insert", void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_remove_at", i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_get", i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_set", void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_get_or", i64_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_len", i64_type.fn_type(&[ptr_type.into()], false), ext);

        // Lists — print
        self.module.add_function("ore_list_print", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_print_typed", void_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_print_str", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_print_float", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_print_bool", void_type.fn_type(&[ptr_type.into()], false), ext);

        // Lists — higher-order methods (fn_ptr, env_ptr)
        self.module.add_function("ore_list_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_filter", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_each", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_find_index", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_find", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_fold", i64_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_reduce", i64_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_reduce1", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_scan", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_any", i8_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_all", i8_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_flat_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_count", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_count_by", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_count_by_int", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_group_by", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_by", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_by_key", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_by_key_str", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_min_by", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_max_by", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_partition", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_take_while", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_drop_while", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_tap", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_map_with_index", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_each_with_index", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_unique_by", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_to_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_par_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_par_each", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_zip_with", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);

        // Lists — non-higher-order methods
        self.module.add_function("ore_list_sort", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_str", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_float", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_reverse", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_reverse_new", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_contains", i8_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_contains_str", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_concat", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_zip", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_enumerate", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_join", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_join_str", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_join_float", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_take", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_skip", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_step", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_slice", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_window", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_chunks", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_flatten", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_index_of", i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_index_of_str", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_unique", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_unique_str", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_dedup", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_intersperse", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_frequencies", ptr_type.fn_type(&[ptr_type.into(), i8_type.into()], false), ext);

        // Lists — aggregation
        self.module.add_function("ore_list_sum", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_product", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_min", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_max", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sum_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_product_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_min_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_max_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_min_str", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_max_str", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_average", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_average_float", f64_type.fn_type(&[ptr_type.into()], false), ext);

        // Lists — range/repeat
        self.module.add_function("ore_range", ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_range_step", ptr_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_repeat", ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false), ext);

        // Maps
        self.module.add_function("ore_map_new", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_map_set", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_map_set_typed", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into(), i8_type.into()], false), ext);
        self.module.add_function("ore_map_get", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_get_or", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_map_contains", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_len", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_remove", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_keys", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_values", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_entries", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_print", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_print_str", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_merge", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_clear", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_each", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_map_values", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_filter", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);

        // Concurrency
        self.module.add_function("ore_spawn", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_spawn_with_arg", void_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_spawn_with_2args", void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_spawn_with_3args", void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_thread_join_all", void_type.fn_type(&[], false), ext);
        self.module.add_function("ore_sleep", void_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_channel_new", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_channel_send", void_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_channel_recv", i64_type.fn_type(&[ptr_type.into()], false), ext);

        // Math
        self.module.add_function("ore_int_pow", i64_type.fn_type(&[i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_math_sqrt", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_sin", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_cos", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_tan", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_log", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_log10", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_exp", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_pow", f64_type.fn_type(&[f64_type.into(), f64_type.into()], false), ext);
        self.module.add_function("ore_math_abs", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_floor", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_ceil", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_round", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_pi", f64_type.fn_type(&[], false), ext);
        self.module.add_function("ore_math_e", f64_type.fn_type(&[], false), ext);
        self.module.add_function("ore_math_atan2", f64_type.fn_type(&[f64_type.into(), f64_type.into()], false), ext);
        self.module.add_function("ore_float_round_to", f64_type.fn_type(&[f64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_float_format", ptr_type.fn_type(&[f64_type.into(), i64_type.into()], false), ext);

        // I/O
        self.module.add_function("ore_readln", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_file_read", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_file_read_lines", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_file_write", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_file_exists", i8_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_file_append", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);

        // Process / environment
        self.module.add_function("ore_args", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_exit", void_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_exec", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_type_of", ptr_type.fn_type(&[i8_type.into()], false), ext);
        self.module.add_function("ore_env_get", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_env_set", void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);

        // JSON
        self.module.add_function("ore_json_parse", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_json_stringify", ptr_type.fn_type(&[ptr_type.into()], false), ext);

        // Time
        self.module.add_function("ore_time_now", i64_type.fn_type(&[], false), ext);
        self.module.add_function("ore_time_ms", i64_type.fn_type(&[], false), ext);

        // Random
        self.module.add_function("ore_rand_int", i64_type.fn_type(&[i64_type.into(), i64_type.into()], false), ext);
    }
}
