use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue};
use inkwell::IntPredicate;

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_record_construct(
        &mut self,
        type_name: &str,
        fields: &[(String, Expr)],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let info = self.records.get(type_name).ok_or_else(|| CodeGenError {
            line: Some(self.current_line), msg: format!("undefined type '{}'", type_name),
        })?;
        let struct_type = info.struct_type;
        let field_names = info.field_names.clone();

        let alloca = bld!(self.builder.build_alloca(struct_type, type_name))?;

        for (name, expr) in fields {
            let idx = field_names.iter().position(|n| n == name).ok_or_else(|| CodeGenError {
                line: Some(self.current_line), msg: format!("unknown field '{}' on type '{}'", name, type_name),
            })?;
            let val = self.compile_expr(expr, func)?;
            let field_ptr = bld!(self.builder.build_struct_gep(struct_type, alloca, idx as u32, &format!("{}.{}", type_name, name)))?;
            bld!(self.builder.build_store(field_ptr, val))?;
        }

        let result = bld!(self.builder.build_load(struct_type, alloca, "record"))?;
        Ok((result, ValKind::Record(type_name.to_string())))
    }

    pub(crate) fn compile_field_access(
        &mut self,
        object: &Expr,
        field: &str,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        let type_name = match &obj_kind {
            ValKind::Record(name) => name.clone(),
            _ => return Err(CodeGenError { line: Some(self.current_line), msg: "field access on non-record type".into() }),
        };

        let info = self.records.get(&type_name).ok_or_else(|| CodeGenError {
            line: Some(self.current_line), msg: format!("undefined type '{}'", type_name),
        })?;
        let struct_type = info.struct_type;
        let idx = info.field_names.iter().position(|n| n == field).ok_or_else(|| CodeGenError {
            line: Some(self.current_line), msg: format!("unknown field '{}' on type '{}'", field, type_name),
        })?;
        let field_kind = info.field_kinds[idx].clone();

        // Store the struct to an alloca so we can GEP into it
        let alloca = bld!(self.builder.build_alloca(struct_type, "tmp"))?;
        bld!(self.builder.build_store(alloca, obj_val))?;
        let field_ty = self.kind_to_llvm_type(&field_kind);
        let field_ptr = bld!(self.builder.build_struct_gep(struct_type, alloca, idx as u32, field))?;
        let result = bld!(self.builder.build_load(field_ty, field_ptr, field))?;
        Ok((result, field_kind))
    }

    pub(crate) fn compile_optional_chain(
        &mut self,
        object: &Expr,
        field: &str,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        if obj_kind != ValKind::Option {
            return Err(CodeGenError { line: Some(self.current_line), msg: "?. operator requires an Option value".into() });
        }

        let opt_ty = self.option_type();
        let alloca = bld!(self.builder.build_alloca(opt_ty, "optchain"))?;
        bld!(self.builder.build_store(alloca, obj_val))?;

        let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
        let is_some = bld!(self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
        ))?;

        let some_bb = self.context.append_basic_block(func, "optchain_some");
        let none_bb = self.context.append_basic_block(func, "optchain_none");
        let merge_bb = self.context.append_basic_block(func, "optchain_merge");

        bld!(self.builder.build_conditional_branch(is_some, some_bb, none_bb))?;

        // Some branch: unwrap, field access, wrap in Some
        self.builder.position_at_end(some_bb);
        let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 2, "val_ptr"))?;
        let inner_i64 = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?.into_int_value();

        // Perform field access on the inner value
        let inner_expr = Expr::FieldAccess {
            object: Box::new(object.clone()),
            field: field.to_string(),
        };
        // Instead, use the inner value directly - reinterpret as the record type
        // For simplicity, wrap the result in Some
        let kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 1, "kind_ptr"))?;
        let inner_kind_tag = bld!(self.builder.build_load(self.context.i8_type(), kind_ptr, "ikind"))?.into_int_value();
        let _ = inner_kind_tag;
        let _ = inner_expr;

        // Build a new Some option with the field value
        // For now, we just pass through the i64 payload as the field result
        // This works for record fields stored as i64
        let result_alloca = bld!(self.builder.build_alloca(opt_ty, "optres"))?;
        let res_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_alloca, 0, "res_tag"))?;
        bld!(self.builder.build_store(res_tag_ptr, self.context.i8_type().const_int(1, false)))?;
        let res_kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_alloca, 1, "res_kind"))?;
        // Store Int kind for now (we don't know the actual kind of the field)
        bld!(self.builder.build_store(res_kind_ptr, self.context.i8_type().const_int(0, false)))?;
        let res_val_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_alloca, 2, "res_val"))?;
        bld!(self.builder.build_store(res_val_ptr, inner_i64))?;
        let some_result = bld!(self.builder.build_load(opt_ty, result_alloca, "some_res"))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;

        // None branch: return None
        self.builder.position_at_end(none_bb);
        let none_alloca = bld!(self.builder.build_alloca(opt_ty, "none_res"))?;
        let none_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, none_alloca, 0, "none_tag"))?;
        bld!(self.builder.build_store(none_tag_ptr, self.context.i8_type().const_int(0, false)))?;
        let none_result = bld!(self.builder.build_load(opt_ty, none_alloca, "none_res"))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;

        self.builder.position_at_end(merge_bb);
        let phi = bld!(self.builder.build_phi(opt_ty, "optchain_result"))?;
        phi.add_incoming(&[(&some_result, some_bb), (&none_result, none_bb)]);

        Ok((phi.as_basic_value(), ValKind::Option))
    }

    pub(crate) fn compile_optional_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        if obj_kind != ValKind::Option {
            return Err(CodeGenError { line: Some(self.current_line), msg: "?. operator requires an Option value".into() });
        }

        let opt_ty = self.option_type();
        let alloca = bld!(self.builder.build_alloca(opt_ty, "optmethod"))?;
        bld!(self.builder.build_store(alloca, obj_val))?;

        let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
        let is_some = bld!(self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
        ))?;

        let some_bb = self.context.append_basic_block(func, "optmethod_some");
        let none_bb = self.context.append_basic_block(func, "optmethod_none");
        let merge_bb = self.context.append_basic_block(func, "optmethod_merge");

        bld!(self.builder.build_conditional_branch(is_some, some_bb, none_bb))?;

        // Some branch: unwrap, call method, wrap result in Some
        self.builder.position_at_end(some_bb);
        let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 2, "val_ptr"))?;
        let inner_val = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?;
        let kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 1, "kind_ptr"))?;
        let inner_kind_tag = bld!(self.builder.build_load(self.context.i8_type(), kind_ptr, "ikind"))?.into_int_value();

        // Determine inner ValKind from tag and call method on the inner value
        // For now, try calling method on inner as Int (most common case)
        let _ = inner_kind_tag;
        let inner_kind = ValKind::Int;
        let (result_val, result_kind) = self.call_method_on_value(inner_val, &inner_kind, method, args, func)?;

        // Wrap result in Some
        let result_opt_alloca = bld!(self.builder.build_alloca(opt_ty, "optres"))?;
        let res_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_opt_alloca, 0, "res_tag"))?;
        bld!(self.builder.build_store(res_tag_ptr, self.context.i8_type().const_int(1, false)))?;
        let res_kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_opt_alloca, 1, "res_kind"))?;
        let rk_tag = self.valkind_to_tag(&result_kind);
        bld!(self.builder.build_store(res_kind_ptr, self.context.i8_type().const_int(rk_tag as u64, false)))?;
        let res_val_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_opt_alloca, 2, "res_val"))?;
        let result_i64 = self.value_to_i64(result_val)?;
        bld!(self.builder.build_store(res_val_ptr, result_i64))?;
        let some_result = bld!(self.builder.build_load(opt_ty, result_opt_alloca, "some_res"))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;
        let some_end = self.current_block()?;

        // None branch
        self.builder.position_at_end(none_bb);
        let none_alloca = bld!(self.builder.build_alloca(opt_ty, "none_res"))?;
        let none_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, none_alloca, 0, "none_tag"))?;
        bld!(self.builder.build_store(none_tag_ptr, self.context.i8_type().const_int(0, false)))?;
        let none_result = bld!(self.builder.build_load(opt_ty, none_alloca, "none_res"))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;

        self.builder.position_at_end(merge_bb);
        let phi = bld!(self.builder.build_phi(opt_ty, "optmethod_result"))?;
        phi.add_incoming(&[(&some_result, some_end), (&none_result, none_bb)]);

        Ok((phi.as_basic_value(), ValKind::Option))
    }

    pub(crate) fn call_method_on_value(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: &ValKind,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        // Dispatch to appropriate method handler based on kind
        match kind {
            ValKind::Str => self.compile_str_method(val.into_pointer_value().into(), method, args, func),
            ValKind::List => self.compile_list_method(val.into_pointer_value().into(), method, args, func),
            ValKind::Int => {
                match method {
                    "abs" => {
                        let i = val.into_int_value();
                        let neg = bld!(self.builder.build_int_neg(i, "neg"))?;
                        let is_neg = bld!(self.builder.build_int_compare(
                            IntPredicate::SLT, i, self.context.i64_type().const_int(0, false), "is_neg"
                        ))?;
                        let result = bld!(self.builder.build_select(is_neg, neg, i, "abs"))?;
                        Ok((result, ValKind::Int))
                    }
                    "to_float" => {
                        let f = bld!(self.builder.build_signed_int_to_float(
                            val.into_int_value(), self.context.f64_type(), "itof"
                        ))?;
                        Ok((f.into(), ValKind::Float))
                    }
                    _ => Err(Self::unknown_method_error("Int", method, &["abs", "to_float", "to_str", "pow", "clamp", "min", "max"])),
                }
            }
            _ => Err(CodeGenError { line: Some(self.current_line), msg: format!("cannot call method '{}' on {:?} in optional chain", method, kind) }),
        }
    }

    pub(crate) fn compile_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;

        // Handle list built-in methods
        if obj_kind == ValKind::List {
            let result = self.compile_list_method(obj_val, method, args, func)?;
            // After push, update the variable's element kind tracking
            if method == "push" {
                if let Expr::Ident(var_name) = object {
                    if let Some(ek) = self.last_list_elem_kind.clone() {
                        self.list_element_kinds.insert(var_name.clone(), ek);
                    }
                }
            }
            return Ok(result);
        }

        // Handle string built-in methods
        if obj_kind == ValKind::Str {
            return self.compile_str_method(obj_val, method, args, func);
        }

        // Handle map built-in methods
        if obj_kind == ValKind::Map {
            let result = self.compile_map_method(obj_val, method, args, func)?;
            // After set, update the variable's value kind tracking
            if method == "set" {
                if let Expr::Ident(var_name) = object {
                    if let Some(vk) = self.last_map_val_kind.clone() {
                        self.map_value_kinds.insert(var_name.clone(), vk);
                    }
                }
            }
            return Ok(result);
        }

        // Handle Option methods
        if obj_kind == ValKind::Option {
            return self.compile_option_method(obj_val, method, args, func);
        }

        // Handle Result methods
        if obj_kind == ValKind::Result {
            return self.compile_result_method(obj_val, method, args, func);
        }

        // Handle Channel methods
        if obj_kind == ValKind::Channel {
            return self.compile_channel_method(obj_val, method, args, func);
        }

        // Handle to_str() on primitive types
        if method == "to_str" {
            let str_val = self.value_to_str(obj_val, obj_kind)?;
            return Ok((str_val.into(), ValKind::Str));
        }

        // Bool methods
        if obj_kind == ValKind::Bool {
            match method {
                "to_int" => {
                    let i_val = bld!(self.builder.build_int_z_extend(
                        obj_val.into_int_value(),
                        self.context.i64_type(),
                        "b2i"
                    ))?;
                    return Ok((i_val.into(), ValKind::Int));
                }
                _ => {}
            }
        }

        // Int methods
        if obj_kind == ValKind::Int {
            match method {
                "to_float" => {
                    let f_val = bld!(self.builder.build_signed_int_to_float(
                        obj_val.into_int_value(),
                        self.context.f64_type(),
                        "i2f"
                    ))?;
                    return Ok((f_val.into(), ValKind::Float));
                }
                "abs" => {
                    let int_val = obj_val.into_int_value();
                    let zero = self.context.i64_type().const_zero();
                    let is_neg = bld!(self.builder.build_int_compare(
                        inkwell::IntPredicate::SLT, int_val, zero, "is_neg"
                    ))?;
                    let neg_val = bld!(self.builder.build_int_neg(int_val, "neg"))?;
                    let result = bld!(self.builder.build_select(is_neg, neg_val, int_val, "abs"))?;
                    return Ok((result, ValKind::Int));
                }
                "max" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: Some(self.current_line), msg: "Int.max() takes 1 argument".into() });
                    }
                    let (other, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let a = obj_val.into_int_value();
                    let b = other.into_int_value();
                    let cmp = bld!(self.builder.build_int_compare(IntPredicate::SGT, a, b, "gt"))?;
                    let result = bld!(self.builder.build_select(cmp, a, b, "max"))?;
                    return Ok((result, ValKind::Int));
                }
                "min" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: Some(self.current_line), msg: "Int.min() takes 1 argument".into() });
                    }
                    let (other, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let a = obj_val.into_int_value();
                    let b = other.into_int_value();
                    let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, a, b, "lt"))?;
                    let result = bld!(self.builder.build_select(cmp, a, b, "min"))?;
                    return Ok((result, ValKind::Int));
                }
                "clamp" => {
                    if args.len() != 2 {
                        return Err(CodeGenError { line: Some(self.current_line), msg: "Int.clamp() takes 2 arguments (min, max)".into() });
                    }
                    let (lo_val, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let (hi_val, _) = self.compile_expr_with_kind(&args[1], func)?;
                    let x = obj_val.into_int_value();
                    let lo = lo_val.into_int_value();
                    let hi = hi_val.into_int_value();
                    let cmp_lo = bld!(self.builder.build_int_compare(IntPredicate::SLT, x, lo, "lt_lo"))?;
                    let v1 = bld!(self.builder.build_select(cmp_lo, lo, x, "clamp_lo"))?;
                    let cmp_hi = bld!(self.builder.build_int_compare(IntPredicate::SGT, v1.into_int_value(), hi, "gt_hi"))?;
                    let result = bld!(self.builder.build_select(cmp_hi, hi, v1.into_int_value(), "clamp"))?;
                    return Ok((result, ValKind::Int));
                }
                "pow" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: Some(self.current_line), msg: "Int.pow() takes 1 argument".into() });
                    }
                    let (exp_val, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let rt = self.rt("ore_int_pow")?;
                    let result = bld!(self.builder.build_call(rt, &[obj_val.into(), exp_val.into()], "pow"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Int));
                }
                "to_str" => {
                    let rt = self.rt("ore_int_to_str")?;
                    let result = bld!(self.builder.build_call(rt, &[obj_val.into()], "i2s"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Str));
                }
                _ => return Err(Self::unknown_method_error("Int", method, &["abs", "to_float", "to_str", "pow", "clamp", "min", "max"])),
            }
        }

        // Float methods
        if obj_kind == ValKind::Float {
            match method {
                "to_int" => {
                    let i_val = bld!(self.builder.build_float_to_signed_int(
                        obj_val.into_float_value(),
                        self.context.i64_type(),
                        "f2i"
                    ))?;
                    return Ok((i_val.into(), ValKind::Int));
                }
                "round" => {
                    let round_fn = self.module.get_function("llvm.round.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.round.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(round_fn, &[obj_val.into()], "round"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "floor" => {
                    let floor_fn = self.module.get_function("llvm.floor.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.floor.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(floor_fn, &[obj_val.into()], "floor"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "ceil" => {
                    let ceil_fn = self.module.get_function("llvm.ceil.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.ceil.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(ceil_fn, &[obj_val.into()], "ceil"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "abs" => {
                    let abs_fn = self.module.get_function("llvm.fabs.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.fabs.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(abs_fn, &[obj_val.into()], "fabs"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "sqrt" => {
                    let sqrt_fn = self.module.get_function("llvm.sqrt.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.sqrt.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(sqrt_fn, &[obj_val.into()], "sqrt"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "max" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: Some(self.current_line), msg: "Float.max() takes 1 argument".into() });
                    }
                    let (other, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let a = obj_val.into_float_value();
                    let b = other.into_float_value();
                    let cmp = bld!(self.builder.build_float_compare(inkwell::FloatPredicate::OGT, a, b, "fgt"))?;
                    let result = bld!(self.builder.build_select(cmp, a, b, "fmax"))?;
                    return Ok((result, ValKind::Float));
                }
                "min" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: Some(self.current_line), msg: "Float.min() takes 1 argument".into() });
                    }
                    let (other, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let a = obj_val.into_float_value();
                    let b = other.into_float_value();
                    let cmp = bld!(self.builder.build_float_compare(inkwell::FloatPredicate::OLT, a, b, "flt"))?;
                    let result = bld!(self.builder.build_select(cmp, a, b, "fmin"))?;
                    return Ok((result, ValKind::Float));
                }
                "clamp" => {
                    if args.len() != 2 {
                        return Err(CodeGenError { line: Some(self.current_line), msg: "Float.clamp() takes 2 arguments (min, max)".into() });
                    }
                    let (lo_val, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let (hi_val, _) = self.compile_expr_with_kind(&args[1], func)?;
                    let x = obj_val.into_float_value();
                    let lo = lo_val.into_float_value();
                    let hi = hi_val.into_float_value();
                    let cmp_lo = bld!(self.builder.build_float_compare(inkwell::FloatPredicate::OLT, x, lo, "fclamp_lo"))?;
                    let v1 = bld!(self.builder.build_select(cmp_lo, lo, x, "fclamp1"))?;
                    let cmp_hi = bld!(self.builder.build_float_compare(inkwell::FloatPredicate::OGT, v1.into_float_value(), hi, "fclamp_hi"))?;
                    let result = bld!(self.builder.build_select(cmp_hi, hi, v1.into_float_value(), "fclamp"))?;
                    return Ok((result, ValKind::Float));
                }
                "pow" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: Some(self.current_line), msg: "Float.pow() takes 1 argument".into() });
                    }
                    let (exp, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let pow_fn = self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.pow.f64",
                            f64_type.fn_type(&[f64_type.into(), f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(pow_fn, &[obj_val.into(), exp.into()], "fpow"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "to_str" => {
                    let rt = self.rt("ore_float_to_str")?;
                    let result = bld!(self.builder.build_call(rt, &[obj_val.into()], "f2s"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Str));
                }
                "format" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: Some(self.current_line), msg: "Float.format() takes 1 argument (decimals)".into() });
                    }
                    let (dec_val, dec_kind) = self.compile_expr_with_kind(&args[0], func)?;
                    let dec_i = match dec_kind {
                        ValKind::Int => dec_val.into_int_value(),
                        _ => return Err(CodeGenError { line: Some(self.current_line), msg: "Float.format() argument must be Int (decimals)".into() }),
                    };
                    let rt = self.rt("ore_float_format")?;
                    let result = bld!(self.builder.build_call(rt, &[obj_val.into(), dec_i.into()], "ffmt"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Str));
                }
                _ => return Err(Self::unknown_method_error("Float", method, &["abs", "floor", "ceil", "round", "sqrt", "pow", "to_int", "to_str", "format", "clamp", "min", "max"])),
            }
        }

        let type_name = match &obj_kind {
            ValKind::Record(name) => name.clone(),
            _ => return Err(CodeGenError { line: Some(self.current_line), msg: format!("method call on unsupported type: {:?}", obj_kind) }),
        };

        // Look up the mangled function name
        let mangled_name = format!("{}_{}", type_name, method);
        let (called_fn, ret_kind) = self.resolve_function(&mangled_name)?;

        // Build args: object as first arg, then the rest
        let mut compiled_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = Vec::new();
        compiled_args.push(obj_val.into());
        for arg in args {
            compiled_args.push(self.compile_expr(arg, func)?.into());
        }

        let result = bld!(self.builder.build_call(called_fn, &compiled_args, "mcall"))?;
        let val = self.call_result_to_value(result)?;
        Ok((val, ret_kind))
    }

    pub(crate) fn compile_channel_method(
        &mut self,
        ch_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "send" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "channel.send() takes 1 argument".into() });
                }
                let val = self.compile_expr(&args[0], func)?;
                let i64_val = self.value_to_i64(val)?;
                let rt = self.rt("ore_channel_send")?;
                bld!(self.builder.build_call(rt, &[ch_val.into(), i64_val.into()], ""))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            "recv" => {
                let rt = self.rt("ore_channel_recv")?;
                let result = bld!(self.builder.build_call(rt, &[ch_val.into()], "recv"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            _ => Err(Self::unknown_method_error("Channel", method, &["send", "recv"])),
        }
    }

    pub(crate) fn compile_index(
        &mut self,
        object: &Expr,
        index: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        let idx_val = self.compile_expr(index, func)?;

        match obj_kind {
            ValKind::List => {
                let list_get = self.rt("ore_list_get")?;
                let result = bld!(self.builder.build_call(
                    list_get,
                    &[obj_val.into(), idx_val.into()],
                    "list_get"
                ))?;
                let val = self.call_result_to_value(result)?;
                let elem_kind = self.last_list_elem_kind.clone().unwrap_or(ValKind::Int);
                let typed_val = self.list_elem_from_i64(val, &elem_kind)?;
                Ok((typed_val, elem_kind))
            }
            ValKind::Map => {
                // Convert non-string keys to strings for map access
                let map_key = if idx_val.is_pointer_value() {
                    idx_val // already a string pointer
                } else {
                    self.value_to_str(idx_val, ValKind::Int)?.into()
                };
                let map_get = self.rt("ore_map_get")?;
                let result = bld!(self.builder.build_call(
                    map_get,
                    &[obj_val.into(), map_key.into()],
                    "map_get"
                ))?;
                let val = self.call_result_to_value(result)?;
                // Look up tracked value kind for this map variable
                let val_kind = if let Expr::Ident(name) = object {
                    self.map_value_kinds.get(name).cloned().unwrap_or(ValKind::Int)
                } else {
                    ValKind::Int
                };
                // If the value is a pointer type (Str, List, Map), convert i64 -> ptr
                match val_kind {
                    ValKind::Str | ValKind::List | ValKind::Map => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            val.into_int_value(),
                            self.context.ptr_type(inkwell::AddressSpace::default()),
                            "i2p"
                        ))?;
                        Ok((ptr.into(), val_kind))
                    }
                    _ => Ok((val, val_kind))
                }
            }
            _ => Err(CodeGenError { line: Some(self.current_line), msg: "indexing only supported on lists and maps".into() }),
        }
    }

}
