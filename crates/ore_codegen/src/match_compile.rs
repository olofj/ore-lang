use super::*;
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, IntValue};
use inkwell::IntPredicate;

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_variant_construct(
        &mut self,
        variant_name: &str,
        fields: &[(String, Expr)],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let enum_name = self.variant_to_enum.get(variant_name).ok_or_else(|| {
            self.err(format!("unknown variant '{}'", variant_name))
        })?.clone();

        let enum_info = self.enums.get(&enum_name).ok_or_else(|| {
            self.err(format!("undefined enum '{}'", enum_name))
        })?;
        let enum_type = enum_info.enum_type;

        // Find the variant
        let variant = enum_info.variants.iter().find(|v| v.name == variant_name).ok_or_else(|| {
            self.err(format!("unknown variant '{}'", variant_name))
        })?;
        let tag = variant.tag;
        let payload_type = variant.payload_type;
        let variant_field_names = variant.field_names.clone();

        // Alloca the enum
        let alloca = bld!(self.builder.build_alloca(enum_type, "enum_val"))?;

        // Store tag
        let tag_ptr = bld!(self.builder.build_struct_gep(enum_type, alloca, 0, "tag_ptr"))?;
        let tag_val = self.context.i8_type().const_int(tag as u64, false);
        bld!(self.builder.build_store(tag_ptr, tag_val))?;

        // Store payload fields
        let data_ptr = bld!(self.builder.build_struct_gep(enum_type, alloca, 1, "data_ptr"))?;
        let payload_ptr = bld!(self.builder.build_pointer_cast(
            data_ptr,
            self.ptr_type(),
            "payload_ptr"
        ))?;

        for (name, expr) in fields {
            let idx = variant_field_names.iter().position(|n| n == name).ok_or_else(|| CodeGenError {
                line: Some(self.current_line), msg: format!("unknown field '{}' on variant '{}'", name, variant_name),
            })?;
            let val = self.compile_expr(expr, func)?;
            let field_ptr = bld!(self.builder.build_struct_gep(payload_type, payload_ptr, idx as u32, &format!("{}.{}", variant_name, name)))?;
            bld!(self.builder.build_store(field_ptr, val))?;
        }

        let result = bld!(self.builder.build_load(enum_type, alloca, "enum_loaded"))?;
        Ok((result, ValKind::Enum(enum_name)))
    }

    pub(crate) fn compile_match(
        &mut self,
        subject: &Expr,
        arms: &[MatchArm],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (subject_val, subject_kind) = self.compile_expr_with_kind(subject, func)?;

        if subject_kind == ValKind::Option {
            return self.compile_option_match(subject_val, arms, func);
        }
        if subject_kind == ValKind::Result {
            return self.compile_result_match(subject_val, arms, func);
        }

        // Check if patterns are literal patterns (Int, String, etc.)
        let has_literal_patterns = arms.iter().any(|arm| matches!(
            &arm.pattern,
            Pattern::IntLit(_) | Pattern::FloatLit(_) | Pattern::BoolLit(_) | Pattern::StringLit(_) | Pattern::Range(_, _) | Pattern::Or(_)
        ));
        if has_literal_patterns || matches!(subject_kind, ValKind::Int | ValKind::Float | ValKind::Bool | ValKind::Str) {
            return self.compile_literal_match(subject_val, &subject_kind, arms, func);
        }

        let enum_name = match &subject_kind {
            ValKind::Enum(name) => name.clone(),
            _ => return Err(self.err("match subject must be an enum type")),
        };

        let enum_info = self.enums.get(&enum_name).ok_or_else(|| {
            self.err(format!("undefined enum '{}'", enum_name))
        })?;
        let enum_type = enum_info.enum_type;

        // Store subject to an alloca so we can extract tag and data
        let subject_alloca = bld!(self.builder.build_alloca(enum_type, "match_subject"))?;
        bld!(self.builder.build_store(subject_alloca, subject_val))?;

        // Load the tag
        let tag_ptr = bld!(self.builder.build_struct_gep(enum_type, subject_alloca, 0, "tag_ptr"))?;
        let tag_val = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?;

        let merge_bb = self.context.append_basic_block(func, "match_merge");

        // Build switch cases
        let default_bb = self.context.append_basic_block(func, "match_default");
        let mut case_blocks: Vec<(inkwell::values::IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut branch_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut result_kind = ValKind::Void;
        let mut wildcard_arm: Option<&MatchArm> = None;

        // Pre-collect variant info needed for each arm
        let variant_infos: Vec<_> = enum_info.variants.iter().map(|v| {
            (v.name.clone(), v.tag, v.payload_type, v.field_names.clone(), v.field_kinds.clone())
        }).collect();

        // Track which tags already have a case block (for chaining guard failures)
        let mut tag_to_guard_fail: HashMap<u8, inkwell::basic_block::BasicBlock<'ctx>> = HashMap::new();

        for arm in arms {
            match &arm.pattern {
                Pattern::Variant { name, bindings } => {
                    let variant = variant_infos.iter().find(|v| v.0 == *name).ok_or_else(|| CodeGenError {
                        line: Some(self.current_line), msg: format!("unknown variant '{}' in match", name),
                    })?;
                    let (_, vtag, payload_type, _field_names, field_kinds) = variant;

                    // If this tag already has a case block (duplicate variant with guard),
                    // chain from the previous guard's failure point
                    let case_bb = if let Some(prev_fail_bb) = tag_to_guard_fail.get(vtag) {
                        // Create a new block for this arm, chained from previous guard failure
                        let bb = self.context.append_basic_block(func, &format!("match_{}_guard", name));
                        // Patch previous guard failure to jump here instead of default_bb
                        self.builder.position_at_end(*prev_fail_bb);
                        // The previous fail block should be empty (we'll fill it with a branch)
                        if self.current_block()?.get_terminator().is_none() {
                            bld!(self.builder.build_unconditional_branch(bb))?;
                        }
                        bb
                    } else {
                        let bb = self.context.append_basic_block(func, &format!("match_{}", name));
                        let tag_const = self.context.i8_type().const_int(*vtag as u64, false);
                        case_blocks.push((tag_const, bb));
                        bb
                    };

                    self.builder.position_at_end(case_bb);

                    // Save variables and bind variant fields
                    let saved_vars = self.variables.clone();

                    // Extract payload
                    let data_ptr = bld!(self.builder.build_struct_gep(enum_type, subject_alloca, 1, "data_ptr"))?;
                    let payload_ptr = bld!(self.builder.build_pointer_cast(
                        data_ptr,
                        self.ptr_type(),
                        "payload_ptr"
                    ))?;

                    for (i, binding) in bindings.iter().enumerate() {
                        let field_kind = &field_kinds[i];
                        let field_ty = self.kind_to_llvm_type(field_kind);
                        let field_ptr = bld!(self.builder.build_struct_gep(*payload_type, payload_ptr, i as u32, binding))?;
                        let val = bld!(self.builder.build_load(field_ty, field_ptr, binding))?;
                        let alloca = bld!(self.builder.build_alloca(field_ty, binding))?;
                        bld!(self.builder.build_store(alloca, val))?;
                        self.variables.insert(binding.clone(), VarInfo { ptr: alloca, ty: field_ty, kind: field_kind.clone(), is_mutable: false });
                    }

                    // Guard condition
                    if let Some(guard) = &arm.guard {
                        let (guard_val, _) = self.compile_expr_with_kind(guard, func)?;
                        let guard_bool = guard_val.into_int_value();
                        let body_bb = self.context.append_basic_block(func, &format!("guard_pass_{}", name));
                        let guard_fail_bb = self.context.append_basic_block(func, &format!("guard_fail_{}", name));
                        bld!(self.builder.build_conditional_branch(guard_bool, body_bb, guard_fail_bb))?;
                        // Record the guard failure block for potential chaining
                        tag_to_guard_fail.insert(*vtag, guard_fail_bb);
                        self.builder.position_at_end(body_bb);
                    } else {
                        // No guard — remove any pending guard_fail for this tag
                        tag_to_guard_fail.remove(vtag);
                    }

                    let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
                    result_kind = arm_kind;

                    if self.current_block()?.get_terminator().is_none() {
                        bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    }
                    let end_bb = self.current_block()?;
                    branch_results.push((arm_val, end_bb));

                    self.variables = saved_vars;
                }
                Pattern::Wildcard => {
                    wildcard_arm = Some(arm);
                }
                _ => return Err(self.err("literal patterns not supported in enum match")),
            }
        }

        // Patch any remaining guard failure blocks to jump to default_bb
        for fail_bb in tag_to_guard_fail.values() {
            self.builder.position_at_end(*fail_bb);
            if self.current_block()?.get_terminator().is_none() {
                bld!(self.builder.build_unconditional_branch(default_bb))?;
            }
        }

        // Handle wildcard/default
        self.builder.position_at_end(default_bb);
        if let Some(arm) = wildcard_arm {
            let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
            result_kind = arm_kind;
            if self.current_block()?.get_terminator().is_none() {
                bld!(self.builder.build_unconditional_branch(merge_bb))?;
            }
            let end_bb = self.current_block()?;
            branch_results.push((arm_val, end_bb));
        } else {
            // Unreachable default
            bld!(self.builder.build_unreachable())?;
        }

        // Build the switch
        // Position back at the block before the switch
        let switch_bb = tag_val.as_instruction_value().unwrap().get_parent().unwrap();
        self.builder.position_at_end(switch_bb);
        let switch = bld!(self.builder.build_switch(
            tag_val.into_int_value(),
            default_bb,
            &case_blocks.iter().map(|(v, bb)| (*v, *bb)).collect::<Vec<_>>()
        ))?;
        let _ = switch;

        // Build merge phi
        self.builder.position_at_end(merge_bb);
        if branch_results.is_empty() {
            return Ok(self.void_result());
        }

        let phi = bld!(self.builder.build_phi(branch_results[0].0.get_type(), "match_val"))?;
        for (val, bb) in &branch_results {
            phi.add_incoming(&[(val, *bb)]);
        }

        Ok((phi.as_basic_value(), result_kind))
    }

    pub(crate) fn compile_option_match(
        &mut self,
        subject_val: BasicValueEnum<'ctx>,
        arms: &[MatchArm],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let variant_tags = &[("None", 0u8), ("Some", 1u8)];
        let has_payload = |tag: u8| tag == 1; // Only Some carries a payload
        self.compile_tagged_union_match(subject_val, arms, func, self.option_type(), "opt", "Option", variant_tags, has_payload)
    }

    /// Unified match compilation for tagged union types (Option and Result).
    #[allow(clippy::too_many_arguments)]
    /// `variant_tags` maps variant names to their tag values.
    /// `has_payload` determines whether a given tag carries a payload to bind.
    fn compile_tagged_union_match(
        &mut self,
        subject_val: BasicValueEnum<'ctx>,
        arms: &[MatchArm],
        func: FunctionValue<'ctx>,
        union_ty: inkwell::types::StructType<'ctx>,
        prefix: &str,
        type_name: &str,
        variant_tags: &[(&str, u8)],
        has_payload: impl Fn(u8) -> bool,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let subject_alloca = bld!(self.builder.build_alloca(union_ty, &format!("{}_match", prefix)))?;
        bld!(self.builder.build_store(subject_alloca, subject_val))?;

        let tag_ptr = bld!(self.builder.build_struct_gep(union_ty, subject_alloca, 0, "tag_ptr"))?;
        let tag_val = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();

        let merge_bb = self.context.append_basic_block(func, &format!("{}_merge", prefix));
        let default_bb = self.context.append_basic_block(func, &format!("{}_default", prefix));
        let mut case_blocks: Vec<(inkwell::values::IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut branch_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut result_kind = ValKind::Void;
        let mut wildcard_arm: Option<&MatchArm> = None;

        for arm in arms {
            match &arm.pattern {
                Pattern::Variant { name, bindings } => {
                    let vtag = variant_tags.iter()
                        .find(|(n, _)| *n == name.as_str())
                        .map(|(_, t)| *t)
                        .ok_or_else(|| self.err(format!("unknown {} variant '{}'", type_name, name)))?;

                    let case_bb = self.context.append_basic_block(func, &format!("{}_{}", prefix, name));
                    let tag_const = self.context.i8_type().const_int(vtag as u64, false);
                    case_blocks.push((tag_const, case_bb));

                    self.builder.position_at_end(case_bb);
                    let saved_vars = self.variables.clone();

                    if has_payload(vtag) && !bindings.is_empty() {
                        let kind_ptr = bld!(self.builder.build_struct_gep(union_ty, subject_alloca, 1, "kind_ptr"))?;
                        let kind_i8 = bld!(self.builder.build_load(self.context.i8_type(), kind_ptr, "kind_tag"))?.into_int_value();
                        let val_ptr = bld!(self.builder.build_struct_gep(union_ty, subject_alloca, 2, "val_ptr"))?;
                        let payload = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, &bindings[0]))?;
                        let alloca = bld!(self.builder.build_alloca(self.context.i64_type(), &bindings[0]))?;
                        bld!(self.builder.build_store(alloca, payload))?;
                        let kind_alloca = bld!(self.builder.build_alloca(self.context.i8_type(), &format!("{}_kind", bindings[0])))?;
                        bld!(self.builder.build_store(kind_alloca, kind_i8))?;
                        self.variables.insert(bindings[0].clone(), VarInfo { ptr: alloca, ty: self.context.i64_type().into(), kind: ValKind::Int, is_mutable: false });
                        self.dynamic_kind_tags.insert(bindings[0].clone(), kind_alloca);
                    }

                    let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
                    result_kind = arm_kind;

                    if self.current_block()?.get_terminator().is_none() {
                        bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    }
                    let end_bb = self.current_block()?;
                    branch_results.push((arm_val, end_bb));
                    self.variables = saved_vars;
                }
                Pattern::Wildcard => {
                    wildcard_arm = Some(arm);
                }
                _ => return Err(self.err(format!("literal patterns not supported in {} match", type_name))),
            }
        }

        self.builder.position_at_end(default_bb);
        if let Some(arm) = wildcard_arm {
            let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
            result_kind = arm_kind;
            if self.current_block()?.get_terminator().is_none() {
                bld!(self.builder.build_unconditional_branch(merge_bb))?;
            }
            let end_bb = self.current_block()?;
            branch_results.push((arm_val, end_bb));
        } else {
            bld!(self.builder.build_unreachable())?;
        }

        let switch_bb = tag_ptr.as_instruction_value().unwrap().get_parent().unwrap();
        self.builder.position_at_end(switch_bb);
        bld!(self.builder.build_switch(
            tag_val,
            default_bb,
            &case_blocks.iter().map(|(v, bb)| (*v, *bb)).collect::<Vec<_>>()
        ))?;

        self.builder.position_at_end(merge_bb);
        if branch_results.is_empty() {
            return Ok(self.void_result());
        }

        let phi = bld!(self.builder.build_phi(branch_results[0].0.get_type(), &format!("{}_val", prefix)))?;
        for (val, bb) in &branch_results {
            phi.add_incoming(&[(val, *bb)]);
        }

        Ok((phi.as_basic_value(), result_kind))
    }

    pub(crate) fn compile_literal_match(
        &mut self,
        subject_val: BasicValueEnum<'ctx>,
        subject_kind: &ValKind,
        arms: &[MatchArm],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        // Chain of if-else comparisons for literal patterns
        // Uses phi nodes to handle any result type (including enums/structs)
        let merge_bb = self.context.append_basic_block(func, "lmatch_merge");

        let mut result_kind = ValKind::Int;
        let mut has_wildcard = false;
        let mut branch_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();

        for (i, arm) in arms.iter().enumerate() {
            // Always create a separate else block (never reuse merge_bb directly)
            // because phi nodes require every predecessor to have an incoming value
            let else_bb = self.context.append_basic_block(func, &format!("lmatch_next_{}", i));

            // Check if pattern is a variable binding (identifier that's not a known variant)
            let is_var_binding = matches!(&arm.pattern, Pattern::Variant { name, bindings }
                if bindings.is_empty() && !self.variant_to_enum.contains_key(name));

            match &arm.pattern {
                _ if is_var_binding || matches!(&arm.pattern, Pattern::Wildcard) => {
                    has_wildcard = true;

                    // Bind variable if it's a named pattern (not wildcard)
                    let saved_vars = if let Pattern::Variant { name, .. } = &arm.pattern {
                        let saved = self.variables.clone();
                        let ty = self.kind_to_llvm_type(subject_kind);
                        let alloca = bld!(self.builder.build_alloca(ty, name))?;
                        bld!(self.builder.build_store(alloca, subject_val))?;
                        self.variables.insert(name.clone(), VarInfo { ptr: alloca, ty, kind: subject_kind.clone(), is_mutable: false });
                        Some(saved)
                    } else {
                        None
                    };

                    // Wildcard/variable with guard: check guard, fall through if false
                    if let Some(guard) = &arm.guard {
                        let (guard_val, _) = self.compile_expr_with_kind(guard, func)?;
                        let guard_bool = guard_val.into_int_value();
                        let body_bb = self.context.append_basic_block(func, &format!("lmatch_wguard_{}", i));
                        bld!(self.builder.build_conditional_branch(guard_bool, body_bb, else_bb))?;
                        self.builder.position_at_end(body_bb);
                    }
                    let (body_val, bk) = self.compile_expr_with_kind(&arm.body, func)?;
                    result_kind = bk;
                    if self.current_block()?.get_terminator().is_none() {
                        bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    }
                    let end_bb = self.current_block()?;
                    branch_results.push((body_val, end_bb));

                    // Restore variables
                    if let Some(saved) = saved_vars {
                        self.variables = saved;
                    }

                    self.builder.position_at_end(else_bb);
                }
                _ => {
                    // Build comparison
                    let cmp = self.compile_pattern_cmp(subject_val, subject_kind, &arm.pattern, func)?;

                    let then_bb = self.context.append_basic_block(func, &format!("lmatch_arm_{}", i));

                    bld!(self.builder.build_conditional_branch(cmp, then_bb, else_bb))?;

                    self.builder.position_at_end(then_bb);

                    // Guard condition: if guard fails, jump to else_bb
                    if let Some(guard) = &arm.guard {
                        let (guard_val, _) = self.compile_expr_with_kind(guard, func)?;
                        let guard_bool = guard_val.into_int_value();
                        let body_bb = self.context.append_basic_block(func, &format!("lmatch_guarded_{}", i));
                        bld!(self.builder.build_conditional_branch(guard_bool, body_bb, else_bb))?;
                        self.builder.position_at_end(body_bb);
                    }

                    let (body_val, bk) = self.compile_expr_with_kind(&arm.body, func)?;
                    result_kind = bk;
                    if self.current_block()?.get_terminator().is_none() {
                        bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    }
                    let end_bb = self.current_block()?;
                    branch_results.push((body_val, end_bb));

                    self.builder.position_at_end(else_bb);
                }
            }
        }

        // If no wildcard, the current block is the fallthrough from the last else.
        // Branch to merge with a dummy value so the phi has an incoming for every predecessor.
        if !has_wildcard
            && self.current_block()?.get_terminator().is_none() {
                if !branch_results.is_empty() {
                    let undef = branch_results[0].0.get_type().const_zero();
                    let default_bb = self.current_block()?;
                    bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    branch_results.push((undef, default_bb));
                } else {
                    bld!(self.builder.build_unconditional_branch(merge_bb))?;
                }
            }

        self.builder.position_at_end(merge_bb);

        if branch_results.is_empty() {
            return Ok(self.void_result());
        }

        let phi = bld!(self.builder.build_phi(branch_results[0].0.get_type(), "lmatch_val"))?;
        for (val, bb) in &branch_results {
            phi.add_incoming(&[(val, *bb)]);
        }

        Ok((phi.as_basic_value(), result_kind))
    }

    pub(crate) fn compile_pattern_cmp(
        &mut self,
        subject: BasicValueEnum<'ctx>,
        _subject_kind: &ValKind,
        pattern: &Pattern,
        _func: FunctionValue<'ctx>,
    ) -> Result<IntValue<'ctx>, CodeGenError> {
        match pattern {
            Pattern::IntLit(n) => {
                let const_val = self.context.i64_type().const_int(*n as u64, true);
                bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, subject.into_int_value(), const_val, "pcmp"
                ))
            }
            Pattern::BoolLit(b) => {
                let const_val = self.context.bool_type().const_int(if *b { 1 } else { 0 }, false);
                bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, subject.into_int_value(), const_val, "pcmp"
                ))
            }
            Pattern::StringLit(s) => {
                // Create string constant and compare
                let str_val = self.compile_string_literal(s)?;
                let i8_val = self.call_rt("ore_str_eq", &[subject.into(), str_val.into()], "seq")?.into_int_value();
                bld!(self.builder.build_int_compare(
                    IntPredicate::NE, i8_val,
                    self.context.i8_type().const_int(0, false), "tobool"
                ))
            }
            Pattern::Range(start, end) => {
                let i64_type = self.context.i64_type();
                let start_val = i64_type.const_int(*start as u64, true);
                let end_val = i64_type.const_int(*end as u64, true);
                let subj = subject.into_int_value();
                let ge = bld!(self.builder.build_int_compare(IntPredicate::SGE, subj, start_val, "rge"))?;
                let le = bld!(self.builder.build_int_compare(IntPredicate::SLE, subj, end_val, "rle"))?;
                bld!(self.builder.build_and(ge, le, "range_cmp"))
            }
            Pattern::Or(alternatives) => {
                // Or pattern: check any alternative matches
                let first = self.compile_pattern_cmp(subject, _subject_kind, &alternatives[0], _func)?;
                let mut result = first;
                for alt in &alternatives[1..] {
                    let alt_cmp = self.compile_pattern_cmp(subject, _subject_kind, alt, _func)?;
                    result = bld!(self.builder.build_or(result, alt_cmp, "or_pat"))?;
                }
                Ok(result)
            }
            _ => Err(self.err("unsupported pattern in literal match")),
        }
    }

    pub(crate) fn compile_result_match(
        &mut self,
        subject_val: BasicValueEnum<'ctx>,
        arms: &[MatchArm],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let variant_tags = &[("Ok", 0u8), ("Err", 1u8)];
        let has_payload = |_tag: u8| true; // Both Ok and Err carry payloads
        self.compile_tagged_union_match(subject_val, arms, func, self.result_type(), "res", "Result", variant_tags, has_payload)
    }

    pub(crate) fn compile_try_result(
        &mut self,
        val: BasicValueEnum<'ctx>,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let res_ty = self.result_type();
        let alloca = bld!(self.builder.build_alloca(res_ty, "try_res"))?;
        bld!(self.builder.build_store(alloca, val))?;
        let tag_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
        let is_err = bld!(self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_err"
        ))?;
        let ok_bb = self.context.append_basic_block(func, "try_ok");
        let err_bb = self.context.append_basic_block(func, "try_err");
        bld!(self.builder.build_conditional_branch(is_err, err_bb, ok_bb))?;
        // Err branch: return the Err result from the current function
        self.builder.position_at_end(err_bb);
        let err_ret = bld!(self.builder.build_load(res_ty, alloca, "err_ret"))?;
        bld!(self.builder.build_return(Some(&err_ret)))?;
        // Ok branch: extract the value
        self.builder.position_at_end(ok_bb);
        let val_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 2, "val_ptr"))?;
        let extracted = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "unwrapped"))?;
        Ok((extracted, ValKind::Int))
    }

    /// Build a tagged union value (Option or Result) with the given tag and payload.
    pub(crate) fn build_tagged_union(
        &mut self,
        union_ty: inkwell::types::StructType<'ctx>,
        tag: u8,
        payload: Option<BasicValueEnum<'ctx>>,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let alloca = bld!(self.builder.build_alloca(union_ty, name))?;
        let tag_ptr = bld!(self.builder.build_struct_gep(union_ty, alloca, 0, "tag"))?;
        bld!(self.builder.build_store(tag_ptr, self.context.i8_type().const_int(tag as u64, false)))?;
        if let Some(val) = payload {
            let kind_ptr = bld!(self.builder.build_struct_gep(union_ty, alloca, 1, "kind"))?;
            bld!(self.builder.build_store(kind_ptr, self.context.i8_type().const_int(0, false)))?;
            let val_ptr = bld!(self.builder.build_struct_gep(union_ty, alloca, 2, "val"))?;
            let i64_val = self.value_to_i64(val)?;
            bld!(self.builder.build_store(val_ptr, i64_val))?;
        }
        let result = bld!(self.builder.build_load(union_ty, alloca, name))?;
        Ok(result)
    }

    pub(crate) fn compile_option_method(
        &mut self,
        opt_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let opt_ty = self.option_type();
        let alloca = bld!(self.builder.build_alloca(opt_ty, "opt_m"))?;
        bld!(self.builder.build_store(alloca, opt_val))?;

        let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
        let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 2, "val_ptr"))?;

        match method {
            "unwrap_or" => {
                // Returns inner value if Some, else the provided default
                if args.is_empty() {
                    return Err(self.err("unwrap_or requires a default argument"));
                }
                let (default_val, default_kind) = self.compile_expr_with_kind(&args[0], func)?;
                let is_some = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
                ))?;
                let inner = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?;

                let some_bb = self.context.append_basic_block(func, "unwrap_some");
                let none_bb = self.context.append_basic_block(func, "unwrap_none");
                let merge_bb = self.context.append_basic_block(func, "unwrap_merge");

                bld!(self.builder.build_conditional_branch(is_some, some_bb, none_bb))?;

                self.builder.position_at_end(some_bb);
                let some_result = self.coerce_from_i64(inner, &default_kind)?;
                bld!(self.builder.build_unconditional_branch(merge_bb))?;

                self.builder.position_at_end(none_bb);
                bld!(self.builder.build_unconditional_branch(merge_bb))?;

                self.builder.position_at_end(merge_bb);
                let phi = bld!(self.builder.build_phi(some_result.get_type(), "unwrap_val"))?;
                phi.add_incoming(&[(&some_result, some_bb), (&default_val, none_bb)]);

                Ok((phi.as_basic_value(), default_kind))
            }
            "unwrap" => {
                // Just return inner value (unsafe - crashes on None in real use, but useful)
                let inner = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "unwrapped"))?;
                Ok((inner, ValKind::Int))
            }
            "is_some" => {
                let is_some = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
                ))?;
                Ok((is_some.into(), ValKind::Bool))
            }
            "is_none" => {
                let is_none = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(0, false), "is_none"
                ))?;
                Ok((is_none.into(), ValKind::Bool))
            }
            "map" => {
                // opt.map(fn) -> applies fn to inner value if Some, returns Option
                self.check_arity("map", args, 1)?;
                let is_some = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
                ))?;
                let some_bb = self.context.append_basic_block(func, "optmap_some");
                let none_bb = self.context.append_basic_block(func, "optmap_none");
                let merge_bb = self.context.append_basic_block(func, "optmap_merge");
                bld!(self.builder.build_conditional_branch(is_some, some_bb, none_bb))?;

                // Some branch: unwrap, apply function, wrap result
                self.builder.position_at_end(some_bb);
                let inner = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?;

                // Compile the lambda/function and call it with inner value
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        self.module.get_function(name).ok_or_else(|| {
                            self.err(format!("unknown function '{}'", name))
                        })?
                    }
                    _ => return Err(self.err("map requires a function or lambda")),
                };

                let map_result = bld!(self.builder.build_call(lambda_fn, &[inner.into()], "mapped"))?;
                let mapped_val = self.call_result_to_value(map_result)?;

                // Wrap result in Some
                let opt_ty = self.option_type();
                let some_result = self.build_tagged_union(opt_ty, 1, Some(mapped_val), "some_res")?;
                bld!(self.builder.build_unconditional_branch(merge_bb))?;
                let some_end = self.current_block()?;

                // None branch
                self.builder.position_at_end(none_bb);
                let none_result = self.build_tagged_union(opt_ty, 0, None, "none_res")?;
                bld!(self.builder.build_unconditional_branch(merge_bb))?;

                self.builder.position_at_end(merge_bb);
                let phi = bld!(self.builder.build_phi(opt_ty, "optmap_result"))?;
                phi.add_incoming(&[(&some_result, some_end), (&none_result, none_bb)]);
                Ok((phi.as_basic_value(), ValKind::Option))
            }
            _ => Err(Self::unknown_method_error("Option", method, &["unwrap_or", "unwrap", "map", "is_some", "is_none"])),
        }
    }

    pub(crate) fn compile_result_method(
        &mut self,
        result_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let result_ty = self.result_type();
        let alloca = bld!(self.builder.build_alloca(result_ty, "res_m"))?;
        bld!(self.builder.build_store(alloca, result_val))?;

        let tag_ptr = bld!(self.builder.build_struct_gep(result_ty, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
        let val_ptr = bld!(self.builder.build_struct_gep(result_ty, alloca, 2, "val_ptr"))?;

        match method {
            "unwrap_or" => {
                if args.is_empty() {
                    return Err(self.err("unwrap_or requires a default argument"));
                }
                let (default_val, default_kind) = self.compile_expr_with_kind(&args[0], func)?;
                // tag 0 = Ok
                let is_ok = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(0, false), "is_ok"
                ))?;
                let inner = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?;

                let ok_bb = self.context.append_basic_block(func, "result_ok");
                let err_bb = self.context.append_basic_block(func, "result_err");
                let merge_bb = self.context.append_basic_block(func, "result_merge");

                bld!(self.builder.build_conditional_branch(is_ok, ok_bb, err_bb))?;

                self.builder.position_at_end(ok_bb);
                let ok_result = self.coerce_from_i64(inner, &default_kind)?;
                bld!(self.builder.build_unconditional_branch(merge_bb))?;

                self.builder.position_at_end(err_bb);
                bld!(self.builder.build_unconditional_branch(merge_bb))?;

                self.builder.position_at_end(merge_bb);
                let phi = bld!(self.builder.build_phi(ok_result.get_type(), "result_val"))?;
                phi.add_incoming(&[(&ok_result, ok_bb), (&default_val, err_bb)]);

                Ok((phi.as_basic_value(), default_kind))
            }
            "is_ok" => {
                let is_ok = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(0, false), "is_ok"
                ))?;
                Ok((is_ok.into(), ValKind::Bool))
            }
            "is_err" => {
                let is_err = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_err"
                ))?;
                Ok((is_err.into(), ValKind::Bool))
            }
            "unwrap" => {
                let inner = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?;
                // Default coercion to Int — same as Option.unwrap()
                let coerced = self.coerce_from_i64(inner, &ValKind::Int)?;
                Ok((coerced, ValKind::Int))
            }
            "map" => {
                // result.map(fn) -> applies fn to inner value if Ok, returns Result
                self.check_arity("map", args, 1)?;
                let is_ok = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(0, false), "is_ok"
                ))?;
                let ok_bb = self.context.append_basic_block(func, "resmap_ok");
                let err_bb = self.context.append_basic_block(func, "resmap_err");
                let merge_bb = self.context.append_basic_block(func, "resmap_merge");
                bld!(self.builder.build_conditional_branch(is_ok, ok_bb, err_bb))?;

                // Ok branch: unwrap, apply function, wrap result
                self.builder.position_at_end(ok_bb);
                let inner = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?;

                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        self.module.get_function(name).ok_or_else(|| {
                            self.err(format!("unknown function '{}'", name))
                        })?
                    }
                    _ => return Err(self.err("map requires a function or lambda")),
                };

                let map_result = bld!(self.builder.build_call(lambda_fn, &[inner.into()], "mapped"))?;
                let mapped_val = self.call_result_to_value(map_result)?;

                // Wrap result in Ok
                let res_ty = self.result_type();
                let ok_result = self.build_tagged_union(res_ty, 0, Some(mapped_val), "ok_res")?;
                bld!(self.builder.build_unconditional_branch(merge_bb))?;
                let ok_end = self.current_block()?;

                // Err branch: pass through unchanged
                self.builder.position_at_end(err_bb);
                bld!(self.builder.build_unconditional_branch(merge_bb))?;

                self.builder.position_at_end(merge_bb);
                let phi = bld!(self.builder.build_phi(res_ty, "resmap_result"))?;
                phi.add_incoming(&[(&ok_result, ok_end), (&result_val, err_bb)]);
                Ok((phi.as_basic_value(), ValKind::Result))
            }
            _ => Err(Self::unknown_method_error("Result", method, &["unwrap_or", "unwrap", "map", "is_ok", "is_err"])),
        }
    }

}
