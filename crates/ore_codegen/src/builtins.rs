use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::IntPredicate;

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_typed_list_print(
        &mut self,
        list_ptr: PointerValue<'ctx>,
        elem_kind: &ValKind,
    ) -> Result<(), CodeGenError> {
        // Print "[" using ore_str_print
        let open_bracket = self.compile_string_literal("[")?;
        self.call_rt("ore_str_print_no_newline", &[open_bracket.into()], "")?;
        self.call_rt("ore_str_release", &[open_bracket.into()], "")?;

        let len = self.call_rt("ore_list_len", &[list_ptr.into()], "len")?.into_int_value();
        // Cache list_get for use in loop body
        let list_get = self.rt("ore_list_get")?;

        let current_fn = self.current_fn()?;

        // Loop: for i in 0..len
        let idx_alloca = bld!(self.builder.build_alloca(self.context.i64_type(), "idx"))?;
        bld!(self.builder.build_store(idx_alloca, self.context.i64_type().const_int(0, false)))?;

        let loop_check = self.context.append_basic_block(current_fn, "list_print_check");
        let loop_body = self.context.append_basic_block(current_fn, "list_print_body");
        let loop_end = self.context.append_basic_block(current_fn, "list_print_end");

        bld!(self.builder.build_unconditional_branch(loop_check))?;

        self.builder.position_at_end(loop_check);
        let i = bld!(self.builder.build_load(self.context.i64_type(), idx_alloca, "i"))?.into_int_value();
        let cond = bld!(self.builder.build_int_compare(IntPredicate::SLT, i, len, "cmp"))?;
        bld!(self.builder.build_conditional_branch(cond, loop_body, loop_end))?;

        self.builder.position_at_end(loop_body);
        let i = bld!(self.builder.build_load(self.context.i64_type(), idx_alloca, "i"))?.into_int_value();

        // Print ", " if not first element
        let is_first = bld!(self.builder.build_int_compare(IntPredicate::EQ, i, self.context.i64_type().const_int(0, false), "first"))?;
        let sep_bb = self.context.append_basic_block(current_fn, "print_sep");
        let elem_bb = self.context.append_basic_block(current_fn, "print_elem");
        bld!(self.builder.build_conditional_branch(is_first, elem_bb, sep_bb))?;

        self.builder.position_at_end(sep_bb);
        let sep = self.compile_string_literal(", ")?;
        self.call_rt("ore_str_print_no_newline", &[sep.into()], "")?;
        self.call_rt("ore_str_release", &[sep.into()], "")?;
        bld!(self.builder.build_unconditional_branch(elem_bb))?;

        self.builder.position_at_end(elem_bb);
        let i = bld!(self.builder.build_load(self.context.i64_type(), idx_alloca, "i"))?.into_int_value();

        // Get element
        let elem_result = bld!(self.builder.build_call(list_get, &[list_ptr.into(), i.into()], "elem"))?;
        let elem_i64 = self.call_result_to_value(elem_result)?.into_int_value();

        // Convert and print based on element kind
        match elem_kind {
            ValKind::Str => {
                let elem_ptr = self.i64_to_ptr(elem_i64)?;
                self.call_rt("ore_str_print_no_newline", &[elem_ptr.into()], "")?;
            }
            ValKind::Float => {
                let f = bld!(self.builder.build_bit_cast(elem_i64, self.context.f64_type(), "f"))?.into_float_value();
                self.call_rt("ore_print_float_no_newline", &[f.into()], "")?;
            }
            ValKind::Bool => {
                let b = bld!(self.builder.build_int_truncate(elem_i64, self.context.i8_type(), "b"))?;
                self.call_rt("ore_print_bool_no_newline", &[b.into()], "")?;
            }
            _ => {
                self.call_rt("ore_print_int_no_newline", &[elem_i64.into()], "")?;
            }
        }

        // Increment
        let next_i = bld!(self.builder.build_int_add(i, self.context.i64_type().const_int(1, false), "next_i"))?;
        bld!(self.builder.build_store(idx_alloca, next_i))?;
        bld!(self.builder.build_unconditional_branch(loop_check))?;

        self.builder.position_at_end(loop_end);
        // Print "]\n"
        let close_str = self.compile_string_literal("]")?;
        self.call_rt("ore_str_print", &[close_str.into()], "")?;
        self.call_rt("ore_str_release", &[close_str.into()], "")?;

        Ok(())
    }

    pub(crate) fn compile_print(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: ValKind,
    ) -> Result<(), CodeGenError> {
        match kind {
            ValKind::Str => {
                self.call_rt("ore_str_print", &[val.into()], "")?;
            }
            ValKind::Bool => {
                let ext = self.bool_to_i8(val.into_int_value())?;
                self.call_rt("ore_print_bool", &[ext.into()], "")?;
            }
            ValKind::Float => {
                self.call_rt("ore_print_float", &[val.into()], "")?;
            }
            ValKind::List(_) => {
                self.call_rt("ore_list_print", &[val.into()], "")?;
            }
            ValKind::Map => {
                self.call_rt("ore_map_print", &[val.into()], "")?;
            }
            ValKind::Record(ref name) | ValKind::Enum(ref name) => {
                let s = if matches!(kind, ValKind::Record(_)) {
                    self.record_to_str(val, name)?
                } else {
                    self.enum_to_str(val, name)?
                };
                self.call_rt("ore_str_print", &[s.into()], "")?;
                self.call_rt("ore_str_release", &[s.into()], "")?;
            }
            _ => {
                self.call_rt("ore_print_int", &[val.into()], "")?;
            }
        }
        Ok(())
    }

    /// Try to compile a call to a built-in stdlib function.
    /// Returns `Ok(Some(...))` if the name matched a builtin, `Ok(None)` if not.
    pub(crate) fn compile_builtin_call(
        &mut self,
        name: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<Option<(BasicValueEnum<'ctx>, ValKind)>, CodeGenError> {
        match name {
            "abs" => {
                self.check_arity("abs", args, 1)?;
                let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                match kind {
                    ValKind::Int => {
                        let x = val.into_int_value();
                        let shift = self.context.i64_type().const_int(63, false);
                        let sign = bld!(self.builder.build_right_shift(x, shift, true, "sign"))?;
                        let xored = bld!(self.builder.build_xor(x, sign, "xor"))?;
                        let result = bld!(self.builder.build_int_sub(xored, sign, "abs"))?;
                        Ok(Some((result.into(), ValKind::Int)))
                    }
                    ValKind::Float => {
                        let x = val.into_float_value();
                        let neg = bld!(self.builder.build_float_neg(x, "neg"))?;
                        let zero = self.context.f64_type().const_float(0.0);
                        let is_neg = bld!(self.builder.build_float_compare(
                            inkwell::FloatPredicate::OLT, x, zero, "is_neg"
                        ))?;
                        let result = bld!(self.builder.build_select(is_neg, neg, x, "abs"))?;
                        Ok(Some((result, ValKind::Float)))
                    }
                    _ => Err(self.err("abs requires Int or Float")),
                }
            }
            "min" | "max" => {
                self.check_arity(name, args, 2)?;
                let (a, ak) = self.compile_expr_with_kind(&args[0], func)?;
                let (b, _) = self.compile_expr_with_kind(&args[1], func)?;
                let is_min = name == "min";
                if ak == ValKind::Float {
                    let fpred = if is_min { inkwell::FloatPredicate::OLT } else { inkwell::FloatPredicate::OGT };
                    let cmp = bld!(self.builder.build_float_compare(fpred, a.into_float_value(), b.into_float_value(), "cmp"))?;
                    let result = bld!(self.builder.build_select(cmp, a, b, name))?;
                    return Ok(Some((result, ValKind::Float)));
                }
                let ipred = if is_min { inkwell::IntPredicate::SLT } else { inkwell::IntPredicate::SGT };
                let cmp = bld!(self.builder.build_int_compare(ipred, a.into_int_value(), b.into_int_value(), "cmp"))?;
                let result = bld!(self.builder.build_select(cmp, a, b, name))?;
                Ok(Some((result, ValKind::Int)))
            }
            "channel" => {
                let val = self.call_rt("ore_channel_new", &[], "ch")?;
                Ok(Some((val, ValKind::Channel)))
            }
            "readln" | "input" => {
                if args.len() == 1 {
                    let (prompt, _) = self.compile_expr_with_kind(&args[0], func)?;
                    self.call_rt("ore_str_print_no_newline", &[prompt.into()], "")?;
                }
                let val = self.call_rt("ore_readln", &[], "readln")?;
                Ok(Some((val, ValKind::Str)))
            }
            "file_read" => {
                self.check_arity("file_read", args, 1)?;
                let path_val = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_file_read", &[path_val.into()], "file_read")?;
                Ok(Some((val, ValKind::Str)))
            }
            "file_read_lines" => {
                self.check_arity("file_read_lines", args, 1)?;
                let path_val = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_file_read_lines", &[path_val.into()], "file_read_lines")?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok(Some((val, ValKind::list_of(ValKind::Str))))
            }
            "file_write" | "file_append" => {
                self.check_arity(name, args, 2)?;
                let path_val = self.compile_expr(&args[0], func)?;
                let content_val = self.compile_expr(&args[1], func)?;
                let rt_name = format!("ore_{}", name);
                let val = self.call_rt(&rt_name, &[path_val.into(), content_val.into()], name)?;
                Ok(Some((val, ValKind::Bool)))
            }
            "file_exists" => {
                self.check_arity("file_exists", args, 1)?;
                let path_val = self.compile_expr(&args[0], func)?;
                let i8_val = self.call_rt("ore_file_exists", &[path_val.into()], "file_exists")?.into_int_value();
                let bool_val = self.i8_to_bool(i8_val)?;
                Ok(Some((bool_val.into(), ValKind::Bool)))
            }
            "env_get" => {
                self.check_arity("env_get", args, 1)?;
                let key = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_env_get", &[key.into()], "env_get")?;
                Ok(Some((val, ValKind::Str)))
            }
            "env_set" => {
                self.check_arity("env_set", args, 2)?;
                let key = self.compile_expr(&args[0], func)?;
                let value = self.compile_expr(&args[1], func)?;
                self.call_rt("ore_env_set", &[key.into(), value.into()], "")?;
                Ok(Some(self.void_result()))
            }
            "args" => {
                let val = self.call_rt("ore_args", &[], "args")?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok(Some((val, ValKind::list_of(ValKind::Str))))
            }
            "eprint" => {
                self.check_arity("eprint", args, 1)?;
                let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                let rt_name = match kind {
                    ValKind::Str => "ore_eprint_str",
                    ValKind::Float => "ore_eprint_float",
                    ValKind::Bool => "ore_eprint_bool",
                    _ => "ore_eprint_int",
                };
                self.call_rt(rt_name, &[val.into()], "")?;
                Ok(Some(self.void_result()))
            }
            "exit" => {
                self.check_arity("exit", args, 1)?;
                let code = self.compile_expr(&args[0], func)?;
                self.call_rt("ore_exit", &[code.into()], "")?;
                Ok(Some(self.void_result()))
            }
            "exec" => {
                self.check_arity("exec", args, 1)?;
                let cmd_val = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_exec", &[cmd_val.into()], "exec")?;
                Ok(Some((val, ValKind::Str)))
            }
            "str" => {
                self.check_arity("str", args, 1)?;
                let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                let str_val = self.value_to_str(val, kind)?;
                Ok(Some((str_val.into(), ValKind::Str)))
            }
            "int" => {
                self.check_arity("int", args, 1)?;
                let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                match kind {
                    ValKind::Int => Ok(Some((val, ValKind::Int))),
                    ValKind::Float => {
                        let i = bld!(self.builder.build_float_to_signed_int(val.into_float_value(), self.context.i64_type(), "ftoi"))?;
                        Ok(Some((i.into(), ValKind::Int)))
                    }
                    ValKind::Bool => {
                        let i = bld!(self.builder.build_int_z_extend(val.into_int_value(), self.context.i64_type(), "btoi"))?;
                        Ok(Some((i.into(), ValKind::Int)))
                    }
                    ValKind::Str => {
                        let v = self.call_rt("ore_str_to_int", &[val.into()], "stoi")?;
                        Ok(Some((v, ValKind::Int)))
                    }
                    _ => Err(self.err("int() cannot convert this type")),
                }
            }
            "float" => {
                self.check_arity("float", args, 1)?;
                let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                match kind {
                    ValKind::Float => Ok(Some((val, ValKind::Float))),
                    ValKind::Int => {
                        let f = self.coerce_to_float(val, &kind, "float()")?;
                        Ok(Some((f.into(), ValKind::Float)))
                    }
                    ValKind::Str => {
                        let v = self.call_rt("ore_str_to_float", &[val.into()], "stof")?;
                        Ok(Some((v, ValKind::Float)))
                    }
                    _ => Err(self.err("float() cannot convert this type")),
                }
            }
            "ord" => {
                self.check_arity("ord", args, 1)?;
                let str_val = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_ord", &[str_val.into()], "ord")?;
                Ok(Some((val, ValKind::Int)))
            }
            "chr" => {
                self.check_arity("chr", args, 1)?;
                let int_val = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_chr", &[int_val.into()], "chr")?;
                Ok(Some((val, ValKind::Str)))
            }
            "type_of" | "typeof" => {
                self.check_arity(name, args, 1)?;
                let (_, kind) = self.compile_expr_with_kind(&args[0], func)?;
                let type_name = Self::valkind_to_name(&kind);
                let str_val = self.compile_string_literal(&type_name)?;
                Ok(Some((str_val.into(), ValKind::Str)))
            }
            "rand_int" => {
                self.check_arity("rand_int", args, 2)?;
                let low = self.compile_expr(&args[0], func)?;
                let high = self.compile_expr(&args[1], func)?;
                let val = self.call_rt("ore_rand_int", &[low.into(), high.into()], "rand")?;
                Ok(Some((val, ValKind::Int)))
            }
            "time_now" | "time_ms" => {
                let rt_name = format!("ore_{}", name);
                let val = self.call_rt(&rt_name, &[], name)?;
                Ok(Some((val, ValKind::Int)))
            }
            "json_parse" => {
                self.check_arity("json_parse", args, 1)?;
                let str_val = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_json_parse", &[str_val.into()], "json_parse")?;
                Ok(Some((val, ValKind::Map)))
            }
            "json_stringify" => {
                self.check_arity("json_stringify", args, 1)?;
                let map_val = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_json_stringify", &[map_val.into()], "json_stringify")?;
                Ok(Some((val, ValKind::Str)))
            }
            "repeat" => {
                self.check_arity("repeat", args, 2)?;
                let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                let val_i64 = self.value_to_i64(val)?;
                let count = self.compile_expr(&args[1], func)?;
                let list_val = self.call_rt("ore_list_repeat", &[val_i64.into(), count.into()], "repeat")?;
                let kind_for_list = kind.clone();
                self.last_list_elem_kind = Some(kind);
                Ok(Some((list_val, ValKind::list_of(kind_for_list))))
            }
            "range" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(self.err("range takes 2-3 arguments (start, end, [step])"));
                }
                let start = self.compile_expr(&args[0], func)?;
                let end = self.compile_expr(&args[1], func)?;
                let val = if args.len() == 3 {
                    let step = self.compile_expr(&args[2], func)?;
                    self.call_rt("ore_range_step", &[start.into(), end.into(), step.into()], "range")?
                } else {
                    self.call_rt("ore_range", &[start.into(), end.into()], "range")?
                };
                self.last_list_elem_kind = Some(ValKind::Int);
                Ok(Some((val, ValKind::list_of(ValKind::Int))))
            }
            "len" => {
                self.check_arity("len()", args, 1)?;
                let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                match kind {
                    ValKind::Str => Ok(Some((self.call_rt("ore_str_len", &[val.into()], "slen")?, ValKind::Int))),
                    ValKind::List(_) => Ok(Some((self.call_rt("ore_list_len", &[val.into()], "llen")?, ValKind::Int))),
                    ValKind::Map => Ok(Some((self.call_rt("ore_map_len", &[val.into()], "mlen")?, ValKind::Int))),
                    _ => Err(self.err("len() not supported on this type")),
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
                self.call_rt("ore_assert_fail", &[msg.into()], "")?;
                bld!(self.builder.build_unreachable())?;

                self.builder.position_at_end(pass_bb);
                Ok(Some(self.void_result()))
            }
            // Math functions
            "sqrt" | "sin" | "cos" | "tan" | "log" | "log10" | "exp" | "floor" | "ceil" | "round" | "math_abs" | "math_floor" | "math_ceil" | "math_round" => {
                // round(x, decimals) — 2-arg overload
                if (name == "round" || name == "math_round") && args.len() == 2 {
                    let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                    let f_val = self.coerce_to_float(val, &kind, "round()")?;
                    let (dec_val, dec_kind) = self.compile_expr_with_kind(&args[1], func)?;
                    let dec_i = match dec_kind {
                        ValKind::Int => dec_val.into_int_value(),
                        _ => return Err(self.err("round() second argument must be Int (decimals)")),
                    };
                    let val = self.call_rt("ore_float_round_to", &[f_val.into(), dec_i.into()], "round_to")?;
                    return Ok(Some((val, ValKind::Float)));
                }
                self.check_arity(name, args, 1)?;
                let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                let f_val = self.coerce_to_float(val, &kind, &format!("{}()", name))?;
                let rt_name = format!("ore_math_{}", name.strip_prefix("math_").unwrap_or(name));
                let val = self.call_rt(&rt_name, &[f_val.into()], name)?;
                Ok(Some((val, ValKind::Float)))
            }
            "pow" | "atan2" => {
                self.check_arity(name, args, 2)?;
                let (a, ak) = self.compile_expr_with_kind(&args[0], func)?;
                let (b, bk) = self.compile_expr_with_kind(&args[1], func)?;
                let a_f = self.coerce_to_float(a, &ak, name)?;
                let b_f = self.coerce_to_float(b, &bk, name)?;
                let rt_name = format!("ore_math_{}", name);
                let val = self.call_rt(&rt_name, &[a_f.into(), b_f.into()], name)?;
                Ok(Some((val, ValKind::Float)))
            }
            "pi" | "euler" | "e" => {
                let rt_name = if name == "pi" { "ore_math_pi" } else { "ore_math_e" };
                let val = self.call_rt(rt_name, &[], name)?;
                Ok(Some((val, ValKind::Float)))
            }
            _ => Ok(None),
        }
    }

}
