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
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "len" => {
                let list_len = self.rt("ore_list_len")?;
                let result = bld!(self.builder.build_call(list_len, &[list_val.into()], "len"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "is_empty" => {
                let list_len = self.rt("ore_list_len")?;
                let result = bld!(self.builder.build_call(list_len, &[list_val.into()], "len"))?;
                let len_val = self.call_result_to_value(result)?.into_int_value();
                let is_zero = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    len_val,
                    self.context.i64_type().const_zero(),
                    "is_empty"
                ))?;
                Ok((is_zero.into(), ValKind::Bool))
            }
            "clear" => {
                let rt = self.rt("ore_list_clear")?;
                bld!(self.builder.build_call(rt, &[list_val.into()], ""))?;
                Ok((list_val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "push" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "push takes exactly 1 argument".into() });
                }
                let (arg, arg_kind) = self.compile_expr_with_kind(&args[0], func)?;
                let list_push = self.rt("ore_list_push")?;
                // For enums/records, heap-allocate and push pointer as i64
                let push_val: BasicValueEnum = match &arg_kind {
                    ValKind::Enum(name) => {
                        let et = self.enums[name].enum_type;
                        let heap_ptr = bld!(self.builder.build_malloc(et, "heap_enum"))?;
                        bld!(self.builder.build_store(heap_ptr, arg))?;
                        let i64_val = bld!(self.builder.build_ptr_to_int(heap_ptr, self.context.i64_type(), "p2i"))?;
                        i64_val.into()
                    }
                    ValKind::Record(name) => {
                        let st = self.records[name].struct_type;
                        let heap_ptr = bld!(self.builder.build_malloc(st, "heap_rec"))?;
                        bld!(self.builder.build_store(heap_ptr, arg))?;
                        let i64_val = bld!(self.builder.build_ptr_to_int(heap_ptr, self.context.i64_type(), "p2i"))?;
                        i64_val.into()
                    }
                    ValKind::Float => {
                        let i64_val = bld!(self.builder.build_bit_cast(arg, self.context.i64_type(), "f2i"))?;
                        i64_val
                    }
                    ValKind::Bool => {
                        let i64_val = bld!(self.builder.build_int_z_extend(arg.into_int_value(), self.context.i64_type(), "b2i"))?;
                        i64_val.into()
                    }
                    ValKind::Str => {
                        let i64_val = bld!(self.builder.build_ptr_to_int(arg.into_pointer_value(), self.context.i64_type(), "p2i"))?;
                        i64_val.into()
                    }
                    _ => arg,
                };
                bld!(self.builder.build_call(list_push, &[list_val.into(), push_val.into()], ""))?;
                // Track element kind so join/pop/iteration know the type
                self.last_list_elem_kind = Some(arg_kind.clone());
                Ok((list_val, ValKind::list_of(arg_kind)))
            }
            "pop" => {
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let list_pop = self.rt("ore_list_pop")?;
                let result = bld!(self.builder.build_call(list_pop, &[list_val.into()], "pop"))?;
                let raw_val = self.call_result_to_value(result)?;
                let typed_val = self.list_elem_from_i64(raw_val, &elem_kind)?;
                Ok((typed_val, elem_kind))
            }
            "insert" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "insert takes 2 arguments (index, value)".into() });
                }
                let idx = self.compile_expr(&args[0], func)?;
                let (val, _) = self.compile_expr_with_kind(&args[1], func)?;
                let i64_val = self.value_to_i64(val)?;
                let rt = self.rt("ore_list_insert")?;
                bld!(self.builder.build_call(rt, &[list_val.into(), idx.into(), i64_val.into()], ""))?;
                Ok((list_val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "remove_at" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "remove_at takes 1 argument (index)".into() });
                }
                let idx = self.compile_expr(&args[0], func)?;
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let rt = self.rt("ore_list_remove_at")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), idx.into()], "removed"))?;
                let raw_val = self.call_result_to_value(result)?;
                match &elem_kind {
                    ValKind::Str => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            raw_val.into_int_value(),
                            self.context.ptr_type(inkwell::AddressSpace::default()), "i2p"
                        ))?;
                        Ok((ptr.into(), ValKind::Str))
                    }
                    _ => Ok((raw_val, elem_kind))
                }
            }
            "get" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "get takes exactly 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let idx = self.compile_expr(&args[0], func)?;
                let list_get = self.rt("ore_list_get")?;
                let result = bld!(self.builder.build_call(list_get, &[list_val.into(), idx.into()], "get"))?;
                let raw_val = self.call_result_to_value(result)?;
                let typed_val = self.list_elem_from_i64(raw_val, &elem_kind)?;
                Ok((typed_val, elem_kind))
            }
            "set" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "set takes 2 arguments (index, value)".into() });
                }
                let idx = self.compile_expr(&args[0], func)?;
                let val = self.compile_expr(&args[1], func)?;
                let val_i64 = self.value_to_i64(val)?;
                let rt = self.rt("ore_list_set")?;
                bld!(self.builder.build_call(rt, &[list_val.into(), idx.into(), val_i64.into()], ""))?;
                Ok((list_val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "get_or" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "get_or takes 2 arguments (index, default)".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let idx = self.compile_expr(&args[0], func)?;
                let default = self.compile_expr(&args[1], func)?;
                let default_i64 = self.value_to_i64(default)?;
                let rt = self.rt("ore_list_get_or")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), idx.into(), default_i64.into()], "getor"))?;
                let raw_val = self.call_result_to_value(result)?;
                let typed_val = self.list_elem_from_i64(raw_val, &elem_kind)?;
                Ok((typed_val, elem_kind))
            }
            "map" | "filter" | "flat_map" | "take_while" | "drop_while" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: format!("{} takes exactly 1 argument", method) });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                self.last_lambda_return_kind = None;
                let kinds = vec![elem_kind.clone().unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, true)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let runtime_fn_name = format!("ore_list_{}", method);
                let runtime_fn = self.rt(&runtime_fn_name)?;
                let result = bld!(self.builder.build_call(
                    runtime_fn,
                    &[list_val.into(), fn_ptr.into(), env_ptr.into()],
                    method
                ))?;
                let val = self.call_result_to_value(result)?;
                // For map, update element kind based on lambda return type
                if method == "map" {
                    if let Some(ret_kind) = self.last_lambda_return_kind.take() {
                        self.last_list_elem_kind = Some(ret_kind);
                    }
                }
                // filter preserves element kind, no update needed
                let ret_elem = self.last_list_elem_kind.clone();
                Ok((val, ValKind::List(ret_elem.map(Box::new))))
            }
            "partition" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "partition takes exactly 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.clone().unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_partition")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "part"))?;
                let val = self.call_result_to_value(result)?;
                let inner_elem = elem_kind.map(Box::new);
                self.last_list_elem_kind = Some(ValKind::List(inner_elem.clone()));
                Ok((val, ValKind::list_of(ValKind::List(inner_elem))))
            }
            "find_index" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "find_index takes exactly 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_find_index")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "fidx"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "fold" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "fold takes 2 arguments: initial value and function".into() });
                }
                let (init_val, _) = self.compile_expr_with_kind(&args[0], func)?;
                let elem_kind = self.last_list_elem_kind.clone();
                // fold lambda receives (acc, elem) — both as Int/i64
                let kinds = vec![ValKind::Int, elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[1], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_fold")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), init_val.into(), fn_ptr.into(), env_ptr.into()], "fold"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "each" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "each takes exactly 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let runtime_fn = self.rt("ore_list_each")?;
                bld!(self.builder.build_call(
                    runtime_fn,
                    &[list_val.into(), fn_ptr.into(), env_ptr.into()],
                    ""
                ))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            "tap" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "tap takes 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.clone().unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_tap")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "tap"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List(elem_kind.map(Box::new))))
            }
            "map_with_index" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "map_with_index takes 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![ValKind::Int, elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_map_with_index")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "mwi"))?;
                let val = self.call_result_to_value(result)?;
                let ret_elem = self.last_lambda_return_kind.take().or(self.last_list_elem_kind.clone());
                if let Some(ref rk) = ret_elem {
                    self.last_list_elem_kind = Some(rk.clone());
                }
                Ok((val, ValKind::List(ret_elem.map(Box::new))))
            }
            "each_with_index" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "each_with_index takes 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![ValKind::Int, elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_each_with_index")?;
                bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], ""))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            "par_map" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "par_map takes exactly 1 argument".into() });
                }
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &[ValKind::Int], method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_par_map")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "par_map"))?;
                let val = self.call_result_to_value(result)?;
                let ret_elem = self.last_lambda_return_kind.take().or(self.last_list_elem_kind.clone());
                if let Some(ref rk) = ret_elem {
                    self.last_list_elem_kind = Some(rk.clone());
                }
                Ok((val, ValKind::List(ret_elem.map(Box::new))))
            }
            "par_each" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "par_each takes exactly 1 argument".into() });
                }
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &[ValKind::Int], method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_par_each")?;
                bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], ""))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            "sort" => {
                if args.is_empty() {
                    let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                    let rt_name = match elem_kind {
                        ValKind::Str => "ore_list_sort_str",
                        ValKind::Float => "ore_list_sort_float",
                        _ => "ore_list_sort",
                    };
                    let rt = self.rt(rt_name)?;
                    let result = bld!(self.builder.build_call(rt, &[list_val.into()], "sorted"))?;
                    let sorted_val = self.call_result_to_value(result)?;
                    return Ok((sorted_val, ValKind::list_of(elem_kind)));
                }
                // sort(comparator) - sort_by
                let elem_kind = self.last_list_elem_kind.clone();
                let ek = elem_kind.unwrap_or(ValKind::Int);
                let kinds = vec![ek.clone(), ek];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_sort_by")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "sorted"))?;
                let sorted_val = self.call_result_to_value(result)?;
                Ok((sorted_val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "sort_by" => {
                // sort_by(key_fn) - sort by a key extracted from each element
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "sort_by takes 1 argument (key function)".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.clone().unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                // Determine which runtime to use based on key return type
                let rt = self.rt("ore_list_sort_by_key")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "sorted"))?;
                let sorted_val = self.call_result_to_value(result)?;
                Ok((sorted_val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "min_by" | "max_by" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: format!("{} takes 1 argument (key function)", method) });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.clone().unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let fn_name = if method == "min_by" { "ore_list_min_by" } else { "ore_list_max_by" };
                let rt = self.rt(fn_name)?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "mby"))?;
                let val = self.call_result_to_value(result)?;
                let ek = elem_kind.unwrap_or(ValKind::Int);
                match &ek {
                    ValKind::Str | ValKind::List(_) | ValKind::Map => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            val.into_int_value(), self.ptr_type(), "mby2p"
                        ))?;
                        Ok((ptr.into(), ek))
                    }
                    _ => Ok((val, ek))
                }
            }
            "reverse" => {
                let rt = self.rt("ore_list_reverse_new")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "reversed"))?;
                let rev_val = self.call_result_to_value(result)?;
                Ok((rev_val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "contains" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "contains takes exactly 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let (val, _) = self.compile_expr_with_kind(&args[0], func)?;
                let result = if matches!(elem_kind, ValKind::Str) {
                    let rt = self.rt("ore_list_contains_str")?;
                    bld!(self.builder.build_call(rt, &[list_val.into(), val.into()], "lcontains"))?
                } else {
                    let rt = self.rt("ore_list_contains")?;
                    let i64_val = if val.is_pointer_value() {
                        bld!(self.builder.build_ptr_to_int(val.into_pointer_value(), self.context.i64_type(), "p2i"))?.into()
                    } else {
                        val.into()
                    };
                    bld!(self.builder.build_call(rt, &[list_val.into(), i64_val], "lcontains"))?
                };
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    i8_val,
                    self.context.i8_type().const_int(0, false),
                    "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "reduce" => {
                // reduce(fn) uses first element as init, or reduce(init, fn)
                if args.is_empty() || args.len() > 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "reduce takes 1 or 2 arguments".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let fn_arg = if args.len() == 1 { &args[0] } else { &args[1] };
                let kinds = vec![ValKind::Int, elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(fn_arg, &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                if args.len() == 1 {
                    let rt = self.rt("ore_list_reduce1")?;
                    let result = bld!(self.builder.build_call(
                        rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "reduce"
                    ))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ValKind::Int))
                } else {
                    let init_val = self.compile_expr(&args[0], func)?;
                    let rt = self.rt("ore_list_reduce")?;
                    let result = bld!(self.builder.build_call(
                        rt, &[list_val.into(), init_val.into(), fn_ptr.into(), env_ptr.into()], "reduce"
                    ))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ValKind::Int))
                }
            }
            "scan" => {
                // scan(init, fn(acc, elem) -> acc) -> list of all acc values
                if args.len() != 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "scan takes 2 arguments (init, fn)".into() });
                }
                let init_val = self.compile_expr(&args[0], func)?;
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![ValKind::Int, elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[1], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_scan")?;
                let result = bld!(self.builder.build_call(
                    rt, &[list_val.into(), init_val.into(), fn_ptr.into(), env_ptr.into()], "scan"
                ))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::Int);
                Ok((val, ValKind::list_of(ValKind::Int)))
            }
            "find" => {
                // find(fn(elem) -> bool) — returns element or 0
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "find takes 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.clone().unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let default_val = self.context.i64_type().const_int(0, false);
                let rt = self.rt("ore_list_find")?;
                let result = bld!(self.builder.build_call(
                    rt, &[list_val.into(), fn_ptr.into(), env_ptr.into(), default_val.into()], "find"
                ))?;
                let val = self.call_result_to_value(result)?;
                let ek = elem_kind.unwrap_or(ValKind::Int);
                match ek {
                    ValKind::Str | ValKind::List(_) | ValKind::Map => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            val.into_int_value(),
                            self.context.ptr_type(inkwell::AddressSpace::default()), "find2p"
                        ))?;
                        Ok((ptr.into(), ek))
                    }
                    _ => Ok((val, ek))
                }
            }
            "join" => {
                // join(separator_str)
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "join takes 1 argument (separator)".into() });
                }
                let sep = self.compile_expr(&args[0], func)?;
                // Use join_str for string lists, join for int lists
                let elem_kind = self.last_list_elem_kind.clone();
                let fn_name = match &elem_kind {
                    Some(ValKind::Str) => "ore_list_join_str",
                    Some(ValKind::Float) => "ore_list_join_float",
                    _ => "ore_list_join",
                };
                let rt = self.rt(fn_name)?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), sep.into()], "join"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "take" | "skip" | "step" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: format!("{} takes 1 argument (count)", method) });
                }
                let n = self.compile_expr(&args[0], func)?;
                let runtime_fn_name = format!("ore_list_{}", method);
                let rt = self.rt(&runtime_fn_name)?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), n.into()], method))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "sum" => {
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                if matches!(elem_kind, ValKind::Float) {
                    let rt = self.rt("ore_list_sum_float")?;
                    let result = bld!(self.builder.build_call(rt, &[list_val.into()], "sumf"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ValKind::Float))
                } else {
                    let rt = self.rt("ore_list_sum")?;
                    let result = bld!(self.builder.build_call(rt, &[list_val.into()], "sum"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ValKind::Int))
                }
            }
            "product" => {
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                if matches!(elem_kind, ValKind::Float) {
                    let rt = self.rt("ore_list_product_float")?;
                    let result = bld!(self.builder.build_call(rt, &[list_val.into()], "prodf"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                let rt = self.rt("ore_list_product")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "product"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "average" => {
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let rt_name = if matches!(elem_kind, ValKind::Float) { "ore_list_average_float" } else { "ore_list_average" };
                let rt = self.rt(rt_name)?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "avg"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Float))
            }
            "any" | "all" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: format!("{} takes 1 argument (predicate)", method) });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let runtime_fn_name = format!("ore_list_{}", method);
                let rt = self.rt(&runtime_fn_name)?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], method))?;
                let val = self.call_result_to_value(result)?;
                let bool_val = bld!(self.builder.build_int_truncate(val.into_int_value(), self.context.bool_type(), &format!("{}_bool", method)))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "zip" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "zip takes 1 argument (other list)".into() });
                }
                let other = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_list_zip")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), other.into()], "zip"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::List(None));
                Ok((val, ValKind::list_of(ValKind::List(None))))
            }
            "zip_with" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "zip_with takes 2 arguments (other_list, fn)".into() });
                }
                let other = self.compile_expr(&args[0], func)?;
                let elem_kind = self.last_list_elem_kind.clone();
                let ek = elem_kind.unwrap_or(ValKind::Int);
                let kinds = vec![ek.clone(), ek];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[1], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_zip_with")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), other.into(), fn_ptr.into(), env_ptr.into()], "zipw"))?;
                let val = self.call_result_to_value(result)?;
                if let Some(rk) = self.last_lambda_return_kind.take() {
                    self.last_list_elem_kind = Some(rk);
                }
                let ret_elem = self.last_list_elem_kind.clone();
                Ok((val, ValKind::List(ret_elem.map(Box::new))))
            }
            "enumerate" => {
                let rt = self.rt("ore_list_enumerate")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "enum"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::List(None));
                Ok((val, ValKind::list_of(ValKind::List(None))))
            }
            "slice" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "slice takes 2 arguments (start, end)".into() });
                }
                let start = self.compile_expr(&args[0], func)?;
                let end = self.compile_expr(&args[1], func)?;
                let rt = self.rt("ore_list_slice")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), start.into(), end.into()], "lslice"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "index_of" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "index_of takes 1 argument".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let val = self.compile_expr(&args[0], func)?;
                let result = if matches!(elem_kind, ValKind::Str) {
                    let rt = self.rt("ore_list_index_of_str")?;
                    bld!(self.builder.build_call(rt, &[list_val.into(), val.into()], "lidx"))?
                } else {
                    let rt = self.rt("ore_list_index_of")?;
                    bld!(self.builder.build_call(rt, &[list_val.into(), val.into()], "lidx"))?
                };
                let v = self.call_result_to_value(result)?;
                Ok((v, ValKind::Int))
            }
            "unique" => {
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let rt_name = if matches!(elem_kind, ValKind::Str) { "ore_list_unique_str" } else { "ore_list_unique" };
                let rt = self.rt(rt_name)?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "luniq"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::list_of(elem_kind)))
            }
            "unique_by" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "unique_by takes 1 argument (key function)".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_unique_by")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "uniqby"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "flatten" => {
                let rt = self.rt("ore_list_flatten")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lflat"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List(None)))
            }
            "window" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "window takes 1 argument (size)".into() });
                }
                let n = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_list_window")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), n.into()], "lwin"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::List(None));
                Ok((val, ValKind::list_of(ValKind::List(None))))
            }
            "chunks" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "chunks takes 1 argument (size)".into() });
                }
                let n = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_list_chunks")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), n.into()], "lchk"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::List(None));
                Ok((val, ValKind::list_of(ValKind::List(None))))
            }
            "first" => {
                let rt = self.rt("ore_list_get")?;
                let zero = self.context.i64_type().const_int(0, false);
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), zero.into()], "first"))?;
                let val = self.call_result_to_value(result)?;
                let ek = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                match ek {
                    ValKind::Str | ValKind::List(_) | ValKind::Map => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            val.into_int_value(),
                            self.context.ptr_type(inkwell::AddressSpace::default()), "first2p"
                        ))?;
                        Ok((ptr.into(), ek))
                    }
                    _ => Ok((val, ek))
                }
            }
            "last" => {
                let rt = self.rt("ore_list_get")?;
                let neg_one = self.context.i64_type().const_int((-1i64) as u64, true);
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), neg_one.into()], "last"))?;
                let val = self.call_result_to_value(result)?;
                let ek = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                match ek {
                    ValKind::Str | ValKind::List(_) | ValKind::Map => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            val.into_int_value(),
                            self.context.ptr_type(inkwell::AddressSpace::default()), "last2p"
                        ))?;
                        Ok((ptr.into(), ek))
                    }
                    _ => Ok((val, ek))
                }
            }
            "min" => {
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                match elem_kind {
                    ValKind::Float => {
                        let rt = self.rt("ore_list_min_float")?;
                        let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lminf"))?;
                        let val = self.call_result_to_value(result)?;
                        Ok((val, ValKind::Float))
                    }
                    ValKind::Str => {
                        let rt = self.rt("ore_list_min_str")?;
                        let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lmins"))?;
                        let val = self.call_result_to_value(result)?;
                        Ok((val, ValKind::Str))
                    }
                    _ => {
                        let rt = self.rt("ore_list_min")?;
                        let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lmin"))?;
                        let val = self.call_result_to_value(result)?;
                        Ok((val, ValKind::Int))
                    }
                }
            }
            "max" => {
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                match elem_kind {
                    ValKind::Float => {
                        let rt = self.rt("ore_list_max_float")?;
                        let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lmaxf"))?;
                        let val = self.call_result_to_value(result)?;
                        Ok((val, ValKind::Float))
                    }
                    ValKind::Str => {
                        let rt = self.rt("ore_list_max_str")?;
                        let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lmaxs"))?;
                        let val = self.call_result_to_value(result)?;
                        Ok((val, ValKind::Str))
                    }
                    _ => {
                        let rt = self.rt("ore_list_max")?;
                        let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lmax"))?;
                        let val = self.call_result_to_value(result)?;
                        Ok((val, ValKind::Int))
                    }
                }
            }
            "count" => {
                // count() with no args returns list length, count(pred) counts matching
                if args.is_empty() {
                    let rt = self.rt("ore_list_len")?;
                    let result = bld!(self.builder.build_call(rt, &[list_val.into()], "count"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Int));
                }
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "count takes 0 or 1 arguments".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_count")?;
                let result = bld!(self.builder.build_call(
                    rt,
                    &[list_val.into(), fn_ptr.into(), env_ptr.into()],
                    "count"
                ))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "count_by" | "group_by" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: format!("{} takes 1 argument (key function)", method) });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let lambda_ret_kind = self.last_lambda_return_kind.take();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                // Use _int variant for count_by when lambda returns non-string
                let rt_name = if method == "count_by" && lambda_ret_kind.as_ref() != Some(&ValKind::Str) {
                    "ore_list_count_by_int".to_string()
                } else {
                    format!("ore_list_{}", method)
                };
                let rt = self.rt(&rt_name)?;
                let result = bld!(self.builder.build_call(
                    rt,
                    &[list_val.into(), fn_ptr.into(), env_ptr.into()],
                    method
                ))?;
                let val = self.call_result_to_value(result)?;
                let val_kind = if method == "count_by" { ValKind::Int } else { ValKind::List(self.last_list_elem_kind.clone().map(Box::new)) };
                self.last_map_val_kind = Some(val_kind);
                Ok((val.into(), ValKind::Map))
            }
            "to_map" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "to_map takes 1 argument (key function)".into() });
                }
                let elem_kind = self.last_list_elem_kind.clone();
                let kinds = vec![elem_kind.clone().unwrap_or(ValKind::Int)];
                let (lambda_fn, env_ptr) = self.resolve_list_lambda_arg(&args[0], &kinds, method, func, false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_list_to_map")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "tomap"))?;
                let val = self.call_result_to_value(result)?;
                let vk = elem_kind.unwrap_or(ValKind::Int);
                self.last_map_val_kind = Some(vk);
                Ok((val.into(), ValKind::Map))
            }
            "dedup" => {
                let rt = self.rt("ore_list_dedup")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "dedup"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
            }
            "frequencies" => {
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let kind_val = self.context.i8_type().const_int(match elem_kind {
                    ValKind::Int => 0,
                    ValKind::Float => 1,
                    ValKind::Bool => 2,
                    ValKind::Str => 3,
                    _ => 0,
                }, false);
                let rt = self.rt("ore_list_frequencies")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), kind_val.into()], "freq"))?;
                let val = self.call_result_to_value(result)?;
                self.last_map_val_kind = Some(ValKind::Int);
                Ok((val.into(), ValKind::Map))
            }
            "intersperse" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "intersperse takes 1 argument".to_string() });
                }
                let (sep_val, sep_kind) = self.compile_expr_with_kind(&args[0], func)?;
                let sep_i64: IntValue = match sep_kind {
                    ValKind::Str | ValKind::List(_) | ValKind::Map => {
                        bld!(self.builder.build_ptr_to_int(sep_val.into_pointer_value(), self.context.i64_type(), "sep2i"))?
                    }
                    _ => sep_val.into_int_value(),
                };
                let rt = self.rt("ore_list_intersperse")?;
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), sep_i64.into()], "inter"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val.into(), ValKind::List(self.last_list_elem_kind.clone().map(Box::new))))
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
        let list_new = self.rt("ore_list_new")?;
        let list_push = self.rt("ore_list_push")?;

        let list_result = bld!(self.builder.build_call(list_new, &[], "list"))?;
        let list_ptr = self.call_result_to_value(list_result)?.into_pointer_value();

        let mut elem_kind = ValKind::Int;
        for elem in elements {
            let (val, kind) = self.compile_expr_with_kind(elem, func)?;
            elem_kind = kind.clone();
            // For records/enums, heap-allocate and push the pointer
            let push_val = match &kind {
                ValKind::Record(name) => {
                    let info = &self.records[name];
                    let st = info.struct_type;
                    let heap_ptr = bld!(self.builder.build_malloc(st, "heap_rec"))?;
                    bld!(self.builder.build_store(heap_ptr, val))?;
                    let i64_val = bld!(self.builder.build_ptr_to_int(heap_ptr, self.context.i64_type(), "p2i"))?;
                    i64_val.into()
                }
                ValKind::Str => {
                    // Strings are already pointers, convert to i64
                    let i64_val = bld!(self.builder.build_ptr_to_int(val.into_pointer_value(), self.context.i64_type(), "p2i"))?;
                    i64_val.into()
                }
                ValKind::Float => {
                    // Floats need bitcast to i64 for storage
                    let i64_val = bld!(self.builder.build_bit_cast(val, self.context.i64_type(), "f2i"))?;
                    i64_val
                }
                ValKind::Bool => {
                    // Bools need zero-extension to i64
                    let i64_val = bld!(self.builder.build_int_z_extend(val.into_int_value(), self.context.i64_type(), "b2i"))?;
                    i64_val.into()
                }
                ValKind::Enum(name) => {
                    let et = self.enums[name].enum_type;
                    let heap_ptr = bld!(self.builder.build_malloc(et, "heap_enum"))?;
                    bld!(self.builder.build_store(heap_ptr, val))?;
                    let i64_val = bld!(self.builder.build_ptr_to_int(heap_ptr, self.context.i64_type(), "p2i"))?;
                    i64_val.into()
                }
                _ => val,
            };
            bld!(self.builder.build_call(list_push, &[list_ptr.into(), push_val.into()], ""))?;
        }

        // Store element kind for later extraction
        self.last_list_elem_kind = Some(elem_kind.clone());

        Ok((list_ptr.into(), ValKind::list_of(elem_kind)))
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
        let list_new = self.rt("ore_list_new")?;
        let list_push = self.rt("ore_list_push")?;

        // Create output list
        let list_result = bld!(self.builder.build_call(list_new, &[], "comp_list"))?;
        let list_ptr = self.call_result_to_value(list_result)?.into_pointer_value();

        // Check if iterable is a range (__range call)
        let is_range = matches!(iterable, Expr::Call { func: f, .. } if matches!(f.as_ref(), Expr::Ident(n) if n == "__range"));

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
            self.variables.insert(var.to_string(), (var_alloca, i64_type.into(), ValKind::Int, false));

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
                let bool_val = if cond_val.is_int_value() && cond_val.into_int_value().get_type().get_bit_width() > 1 {
                    bld!(self.builder.build_int_compare(IntPredicate::NE, cond_val.into_int_value(), i64_type.const_int(0, false), "tobool"))?
                } else {
                    cond_val.into_int_value()
                };
                bld!(self.builder.build_conditional_branch(bool_val, filter_bb, inc_bb))?;
                self.builder.position_at_end(filter_bb);
                filter_bb
            } else {
                body_bb
            };

            let (val, kind) = self.compile_expr_with_kind(expr, func)?;
            let push_val = match &kind {
                ValKind::Str => {
                    let i64_val = bld!(self.builder.build_ptr_to_int(val.into_pointer_value(), i64_type, "p2i"))?;
                    i64_val.into()
                }
                ValKind::Float => {
                    bld!(self.builder.build_bit_cast(val, i64_type, "f2i"))?
                }
                ValKind::Bool => {
                    let i64_val = bld!(self.builder.build_int_z_extend(val.into_int_value(), i64_type, "b2i"))?;
                    i64_val.into()
                }
                _ => val,
            };
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
            self.last_list_elem_kind = Some(kind);
        } else {
            // List-based comprehension
            let list_src = self.compile_expr(iterable, func)?.into_pointer_value();
            let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);

            let list_len_fn = self.rt("ore_list_len")?;
            let len_result = bld!(self.builder.build_call(list_len_fn, &[list_src.into()], "len"))?;
            let len_val = self.call_result_to_value(len_result)?.into_int_value();

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
            self.variables.insert(var.to_string(), (var_alloca, var_ty, elem_kind.clone(), false));

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
            let list_get_fn = self.rt("ore_list_get")?;
            let elem_result = bld!(self.builder.build_call(list_get_fn, &[list_src.into(), idx.into()], "elem"))?;
            let raw_val = self.call_result_to_value(elem_result)?;

            match &elem_kind {
                ValKind::Str => {
                    let ptr = bld!(self.builder.build_int_to_ptr(
                        raw_val.into_int_value(),
                        self.context.ptr_type(inkwell::AddressSpace::default()), "i2p"
                    ))?;
                    bld!(self.builder.build_store(var_alloca, ptr))?;
                }
                _ => {
                    bld!(self.builder.build_store(var_alloca, raw_val))?;
                }
            }

            if let Some(c) = cond {
                let filter_bb = self.context.append_basic_block(func, "comp_filter");
                let (cond_val, _) = self.compile_expr_with_kind(c, func)?;
                let bool_val = if cond_val.is_int_value() && cond_val.into_int_value().get_type().get_bit_width() > 1 {
                    bld!(self.builder.build_int_compare(IntPredicate::NE, cond_val.into_int_value(), i64_type.const_int(0, false), "tobool"))?
                } else {
                    cond_val.into_int_value()
                };
                bld!(self.builder.build_conditional_branch(bool_val, filter_bb, inc_bb))?;
                self.builder.position_at_end(filter_bb);
            }

            let (val, kind) = self.compile_expr_with_kind(expr, func)?;
            let push_val = match &kind {
                ValKind::Str => {
                    let i64_val = bld!(self.builder.build_ptr_to_int(val.into_pointer_value(), i64_type, "p2i"))?;
                    i64_val.into()
                }
                ValKind::Float => {
                    bld!(self.builder.build_bit_cast(val, i64_type, "f2i"))?
                }
                ValKind::Bool => {
                    let i64_val = bld!(self.builder.build_int_z_extend(val.into_int_value(), i64_type, "b2i"))?;
                    i64_val.into()
                }
                _ => val,
            };
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
            self.last_list_elem_kind = Some(kind);
        }

        let comp_elem = self.last_list_elem_kind.clone();
        Ok((list_ptr.into(), ValKind::List(comp_elem.map(Box::new))))
    }

}
