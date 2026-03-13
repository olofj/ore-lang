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
                let val = self.call_rt("ore_str_len", &[str_val.into()], "slen")?;
                Ok((val, ValKind::Int))
            }
            "is_empty" => {
                let len_val = self.call_rt("ore_str_len", &[str_val.into()], "slen")?.into_int_value();
                let is_zero = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    len_val,
                    self.context.i64_type().const_zero(),
                    "is_empty"
                ))?;
                Ok((is_zero.into(), ValKind::Bool))
            }
            "contains" => {
                self.check_arity("contains", args, 1)?;
                let needle = self.compile_expr(&args[0], func)?;
                let i8_val = self.call_rt("ore_str_contains", &[str_val.into(), needle.into()], "scontains")?.into_int_value();
                let bool_val = self.i8_to_bool(i8_val)?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "trim" | "trim_start" | "trim_end" => {
                let fn_name = format!("ore_str_{}", method);
                let val = self.call_rt(&fn_name, &[str_val.into()], "strim")?;
                Ok((val, ValKind::Str))
            }
            "words" | "lines" | "chars" => {
                let rt_name = match method {
                    "words" => "ore_str_split_whitespace",
                    "lines" => "ore_str_lines",
                    _ => "ore_str_chars",
                };
                let val = self.call_rt(rt_name, &[str_val.into()], method)?;
                Ok((val, ValKind::list_of(ValKind::Str)))
            }
            "split" => {
                if args.is_empty() {
                    // split() with no args = split on whitespace
                    let val = self.call_rt("ore_str_split_whitespace", &[str_val.into()], "ssplit")?;
                    return Ok((val, ValKind::list_of(ValKind::Str)));
                }
                self.check_arity("split", args, 1)?;
                let delim = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_str_split", &[str_val.into(), delim.into()], "ssplit")?;
                Ok((val, ValKind::list_of(ValKind::Str)))
            }
            "to_int" => {
                let val = self.call_rt("ore_str_to_int", &[str_val.into()], "stoi")?;
                Ok((val, ValKind::Int))
            }
            "to_float" => {
                let val = self.call_rt("ore_str_to_float", &[str_val.into()], "stof")?;
                Ok((val, ValKind::Float))
            }
            "replace" => {
                self.check_arity("replace", args, 2)?;
                let from = self.compile_expr(&args[0], func)?;
                let to = self.compile_expr(&args[1], func)?;
                let val = self.call_rt("ore_str_replace", &[str_val.into(), from.into(), to.into()], "sreplace")?;
                Ok((val, ValKind::Str))
            }
            "starts_with" | "ends_with" => {
                self.check_arity(method, args, 1)?;
                let arg = self.compile_expr(&args[0], func)?;
                let rt_name = format!("ore_str_{}", method);
                let i8_val = self.call_rt(&rt_name, &[str_val.into(), arg.into()], method)?.into_int_value();
                let bool_val = self.i8_to_bool(i8_val)?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "to_upper" | "capitalize" | "to_lower" | "reverse" => {
                let rt_name = format!("ore_str_{}", method);
                let val = self.call_rt(&rt_name, &[str_val.into()], method)?;
                Ok((val, ValKind::Str))
            }
            "substr" => {
                self.check_arity("substr", args, 2)?;
                let start = self.compile_expr(&args[0], func)?;
                let len = self.compile_expr(&args[1], func)?;
                let val = self.call_rt("ore_str_substr", &[str_val.into(), start.into(), len.into()], "ssub")?;
                Ok((val, ValKind::Str))
            }
            "char_at" => {
                self.check_arity("char_at", args, 1)?;
                let idx = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_str_char_at", &[str_val.into(), idx.into()], "charat")?;
                Ok((val, ValKind::Str))
            }
            "index_of" | "find" => {
                self.check_arity("index_of/find", args, 1)?;
                let needle = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_str_index_of", &[str_val.into(), needle.into()], "sidx")?;
                Ok((val, ValKind::Int))
            }
            "slice" => {
                self.check_arity("slice", args, 2)?;
                let start = self.compile_expr(&args[0], func)?;
                let end = self.compile_expr(&args[1], func)?;
                let val = self.call_rt("ore_str_slice", &[str_val.into(), start.into(), end.into()], "sslice")?;
                Ok((val, ValKind::Str))
            }
            "parse_int" => {
                let val = self.call_rt("ore_str_to_int", &[str_val.into()], "parse_int")?;
                Ok((val, ValKind::Int))
            }
            "parse_float" => {
                let val = self.call_rt("ore_str_parse_float", &[str_val.into()], "parse_float")?;
                Ok((val, ValKind::Float))
            }
            "repeat" => {
                self.check_arity("repeat", args, 1)?;
                let count = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_str_repeat", &[str_val.into(), count.into()], "srep")?;
                Ok((val, ValKind::Str))
            }
            "count" => {
                self.check_arity("count", args, 1)?;
                let needle = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_str_count", &[str_val.into(), needle.into()], "scount")?;
                Ok((val, ValKind::Int))
            }
            "strip_prefix" | "strip_suffix" => {
                self.check_arity(method, args, 1)?;
                let arg = self.compile_expr(&args[0], func)?;
                let fn_name = format!("ore_str_{}", method);
                let val = self.call_rt(&fn_name, &[str_val.into(), arg.into()], "sstrip")?;
                Ok((val, ValKind::Str))
            }
            "pad_left" | "pad_right" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(self.err(format!("{} takes 1-2 arguments (width, [pad_char])", method)));
                }
                let width = self.compile_expr(&args[0], func)?;
                let pad_char = if args.len() > 1 {
                    self.compile_expr(&args[1], func)?
                } else {
                    // Default pad char: space
                    let space = " ";
                    let space_ptr = bld!(self.builder.build_global_string_ptr(space, "pad_space"))?.as_pointer_value();
                    self.call_rt("ore_str_new", &[space_ptr.into(), self.context.i32_type().const_int(1, false).into()], "spad")?
                };
                let fn_name = if method == "pad_left" { "ore_str_pad_left" } else { "ore_str_pad_right" };
                let val = self.call_rt(fn_name, &[str_val.into(), width.into(), pad_char.into()], "spad")?;
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
        let ptr = self.builder_string_const(s);
        let len = self.context.i32_type().const_int(s.len() as u64, false);
        Ok(self.call_rt("ore_str_new", &[ptr.into(), len.into()], "str")?.into_pointer_value())
    }

    /// Concatenate two OreStr pointers and release both inputs.
    pub(crate) fn str_concat_release(
        &mut self,
        a: PointerValue<'ctx>,
        b: PointerValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let result = self.call_rt("ore_str_concat", &[a.into(), b.into()], "cat")?.into_pointer_value();
        self.call_rt("ore_str_release", &[a.into()], "")?;
        self.call_rt("ore_str_release", &[b.into()], "")?;
        Ok(result)
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
                            self.call_rt("ore_dynamic_to_str", &[val.into(), kind_i8.into()], "dyntos")?.into_pointer_value()
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
            self.call_rt("ore_str_retain", &[final_ptr.into()], "")?;
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
                let ptr = val.into_pointer_value();
                self.call_rt("ore_str_retain", &[ptr.into()], "")?;
                Ok(ptr)
            }
            ValKind::Int => {
                Ok(self.call_rt("ore_int_to_str", &[val.into()], "itos")?.into_pointer_value())
            }
            ValKind::Float => {
                Ok(self.call_rt("ore_float_to_str", &[val.into()], "ftos")?.into_pointer_value())
            }
            ValKind::Bool => {
                let ext = self.bool_to_i8(val.into_int_value())?;
                Ok(self.call_rt("ore_bool_to_str", &[ext.into()], "btos")?.into_pointer_value())
            }
            ValKind::Record(ref name) => {
                self.record_to_str(val, name)
            }
            ValKind::Enum(ref name) => {
                self.enum_to_str(val, name)
            }
            _ => {
                // Fallback: convert as int
                Ok(self.call_rt("ore_int_to_str", &[val.into()], "itos")?.into_pointer_value())
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

        // Store the struct to an alloca so we can GEP into it
        let alloca = bld!(self.builder.build_alloca(struct_type, "rec_tmp"))?;
        bld!(self.builder.build_store(alloca, val))?;

        // Start with "TypeName("
        let mut current = self.compile_string_literal(&format!("{}(", type_name))?;

        for (i, (fname, fkind)) in field_names.iter().zip(field_kinds.iter()).enumerate() {
            let label = if i == 0 { format!("{}: ", fname) } else { format!(", {}: ", fname) };
            let label_str = self.compile_string_literal(&label)?;
            current = self.str_concat_release(current, label_str)?;

            // Extract field value and convert to string
            let field_ptr = bld!(self.builder.build_struct_gep(struct_type, alloca, i as u32, &format!("f_{}", fname)))?;
            let field_ty = struct_type.get_field_type_at_index(i as u32).unwrap();
            let field_val = bld!(self.builder.build_load(field_ty, field_ptr, fname))?;
            let field_str = self.value_to_str(field_val, fkind.clone())?;
            current = self.str_concat_release(current, field_str)?;
        }

        // Append ")"
        let suffix_str = self.compile_string_literal(")")?;
        current = self.str_concat_release(current, suffix_str)?;

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

        // Store enum to alloca
        let alloca = bld!(self.builder.build_alloca(enum_type, "enum_tmp"))?;
        bld!(self.builder.build_store(alloca, val))?;

        // Result alloca (must be before the switch)
        let result_alloca = bld!(self.builder.build_alloca(self.ptr_type(), "enum_str_result"))?;

        let tag = self.load_tag(enum_type, alloca)?;

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
                let name_val = self.compile_string_literal(vname)?;
                bld!(self.builder.build_store(result_alloca, name_val))?;
            } else {
                let mut current = self.compile_string_literal(&format!("{}(", vname))?;

                let data_ptr = bld!(self.builder.build_struct_gep(enum_type, alloca, 1, "data_ptr"))?;
                let payload_ptr = bld!(self.builder.build_pointer_cast(data_ptr, self.ptr_type(), "payload"))?;

                for (i, (fname, fkind)) in field_names.iter().zip(field_kinds.iter()).enumerate() {
                    let label = if i == 0 { format!("{}: ", fname) } else { format!(", {}: ", fname) };
                    let label_str = self.compile_string_literal(&label)?;
                    current = self.str_concat_release(current, label_str)?;

                    let field_ptr = bld!(self.builder.build_struct_gep(*payload_type, payload_ptr, i as u32, &format!("f_{}", fname)))?;
                    let field_ty = payload_type.get_field_type_at_index(i as u32).unwrap();
                    let field_val = bld!(self.builder.build_load(field_ty, field_ptr, fname))?;
                    let field_str = self.value_to_str(field_val, fkind.clone())?;
                    current = self.str_concat_release(current, field_str)?;
                }

                let suffix_str = self.compile_string_literal(")")?;
                current = self.str_concat_release(current, suffix_str)?;
                bld!(self.builder.build_store(result_alloca, current))?;
            }

            bld!(self.builder.build_unconditional_branch(merge_bb))?;
            cases.push((self.context.i8_type().const_int(*vtag as u64, false), bb));
        }

        // Default block
        self.builder.position_at_end(default_bb);
        let unknown_ptr = self.compile_string_literal("<unknown>")?;
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
