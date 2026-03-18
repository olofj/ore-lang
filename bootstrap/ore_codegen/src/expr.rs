use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue};
use inkwell::IntPredicate;

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_expr(
        &mut self,
        expr: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        self.compile_expr_with_kind(expr, func).map(|(v, _)| v)
    }

    pub(crate) fn compile_expr_with_kind(
        &mut self,
        expr: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match expr {
            Expr::IntLit(n) => {
                Ok((self.context.i64_type().const_int(*n as u64, true).into(), ValKind::Int))
            }
            Expr::FloatLit(f) => {
                Ok((self.context.f64_type().const_float(*f).into(), ValKind::Float))
            }
            Expr::BoolLit(b) => {
                Ok((self.context.bool_type().const_int(*b as u64, false).into(), ValKind::Bool))
            }
            Expr::StringLit(s) => {
                let ptr = self.compile_string_literal(s)?;
                Ok((ptr.into(), ValKind::Str))
            }
            Expr::StringInterp(parts) => {
                let ptr = self.compile_string_interp(parts, func)?;
                Ok((ptr.into(), ValKind::Str))
            }
            Expr::BlockExpr(block) => {
                self.compile_block_stmts_with_kind(block, func).map(|(v, k)| {
                    (v.unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()), k)
                })
            }
            Expr::Lambda { params, body } => {
                let lambda_fn = self.compile_lambda(params, body)?;
                // Return the function pointer
                let ptr = lambda_fn.as_global_value().as_pointer_value();
                Ok((ptr.into(), ValKind::Int)) // Kind is approximate; lambdas are function pointers
            }
            Expr::Ident(name) => {
                // Check if it's a zero-arg enum variant (e.g., `Red` instead of `Red()`)
                if !self.variables.contains_key(name) && self.variant_to_enum.contains_key(name) {
                    let construct = Expr::RecordConstruct {
                        type_name: name.clone(),
                        fields: vec![],
                    };
                    return self.compile_expr_with_kind(&construct, func);
                }

                // Check if it's a function reference (for passing functions as values)
                if !self.variables.contains_key(name) {
                    if let Ok((f, _ret_kind)) = self.resolve_function(name) {
                        let fn_ptr = f.as_global_value().as_pointer_value();
                        return Ok((fn_ptr.into(), ValKind::Int));
                    }
                }

                let var = self.variables.get(name).ok_or_else(|| self.undefined_var_error(name))?;
                let val = bld!(self.builder.build_load(var.ty, var.ptr, name))?;
                // Enrich kind with latest tracked element/value kinds
                let kind = match &var.kind {
                    ValKind::List(_) => {
                        if let Some(ek) = self.list_element_kinds.get(name) {
                            ValKind::list_of(ek.clone())
                        } else {
                            var.kind.clone()
                        }
                    }
                    ValKind::Map(_) => {
                        if let Some(vk) = self.map_value_kinds.get(name) {
                            ValKind::map_of(vk.clone())
                        } else {
                            var.kind.clone()
                        }
                    }
                    _ => var.kind.clone(),
                };
                Ok((val, kind))
            }
            Expr::BinOp { op, left, right } => {
                if *op == BinOp::Pipe {
                    return self.compile_pipeline_with_kind(left, right, func);
                }
                // Short-circuit evaluation for and/or
                if *op == BinOp::And || *op == BinOp::Or {
                    return self.compile_short_circuit(*op, left, right, func);
                }
                let (lhs, lk) = self.compile_expr_with_kind(left, func)?;
                let (rhs, rk) = self.compile_expr_with_kind(right, func)?;

                // List concatenation: list + list
                if lk.is_list() && *op == BinOp::Add {
                    let val = self.call_rt("ore_list_concat", &[lhs.into(), rhs.into()], "lcat")?;
                    // Preserve element kind: prefer RHS (the appended elements), fall back to LHS
                    let elem = match (&rk, &lk) {
                        (ValKind::List(Some(ek)), _) | (_, ValKind::List(Some(ek))) => Some(ek.clone()),
                        _ => None,
                    };
                    return Ok((val, ValKind::List(elem)));
                }

                // String repetition: str * int
                if lk == ValKind::Str && *op == BinOp::Mul {
                    let val = self.call_rt("ore_str_repeat", &[lhs.into(), rhs.into()], "srepeat")?;
                    return Ok((val, ValKind::Str));
                }

                // If both sides are strings but represented as i64 (e.g. in lambdas), convert to pointers
                let (lhs, rhs) = if lk == ValKind::Str && rk == ValKind::Str {
                    let l = if lhs.is_int_value() {
                        self.i64_to_ptr(lhs.into_int_value())?.into()
                    } else { lhs };
                    let r = if rhs.is_int_value() {
                        self.i64_to_ptr(rhs.into_int_value())?.into()
                    } else { rhs };
                    (l, r)
                } else {
                    (lhs, rhs)
                };
                let result = self.compile_binop(*op, lhs, rhs)?;
                // Determine result kind
                let result_kind = match op {
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq
                    | BinOp::And | BinOp::Or => ValKind::Bool,
                    _ => if lk == ValKind::Float || rk == ValKind::Float { ValKind::Float } else { lk },
                };
                Ok((result, result_kind))
            }
            Expr::UnaryMinus(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                match val {
                    BasicValueEnum::IntValue(v) => {
                        Ok((bld!(self.builder.build_int_neg(v, "neg"))?.into(), kind))
                    }
                    BasicValueEnum::FloatValue(v) => {
                        Ok((bld!(self.builder.build_float_neg(v, "fneg"))?.into(), kind))
                    }
                    _ => Err(self.err("cannot negate this type")),
                }
            }
            Expr::UnaryNot(inner) => {
                let val = self.compile_expr(inner, func)?;
                match val {
                    BasicValueEnum::IntValue(v) => {
                        Ok((bld!(self.builder.build_not(v, "not"))?.into(), ValKind::Bool))
                    }
                    _ => Err(self.err("cannot apply 'not' to this type")),
                }
            }
            Expr::Print(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                self.compile_print_expr(val, &kind, inner)?;
                Ok(self.void_result())
            }
            Expr::Sleep(inner) => {
                let val = self.compile_expr(inner, func)?;
                self.call_rt("ore_sleep", &[val.into()], "")?;
                Ok(self.void_result())
            }
            Expr::Assert { cond, message } => {
                let cond_val = self.compile_expr(cond, func)?;
                let msg_str = message.as_deref().unwrap_or("assertion failed");
                let msg_ptr = self.build_c_string_global(msg_str, &format!("assert_msg_{}", self.current_line))?;
                let line_val = self.context.i64_type().const_int(self.current_line as u64, false);
                self.call_rt("ore_assert", &[cond_val.into(), msg_ptr.into(), line_val.into()], "")?;
                Ok(self.void_result())
            }
            Expr::AssertEq { left, right, message } => {
                self.compile_assert_cmp(left, right, message.as_deref(), "eq", func)
            }
            Expr::AssertNe { left, right, message } => {
                self.compile_assert_cmp(left, right, message.as_deref(), "ne", func)
            }
            Expr::Call { func: callee, args } => {
                let name = match callee.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(self.err("only named function calls supported")),
                };

                // Try built-in stdlib functions first
                if let Some(result) = self.compile_builtin_call(&name, args, func)? {
                    return Ok(result);
                }

                // Check if this is a variant construction (e.g. Red() or Circle(radius: 5.0))
                if self.variant_to_enum.contains_key(&name) {
                    // Treat as RecordConstruct with variant name
                    let construct = Expr::RecordConstruct {
                        type_name: name.clone(),
                        fields: vec![], // Zero-field variant
                    };
                    return self.compile_expr_with_kind(&construct, func);
                }

                // Try resolving as a named function first, or monomorphize generic
                let resolved = match self.resolve_function(&name) {
                    Ok(fk) => Some(fk),
                    Err(_) if self.generic_fns.contains_key(&name) => {
                        // Compile args to determine their kinds for monomorphization
                        let mut compiled_args = Vec::new();
                        let mut arg_kinds = Vec::new();
                        for arg in args {
                            let (val, kind) = self.compile_expr_with_kind(arg, func)?;
                            compiled_args.push(val.into());
                            arg_kinds.push(kind);
                        }
                        let (called_fn, ret_kind) = self.monomorphize(&name, &arg_kinds)?;
                        let result = bld!(self.builder.build_call(called_fn, &compiled_args, "call"))?;
                        let val = self.call_result_to_value(result)?;
                        return Ok((val, ret_kind));
                    }
                    Err(_) => None,
                };

                if let Some((called_fn, ret_kind)) = resolved {
                    let mut compiled_args = Vec::new();
                    for arg in args {
                        compiled_args.push(self.compile_expr(arg, func)?.into());
                    }
                    self.fill_default_args(&name, &mut compiled_args, func)?;
                    let result = bld!(self.builder.build_call(called_fn, &compiled_args, "call"))?;
                    let val = self.call_result_to_value(result)?;
                    // Propagate list/map element kind from function return type annotation
                    let ret_kind = if ret_kind.is_list() {
                        if let Some(ek) = self.fn_return_list_elem_kind.get(&name) {
                            ValKind::list_of(ek.clone())
                        } else {
                            ret_kind
                        }
                    } else if ret_kind.is_map() {
                        if let Some(vk) = self.fn_return_map_val_kind.get(&name) {
                            ValKind::map_of(vk.clone())
                        } else {
                            ret_kind
                        }
                    } else {
                        ret_kind
                    };
                    Ok((val, ret_kind))
                } else {
                    // Check if it's a variable holding a function pointer (closure)
                    if let Some(var) = self.variables.get(&name).cloned() {
                        let fn_ptr_val = bld!(self.builder.build_load(self.ptr_type(), var.ptr, "fn_ptr"))?;
                        let fn_ptr = fn_ptr_val.into_pointer_value();

                        // Check for closure (env_ptr stored alongside)
                        let env_var_name = format!("{}_env", name);
                        let has_env = self.variables.contains_key(&env_var_name);

                        let mut compiled_args = Vec::new();
                        for arg in args {
                            compiled_args.push(self.compile_expr(arg, func)?.into());
                        }

                        if has_env {
                            let env_var = self.variables[&env_var_name].clone();
                            let env_val = bld!(self.builder.build_load(self.ptr_type(), env_var.ptr, "env"))?;
                            let mut all_args = vec![env_val.into()];
                            all_args.extend(compiled_args);

                            let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![self.ptr_type().into()];
                            for _ in &all_args[1..] {
                                param_types.push(self.context.i64_type().into());
                            }
                            let fn_type = self.context.i64_type().fn_type(&param_types, false);
                            let result = bld!(self.builder.build_indirect_call(fn_type, fn_ptr, &all_args, "closurecall"))?;
                            let val = self.call_result_to_value(result)?;
                            Ok((val, ValKind::Int))
                        } else {
                            let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = compiled_args.iter().map(|_| self.context.i64_type().into()).collect();
                            let fn_type = self.context.i64_type().fn_type(&param_types, false);
                            let result = bld!(self.builder.build_indirect_call(fn_type, fn_ptr, &compiled_args, "fncall"))?;
                            let val = self.call_result_to_value(result)?;
                            Ok((val, ValKind::Int))
                        }
                    } else {
                        Err(self.undefined_fn_error(&name))
                    }
                }
            }
            Expr::IfElse { cond, then_block, else_block } => {
                // Pre-scan both branches for map.set() calls so map value kinds
                // are available regardless of compilation order
                self.prescan_map_value_kinds(then_block);
                if let Some(eb) = else_block {
                    self.prescan_map_value_kinds(eb);
                }
                self.compile_if_else_with_kind(cond, then_block, else_block.as_ref(), func)
            }
            Expr::ColonMatch { cond, then_expr, else_expr } => {
                self.compile_colon_match_with_kind(cond, then_expr, else_expr.as_deref(), func)
            }
            Expr::RecordConstruct { type_name, fields } => {
                // Check if this is actually an enum variant construction
                if self.variant_to_enum.contains_key(type_name) {
                    return self.compile_variant_construct(type_name, fields, func);
                }
                self.compile_record_construct(type_name, fields, func)
            }
            Expr::Match { subject, arms } => {
                self.compile_match(subject, arms, func)
            }
            Expr::FieldAccess { object, field } => {
                self.compile_field_access(object, field, func)
            }
            Expr::MethodCall { object, method, args } => {
                self.compile_method_call(object, method, args, func)
            }
            Expr::ListLit(elements) => {
                self.compile_list_lit(elements, func)
            }
            Expr::ListComp { expr, var, iterable, cond } => {
                self.compile_list_comp(expr, var, iterable, cond.as_deref(), func)
            }
            Expr::MapLit(entries) => {
                self.compile_map_lit(entries, func)
            }
            Expr::Index { object, index } => {
                self.compile_index(object, index, func)
            }
            Expr::OptionNone => {
                let result = self.build_tagged_union(self.option_type(), 0, None::<(BasicValueEnum, &ValKind)>, "none_val")?;
                Ok((result, ValKind::Option))
            }
            Expr::OptionSome(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                let result = self.build_tagged_union(self.option_type(), 1, Some((val, &kind)), "some_val")?;
                Ok((result, ValKind::Option))
            }
            Expr::ResultOk(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                let result = self.build_tagged_union(self.result_type(), 0, Some((val, &kind)), "ok_val")?;
                Ok((result, ValKind::Result))
            }
            Expr::ResultErr(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                let result = self.build_tagged_union(self.result_type(), 1, Some((val, &kind)), "err_val")?;
                Ok((result, ValKind::Result))
            }
            Expr::Try(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                if kind == ValKind::Result {
                    return self.compile_try_result(val, func);
                }
                let opt_ty = self.option_type();
                let alloca = bld!(self.builder.build_alloca(opt_ty, "try_opt"))?;
                bld!(self.builder.build_store(alloca, val))?;
                let tag = self.load_tag(opt_ty, alloca)?;
                let is_none = self.check_tag(tag, 0, "is_none")?;
                let some_bb = self.context.append_basic_block(func, "try_some");
                let none_bb = self.context.append_basic_block(func, "try_none");
                bld!(self.builder.build_conditional_branch(is_none, none_bb, some_bb))?;
                // None branch: return None from current function
                self.builder.position_at_end(none_bb);
                let none_ret = self.build_tagged_union(opt_ty, 0, None::<(BasicValueEnum, &ValKind)>, "none_ret")?;
                bld!(self.builder.build_return(Some(&none_ret)))?;
                // Some branch: extract value
                self.builder.position_at_end(some_bb);
                let extracted = self.load_tagged_value(opt_ty, alloca)?;
                Ok((extracted, ValKind::Int))
            }
            Expr::OptionalChain { object, field } => {
                self.compile_optional_chain(object, field, func)
            }
            Expr::OptionalMethodCall { object, method, args } => {
                self.compile_optional_method_call(object, method, args, func)
            }
            Expr::Break => {
                if let Some(target) = self.break_target {
                    bld!(self.builder.build_unconditional_branch(target))?;
                } else {
                    return Err(self.err("break outside of loop"));
                }
                Ok(self.void_result())
            }
        }
    }

    pub(crate) fn compile_short_circuit(
        &mut self,
        op: BinOp,
        left: &Expr,
        right: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        // Use alloca-based approach instead of phi nodes to avoid LLVM backend
        // issues when function calls appear in conditional branches.
        // Place alloca in entry block for proper stack allocation.
        let result_alloca = {
            self.build_entry_alloca(func, self.context.i64_type(), "sc_tmp")?
        };

        let (lhs, _lk) = self.compile_expr_with_kind(left, func)?;
        let lhs_bool = if lhs.is_int_value() {
            let lv = lhs.into_int_value();
            if lv.get_type().get_bit_width() != 1 {
                bld!(self.builder.build_int_truncate(lv, self.context.bool_type(), "tobool"))?
            } else {
                lv
            }
        } else {
            return Err(self.err("short-circuit: expected boolean operand"));
        };

        let rhs_block = self.context.append_basic_block(func, "sc_rhs");
        let merge_block = self.context.append_basic_block(func, "sc_merge");

        // For AND: if lhs is false, short-circuit to false; else eval RHS
        // For OR:  if lhs is true, short-circuit to true; else eval RHS
        let short_val = if op == BinOp::And { 0u64 } else { 1u64 };
        bld!(self.builder.build_store(
            result_alloca,
            self.context.i64_type().const_int(short_val, false)
        ))?;

        if op == BinOp::And {
            bld!(self.builder.build_conditional_branch(lhs_bool, rhs_block, merge_block))?;
        } else {
            bld!(self.builder.build_conditional_branch(lhs_bool, merge_block, rhs_block))?;
        }

        // Compile RHS
        self.builder.position_at_end(rhs_block);
        let (rhs, _rk) = self.compile_expr_with_kind(right, func)?;
        let rhs_i64 = if rhs.is_int_value() {
            let rv = rhs.into_int_value();
            if rv.get_type().get_bit_width() != 64 {
                bld!(self.builder.build_int_z_extend(rv, self.context.i64_type(), "rhs_ext"))?
            } else {
                rv
            }
        } else {
            return Err(self.err("short-circuit: expected boolean operand for RHS"));
        };
        bld!(self.builder.build_store(result_alloca, rhs_i64))?;
        bld!(self.builder.build_unconditional_branch(merge_block))?;

        // Load result from alloca
        self.builder.position_at_end(merge_block);
        let result = bld!(self.builder.build_load(self.context.i64_type(), result_alloca, "sc_result"))?;

        Ok((result, ValKind::Bool))
    }

    /// Compile a pipeline to a named function: `arg | name(extra_args...)`.
    /// Falls back to method call if the name isn't a known function.
    fn compile_pipe_to_named(
        &mut self,
        arg: &Expr,
        name: &str,
        extra_args: &[Expr],
        current_fn: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        if self.functions.contains_key(name) || self.module.get_function(name).is_some() {
            let arg_val = self.compile_expr(arg, current_fn)?;
            let (called_fn, ret_kind) = self.resolve_function(name)?;
            let mut compiled_args = vec![arg_val.into()];
            for a in extra_args {
                compiled_args.push(self.compile_expr(a, current_fn)?.into());
            }
            self.fill_default_args(name, &mut compiled_args, current_fn)?;
            let result = bld!(self.builder.build_call(called_fn, &compiled_args, "pipe"))?;
            let val = self.call_result_to_value(result)?;
            Ok((val, ret_kind))
        } else if self.generic_fns.contains_key(name) {
            let (arg_val, arg_kind) = self.compile_expr_with_kind(arg, current_fn)?;
            let mut compiled_args = vec![arg_val.into()];
            let mut arg_kinds = vec![arg_kind];
            for a in extra_args {
                let (v, k) = self.compile_expr_with_kind(a, current_fn)?;
                compiled_args.push(v.into());
                arg_kinds.push(k);
            }
            let (called_fn, ret_kind) = self.monomorphize(name, &arg_kinds)?;
            let result = bld!(self.builder.build_call(called_fn, &compiled_args, "pipe"))?;
            let val = self.call_result_to_value(result)?;
            Ok((val, ret_kind))
        } else {
            let method_call = Expr::MethodCall {
                object: Box::new(arg.clone()),
                method: name.to_string(),
                args: extra_args.to_vec(),
            };
            self.compile_expr_with_kind(&method_call, current_fn)
        }
    }

    pub(crate) fn compile_pipeline_with_kind(
        &mut self,
        arg: &Expr,
        func_expr: &Expr,
        current_fn: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        // Desugar pipeline: if the target is a function name/call that isn't a known
        // function, convert to a method call on the piped argument instead.
        // e.g. `list | each(lambda)` becomes `list.each(lambda)`
        // e.g. `list | map(lambda)` becomes `list.map(lambda)`
        match func_expr {
            Expr::Ident(name) => {
                self.compile_pipe_to_named(arg, name, &[], current_fn)
            }
            Expr::Call { func, args } => {
                let name = match func.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(self.err("pipeline target must be a function")),
                };
                self.compile_pipe_to_named(arg, &name, args, current_fn)
            }
            Expr::Lambda { params, body } => {
                let arg_val = self.compile_expr(arg, current_fn)?;
                let lambda_fn = self.compile_lambda(params, body)?;
                let lambda_name = Self::get_lambda_name(lambda_fn);

                let mut call_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = Vec::new();
                if self.lambda_captures.contains_key(&lambda_name) {
                    let env_ptr = self.build_captures_struct(&lambda_name)?;
                    call_args.push(env_ptr.into());
                }
                call_args.push(arg_val.into());

                let result = bld!(self.builder.build_call(lambda_fn, &call_args, "pipe"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            _ => Err(self.err("unsupported pipeline target")),
        }
    }

    pub(crate) fn emit_div_by_zero_check(&mut self, divisor: IntValue<'ctx>) -> Result<(), CodeGenError> {
        let current_fn = self.current_fn()?;
        let zero = divisor.get_type().const_zero();
        let is_zero = bld!(self.builder.build_int_compare(
            IntPredicate::EQ, divisor, zero, "is_zero"
        ))?;
        let err_bb = self.context.append_basic_block(current_fn, "div_zero");
        let ok_bb = self.context.append_basic_block(current_fn, "div_ok");
        bld!(self.builder.build_conditional_branch(is_zero, err_bb, ok_bb))?;
        self.builder.position_at_end(err_bb);
        self.call_rt("ore_div_by_zero", &[], "")?;
        bld!(self.builder.build_unreachable())?;
        self.builder.position_at_end(ok_bb);
        Ok(())
    }

    pub(crate) fn compile_binop(
        &mut self,
        op: BinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match (lhs, rhs) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                if l.get_type().get_bit_width() == 1 && r.get_type().get_bit_width() == 1 {
                    return self.compile_bool_binop(op, l, r);
                }
                let result: IntValue<'ctx> = match op {
                    BinOp::Add => bld!(self.builder.build_int_add(l, r, "add")),
                    BinOp::Sub => bld!(self.builder.build_int_sub(l, r, "sub")),
                    BinOp::Mul => bld!(self.builder.build_int_mul(l, r, "mul")),
                    BinOp::Div => {
                        self.emit_div_by_zero_check(r)?;
                        bld!(self.builder.build_int_signed_div(l, r, "div"))
                    },
                    BinOp::Mod => {
                        self.emit_div_by_zero_check(r)?;
                        bld!(self.builder.build_int_signed_rem(l, r, "rem"))
                    },
                    BinOp::Eq => bld!(self.builder.build_int_compare(IntPredicate::EQ, l, r, "eq")),
                    BinOp::NotEq => bld!(self.builder.build_int_compare(IntPredicate::NE, l, r, "ne")),
                    BinOp::Lt => bld!(self.builder.build_int_compare(IntPredicate::SLT, l, r, "lt")),
                    BinOp::Gt => bld!(self.builder.build_int_compare(IntPredicate::SGT, l, r, "gt")),
                    BinOp::LtEq => bld!(self.builder.build_int_compare(IntPredicate::SLE, l, r, "le")),
                    BinOp::GtEq => bld!(self.builder.build_int_compare(IntPredicate::SGE, l, r, "ge")),
                    BinOp::And => bld!(self.builder.build_and(l, r, "and")),
                    BinOp::Or => bld!(self.builder.build_or(l, r, "or")),
                    BinOp::Pipe => unreachable!("pipe handled separately"),
                }?;
                Ok(result.into())
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                self.compile_float_binop(op, l, r)
            }
            (BasicValueEnum::PointerValue(l), BasicValueEnum::PointerValue(r)) => {
                // String comparison via ore_str_eq
                match op {
                    BinOp::Eq => {
                        let i8_val = self.call_rt("ore_str_eq", &[l.into(), r.into()], "seq")?.into_int_value();
                        let bool_val = self.i8_to_bool(i8_val)?;
                        Ok(bool_val.into())
                    }
                    BinOp::NotEq => {
                        let i8_val = self.call_rt("ore_str_eq", &[l.into(), r.into()], "seq")?.into_int_value();
                        // EQ against 0 means "not equal"
                        let bool_val = bld!(self.builder.build_int_compare(
                            IntPredicate::EQ, i8_val,
                            self.context.i8_type().const_int(0, false), "sneq"
                        ))?;
                        Ok(bool_val.into())
                    }
                    BinOp::Add => {
                        // String concatenation
                        let val = self.call_rt("ore_str_concat", &[l.into(), r.into()], "sconcat")?;
                        Ok(val)
                    }
                    BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                        // String ordering via ore_str_cmp
                        let cmp_val = self.call_rt("ore_str_cmp", &[l.into(), r.into()], "scmp")?.into_int_value();
                        let zero = self.context.i64_type().const_int(0, false);
                        let pred = match op {
                            BinOp::Lt => IntPredicate::SLT,
                            BinOp::Gt => IntPredicate::SGT,
                            BinOp::LtEq => IntPredicate::SLE,
                            BinOp::GtEq => IntPredicate::SGE,
                            _ => unreachable!(),
                        };
                        let bool_val = bld!(self.builder.build_int_compare(pred, cmp_val, zero, "scmpres"))?;
                        Ok(bool_val.into())
                    }
                    _ => Err(self.err(format!("unsupported pointer op {:?}", op))),
                }
            }
            // Int-Float promotion: promote the int side to float
            (BasicValueEnum::IntValue(l), BasicValueEnum::FloatValue(r)) => {
                let l_f = bld!(self.builder.build_signed_int_to_float(l, self.context.f64_type(), "itof"))?;
                self.compile_float_binop(op, l_f, r)
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::IntValue(r)) => {
                let r_f = bld!(self.builder.build_signed_int_to_float(r, self.context.f64_type(), "itof"))?;
                self.compile_float_binop(op, l, r_f)
            }
            // Int-Pointer promotion: treat the int as a pointer (e.g., string stored as i64 in list)
            (BasicValueEnum::IntValue(l), BasicValueEnum::PointerValue(r)) => {
                let l_ptr = self.i64_to_ptr(l)?;
                self.compile_binop(op, l_ptr.into(), r.into())
            }
            (BasicValueEnum::PointerValue(l), BasicValueEnum::IntValue(r)) => {
                let r_ptr = self.i64_to_ptr(r)?;
                self.compile_binop(op, l.into(), r_ptr.into())
            }
            _ => Err(self.err("type mismatch in binary operation")),
        }
    }

    fn compile_float_binop(
        &mut self,
        op: BinOp,
        l: inkwell::values::FloatValue<'ctx>,
        r: inkwell::values::FloatValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        use inkwell::FloatPredicate;
        let result: BasicValueEnum<'ctx> = match op {
            BinOp::Add => bld!(self.builder.build_float_add(l, r, "fadd"))?.into(),
            BinOp::Sub => bld!(self.builder.build_float_sub(l, r, "fsub"))?.into(),
            BinOp::Mul => bld!(self.builder.build_float_mul(l, r, "fmul"))?.into(),
            BinOp::Div => bld!(self.builder.build_float_div(l, r, "fdiv"))?.into(),
            BinOp::Mod => bld!(self.builder.build_float_rem(l, r, "fmod"))?.into(),
            BinOp::Lt => bld!(self.builder.build_float_compare(FloatPredicate::OLT, l, r, "flt"))?.into(),
            BinOp::Gt => bld!(self.builder.build_float_compare(FloatPredicate::OGT, l, r, "fgt"))?.into(),
            BinOp::Eq => bld!(self.builder.build_float_compare(FloatPredicate::OEQ, l, r, "feq"))?.into(),
            BinOp::NotEq => bld!(self.builder.build_float_compare(FloatPredicate::ONE, l, r, "fne"))?.into(),
            BinOp::LtEq => bld!(self.builder.build_float_compare(FloatPredicate::OLE, l, r, "fle"))?.into(),
            BinOp::GtEq => bld!(self.builder.build_float_compare(FloatPredicate::OGE, l, r, "fge"))?.into(),
            _ => return Err(self.err(format!("unsupported float op {:?}", op))),
        };
        Ok(result)
    }

    pub(crate) fn compile_bool_binop(
        &mut self,
        op: BinOp,
        l: IntValue<'ctx>,
        r: IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let result = match op {
            BinOp::And => bld!(self.builder.build_and(l, r, "band")),
            BinOp::Or => bld!(self.builder.build_or(l, r, "bor")),
            BinOp::Eq => bld!(self.builder.build_int_compare(IntPredicate::EQ, l, r, "beq")),
            BinOp::NotEq => bld!(self.builder.build_int_compare(IntPredicate::NE, l, r, "bne")),
            _ => return Err(self.err(format!("unsupported bool op {:?}", op))),
        }?;
        Ok(result.into())
    }

    fn compile_assert_cmp(
        &mut self,
        left: &Expr,
        right: &Expr,
        message: Option<&str>,
        op: &str, // "eq" or "ne"
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (left_val, left_kind) = self.compile_expr_with_kind(left, func)?;
        let (right_val, right_kind) = self.compile_expr_with_kind(right, func)?;
        let default_msg = format!("assert_{} failed", op);
        let msg_str = message.unwrap_or(&default_msg);
        let msg_ptr = self.build_c_string_global(msg_str, &format!("assert_{}_msg_{}", op, self.current_line))?;
        let line_val = self.context.i64_type().const_int(self.current_line as u64, false);
        let fn_name = match (&left_kind, &right_kind) {
            (ValKind::Float, _) | (_, ValKind::Float) if op == "eq" => "ore_assert_eq_float",
            (ValKind::Str, _) | (_, ValKind::Str) => &format!("ore_assert_{}_str", op),
            _ => &format!("ore_assert_{}_int", op),
        };
        self.call_rt(fn_name, &[left_val.into(), right_val.into(), msg_ptr.into(), line_val.into()], "")?;
        Ok(self.void_result())
    }

    /// Compile a print expression with full type dispatch (dynamic kinds, typed lists, maps).
    fn compile_print_expr(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: &ValKind,
        inner: &Expr,
    ) -> Result<(), CodeGenError> {
        // Dynamic-kind variables (from Result/Option match bindings)
        if let Expr::Ident(name) = inner {
            if let Some(kind_alloca) = self.dynamic_kind_tags.get(name).copied() {
                let kind_i8 = bld!(self.builder.build_load(self.context.i8_type(), kind_alloca, "dyn_kind"))?.into_int_value();
                let str_ptr = self.call_rt("ore_dynamic_to_str", &[val.into(), kind_i8.into()], "dyntos")?.into_pointer_value();
                self.call_rt("ore_str_print", &[str_ptr.into()], "")?;
                self.call_rt("ore_str_release", &[str_ptr.into()], "")?;
                return Ok(());
            }
        }
        // Typed list printing: kind is already enriched via Ident load
        if let Some(ek) = kind.list_elem_kind() {
            if *ek != ValKind::Int {
                self.compile_typed_list_print(val.into_pointer_value(), ek)?;
                return Ok(());
            }
        }
        // String-valued map printing: kind is already enriched via Ident load
        if kind.map_val_kind() == Some(&ValKind::Str) {
            self.call_rt("ore_map_print_str", &[val.into()], "")?;
            return Ok(());
        }
        self.compile_print(val, kind.clone())?;
        Ok(())
    }

    /// Append default parameter values for any missing arguments in a function call.
    pub(crate) fn fill_default_args(
        &mut self,
        name: &str,
        compiled_args: &mut Vec<inkwell::values::BasicMetadataValueEnum<'ctx>>,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        if let Some(defaults) = self.fn_defaults.get(name).cloned() {
            let num_args = compiled_args.len();
            for default_expr in defaults.iter().skip(num_args).flatten() {
                compiled_args.push(self.compile_expr(default_expr, func)?.into());
            }
        }
        Ok(())
    }

}
