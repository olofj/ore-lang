use super::*;
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue};

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_str_method(
        &mut self,
        str_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "len" => {
                let rt = self.rt("ore_str_len")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "slen"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "is_empty" => {
                let rt = self.rt("ore_str_len")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "slen"))?;
                let len_val = self.call_result_to_value(result)?.into_int_value();
                let is_zero = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    len_val,
                    self.context.i64_type().const_zero(),
                    "is_empty"
                ))?;
                Ok((is_zero.into(), ValKind::Bool))
            }
            "contains" => {
                self.check_arity("contains", &args, 1)?;
                let needle = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_str_contains")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), needle.into()], "scontains"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    i8_val,
                    self.context.i8_type().const_int(0, false),
                    "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "trim" | "trim_start" | "trim_end" => {
                let fn_name = format!("ore_str_{}", method);
                let rt = self.rt(&fn_name)?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "strim"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "words" => {
                let rt = self.rt("ore_str_split_whitespace")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "words"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok((val, ValKind::list_of(ValKind::Str)))
            }
            "lines" => {
                let rt = self.rt("ore_str_lines")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "lines"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok((val, ValKind::list_of(ValKind::Str)))
            }
            "split" => {
                if args.is_empty() {
                    // split() with no args = split on whitespace
                    let rt = self.rt("ore_str_split_whitespace")?;
                    let result = bld!(self.builder.build_call(rt, &[str_val.into()], "ssplit"))?;
                    let val = self.call_result_to_value(result)?;
                    self.last_list_elem_kind = Some(ValKind::Str);
                    return Ok((val, ValKind::list_of(ValKind::Str)));
                }
                self.check_arity("split", &args, 1)?;
                let delim = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_str_split")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), delim.into()], "ssplit"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok((val, ValKind::list_of(ValKind::Str)))
            }
            "to_int" => {
                let rt = self.rt("ore_str_to_int")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "stoi"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "to_float" => {
                let rt = self.rt("ore_str_to_float")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "stof"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Float))
            }
            "replace" => {
                self.check_arity("replace", &args, 2)?;
                let from = self.compile_expr(&args[0], func)?;
                let to = self.compile_expr(&args[1], func)?;
                let rt = self.rt("ore_str_replace")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), from.into(), to.into()], "sreplace"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "starts_with" => {
                self.check_arity("starts_with", &args, 1)?;
                let prefix = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_str_starts_with")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), prefix.into()], "ssw"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE, i8_val,
                    self.context.i8_type().const_int(0, false), "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "ends_with" => {
                self.check_arity("ends_with", &args, 1)?;
                let suffix = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_str_ends_with")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), suffix.into()], "sew"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE, i8_val,
                    self.context.i8_type().const_int(0, false), "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "to_upper" => {
                let rt = self.rt("ore_str_to_upper")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "supper"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "capitalize" => {
                let rt = self.rt("ore_str_capitalize")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "scap"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "to_lower" => {
                let rt = self.rt("ore_str_to_lower")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "slower"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "substr" => {
                self.check_arity("substr", &args, 2)?;
                let start = self.compile_expr(&args[0], func)?;
                let len = self.compile_expr(&args[1], func)?;
                let rt = self.rt("ore_str_substr")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), start.into(), len.into()], "ssub"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "chars" => {
                let rt = self.rt("ore_str_chars")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "schars"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok((val, ValKind::list_of(ValKind::Str)))
            }
            "char_at" => {
                self.check_arity("char_at", &args, 1)?;
                let idx = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_str_char_at")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), idx.into()], "charat"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "index_of" | "find" => {
                self.check_arity("index_of/find", &args, 1)?;
                let needle = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_str_index_of")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), needle.into()], "sidx"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "slice" => {
                self.check_arity("slice", &args, 2)?;
                let start = self.compile_expr(&args[0], func)?;
                let end = self.compile_expr(&args[1], func)?;
                let rt = self.rt("ore_str_slice")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), start.into(), end.into()], "sslice"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "reverse" => {
                let rt = self.rt("ore_str_reverse")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "srev"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "parse_int" => {
                let rt = self.rt("ore_str_parse_int")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "parse_int"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "parse_float" => {
                let rt = self.rt("ore_str_parse_float")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "parse_float"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Float))
            }
            "repeat" => {
                self.check_arity("repeat", &args, 1)?;
                let count = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_str_repeat")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), count.into()], "srep"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "count" => {
                self.check_arity("count", &args, 1)?;
                let needle = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_str_count")?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), needle.into()], "scount"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "strip_prefix" | "strip_suffix" => {
                self.check_arity(method, &args, 1)?;
                let arg = self.compile_expr(&args[0], func)?;
                let fn_name = format!("ore_str_{}", method);
                let rt = self.rt(&fn_name)?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), arg.into()], "sstrip"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "pad_left" | "pad_right" => {
                if args.len() < 1 || args.len() > 2 {
                    return Err(self.err(format!("{} takes 1-2 arguments (width, [pad_char])", method)));
                }
                let width = self.compile_expr(&args[0], func)?;
                let pad_char = if args.len() > 1 {
                    self.compile_expr(&args[1], func)?
                } else {
                    // Default pad char: space
                    let space = " ";
                    let rt = self.rt("ore_str_new")?;
                    let space_ptr = bld!(self.builder.build_global_string_ptr(space, "pad_space"))?.as_pointer_value();
                    let result = bld!(self.builder.build_call(rt, &[space_ptr.into(), self.context.i32_type().const_int(1, false).into()], "spad"))?;
                    self.call_result_to_value(result)?
                };
                let fn_name = if method == "pad_left" { "ore_str_pad_left" } else { "ore_str_pad_right" };
                let rt = self.rt(fn_name)?;
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), width.into(), pad_char.into()], "spad"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            _ => Err(Self::unknown_method_error("Str", method, &[
                "len", "contains", "starts_with", "ends_with", "to_upper", "to_lower",
                "trim", "trim_start", "trim_end", "replace", "split", "join", "repeat",
                "reverse", "chars", "char_at", "index_of", "find", "slice", "substr",
                "parse_int", "parse_float", "pad_left", "pad_right",
                "capitalize", "count", "strip_prefix", "strip_suffix", "is_empty",
                "words", "lines",
            ])),
        }
    }

    pub(crate) fn compile_string_literal(&mut self, s: &str) -> Result<PointerValue<'ctx>, CodeGenError> {
        let bytes = s.as_bytes();
        let global_name = format!(".str.{}", self.str_counter);
        self.str_counter += 1;

        let i8_type = self.context.i8_type();
        let arr_type = i8_type.array_type(bytes.len() as u32);
        let global = self.module.add_global(arr_type, None, &global_name);
        global.set_initializer(&i8_type.const_array(
            &bytes.iter().map(|&b| i8_type.const_int(b as u64, false)).collect::<Vec<_>>(),
        ));
        global.set_constant(true);

        let str_new = self.rt("ore_str_new")?;
        let ptr = bld!(self.builder.build_pointer_cast(
            global.as_pointer_value(),
            self.ptr_type(),
            "strptr"
        ))?;
        let len = self.context.i32_type().const_int(bytes.len() as u64, false);
        let result = bld!(self.builder.build_call(str_new, &[ptr.into(), len.into()], "str"))?;
        match result.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(BasicValueEnum::PointerValue(p)) => Ok(p),
            inkwell::values::ValueKind::Basic(v) => {
                // Should be pointer but handle gracefully
                Ok(v.into_pointer_value())
            }
            _ => Err(self.err("ore_str_new did not return a pointer")),
        }
    }

    /// Create a global constant string and return a pointer to its data.
    pub(crate) fn builder_string_const(&mut self, s: &str) -> PointerValue<'ctx> {
        let bytes = s.as_bytes();
        let global_name = format!(".str.{}", self.str_counter);
        self.str_counter += 1;
        let i8_type = self.context.i8_type();
        let arr_type = i8_type.array_type(bytes.len() as u32);
        let global = self.module.add_global(arr_type, None, &global_name);
        global.set_initializer(&i8_type.const_array(
            &bytes.iter().map(|&b| i8_type.const_int(b as u64, false)).collect::<Vec<_>>(),
        ));
        global.set_constant(true);
        // build_pointer_cast can't fail for globals in practice
        self.builder.build_pointer_cast(
            global.as_pointer_value(),
            self.ptr_type(),
            "strptr",
        ).unwrap()
    }

    pub(crate) fn compile_string_interp(
        &mut self,
        parts: &[StringPart],
        func: FunctionValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let concat_fn = self.rt("ore_str_concat")?;
        let release_fn = self.rt("ore_str_release")?;

        let mut result: Option<PointerValue<'ctx>> = None;
        let mut temps: Vec<PointerValue<'ctx>> = Vec::new();

        for part in parts {
            let part_ptr = match part {
                StringPart::Lit(s) => {
                    let p = self.compile_string_literal(s)?;
                    temps.push(p);
                    p
                }
                StringPart::Expr(expr) => {
                    let (val, kind) = self.compile_expr_with_kind(expr, func)?;
                    // Check if this is a variable with a dynamic kind tag (from Result/Option match)
                    let p = if let Expr::Ident(name) = expr {
                        if let Some(kind_alloca) = self.dynamic_kind_tags.get(name).copied() {
                            let kind_i8 = bld!(self.builder.build_load(self.context.i8_type(), kind_alloca, "dyn_kind"))?.into_int_value();
                            let dyn_fn = self.rt("ore_dynamic_to_str")?;
                            let result = bld!(self.builder.build_call(dyn_fn, &[val.into(), kind_i8.into()], "dyntos"))?;
                            self.call_result_to_value(result)?.into_pointer_value()
                        } else {
                            self.value_to_str(val, kind)?
                        }
                    } else {
                        self.value_to_str(val, kind)?
                    };
                    temps.push(p);
                    p
                }
            };

            result = Some(match result {
                None => part_ptr,
                Some(acc) => {
                    let concat_result = bld!(self.builder.build_call(
                        concat_fn,
                        &[acc.into(), part_ptr.into()],
                        "concat"
                    ))?;
                    let new_ptr = self.call_result_to_value(concat_result)?.into_pointer_value();
                    // Release the old accumulator (it was a concat result, not a literal or conversion)
                    // We'll release all temps at the end
                    temps.push(new_ptr);
                    new_ptr
                }
            });
        }

        let final_ptr = result.unwrap_or_else(|| self.ptr_type().const_null());

        // Retain the final result before releasing temps
        if !final_ptr.is_null() {
            let retain_fn = self.rt("ore_str_retain")?;
            bld!(self.builder.build_call(retain_fn, &[final_ptr.into()], ""))?;
        }

        // Release all temporaries
        for temp in &temps {
            bld!(self.builder.build_call(release_fn, &[(*temp).into()], ""))?;
        }

        Ok(final_ptr)
    }

    pub(crate) fn value_to_str(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: ValKind,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        match kind {
            ValKind::Str => {
                // Already a string pointer, retain it
                let retain_fn = self.rt("ore_str_retain")?;
                let ptr = val.into_pointer_value();
                bld!(self.builder.build_call(retain_fn, &[ptr.into()], ""))?;
                Ok(ptr)
            }
            ValKind::Int => {
                let int_to_str = self.rt("ore_int_to_str")?;
                let result = bld!(self.builder.build_call(int_to_str, &[val.into()], "itos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
            ValKind::Float => {
                let float_to_str = self.rt("ore_float_to_str")?;
                let result = bld!(self.builder.build_call(float_to_str, &[val.into()], "ftos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
            ValKind::Bool => {
                let bool_to_str = self.rt("ore_bool_to_str")?;
                let int_val = val.into_int_value();
                let ext = {
                    let bw = int_val.get_type().get_bit_width();
                    if bw < 8 {
                        bld!(self.builder.build_int_z_extend(int_val, self.context.i8_type(), "zext"))?
                    } else if bw > 8 {
                        bld!(self.builder.build_int_truncate(int_val, self.context.i8_type(), "trunc"))?
                    } else {
                        int_val
                    }
                };
                let result = bld!(self.builder.build_call(bool_to_str, &[ext.into()], "btos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
            ValKind::Record(ref name) => {
                self.record_to_str(val, name)
            }
            ValKind::Enum(ref name) => {
                self.enum_to_str(val, name)
            }
            _ => {
                // Fallback: convert as int
                let int_to_str = self.rt("ore_int_to_str")?;
                let result = bld!(self.builder.build_call(int_to_str, &[val.into()], "itos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
        }
    }

    pub(crate) fn record_to_str(
        &mut self,
        val: BasicValueEnum<'ctx>,
        type_name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let info = self.records.get(type_name).ok_or_else(|| self.err(format!("undefined type '{}' for display", type_name)))?;
        let struct_type = info.struct_type;
        let field_names = info.field_names.clone();
        let field_kinds = info.field_kinds.clone();

        let str_new = self.rt("ore_str_new")?;
        let concat_fn = self.rt("ore_str_concat")?;
        let release_fn = self.rt("ore_str_release")?;

        // Store the struct to an alloca so we can GEP into it
        let alloca = bld!(self.builder.build_alloca(struct_type, "rec_tmp"))?;
        bld!(self.builder.build_store(alloca, val))?;

        // Helper: call ore_str_new and get pointer
        let make_str = |cg: &mut Self, s: &str| -> Result<PointerValue<'ctx>, CodeGenError> {
            let ptr = cg.builder_string_const(s);
            let len = cg.context.i32_type().const_int(s.len() as u64, false);
            let result = bld!(cg.builder.build_call(str_new, &[ptr.into(), len.into()], "s"))?;
            Ok(cg.call_result_to_value(result)?.into_pointer_value())
        };

        // Helper: concat two strings, releasing both inputs
        let concat_and_release = |cg: &mut Self, a: PointerValue<'ctx>, b: PointerValue<'ctx>| -> Result<PointerValue<'ctx>, CodeGenError> {
            let result = bld!(cg.builder.build_call(concat_fn, &[a.into(), b.into()], "cat"))?;
            let p = cg.call_result_to_value(result)?.into_pointer_value();
            bld!(cg.builder.build_call(release_fn, &[a.into()], ""))?;
            bld!(cg.builder.build_call(release_fn, &[b.into()], ""))?;
            Ok(p)
        };

        // Start with "TypeName("
        let prefix = format!("{}(", type_name);
        let mut current = make_str(self, &prefix)?;

        for (i, (fname, fkind)) in field_names.iter().zip(field_kinds.iter()).enumerate() {
            let label = if i == 0 { format!("{}: ", fname) } else { format!(", {}: ", fname) };
            let label_str = make_str(self, &label)?;
            current = concat_and_release(self, current, label_str)?;

            // Extract field value and convert to string
            let field_ptr = bld!(self.builder.build_struct_gep(struct_type, alloca, i as u32, &format!("f_{}", fname)))?;
            let field_ty = struct_type.get_field_type_at_index(i as u32).unwrap();
            let field_val = bld!(self.builder.build_load(field_ty, field_ptr, fname))?;
            let field_str = self.value_to_str(field_val, fkind.clone())?;
            current = concat_and_release(self, current, field_str)?;
        }

        // Append ")"
        let suffix_str = make_str(self, ")")?;
        current = concat_and_release(self, current, suffix_str)?;

        Ok(current)
    }

    pub(crate) fn enum_to_str(
        &mut self,
        val: BasicValueEnum<'ctx>,
        enum_name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let enum_info = self.enums.get(enum_name).ok_or_else(|| self.err(format!("undefined enum '{}' for display", enum_name)))?;
        let enum_type = enum_info.enum_type;
        let variants: Vec<_> = enum_info.variants.iter().map(|v| {
            (v.name.clone(), v.tag, v.field_names.clone(), v.field_kinds.clone(), v.payload_type)
        }).collect();

        let str_new = self.rt("ore_str_new")?;
        let concat_fn = self.rt("ore_str_concat")?;
        let release_fn = self.rt("ore_str_release")?;

        // Store enum to alloca
        let alloca = bld!(self.builder.build_alloca(enum_type, "enum_tmp"))?;
        bld!(self.builder.build_store(alloca, val))?;

        // Result alloca (must be before the switch)
        let result_alloca = bld!(self.builder.build_alloca(self.ptr_type(), "enum_str_result"))?;

        // Read tag
        let tag_ptr = bld!(self.builder.build_struct_gep(enum_type, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();

        let current_fn = self.current_fn()?;

        // Create merge block and default block
        let merge_bb = self.context.append_basic_block(current_fn, "enum_str_merge");
        let default_bb = self.context.append_basic_block(current_fn, "enum_str_default");

        // Build variant blocks first (collect cases), then build switch in entry block
        let mut cases = Vec::new();
        for (vname, vtag, field_names, field_kinds, payload_type) in &variants {
            let bb = self.context.append_basic_block(current_fn, &format!("enum_str_{}", vname));
            self.builder.position_at_end(bb);

            if field_names.is_empty() {
                let name_ptr = self.builder_string_const(vname);
                let name_str = bld!(self.builder.build_call(str_new, &[name_ptr.into(), self.context.i32_type().const_int(vname.len() as u64, false).into()], "s"))?;
                let name_val = self.call_result_to_value(name_str)?.into_pointer_value();
                bld!(self.builder.build_store(result_alloca, name_val))?;
            } else {
                let prefix = format!("{}(", vname);
                let prefix_ptr = self.builder_string_const(&prefix);
                let prefix_len = self.context.i32_type().const_int(prefix.len() as u64, false);
                let prefix_str = bld!(self.builder.build_call(str_new, &[prefix_ptr.into(), prefix_len.into()], "s"))?;
                let mut current = self.call_result_to_value(prefix_str)?.into_pointer_value();

                let data_ptr = bld!(self.builder.build_struct_gep(enum_type, alloca, 1, "data_ptr"))?;
                let payload_ptr = bld!(self.builder.build_pointer_cast(data_ptr, self.ptr_type(), "payload"))?;

                for (i, (fname, fkind)) in field_names.iter().zip(field_kinds.iter()).enumerate() {
                    let label = if i == 0 { format!("{}: ", fname) } else { format!(", {}: ", fname) };
                    let label_ptr = self.builder_string_const(&label);
                    let label_len = self.context.i32_type().const_int(label.len() as u64, false);
                    let label_str = bld!(self.builder.build_call(str_new, &[label_ptr.into(), label_len.into()], "s"))?;
                    let label_val = self.call_result_to_value(label_str)?.into_pointer_value();
                    let next = bld!(self.builder.build_call(concat_fn, &[current.into(), label_val.into()], "cat"))?;
                    let next_ptr = self.call_result_to_value(next)?.into_pointer_value();
                    bld!(self.builder.build_call(release_fn, &[current.into()], ""))?;
                    bld!(self.builder.build_call(release_fn, &[label_val.into()], ""))?;
                    current = next_ptr;

                    let field_ptr = bld!(self.builder.build_struct_gep(*payload_type, payload_ptr, i as u32, &format!("f_{}", fname)))?;
                    let field_ty = payload_type.get_field_type_at_index(i as u32).unwrap();
                    let field_val = bld!(self.builder.build_load(field_ty, field_ptr, fname))?;
                    let field_str = self.value_to_str(field_val, fkind.clone())?;

                    let next2 = bld!(self.builder.build_call(concat_fn, &[current.into(), field_str.into()], "cat"))?;
                    let next2_ptr = self.call_result_to_value(next2)?.into_pointer_value();
                    bld!(self.builder.build_call(release_fn, &[current.into()], ""))?;
                    bld!(self.builder.build_call(release_fn, &[field_str.into()], ""))?;
                    current = next2_ptr;
                }

                let suffix_ptr = self.builder_string_const(")");
                let suffix_str = bld!(self.builder.build_call(str_new, &[suffix_ptr.into(), self.context.i32_type().const_int(1, false).into()], "s"))?;
                let suffix_val = self.call_result_to_value(suffix_str)?.into_pointer_value();
                let final_str = bld!(self.builder.build_call(concat_fn, &[current.into(), suffix_val.into()], "cat"))?;
                let final_ptr = self.call_result_to_value(final_str)?.into_pointer_value();
                bld!(self.builder.build_call(release_fn, &[current.into()], ""))?;
                bld!(self.builder.build_call(release_fn, &[suffix_val.into()], ""))?;
                bld!(self.builder.build_store(result_alloca, final_ptr))?;
            }

            bld!(self.builder.build_unconditional_branch(merge_bb))?;
            cases.push((self.context.i8_type().const_int(*vtag as u64, false), bb));
        }

        // Default block
        self.builder.position_at_end(default_bb);
        let unknown_s = self.builder_string_const("<unknown>");
        let unknown_str = bld!(self.builder.build_call(str_new, &[unknown_s.into(), self.context.i32_type().const_int(9, false).into()], "s"))?;
        let unknown_ptr = self.call_result_to_value(unknown_str)?.into_pointer_value();
        bld!(self.builder.build_store(result_alloca, unknown_ptr))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;

        // Now we need to insert the switch. The entry block where we read the tag
        // needs a terminator (the switch). But we already moved the builder away.
        // We need to go back to the entry block and add the switch there.
        // The entry block is the one containing the tag load. Let's use a dedicated block.
        // Actually, the tag load was in the current insert block before we started building variant blocks.
        // We need to split: create a switch block right after the tag load.

        // The trick: the alloca + tag load were in the original block. We need to terminate
        // that block with a branch to a switch block. But the original block might already
        // have other code. Let's just use an unconditional branch from wherever we were
        // to a new switch block.

        // Actually, the simplest approach: the entry block (where tag was loaded) doesn't have
        // a terminator yet. We need to go back there and add one.
        // But we've moved the builder. The tag was loaded in the block that was current
        // when enum_to_str was called. That block now has no terminator.

        // Let's find that block: it's the one containing the alloca instruction
        let entry_block = alloca.as_instruction_value().unwrap().get_parent().unwrap();
        self.builder.position_at_end(entry_block);
        bld!(self.builder.build_switch(tag, default_bb, &cases))?;

        // Position at merge for subsequent code
        self.builder.position_at_end(merge_bb);
        let result = bld!(self.builder.build_load(self.ptr_type(), result_alloca, "enum_str_val"))?.into_pointer_value();
        Ok(result)
    }

}
