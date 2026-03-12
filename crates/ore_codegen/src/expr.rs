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
                let lambda_fn = self.compile_lambda(params, body, func)?;
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

                let var = self.variables.get(name).ok_or_else(|| {
                    let mut msg = format!("undefined variable '{}'", name);
                    let candidates: Vec<&str> = self.variables.keys().map(|s| s.as_str()).collect();
                    if let Some(suggestion) = Self::find_similar(name, &candidates) {
                        msg.push_str(&format!("; did you mean '{}'?", suggestion));
                    }
                    CodeGenError { line: None, msg }
                })?;
                let val = bld!(self.builder.build_load(var.ty, var.ptr, name))?;
                let kind = var.kind.clone();
                // Restore list element kind tracking for method dispatch
                if kind.is_list() {
                    if let Some(elem_kind) = self.list_element_kinds.get(name) {
                        self.last_list_elem_kind = Some(elem_kind.clone());
                    }
                }
                // Restore map value kind tracking for method dispatch
                if kind == ValKind::Map {
                    if let Some(val_kind) = self.map_value_kinds.get(name) {
                        self.last_map_val_kind = Some(val_kind.clone());
                    }
                }
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
                let lhs_elem_kind = self.last_list_elem_kind.clone();
                let (rhs, _rk) = self.compile_expr_with_kind(right, func)?;
                let rhs_elem_kind = self.last_list_elem_kind.clone();

                // List concatenation: list + list
                if lk.is_list() && *op == BinOp::Add {
                    let val = self.call_rt("ore_list_concat", &[lhs.into(), rhs.into()], "lcat")?;
                    // Preserve element kind: prefer RHS (the appended elements) if it has a concrete kind
                    if rhs_elem_kind.is_some() {
                        self.last_list_elem_kind = rhs_elem_kind;
                    } else if lhs_elem_kind.is_some() {
                        self.last_list_elem_kind = lhs_elem_kind;
                    }
                    let elem = self.last_list_elem_kind.clone();
                    return Ok((val, ValKind::List(elem.map(Box::new))));
                }

                // String repetition: str * int
                if lk == ValKind::Str && *op == BinOp::Mul {
                    let val = self.call_rt("ore_str_repeat", &[lhs.into(), rhs.into()], "srepeat")?;
                    return Ok((val, ValKind::Str));
                }

                // If both sides are strings but represented as i64 (e.g. in lambdas), convert to pointers
                let (lhs, rhs) = if lk == ValKind::Str && _rk == ValKind::Str {
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
                    _ => if lk == ValKind::Float || _rk == ValKind::Float { ValKind::Float } else { lk },
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
                // Check if printing a dynamic-kind variable (from Result/Option match)
                if let Expr::Ident(name) = inner.as_ref() {
                    if let Some(kind_alloca) = self.dynamic_kind_tags.get(name).copied() {
                        let kind_i8 = bld!(self.builder.build_load(self.context.i8_type(), kind_alloca, "dyn_kind"))?.into_int_value();
                        let str_ptr = self.call_rt("ore_dynamic_to_str", &[val.into(), kind_i8.into()], "dyntos")?.into_pointer_value();
                        let pf = self.rt("ore_str_print")?;
                        bld!(self.builder.build_call(pf, &[str_ptr.into()], ""))?;
                        let release = self.rt("ore_str_release")?;
                        bld!(self.builder.build_call(release, &[str_ptr.into()], ""))?;
                        return Ok(self.void_result());
                    }
                    // Check for typed list printing
                    if kind.is_list() {
                        if let Some(elem_kind) = self.list_element_kinds.get(name).cloned() {
                            match elem_kind {
                                ValKind::Int => {} // Fall through to default int list print
                                _ => {
                                    // Generate inline typed list print loop
                                    self.compile_typed_list_print(val.into_pointer_value(), &elem_kind)?;
                                    return Ok(self.void_result());
                                }
                            }
                        }
                    }
                }
                // Check for typed list printing via last_list_elem_kind (for method calls etc.)
                if kind.is_list() {
                    if let Some(elem_kind) = self.last_list_elem_kind.take() {
                        if elem_kind != ValKind::Int {
                            self.compile_typed_list_print(val.into_pointer_value(), &elem_kind)?;
                            return Ok(self.void_result());
                        }
                    }
                }
                // Check for string-valued map printing
                if kind == ValKind::Map {
                    if let Some(ValKind::Str) = self.last_map_val_kind.take() {
                        let pf = self.rt("ore_map_print_str")?;
                        bld!(self.builder.build_call(pf, &[val.into()], ""))?;
                        return Ok(self.void_result());
                    }
                }
                self.compile_print(val, kind)?;
                Ok(self.void_result())
            }
            Expr::Sleep(inner) => {
                let val = self.compile_expr(inner, func)?;
                let ore_sleep = self.rt("ore_sleep")?;
                bld!(self.builder.build_call(ore_sleep, &[val.into()], ""))?;
                Ok(self.void_result())
            }
            Expr::Assert { cond, message } => {
                let cond_val = self.compile_expr(cond, func)?;
                let ore_assert = self.rt("ore_assert")?;
                let msg_str = message.as_deref().unwrap_or("assertion failed");
                let msg_ptr = self.build_c_string_global(msg_str, &format!("assert_msg_{}", self.current_line))?;
                let line_val = self.context.i64_type().const_int(self.current_line as u64, false);
                bld!(self.builder.build_call(ore_assert, &[cond_val.into(), msg_ptr.into(), line_val.into()], ""))?;
                Ok(self.void_result())
            }
            Expr::AssertEq { left, right, message } => {
                let (left_val, left_kind) = self.compile_expr_with_kind(left, func)?;
                let (right_val, right_kind) = self.compile_expr_with_kind(right, func)?;
                let msg_str = message.as_deref().unwrap_or("assert_eq failed");
                let msg_ptr = self.build_c_string_global(msg_str, &format!("assert_eq_msg_{}", self.current_line))?;
                let line_val = self.context.i64_type().const_int(self.current_line as u64, false);
                let fn_name = match (&left_kind, &right_kind) {
                    (ValKind::Float, _) | (_, ValKind::Float) => "ore_assert_eq_float",
                    (ValKind::Str, _) | (_, ValKind::Str) => "ore_assert_eq_str",
                    _ => "ore_assert_eq_int",
                };
                let assert_fn = self.rt(fn_name)?;
                bld!(self.builder.build_call(assert_fn, &[left_val.into(), right_val.into(), msg_ptr.into(), line_val.into()], ""))?;
                Ok(self.void_result())
            }
            Expr::AssertNe { left, right, message } => {
                let (left_val, left_kind) = self.compile_expr_with_kind(left, func)?;
                let (right_val, right_kind) = self.compile_expr_with_kind(right, func)?;
                let msg_str = message.as_deref().unwrap_or("assert_ne failed");
                let msg_ptr = self.build_c_string_global(msg_str, &format!("assert_ne_msg_{}", self.current_line))?;
                let line_val = self.context.i64_type().const_int(self.current_line as u64, false);
                let fn_name = match (&left_kind, &right_kind) {
                    (ValKind::Str, _) | (_, ValKind::Str) => "ore_assert_ne_str",
                    _ => "ore_assert_ne_int",
                };
                let assert_fn = self.rt(fn_name)?;
                bld!(self.builder.build_call(assert_fn, &[left_val.into(), right_val.into(), msg_ptr.into(), line_val.into()], ""))?;
                Ok(self.void_result())
            }
            Expr::Call { func: callee, args } => {
                let name = match callee.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(self.err("only named function calls supported")),
                };

                // Built-in stdlib functions
                match name.as_str() {
                    "abs" => {
                        self.check_arity("abs", args, 1)?;
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        match kind {
                            ValKind::Int => {
                                // abs for int: (x ^ (x >> 63)) - (x >> 63)
                                let x = val.into_int_value();
                                let shift = self.context.i64_type().const_int(63, false);
                                let sign = bld!(self.builder.build_right_shift(x, shift, true, "sign"))?;
                                let xored = bld!(self.builder.build_xor(x, sign, "xor"))?;
                                let result = bld!(self.builder.build_int_sub(xored, sign, "abs"))?;
                                return Ok((result.into(), ValKind::Int));
                            }
                            ValKind::Float => {
                                let x = val.into_float_value();
                                let neg = bld!(self.builder.build_float_neg(x, "neg"))?;
                                let zero = self.context.f64_type().const_float(0.0);
                                let is_neg = bld!(self.builder.build_float_compare(
                                    inkwell::FloatPredicate::OLT, x, zero, "is_neg"
                                ))?;
                                let result = bld!(self.builder.build_select(is_neg, neg, x, "abs"))?;
                                return Ok((result, ValKind::Float));
                            }
                            _ => return Err(self.err("abs requires Int or Float")),
                        }
                    }
                    "min" => {
                        self.check_arity("min", args, 2)?;
                        let (a, ak) = self.compile_expr_with_kind(&args[0], func)?;
                        let (b, _) = self.compile_expr_with_kind(&args[1], func)?;
                        if ak == ValKind::Float {
                            let cmp = bld!(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OLT, a.into_float_value(), b.into_float_value(), "cmp"
                            ))?;
                            let result = bld!(self.builder.build_select(cmp, a, b, "min"))?;
                            return Ok((result, ValKind::Float));
                        }
                        let cmp = bld!(self.builder.build_int_compare(
                            inkwell::IntPredicate::SLT, a.into_int_value(), b.into_int_value(), "cmp"
                        ))?;
                        let result = bld!(self.builder.build_select(cmp, a, b, "min"))?;
                        return Ok((result, ValKind::Int));
                    }
                    "max" => {
                        self.check_arity("max", args, 2)?;
                        let (a, ak) = self.compile_expr_with_kind(&args[0], func)?;
                        let (b, _) = self.compile_expr_with_kind(&args[1], func)?;
                        if ak == ValKind::Float {
                            let cmp = bld!(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OGT, a.into_float_value(), b.into_float_value(), "cmp"
                            ))?;
                            let result = bld!(self.builder.build_select(cmp, a, b, "max"))?;
                            return Ok((result, ValKind::Float));
                        }
                        let cmp = bld!(self.builder.build_int_compare(
                            inkwell::IntPredicate::SGT, a.into_int_value(), b.into_int_value(), "cmp"
                        ))?;
                        let result = bld!(self.builder.build_select(cmp, a, b, "max"))?;
                        return Ok((result, ValKind::Int));
                    }
                    "channel" => {
                        let val = self.call_rt("ore_channel_new", &[], "ch")?;
                        return Ok((val, ValKind::Channel));
                    }
                    "readln" | "input" => {
                        // input("prompt") prints prompt then reads line
                        // input() / readln() just reads line
                        if args.len() == 1 {
                            let (prompt, _) = self.compile_expr_with_kind(&args[0], func)?;
                            let print_fn = self.rt("ore_str_print_no_newline")?;
                            bld!(self.builder.build_call(print_fn, &[prompt.into()], ""))?;
                        }
                        let val = self.call_rt("ore_readln", &[], "readln")?;
                        return Ok((val, ValKind::Str));
                    }
                    "file_read" => {
                        self.check_arity("file_read", args, 1)?;
                        let path_val = self.compile_expr(&args[0], func)?;
                        let val = self.call_rt("ore_file_read", &[path_val.into()], "file_read")?;
                        return Ok((val, ValKind::Str));
                    }
                    "file_read_lines" => {
                        self.check_arity("file_read_lines", args, 1)?;
                        let path_val = self.compile_expr(&args[0], func)?;
                        let val = self.call_rt("ore_file_read_lines", &[path_val.into()], "file_read_lines")?;
                        self.last_list_elem_kind = Some(ValKind::Str);
                        return Ok((val, ValKind::list_of(ValKind::Str)));
                    }
                    "file_write" | "file_append" => {
                        self.check_arity(&name, args, 2)?;
                        let path_val = self.compile_expr(&args[0], func)?;
                        let content_val = self.compile_expr(&args[1], func)?;
                        let rt_name = format!("ore_{}", name);
                        let val = self.call_rt(&rt_name, &[path_val.into(), content_val.into()], name.as_str())?;
                        return Ok((val, ValKind::Bool));
                    }
                    "file_exists" => {
                        self.check_arity("file_exists", args, 1)?;
                        let path_val = self.compile_expr(&args[0], func)?;
                        let i8_val = self.call_rt("ore_file_exists", &[path_val.into()], "file_exists")?.into_int_value();
                        let bool_val = bld!(self.builder.build_int_compare(
                            inkwell::IntPredicate::NE,
                            i8_val,
                            self.context.i8_type().const_int(0, false),
                            "tobool"
                        ))?;
                        return Ok((bool_val.into(), ValKind::Bool));
                    }
                    "env_get" => {
                        self.check_arity("env_get", args, 1)?;
                        let key = self.compile_expr(&args[0], func)?;
                        let val = self.call_rt("ore_env_get", &[key.into()], "env_get")?;
                        return Ok((val, ValKind::Str));
                    }
                    "env_set" => {
                        self.check_arity("env_set", args, 2)?;
                        let key = self.compile_expr(&args[0], func)?;
                        let value = self.compile_expr(&args[1], func)?;
                        let rt = self.rt("ore_env_set")?;
                        bld!(self.builder.build_call(rt, &[key.into(), value.into()], ""))?;
                        return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Int));
                    }
                    "args" => {
                        let val = self.call_rt("ore_args", &[], "args")?;
                        self.last_list_elem_kind = Some(ValKind::Str);
                        return Ok((val, ValKind::list_of(ValKind::Str)));
                    }
                    "eprint" => {
                        self.check_arity("eprint", args, 1)?;
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        match kind {
                            ValKind::Str => {
                                let rt = self.rt("ore_eprint_str")?;
                                bld!(self.builder.build_call(rt, &[val.into()], ""))?;
                            }
                            ValKind::Float => {
                                let rt = self.rt("ore_eprint_float")?;
                                bld!(self.builder.build_call(rt, &[val.into()], ""))?;
                            }
                            ValKind::Bool => {
                                let rt = self.rt("ore_eprint_bool")?;
                                bld!(self.builder.build_call(rt, &[val.into()], ""))?;
                            }
                            _ => {
                                let rt = self.rt("ore_eprint_int")?;
                                bld!(self.builder.build_call(rt, &[val.into()], ""))?;
                            }
                        }
                        return Ok(self.void_result());
                    }
                    "exit" => {
                        self.check_arity("exit", args, 1)?;
                        let code = self.compile_expr(&args[0], func)?;
                        let rt = self.rt("ore_exit")?;
                        bld!(self.builder.build_call(rt, &[code.into()], ""))?;
                        return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Int));
                    }
                    "exec" => {
                        self.check_arity("exec", args, 1)?;
                        let cmd_val = self.compile_expr(&args[0], func)?;
                        let val = self.call_rt("ore_exec", &[cmd_val.into()], "exec")?;
                        return Ok((val, ValKind::Str));
                    }
                    "str" => {
                        self.check_arity("str", args, 1)?;
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        let str_val = self.value_to_str(val, kind)?;
                        return Ok((str_val.into(), ValKind::Str));
                    }
                    "int" => {
                        self.check_arity("int", args, 1)?;
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        match kind {
                            ValKind::Int => return Ok((val, ValKind::Int)),
                            ValKind::Float => {
                                let i = bld!(self.builder.build_float_to_signed_int(val.into_float_value(), self.context.i64_type(), "ftoi"))?;
                                return Ok((i.into(), ValKind::Int));
                            }
                            ValKind::Bool => {
                                let i = bld!(self.builder.build_int_z_extend(val.into_int_value(), self.context.i64_type(), "btoi"))?;
                                return Ok((i.into(), ValKind::Int));
                            }
                            ValKind::Str => {
                                let v = self.call_rt("ore_str_to_int", &[val.into()], "stoi")?;
                                return Ok((v, ValKind::Int));
                            }
                            _ => return Err(self.err("int() cannot convert this type")),
                        }
                    }
                    "float" => {
                        self.check_arity("float", args, 1)?;
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        match kind {
                            ValKind::Float => return Ok((val, ValKind::Float)),
                            ValKind::Int => {
                                let f = self.to_float_val(val, &kind, "float()")?;
                                return Ok((f.into(), ValKind::Float));
                            }
                            ValKind::Str => {
                                let v = self.call_rt("ore_str_to_float", &[val.into()], "stof")?;
                                return Ok((v, ValKind::Float));
                            }
                            _ => return Err(self.err("float() cannot convert this type")),
                        }
                    }
                    "ord" => {
                        self.check_arity("ord", args, 1)?;
                        let str_val = self.compile_expr(&args[0], func)?;
                        let val = self.call_rt("ore_ord", &[str_val.into()], "ord")?;
                        return Ok((val, ValKind::Int));
                    }
                    "chr" => {
                        self.check_arity("chr", args, 1)?;
                        let int_val = self.compile_expr(&args[0], func)?;
                        let val = self.call_rt("ore_chr", &[int_val.into()], "chr")?;
                        return Ok((val, ValKind::Str));
                    }
                    "type_of" => {
                        self.check_arity("type_of", args, 1)?;
                        let (_, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        let kind_tag = self.valkind_to_tag(&kind);
                        let kind_val = self.context.i8_type().const_int(kind_tag as u64, false);
                        let val = self.call_rt("ore_type_of", &[kind_val.into()], "typeof")?;
                        return Ok((val, ValKind::Str));
                    }
                    "rand_int" => {
                        self.check_arity("rand_int", args, 2)?;
                        let low = self.compile_expr(&args[0], func)?;
                        let high = self.compile_expr(&args[1], func)?;
                        let val = self.call_rt("ore_rand_int", &[low.into(), high.into()], "rand")?;
                        return Ok((val, ValKind::Int));
                    }
                    "time_now" | "time_ms" => {
                        let rt_name = format!("ore_{}", name);
                        let val = self.call_rt(&rt_name, &[], name.as_str())?;
                        return Ok((val, ValKind::Int));
                    }
                    "json_parse" => {
                        self.check_arity("json_parse", args, 1)?;
                        let str_val = self.compile_expr(&args[0], func)?;
                        let val = self.call_rt("ore_json_parse", &[str_val.into()], "json_parse")?;
                        return Ok((val, ValKind::Map));
                    }
                    "json_stringify" => {
                        self.check_arity("json_stringify", args, 1)?;
                        let map_val = self.compile_expr(&args[0], func)?;
                        let val = self.call_rt("ore_json_stringify", &[map_val.into()], "json_stringify")?;
                        return Ok((val, ValKind::Str));
                    }
                    "repeat" => {
                        self.check_arity("repeat", args, 2)?;
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        let val_i64 = self.value_to_i64(val)?;
                        let count = self.compile_expr(&args[1], func)?;
                        let list_val = self.call_rt("ore_list_repeat", &[val_i64.into(), count.into()], "repeat")?;
                        let kind_for_list = kind.clone();
                        self.last_list_elem_kind = Some(kind);
                        return Ok((list_val, ValKind::list_of(kind_for_list)));
                    }
                    "range" => {
                        if args.len() < 2 || args.len() > 3 {
                            return Err(self.err("range takes 2-3 arguments (start, end, [step])"));
                        }
                        let start = self.compile_expr(&args[0], func)?;
                        let end = self.compile_expr(&args[1], func)?;
                        let result = if args.len() == 3 {
                            let step = self.compile_expr(&args[2], func)?;
                            let rt = self.rt("ore_range_step")?;
                            bld!(self.builder.build_call(rt, &[start.into(), end.into(), step.into()], "range"))?
                        } else {
                            let rt = self.rt("ore_range")?;
                            bld!(self.builder.build_call(rt, &[start.into(), end.into()], "range"))?
                        };
                        let val = self.call_result_to_value(result)?;
                        self.last_list_elem_kind = Some(ValKind::Int);
                        return Ok((val, ValKind::list_of(ValKind::Int)));
                    }
                    "len" => {
                        self.check_arity("len()", args, 1)?;
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        match kind {
                            ValKind::Str => {
                                return Ok((self.call_rt("ore_str_len", &[val.into()], "slen")?, ValKind::Int));
                            }
                            ValKind::List(_) => {
                                return Ok((self.call_rt("ore_list_len", &[val.into()], "llen")?, ValKind::Int));
                            }
                            ValKind::Map => {
                                return Ok((self.call_rt("ore_map_len", &[val.into()], "mlen")?, ValKind::Int));
                            }
                            _ => return Err(self.err("len() not supported on this type")),
                        }
                    }
                    "assert" => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(self.err("assert takes 1-2 arguments (condition, optional message)"));
                        }
                        let (cond, _) = self.compile_expr_with_kind(&args[0], func)?;
                        let cond_bool = cond.into_int_value();

                        let pass_bb = self.context.append_basic_block(func, "assert_pass");
                        let fail_bb = self.context.append_basic_block(func, "assert_fail");
                        bld!(self.builder.build_conditional_branch(cond_bool, pass_bb, fail_bb))?;

                        self.builder.position_at_end(fail_bb);
                        let msg = if args.len() == 2 {
                            self.compile_expr(&args[1], func)?.into_pointer_value()
                        } else {
                            let line = self.current_line;
                            self.compile_string_literal(&format!("assertion failed at line {}", line))?
                        };
                        let rt = self.rt("ore_assert_fail")?;
                        bld!(self.builder.build_call(rt, &[msg.into()], ""))?;
                        bld!(self.builder.build_unreachable())?;

                        self.builder.position_at_end(pass_bb);
                        return Ok(self.void_result());
                    }
                    "typeof" => {
                        self.check_arity("typeof", args, 1)?;
                        let (_, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        let type_name = match kind {
                            ValKind::Int => "Int",
                            ValKind::Float => "Float",
                            ValKind::Bool => "Bool",
                            ValKind::Str => "Str",
                            ValKind::List(_) => "List",
                            ValKind::Map => "Map",
                            ValKind::Option => "Option",
                            ValKind::Result => "Result",
                            ValKind::Void => "Void",
                            ValKind::Record(ref n) => n.as_str(),
                            ValKind::Enum(ref n) => n.as_str(),
                            ValKind::Channel => "Channel",
                        };
                        let str_val = self.compile_string_literal(type_name)?;
                        return Ok((str_val.into(), ValKind::Str));
                    }
                    // Math functions
                    "sqrt" | "sin" | "cos" | "tan" | "log" | "log10" | "exp" | "floor" | "ceil" | "round" | "math_abs" | "math_floor" | "math_ceil" | "math_round" => {
                        // round(x, decimals) — 2-arg overload
                        if (name == "round" || name == "math_round") && args.len() == 2 {
                            let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                            let f_val = self.to_float_val(val, &kind, "round()")?;
                            let (dec_val, dec_kind) = self.compile_expr_with_kind(&args[1], func)?;
                            let dec_i = match dec_kind {
                                ValKind::Int => dec_val.into_int_value(),
                                _ => return Err(self.err("round() second argument must be Int (decimals)")),
                            };
                            let val = self.call_rt("ore_float_round_to", &[f_val.into(), dec_i.into()], "round_to")?;
                            return Ok((val, ValKind::Float));
                        }
                        self.check_arity(&name, args, 1)?;
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        let f_val = self.to_float_val(val, &kind, &format!("{}()", name))?;
                        let rt_name = format!("ore_math_{}", name.strip_prefix("math_").unwrap_or(&name));
                        let val = self.call_rt(&rt_name, &[f_val.into()], &name)?;
                        return Ok((val, ValKind::Float));
                    }
                    "pow" => {
                        self.check_arity("pow()", args, 2)?;
                        let (base, bk) = self.compile_expr_with_kind(&args[0], func)?;
                        let (exp, ek) = self.compile_expr_with_kind(&args[1], func)?;
                        let base_f = self.to_float_val(base, &bk, "pow()")?;
                        let exp_f = self.to_float_val(exp, &ek, "pow()")?;
                        let val = self.call_rt("ore_math_pow", &[base_f.into(), exp_f.into()], "pow")?;
                        return Ok((val, ValKind::Float));
                    }
                    "atan2" => {
                        self.check_arity("atan2()", args, 2)?;
                        let (y, yk) = self.compile_expr_with_kind(&args[0], func)?;
                        let (x, xk) = self.compile_expr_with_kind(&args[1], func)?;
                        let y_f = self.to_float_val(y, &yk, "atan2()")?;
                        let x_f = self.to_float_val(x, &xk, "atan2()")?;
                        let val = self.call_rt("ore_math_atan2", &[y_f.into(), x_f.into()], "atan2")?;
                        return Ok((val, ValKind::Float));
                    }
                    "pi" => {
                        let val = self.call_rt("ore_math_pi", &[], "pi")?;
                        return Ok((val, ValKind::Float));
                    }
                    "euler" | "e" => {
                        let val = self.call_rt("ore_math_e", &[], "euler")?;
                        return Ok((val, ValKind::Float));
                    }
                    _ => {}
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
                        let (called_fn, ret_kind) = self.monomorphize(&name, &arg_kinds, func)?;
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
                    // Fill in default parameter values for missing args
                    if let Some(defaults) = self.fn_defaults.get(&name).cloned() {
                        let num_args = compiled_args.len();
                        for default_expr in defaults.iter().skip(num_args).flatten() {
                            compiled_args.push(self.compile_expr(default_expr, func)?.into());
                        }
                    }
                    let result = bld!(self.builder.build_call(called_fn, &compiled_args, "call"))?;
                    let val = self.call_result_to_value(result)?;
                    // Propagate list element kind from function return type annotation
                    if ret_kind.is_list() {
                        if let Some(ek) = self.fn_return_list_elem_kind.get(&name) {
                            self.last_list_elem_kind = Some(ek.clone());
                        }
                    }
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
                        let mut msg = format!("undefined function '{}'", name);
                        let candidates: Vec<&str> = self.functions.keys().map(|s| s.as_str()).collect();
                        if let Some(suggestion) = Self::find_similar(&name, &candidates) {
                            msg.push_str(&format!("; did you mean '{}'?", suggestion));
                        }
                        Err(CodeGenError { line: None, msg })
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
                let opt_ty = self.option_type();
                let alloca = bld!(self.builder.build_alloca(opt_ty, "opt_none"))?;
                let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
                bld!(self.builder.build_store(tag_ptr, self.context.i8_type().const_int(0, false)))?;
                let result = bld!(self.builder.build_load(opt_ty, alloca, "none_val"))?;
                Ok((result, ValKind::Option))
            }
            Expr::OptionSome(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                let result = self.build_tagged_wrapper(self.option_type(), 1, &kind, val, "opt_some", "some_val")?;
                Ok((result, ValKind::Option))
            }
            Expr::ResultOk(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                let result = self.build_tagged_wrapper(self.result_type(), 0, &kind, val, "res_ok", "ok_val")?;
                Ok((result, ValKind::Result))
            }
            Expr::ResultErr(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                let result = self.build_tagged_wrapper(self.result_type(), 1, &kind, val, "res_err", "err_val")?;
                Ok((result, ValKind::Result))
            }
            Expr::Try(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                if kind == ValKind::Result {
                    return self.compile_try_result(val, func);
                }
                let opt_ty = self.option_type();
                // Store the option value so we can extract from it
                let alloca = bld!(self.builder.build_alloca(opt_ty, "try_opt"))?;
                bld!(self.builder.build_store(alloca, val))?;
                // Load tag
                let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
                let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
                let is_none = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(0, false), "is_none"
                ))?;
                let some_bb = self.context.append_basic_block(func, "try_some");
                let none_bb = self.context.append_basic_block(func, "try_none");
                bld!(self.builder.build_conditional_branch(is_none, none_bb, some_bb))?;
                // None branch: return None from current function
                self.builder.position_at_end(none_bb);
                let none_alloca = bld!(self.builder.build_alloca(opt_ty, "ret_none"))?;
                let none_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, none_alloca, 0, "ret_tag"))?;
                bld!(self.builder.build_store(none_tag_ptr, self.context.i8_type().const_int(0, false)))?;
                let none_ret = bld!(self.builder.build_load(opt_ty, none_alloca, "none_ret"))?;
                bld!(self.builder.build_return(Some(&none_ret)))?;
                // Some branch: extract value
                self.builder.position_at_end(some_bb);
                let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 2, "val_ptr"))?;
                let extracted = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "unwrapped"))?;
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
                bld!(self.builder.build_int_truncate(lv, self.context.bool_type(), "tobool"))
                    .map_err(|e| CodeGenError { line: None, msg: format!("short-circuit trunc: {e}") })?
            } else {
                lv
            }
        } else {
            return Err(CodeGenError { line: None, msg: "short-circuit: expected boolean operand".to_string() });
        };

        let rhs_block = self.context.append_basic_block(func, "sc_rhs");
        let merge_block = self.context.append_basic_block(func, "sc_merge");

        // For AND: if lhs is false, short-circuit to false; else eval RHS
        // For OR:  if lhs is true, short-circuit to true; else eval RHS
        let short_val = if op == BinOp::And { 0u64 } else { 1u64 };
        bld!(self.builder.build_store(
            result_alloca,
            self.context.i64_type().const_int(short_val, false)
        )).map_err(|e| CodeGenError { line: None, msg: format!("sc store1: {e}") })?;

        if op == BinOp::And {
            bld!(self.builder.build_conditional_branch(lhs_bool, rhs_block, merge_block))
                .map_err(|e| CodeGenError { line: None, msg: format!("sc branch: {e}") })?;
        } else {
            bld!(self.builder.build_conditional_branch(lhs_bool, merge_block, rhs_block))
                .map_err(|e| CodeGenError { line: None, msg: format!("sc branch: {e}") })?;
        }

        // Compile RHS
        self.builder.position_at_end(rhs_block);
        let (rhs, _rk) = self.compile_expr_with_kind(right, func)?;
        let rhs_i64 = if rhs.is_int_value() {
            let rv = rhs.into_int_value();
            if rv.get_type().get_bit_width() == 1 {
                bld!(self.builder.build_int_z_extend(rv, self.context.i64_type(), "rhs_ext"))
                    .map_err(|e| CodeGenError { line: None, msg: format!("sc rhs ext: {e}") })?
            } else if rv.get_type().get_bit_width() != 64 {
                bld!(self.builder.build_int_z_extend(rv, self.context.i64_type(), "rhs_ext"))
                    .map_err(|e| CodeGenError { line: None, msg: format!("sc rhs ext: {e}") })?
            } else {
                rv
            }
        } else {
            return Err(CodeGenError { line: None, msg: "short-circuit: expected boolean operand for RHS".to_string() });
        };
        bld!(self.builder.build_store(result_alloca, rhs_i64))
            .map_err(|e| CodeGenError { line: None, msg: format!("sc store2: {e}") })?;
        bld!(self.builder.build_unconditional_branch(merge_block))
            .map_err(|e| CodeGenError { line: None, msg: format!("sc merge: {e}") })?;

        // Load result from alloca
        self.builder.position_at_end(merge_block);
        let result = bld!(self.builder.build_load(self.context.i64_type(), result_alloca, "sc_result"))
            .map_err(|e| CodeGenError { line: None, msg: format!("sc load: {e}") })?;

        Ok((result, ValKind::Bool))
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
                if self.functions.contains_key(name) || self.module.get_function(name).is_some() {
                    let arg_val = self.compile_expr(arg, current_fn)?;
                    let (called_fn, ret_kind) = self.resolve_function(name)?;
                    let result = bld!(self.builder.build_call(called_fn, &[arg_val.into()], "pipe"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ret_kind))
                } else if self.generic_fns.contains_key(name) {
                    let (arg_val, arg_kind) = self.compile_expr_with_kind(arg, current_fn)?;
                    let (called_fn, ret_kind) = self.monomorphize(name, &[arg_kind], current_fn)?;
                    let result = bld!(self.builder.build_call(called_fn, &[arg_val.into()], "pipe"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ret_kind))
                } else {
                    // Treat as method call: arg.name()
                    let method_call = Expr::MethodCall {
                        object: Box::new(arg.clone()),
                        method: name.clone(),
                        args: vec![],
                    };
                    self.compile_expr_with_kind(&method_call, current_fn)
                }
            }
            Expr::Call { func, args } => {
                let name = match func.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(self.err("pipeline target must be a function")),
                };
                if self.functions.contains_key(&name) || self.module.get_function(&name).is_some() {
                    let arg_val = self.compile_expr(arg, current_fn)?;
                    let (called_fn, ret_kind) = self.resolve_function(&name)?;

                    let mut compiled_args = vec![arg_val.into()];
                    for a in args {
                        compiled_args.push(self.compile_expr(a, current_fn)?.into());
                    }
                    // Fill in default parameter values for missing args
                    if let Some(defaults) = self.fn_defaults.get(&name).cloned() {
                        let num_args = compiled_args.len();
                        for default_expr in defaults.iter().skip(num_args).flatten() {
                            compiled_args.push(self.compile_expr(default_expr, current_fn)?.into());
                        }
                    }

                    let result = bld!(self.builder.build_call(called_fn, &compiled_args, "pipe"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ret_kind))
                } else if self.generic_fns.contains_key(&name) {
                    let (arg_val, arg_kind) = self.compile_expr_with_kind(arg, current_fn)?;
                    let mut compiled_args = vec![arg_val.into()];
                    let mut arg_kinds = vec![arg_kind];
                    for a in args {
                        let (v, k) = self.compile_expr_with_kind(a, current_fn)?;
                        compiled_args.push(v.into());
                        arg_kinds.push(k);
                    }
                    let (called_fn, ret_kind) = self.monomorphize(&name, &arg_kinds, current_fn)?;
                    let result = bld!(self.builder.build_call(called_fn, &compiled_args, "pipe"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ret_kind))
                } else {
                    // Treat as method call: arg.name(args...)
                    let method_call = Expr::MethodCall {
                        object: Box::new(arg.clone()),
                        method: name.clone(),
                        args: args.clone(),
                    };
                    self.compile_expr_with_kind(&method_call, current_fn)
                }
            }
            Expr::Lambda { params, body } => {
                let arg_val = self.compile_expr(arg, current_fn)?;
                let lambda_fn = self.compile_lambda(params, body, current_fn)?;
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
        let rt = self.rt("ore_div_by_zero")?;
        bld!(self.builder.build_call(rt, &[], ""))?;
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
                        let bool_val = bld!(self.builder.build_int_compare(
                            IntPredicate::NE, i8_val,
                            self.context.i8_type().const_int(0, false), "tobool"
                        ))?;
                        Ok(bool_val.into())
                    }
                    BinOp::NotEq => {
                        let i8_val = self.call_rt("ore_str_eq", &[l.into(), r.into()], "seq")?.into_int_value();
                        let bool_val = bld!(self.builder.build_int_compare(
                            IntPredicate::EQ, i8_val,
                            self.context.i8_type().const_int(0, false), "tobool"
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

}
