use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_list_method(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
        elem_kind: &ValKind,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "len" => {
                let val = self.call_rt("ore_list_len", &[list_val.into()], "len")?;
                Ok((val, ValKind::Int))
            }
            "is_empty" => {
                let len_val = self.call_rt("ore_list_len", &[list_val.into()], "len")?.into_int_value();
                let is_zero = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    len_val,
                    self.context.i64_type().const_zero(),
                    "is_empty"
                ))?;
                Ok((is_zero.into(), ValKind::Bool))
            }
            "clear" => {
                self.call_rt("ore_list_clear", &[list_val.into()], "")?;
                Ok((list_val, ValKind::list_of(elem_kind.clone())))
            }
            "push" => {
                self.check_arity("push", args, 1)?;
                let (arg, arg_kind) = self.compile_expr_with_kind(&args[0], func)?;
                let push_val = self.val_to_list_i64(arg, &arg_kind)?;
                self.call_rt("ore_list_push", &[list_val.into(), push_val.into()], "")?;
                Ok((list_val, ValKind::list_of(arg_kind)))
            }
            "pop" => {
                let raw_val = self.call_rt("ore_list_pop", &[list_val.into()], "pop")?;
                let typed_val = self.list_elem_from_i64(raw_val, elem_kind)?;
                Ok((typed_val, elem_kind.clone()))
            }
            "insert" => {
                self.check_arity("insert", args, 2)?;
                let idx = self.compile_expr(&args[0], func)?;
                let (val, _) = self.compile_expr_with_kind(&args[1], func)?;
                let i64_val = self.value_to_i64(val)?;
                self.call_rt("ore_list_insert", &[list_val.into(), idx.into(), i64_val.into()], "")?;
                Ok((list_val, ValKind::list_of(elem_kind.clone())))
            }
            "remove_at" => {
                self.check_arity("remove_at", args, 1)?;
                let idx = self.compile_expr(&args[0], func)?;
                let raw_val = self.call_rt("ore_list_remove_at", &[list_val.into(), idx.into()], "removed")?;
                self.coerce_list_element(raw_val, elem_kind.clone())
            }
            "get" => {
                self.check_arity("get", args, 1)?;
                let idx = self.compile_expr(&args[0], func)?;
                let raw_val = self.call_rt("ore_list_get", &[list_val.into(), idx.into()], "get")?;
                let typed_val = self.list_elem_from_i64(raw_val, elem_kind)?;
                Ok((typed_val, elem_kind.clone()))
            }
            "set" => {
                self.check_arity("set", args, 2)?;
                let idx = self.compile_expr(&args[0], func)?;
                let val = self.compile_expr(&args[1], func)?;
                let val_i64 = self.value_to_i64(val)?;
                self.call_rt("ore_list_set", &[list_val.into(), idx.into(), val_i64.into()], "")?;
                Ok((list_val, ValKind::list_of(elem_kind.clone())))
            }
            "get_or" => {
                self.check_arity("get_or", args, 2)?;
                let idx = self.compile_expr(&args[0], func)?;
                let default = self.compile_expr(&args[1], func)?;
                let default_i64 = self.value_to_i64(default)?;
                let raw_val = self.call_rt("ore_list_get_or", &[list_val.into(), idx.into(), default_i64.into()], "getor")?;
                let typed_val = self.list_elem_from_i64(raw_val, elem_kind)?;
                Ok((typed_val, elem_kind.clone()))
            }
            "map" | "filter" | "flat_map" | "take_while" | "drop_while" => {
                self.check_arity(method, args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, ret_kind) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let runtime_fn_name = format!("ore_list_{}", method);
                let val = self.call_rt(&runtime_fn_name, &[list_val.into(), fn_ptr.into(), env_ptr.into()], method)?;
                // For map, update element kind based on lambda return type
                let result_elem = if method == "map" {
                    ret_kind
                } else {
                    elem_kind.clone()
                };
                Ok((val, ValKind::list_of(result_elem)))
            }
            "partition" => {
                self.check_arity("partition", args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let val = self.call_rt("ore_list_partition", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "part")?;
                let inner_elem = Some(Box::new(elem_kind.clone()));
                Ok((val, ValKind::list_of(ValKind::List(inner_elem))))
            }
            "find_index" => {
                self.check_arity("find_index", args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let val = self.call_rt("ore_list_find_index", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "fidx")?;
                Ok((val, ValKind::Int))
            }
            "fold" => {
                self.check_arity("fold", args, 2)?;
                let (init_val, _) = self.compile_expr_with_kind(&args[0], func)?;
                // fold lambda receives (acc, elem) — both as Int/i64
                let kinds = [ValKind::Int, elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[1], &kinds, method)?;

                let val = self.call_rt("ore_list_fold", &[list_val.into(), init_val.into(), fn_ptr.into(), env_ptr.into()], "fold")?;
                Ok((val, ValKind::Int))
            }
            "each" => {
                self.check_arity("each", args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                self.call_rt("ore_list_each", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "")?;
                Ok(self.void_result())
            }
            "tap" => {
                self.check_arity("tap", args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let val = self.call_rt("ore_list_tap", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "tap")?;
                Ok((val, ValKind::list_of(elem_kind.clone())))
            }
            "map_with_index" => {
                self.check_arity("map_with_index", args, 1)?;
                let kinds = [ValKind::Int, elem_kind.clone()];
                let (fn_ptr, env_ptr, ret_kind) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let val = self.call_rt("ore_list_map_with_index", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "mwi")?;
                let ret_elem = ret_kind;
                Ok((val, ValKind::list_of(ret_elem)))
            }
            "each_with_index" => {
                self.check_arity("each_with_index", args, 1)?;
                let kinds = [ValKind::Int, elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                self.call_rt("ore_list_each_with_index", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "")?;
                Ok(self.void_result())
            }
            "par_map" => {
                self.check_arity("par_map", args, 1)?;
                let (fn_ptr, env_ptr, ret_kind) = self.resolve_lambda_arg(&args[0], &[ValKind::Int], method)?;

                let val = self.call_rt("ore_list_par_map", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "par_map")?;
                let ret_elem = ret_kind;
                Ok((val, ValKind::list_of(ret_elem)))
            }
            "par_each" => {
                self.check_arity("par_each", args, 1)?;
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &[ValKind::Int], method)?;

                self.call_rt("ore_list_par_each", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "")?;
                Ok(self.void_result())
            }
            "sort" => {
                if args.is_empty() {
                    let rt_name = match elem_kind {
                        ValKind::Str => "ore_list_sort_str",
                        ValKind::Float => "ore_list_sort_float",
                        _ => "ore_list_sort",
                    };
                    let sorted_val = self.call_rt(rt_name, &[list_val.into()], "sorted")?;
                    return Ok((sorted_val, ValKind::list_of(elem_kind.clone())));
                }
                // sort(comparator) - sort_by
                let ek = elem_kind.clone();
                let kinds = [ek.clone(), ek];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let sorted_val = self.call_rt("ore_list_sort_by", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "sorted")?;
                Ok((sorted_val, ValKind::list_of(elem_kind.clone())))
            }
            "sort_by" => {
                // sort_by(key_fn) - sort by a key extracted from each element
                self.check_arity("sort_by", args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let sorted_val = self.call_rt("ore_list_sort_by_key", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "sorted")?;
                Ok((sorted_val, ValKind::list_of(elem_kind.clone())))
            }
            "min_by" | "max_by" => {
                self.check_arity(method, args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let fn_name = if method == "min_by" { "ore_list_min_by" } else { "ore_list_max_by" };
                let val = self.call_rt(fn_name, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "mby")?;
                self.coerce_list_element(val, elem_kind.clone())
            }
            "reverse" => {
                let rev_val = self.call_rt("ore_list_reverse_new", &[list_val.into()], "reversed")?;
                Ok((rev_val, ValKind::list_of(elem_kind.clone())))
            }
            "contains" => {
                self.check_arity("contains", args, 1)?;
                let (val, _) = self.compile_expr_with_kind(&args[0], func)?;
                let i8_val = if matches!(elem_kind, ValKind::Str) {
                    self.call_rt("ore_list_contains_str", &[list_val.into(), val.into()], "lcontains")?
                } else {
                    let i64_val = self.value_to_i64(val)?;
                    self.call_rt("ore_list_contains", &[list_val.into(), i64_val.into()], "lcontains")?
                }.into_int_value();
                let bool_val = self.i8_to_bool(i8_val)?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "reduce" => {
                // reduce(fn) uses first element as init, or reduce(init, fn)
                if args.is_empty() || args.len() > 2 {
                    return Err(self.err("reduce takes 1 or 2 arguments"));
                }
                let fn_arg = if args.len() == 1 { &args[0] } else { &args[1] };
                let kinds = [ValKind::Int, elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(fn_arg, &kinds, method)?;

                if args.len() == 1 {
                    let val = self.call_rt("ore_list_reduce1", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "reduce")?;
                    Ok((val, ValKind::Int))
                } else {
                    let init_val = self.compile_expr(&args[0], func)?;
                    let val = self.call_rt("ore_list_fold", &[list_val.into(), init_val.into(), fn_ptr.into(), env_ptr.into()], "reduce")?;
                    Ok((val, ValKind::Int))
                }
            }
            "scan" => {
                // scan(init, fn(acc, elem) -> acc) -> list of all acc values
                self.check_arity("scan", args, 2)?;
                let init_val = self.compile_expr(&args[0], func)?;
                let kinds = [ValKind::Int, elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[1], &kinds, method)?;

                let val = self.call_rt("ore_list_scan", &[list_val.into(), init_val.into(), fn_ptr.into(), env_ptr.into()], "scan")?;
                Ok((val, ValKind::list_of(ValKind::Int)))
            }
            "find" => {
                // find(fn(elem) -> bool) — returns element or 0
                self.check_arity("find", args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let default_val = self.context.i64_type().const_int(0, false);
                let val = self.call_rt("ore_list_find", &[list_val.into(), fn_ptr.into(), env_ptr.into(), default_val.into()], "find")?;
                self.coerce_list_element(val, elem_kind.clone())
            }
            "join" => {
                // join(separator_str)
                self.check_arity("join", args, 1)?;
                let sep = self.compile_expr(&args[0], func)?;
                // Use join_str for string lists, join for int lists
                let fn_name = match &elem_kind {
                    ValKind::Str => "ore_list_join_str",
                    ValKind::Float => "ore_list_join_float",
                    _ => "ore_list_join",
                };
                let val = self.call_rt(fn_name, &[list_val.into(), sep.into()], "join")?;
                Ok((val, ValKind::Str))
            }
            "take" | "skip" | "step" => {
                self.check_arity(method, args, 1)?;
                let n = self.compile_expr(&args[0], func)?;
                let runtime_fn_name = format!("ore_list_{}", method);
                let val = self.call_rt(&runtime_fn_name, &[list_val.into(), n.into()], method)?;
                Ok((val, ValKind::list_of(elem_kind.clone())))
            }
            "sum" | "product" => {
                let (rt_name, result_kind) = if matches!(elem_kind, ValKind::Float) {
                    (format!("ore_list_{}_float", method), ValKind::Float)
                } else {
                    (format!("ore_list_{}", method), ValKind::Int)
                };
                let val = self.call_rt(&rt_name, &[list_val.into()], method)?;
                Ok((val, result_kind))
            }
            "average" => {
                let rt_name = if matches!(elem_kind, ValKind::Float) { "ore_list_average_float" } else { "ore_list_average" };
                let val = self.call_rt(rt_name, &[list_val.into()], "avg")?;
                Ok((val, ValKind::Float))
            }
            "any" | "all" => {
                self.check_arity(method, args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let runtime_fn_name = format!("ore_list_{}", method);
                let val = self.call_rt(&runtime_fn_name, &[list_val.into(), fn_ptr.into(), env_ptr.into()], method)?;
                let bool_val = bld!(self.builder.build_int_truncate(val.into_int_value(), self.context.bool_type(), &format!("{}_bool", method)))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "zip" => {
                self.check_arity("zip", args, 1)?;
                let other = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_list_zip", &[list_val.into(), other.into()], "zip")?;
                Ok((val, ValKind::list_of(ValKind::List(None))))
            }
            "zip_with" => {
                self.check_arity("zip_with", args, 2)?;
                let other = self.compile_expr(&args[0], func)?;
                let ek = elem_kind.clone();
                let kinds = [ek.clone(), ek];
                let (fn_ptr, env_ptr, ret_kind) = self.resolve_lambda_arg(&args[1], &kinds, method)?;

                let val = self.call_rt("ore_list_zip_with", &[list_val.into(), other.into(), fn_ptr.into(), env_ptr.into()], "zipw")?;
                let result_elem = ret_kind;
                Ok((val, ValKind::list_of(result_elem)))
            }
            "enumerate" => {
                let val = self.call_rt("ore_list_enumerate", &[list_val.into()], "enum")?;
                Ok((val, ValKind::list_of(ValKind::List(None))))
            }
            "slice" => {
                self.check_arity("slice", args, 2)?;
                let start = self.compile_expr(&args[0], func)?;
                let end = self.compile_expr(&args[1], func)?;
                let val = self.call_rt("ore_list_slice", &[list_val.into(), start.into(), end.into()], "lslice")?;
                Ok((val, ValKind::list_of(elem_kind.clone())))
            }
            "index_of" => {
                self.check_arity("index_of", args, 1)?;
                let val = self.compile_expr(&args[0], func)?;
                let rt_name = if matches!(elem_kind, ValKind::Str) { "ore_list_index_of_str" } else { "ore_list_index_of" };
                let v = self.call_rt(rt_name, &[list_val.into(), val.into()], "lidx")?;
                Ok((v, ValKind::Int))
            }
            "unique" => {
                let rt_name = if matches!(elem_kind, ValKind::Str) { "ore_list_unique_str" } else { "ore_list_unique" };
                let val = self.call_rt(rt_name, &[list_val.into()], "luniq")?;
                Ok((val, ValKind::list_of(elem_kind.clone())))
            }
            "unique_by" => {
                self.check_arity("unique_by", args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let val = self.call_rt("ore_list_unique_by", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "uniqby")?;
                Ok((val, ValKind::list_of(elem_kind.clone())))
            }
            "flatten" => {
                let val = self.call_rt("ore_list_flatten", &[list_val.into()], "lflat")?;
                Ok((val, ValKind::List(None)))
            }
            "window" | "chunks" => {
                self.check_arity(method, args, 1)?;
                let n = self.compile_expr(&args[0], func)?;
                let rt_name = format!("ore_list_{}", method);
                let val = self.call_rt(&rt_name, &[list_val.into(), n.into()], method)?;
                Ok((val, ValKind::list_of(ValKind::List(None))))
            }
            "first" | "last" => {
                let idx = if method == "first" {
                    self.context.i64_type().const_int(0, false)
                } else {
                    self.context.i64_type().const_int((-1i64) as u64, true)
                };
                let val = self.call_rt("ore_list_get", &[list_val.into(), idx.into()], method)?;
                let ek = elem_kind.clone();
                self.coerce_list_element(val, ek)
            }
            "min" | "max" => {
                let (rt_name, result_kind) = match elem_kind {
                    ValKind::Float => (format!("ore_list_{}_float", method), ValKind::Float),
                    ValKind::Str => (format!("ore_list_{}_str", method), ValKind::Str),
                    _ => (format!("ore_list_{}", method), ValKind::Int),
                };
                let val = self.call_rt(&rt_name, &[list_val.into()], method)?;
                Ok((val, result_kind))
            }
            "count" => {
                // count() with no args returns list length, count(pred) counts matching
                if args.is_empty() {
                    let val = self.call_rt("ore_list_len", &[list_val.into()], "count")?;
                    return Ok((val, ValKind::Int));
                }
                self.check_arity("count", args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let val = self.call_rt("ore_list_count", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "count")?;
                Ok((val, ValKind::Int))
            }
            "count_by" | "group_by" => {
                self.check_arity(method, args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, ret_kind) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                // Use _int variant for count_by when lambda returns non-string
                let rt_name = if method == "count_by" && ret_kind != ValKind::Str {
                    "ore_list_count_by_int".to_string()
                } else {
                    format!("ore_list_{}", method)
                };
                let val = self.call_rt(&rt_name, &[list_val.into(), fn_ptr.into(), env_ptr.into()], method)?;
                let val_kind = if method == "count_by" { ValKind::Int } else { ValKind::list_of(elem_kind.clone()) };
                Ok((val, ValKind::map_of(val_kind)))
            }
            "to_map" => {
                self.check_arity("to_map", args, 1)?;
                let kinds = [elem_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, method)?;

                let val = self.call_rt("ore_list_to_map", &[list_val.into(), fn_ptr.into(), env_ptr.into()], "tomap")?;
                Ok((val, ValKind::map_of(elem_kind.clone())))
            }
            "dedup" => {
                let val = self.call_rt("ore_list_dedup", &[list_val.into()], "dedup")?;
                Ok((val, ValKind::list_of(elem_kind.clone())))
            }
            "frequencies" => {
                let kind_tag = self.valkind_to_tag(elem_kind);
                let kind_val = self.context.i8_type().const_int(kind_tag as u64, false);
                let val = self.call_rt("ore_list_frequencies", &[list_val.into(), kind_val.into()], "freq")?;
                Ok((val, ValKind::map_of(ValKind::Int)))
            }
            "intersperse" => {
                self.check_arity("intersperse", args, 1)?;
                let (sep_val, sep_kind) = self.compile_expr_with_kind(&args[0], func)?;
                let sep_i64: IntValue = match sep_kind {
                    ValKind::Str | ValKind::List(_) | ValKind::Map(_) => {
                        self.ptr_to_i64(sep_val.into_pointer_value())?
                    }
                    _ => sep_val.into_int_value(),
                };
                let val = self.call_rt("ore_list_intersperse", &[list_val.into(), sep_i64.into()], "inter")?;
                Ok((val, ValKind::list_of(elem_kind.clone())))
            }
            _ => Err(Self::unknown_method_error("List", method, &[
                "len", "push", "pop", "insert", "remove_at", "clear", "join", "reverse", "sort",
                "sort_by", "map", "filter", "reduce", "fold", "scan", "each", "any", "all",
                "find", "find_index", "index_of", "contains", "count", "sum", "product",
                "min", "max", "average", "unique", "dedup", "flatten", "flat_map",
                "take", "skip", "take_while", "drop_while", "slice", "zip", "zip_with",
                "first", "last", "enumerate", "window", "chunks", "frequencies",
                "intersperse", "partition", "group_by", "to_map", "step",
            ])),
        }
    }

    pub(crate) fn compile_list_lit(
        &mut self,
        elements: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let list_push = self.rt("ore_list_push")?;
        let list_ptr = self.call_rt("ore_list_new", &[], "list")?.into_pointer_value();

        let mut elem_kind: Option<ValKind> = None;
        for elem in elements {
            let (val, kind) = self.compile_expr_with_kind(elem, func)?;
            elem_kind = Some(kind.clone());
            let push_val = self.val_to_list_i64(val, &kind)?;
            bld!(self.builder.build_call(list_push, &[list_ptr.into(), push_val.into()], ""))?;
        }

        Ok((list_ptr.into(), ValKind::List(elem_kind.map(Box::new))))
    }

    pub(crate) fn compile_list_comp(
        &mut self,
        expr: &Expr,
        var: &str,
        iterable: &Expr,
        cond: Option<&Expr>,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let i64_type = self.context.i64_type();
        let list_push = self.rt("ore_list_push")?;

        // Create output list
        let list_ptr = self.call_rt("ore_list_new", &[], "comp_list")?.into_pointer_value();

        // Check if iterable is a range (__range call)
        let is_range = matches!(iterable, Expr::Call { func: f, .. } if matches!(f.as_ref(), Expr::Ident(n) if n == "__range"));

        // Track the output element kind from the body expression (set in both branches)
        let output_elem_kind;

        if is_range {
            // Range-based comprehension
            let (start_val, end_val) = if let Expr::Call { args, .. } = iterable {
                let s = self.compile_expr(&args[0], func)?.into_int_value();
                let e = self.compile_expr(&args[1], func)?.into_int_value();
                (s, e)
            } else {
                unreachable!()
            };

            // Loop variable
            let var_alloca = bld!(self.builder.build_alloca(i64_type, var))?;
            bld!(self.builder.build_store(var_alloca, start_val))?;
            self.variables.insert(var.to_string(), VarInfo { ptr: var_alloca, ty: i64_type.into(), kind: ValKind::Int, is_mutable: false });

            let cond_bb = self.context.append_basic_block(func, "comp_cond");
            let body_bb = self.context.append_basic_block(func, "comp_body");
            let inc_bb = self.context.append_basic_block(func, "comp_inc");
            let end_bb = self.context.append_basic_block(func, "comp_end");

            bld!(self.builder.build_unconditional_branch(cond_bb))?;

            // Condition: var < end
            self.builder.position_at_end(cond_bb);
            let cur = bld!(self.builder.build_load(i64_type, var_alloca, "cur"))?.into_int_value();
            let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, cur, end_val, "comp_cmp"))?;
            bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

            // Body
            self.builder.position_at_end(body_bb);
            let push_bb = if let Some(c) = cond {
                let filter_bb = self.context.append_basic_block(func, "comp_filter");
                let (cond_val, _) = self.compile_expr_with_kind(c, func)?;
                let bool_val = self.normalize_to_bool(cond_val)?;
                bld!(self.builder.build_conditional_branch(bool_val, filter_bb, inc_bb))?;
                self.builder.position_at_end(filter_bb);
                filter_bb
            } else {
                body_bb
            };

            let (val, kind) = self.compile_expr_with_kind(expr, func)?;
            output_elem_kind = kind.clone();
            let push_val = self.val_to_list_i64(val, &kind)?;
            bld!(self.builder.build_call(list_push, &[list_ptr.into(), push_val.into()], ""))?;

            if self.current_block()?.get_terminator().is_none() {
                bld!(self.builder.build_unconditional_branch(inc_bb))?;
            }

            // Increment
            self.builder.position_at_end(inc_bb);
            let cur = bld!(self.builder.build_load(i64_type, var_alloca, "cur"))?.into_int_value();
            let next = bld!(self.builder.build_int_add(cur, i64_type.const_int(1, false), "inc"))?;
            bld!(self.builder.build_store(var_alloca, next))?;
            bld!(self.builder.build_unconditional_branch(cond_bb))?;

            self.builder.position_at_end(end_bb);
            let _ = push_bb; // suppress unused warning
        } else {
            // List-based comprehension
            let (list_val, list_kind) = self.compile_expr_with_kind(iterable, func)?;
            let list_src = list_val.into_pointer_value();
            let elem_kind = match &list_kind {
                ValKind::List(Some(ek)) => ek.as_ref().clone(),
                _ => ValKind::Int,
            };

            let len_val = self.call_rt("ore_list_len", &[list_src.into()], "len")?.into_int_value();

            let idx_alloca = bld!(self.builder.build_alloca(i64_type, "comp_idx"))?;
            bld!(self.builder.build_store(idx_alloca, i64_type.const_int(0, false)))?;

            // Element variable
            let (var_alloca, var_ty): (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>) = match &elem_kind {
                ValKind::Str => {
                    let pt = self.context.ptr_type(inkwell::AddressSpace::default());
                    (bld!(self.builder.build_alloca(pt, var))?, pt.into())
                }
                _ => (bld!(self.builder.build_alloca(i64_type, var))?, i64_type.into()),
            };
            self.variables.insert(var.to_string(), VarInfo { ptr: var_alloca, ty: var_ty, kind: elem_kind.clone(), is_mutable: false });

            let cond_bb = self.context.append_basic_block(func, "comp_cond");
            let body_bb = self.context.append_basic_block(func, "comp_body");
            let inc_bb = self.context.append_basic_block(func, "comp_inc");
            let end_bb = self.context.append_basic_block(func, "comp_end");

            bld!(self.builder.build_unconditional_branch(cond_bb))?;

            self.builder.position_at_end(cond_bb);
            let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
            let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, idx, len_val, "comp_cmp"))?;
            bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

            self.builder.position_at_end(body_bb);
            let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
            let raw_val = self.call_rt("ore_list_get", &[list_src.into(), idx.into()], "elem")?;

            match &elem_kind {
                ValKind::Str => {
                    let ptr = self.i64_to_ptr(raw_val.into_int_value())?;
                    bld!(self.builder.build_store(var_alloca, ptr))?;
                }
                _ => {
                    bld!(self.builder.build_store(var_alloca, raw_val))?;
                }
            }

            if let Some(c) = cond {
                let filter_bb = self.context.append_basic_block(func, "comp_filter");
                let (cond_val, _) = self.compile_expr_with_kind(c, func)?;
                let bool_val = self.normalize_to_bool(cond_val)?;
                bld!(self.builder.build_conditional_branch(bool_val, filter_bb, inc_bb))?;
                self.builder.position_at_end(filter_bb);
            }

            let (val, kind) = self.compile_expr_with_kind(expr, func)?;
            output_elem_kind = kind.clone();
            let push_val = self.val_to_list_i64(val, &kind)?;
            bld!(self.builder.build_call(list_push, &[list_ptr.into(), push_val.into()], ""))?;

            if self.current_block()?.get_terminator().is_none() {
                bld!(self.builder.build_unconditional_branch(inc_bb))?;
            }

            self.builder.position_at_end(inc_bb);
            let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
            let next = bld!(self.builder.build_int_add(idx, i64_type.const_int(1, false), "inc"))?;
            bld!(self.builder.build_store(idx_alloca, next))?;
            bld!(self.builder.build_unconditional_branch(cond_bb))?;

            self.builder.position_at_end(end_bb);
        }

        Ok((list_ptr.into(), ValKind::list_of(output_elem_kind)))
    }

}
