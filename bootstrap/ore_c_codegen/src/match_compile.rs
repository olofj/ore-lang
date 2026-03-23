use super::*;

impl CCodeGen {
    pub(crate) fn compile_match(&mut self, subject: &Expr, arms: &[MatchArm]) -> Result<(String, ValKind), CCodeGenError> {
        let (subject_val, subject_kind) = self.compile_expr(subject)?;

        if subject_kind == ValKind::Option {
            return self.compile_option_match(&subject_val, arms);
        }
        if subject_kind == ValKind::Result {
            return self.compile_result_match(&subject_val, arms);
        }

        // Check for enum match
        if let ValKind::Enum(ref enum_name) = subject_kind {
            return self.compile_enum_match(&subject_val, enum_name, arms);
        }

        // Infer enum type from variant patterns when subject kind is Int
        // (e.g. when indexing an untyped list that contains enum values)
        if !matches!(subject_kind, ValKind::Enum(_)) {
            if let Some(enum_name) = self.infer_enum_from_arms(arms) {
                // The subject is a heap pointer to the enum struct, cast it back
                let c_type = format!("struct ore_enum_{}", Self::mangle_name(&enum_name));
                let cast_val = format!("*({c_type}*)(intptr_t)({})", subject_val);
                let tmp = self.tmp();
                self.emit(&format!("{} {} = {};", c_type, tmp, cast_val));
                return self.compile_enum_match(&tmp, &enum_name, arms);
            }
        }

        // Literal match (if/else chain)
        self.compile_literal_match(&subject_val, &subject_kind, arms)
    }

    fn compile_enum_match(&mut self, subject_val: &str, enum_name: &str, arms: &[MatchArm]) -> Result<(String, ValKind), CCodeGenError> {
        let enum_info = self.enums.get(enum_name).ok_or_else(|| self.err(format!("undefined enum '{}'", enum_name)))?;
        let struct_name = format!("struct ore_enum_{}", Self::mangle_name(enum_name));
        let variants: Vec<_> = enum_info.variants.iter().map(|v| {
            (v.name.clone(), v.tag, v.field_names.clone(), v.field_kinds.clone())
        }).collect();

        let subject_tmp = self.tmp();
        self.emit(&format!("{} {} = {};", struct_name, subject_tmp, subject_val));

        let result_tmp = self.tmp();
        self.emit(&format!("int64_t {} = 0;", result_tmp));
        let mut result_kind = ValKind::Int;

        // Group arms by variant tag to avoid duplicate case labels.
        // Arms matching the same variant are merged into one case with if/else chains.
        let mut tag_groups: Vec<(u8, Vec<usize>)> = Vec::new(); // (tag, arm indices)
        let mut wildcard_indices: Vec<usize> = Vec::new();

        for (i, arm) in arms.iter().enumerate() {
            match &arm.pattern {
                Pattern::Variant { name, .. } => {
                    let variant = variants.iter().find(|v| v.0 == *name).ok_or_else(|| self.err(format!("unknown variant '{}'", name)))?;
                    let vtag = variant.1;
                    if let Some(group) = tag_groups.iter_mut().find(|(t, _)| *t == vtag) {
                        group.1.push(i);
                    } else {
                        tag_groups.push((vtag, vec![i]));
                    }
                }
                Pattern::Wildcard => {
                    wildcard_indices.push(i);
                }
                _ => return Err(self.err("unsupported pattern in enum match")),
            }
        }

        self.emit(&format!("switch ({}.tag) {{", subject_tmp));
        self.indent += 1;

        for (vtag, arm_indices) in &tag_groups {
            self.emit(&format!("case {}: {{", vtag));
            self.indent += 1;
            let saved_vars = self.variables.clone();

            // All arms in this group share the same variant. Extract fields once
            // (using the first arm's bindings to determine the payload struct).
            let first_arm = &arms[arm_indices[0]];
            let (variant_name, field_names, field_kinds) = if let Pattern::Variant { name, .. } = &first_arm.pattern {
                let variant = variants.iter().find(|v| v.0 == *name).unwrap();
                (name.clone(), variant.2.clone(), variant.3.clone())
            } else {
                unreachable!()
            };

            // Extract payload fields once for the case block
            let payload_tmp_name = if !field_names.is_empty() {
                let payload_name = format!("ore_payload_{}_{}", Self::mangle_name(enum_name), variant_name);
                let payload_tmp = self.tmp();
                self.emit(&format!("struct {}* {} = (struct {}*){}.data;",
                    payload_name, payload_tmp, payload_name, subject_tmp));
                Some(payload_tmp)
            } else {
                None
            };

            if arm_indices.len() == 1 {
                result_kind = self.emit_single_arm_case(
                    &arms[arm_indices[0]], payload_tmp_name.as_deref(),
                    &field_names, &field_kinds, &result_tmp,
                )?;
            } else {
                result_kind = self.emit_multi_arm_case(
                    arms, arm_indices, &first_arm.pattern, payload_tmp_name.as_deref(),
                    &field_names, &field_kinds, &result_tmp,
                )?;
            }

            self.emit("break;");
            self.variables = saved_vars;
            self.indent -= 1;
            self.emit("}");
        }

        // Emit wildcard/default arm if present
        for &wi in &wildcard_indices {
            let arm = &arms[wi];
            self.emit("default: {");
            self.indent += 1;
            let saved_vars = self.variables.clone();
            let (body_val, body_kind) = self.compile_expr(&arm.body)?;
            result_kind = body_kind.clone();
            self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
            self.emit("break;");
            self.variables = saved_vars;
            self.indent -= 1;
            self.emit("}");
        }

        self.indent -= 1;
        self.emit("}");

        Ok((self.coerce_from_i64_expr(&result_tmp, &result_kind), result_kind))
    }

    fn emit_single_arm_case(
        &mut self,
        arm: &MatchArm,
        payload_tmp: Option<&str>,
        field_names: &[String],
        field_kinds: &[ValKind],
        result_tmp: &str,
    ) -> Result<ValKind, CCodeGenError> {
        let mut result_kind = ValKind::Int;
        if let Pattern::Variant { bindings, .. } = &arm.pattern {
            if !bindings.is_empty() {
                let pt = payload_tmp.unwrap();
                for (i, binding) in bindings.iter().enumerate() {
                    if binding == "_" {
                        continue;
                    }
                    let fkind = &field_kinds[i];
                    let c_type = self.kind_to_c_type_str(fkind);
                    let fname = &field_names[i];
                    self.emit(&format!("{} {} = {}->{};", c_type, binding, pt, fname));
                    self.variables.insert(binding.clone(), VarInfo {
                        c_name: binding.clone(),
                        kind: fkind.clone(),
                        is_mutable: false,
                    });
                }
            }

            if let Some(guard) = &arm.guard {
                let (guard_val, _) = self.compile_expr(guard)?;
                self.emit(&format!("if (!({})) break;", guard_val));
            }

            let (body_val, body_kind) = self.compile_expr(&arm.body)?;
            result_kind = body_kind.clone();
            self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
        }
        Ok(result_kind)
    }

    fn emit_multi_arm_case(
        &mut self,
        arms: &[MatchArm],
        arm_indices: &[usize],
        first_pattern: &Pattern,
        payload_tmp: Option<&str>,
        field_names: &[String],
        field_kinds: &[ValKind],
        result_tmp: &str,
    ) -> Result<ValKind, CCodeGenError> {
        let mut result_kind = ValKind::Int;

        // Declare bindings once before the chain so all arms can use them
        let mut declared_bindings: Vec<(String, ValKind)> = Vec::new();
        if let Pattern::Variant { bindings, .. } = first_pattern {
            if !bindings.is_empty() {
                let pt = payload_tmp.unwrap();
                for (i, binding) in bindings.iter().enumerate() {
                    let fkind = &field_kinds[i];
                    let fname = &field_names[i];
                    if binding == "_" {
                        declared_bindings.push((binding.clone(), fkind.clone()));
                        continue;
                    }
                    let c_type = self.kind_to_c_type_str(fkind);
                    self.emit(&format!("{} {} = {}->{};", c_type, binding, pt, fname));
                    self.variables.insert(binding.clone(), VarInfo {
                        c_name: binding.clone(),
                        kind: fkind.clone(),
                        is_mutable: false,
                    });
                    declared_bindings.push((binding.clone(), fkind.clone()));
                }
            }
        }

        let mut first_branch = true;
        for &arm_idx in arm_indices {
            let arm = &arms[arm_idx];
            if let Pattern::Variant { bindings, .. } = &arm.pattern {
                // Re-register bindings (they may use different names per arm)
                for (i, binding) in bindings.iter().enumerate() {
                    if binding == "_" {
                        continue;
                    }
                    if i < declared_bindings.len() && binding != &declared_bindings[i].0 {
                        // Different binding name — alias it
                        let fkind = &field_kinds[i];
                        let c_type = self.kind_to_c_type_str(fkind);
                        self.emit(&format!("{} {} = {};", c_type, binding, declared_bindings[i].0));
                        self.variables.insert(binding.clone(), VarInfo {
                            c_name: binding.clone(),
                            kind: fkind.clone(),
                            is_mutable: false,
                        });
                    } else if !self.variables.contains_key(binding) {
                        let fkind = &field_kinds[i];
                        self.variables.insert(binding.clone(), VarInfo {
                            c_name: binding.clone(),
                            kind: fkind.clone(),
                            is_mutable: false,
                        });
                    }
                }

                if let Some(guard) = &arm.guard {
                    let (guard_val, _) = self.compile_expr(guard)?;
                    if first_branch {
                        self.emit(&format!("if ({}) {{", guard_val));
                        first_branch = false;
                    } else {
                        self.emit(&format!("}} else if ({}) {{", guard_val));
                    }
                    self.indent += 1;
                    let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                    result_kind = body_kind.clone();
                    self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
                    self.indent -= 1;
                } else {
                    // No guard — this is the final fallback for this tag
                    if first_branch {
                        // Only arm has no guard (shouldn't happen in multi-arm, but handle it)
                        let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                        result_kind = body_kind.clone();
                        self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
                    } else {
                        self.emit("} else {");
                        self.indent += 1;
                        let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                        result_kind = body_kind.clone();
                        self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
                        self.indent -= 1;
                        self.emit("}");
                    }
                    first_branch = false; // mark closed
                }
            }
        }

        // Close the if chain if last arm had a guard (no else)
        let last_arm = &arms[*arm_indices.last().unwrap()];
        if last_arm.guard.is_some() {
            self.emit("}");
        }

        Ok(result_kind)
    }

    fn compile_literal_match(&mut self, subject_val: &str, subject_kind: &ValKind, arms: &[MatchArm]) -> Result<(String, ValKind), CCodeGenError> {
        let result_tmp = self.tmp();
        self.emit(&format!("int64_t {} = 0;", result_tmp));
        let mut result_kind = ValKind::Int;
        let mut first = true;
        let mut closed = false;
        let mut extra_else_depth: usize = 0;

        for arm in arms {
            let is_wildcard = matches!(&arm.pattern, Pattern::Wildcard);
            let is_var_binding = matches!(&arm.pattern, Pattern::Variant { name, bindings }
                if bindings.is_empty() && !self.variant_to_enum.contains_key(name));

            if is_wildcard || (is_var_binding && arm.guard.is_none()) {
                // Unconditional wildcard/variable binding (no guard) — this is
                // the final catch-all arm.
                if first {
                    self.emit("{");
                } else {
                    self.emit("} else {");
                }
                self.indent += 1;
                let saved_vars = self.variables.clone();

                if let Pattern::Variant { name, .. } = &arm.pattern {
                    let c_type = self.kind_to_c_type_str(subject_kind);
                    self.emit(&format!("{} {} = {};", c_type, name, subject_val));
                    self.variables.insert(name.clone(), VarInfo {
                        c_name: name.clone(),
                        kind: subject_kind.clone(),
                        is_mutable: false,
                    });
                }

                let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                result_kind = body_kind.clone();
                self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));

                self.variables = saved_vars;
                self.indent -= 1;
                self.emit("}");
                closed = true;
                break;
            }

            if is_var_binding {
                // Variable binding WITH guard — close previous block first so
                // guard temporaries are emitted in the correct scope.
                let guard = arm.guard.as_ref().unwrap();

                if !first {
                    // Close the previous arm and open an else block so guard
                    // setup code (temporaries, div-by-zero checks) is in scope
                    // for the subsequent if condition.
                    self.emit("} else {");
                    self.indent += 1;
                    extra_else_depth += 1;
                }

                let saved_vars = self.variables.clone();

                if let Pattern::Variant { name, .. } = &arm.pattern {
                    // Bind variable before guard evaluation
                    let c_type = self.kind_to_c_type_str(subject_kind);
                    if !self.variables.contains_key(name) || first {
                        self.emit(&format!("{} {} = {};", c_type, name, subject_val));
                    }
                    self.variables.insert(name.clone(), VarInfo {
                        c_name: name.clone(),
                        kind: subject_kind.clone(),
                        is_mutable: false,
                    });
                }

                let (guard_val, _) = self.compile_expr(guard)?;
                self.emit(&format!("if ({}) {{", guard_val));
                first = false;
                self.indent += 1;

                let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                result_kind = body_kind.clone();
                self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
                self.variables = saved_vars;
                self.indent -= 1;
                continue;
            }

            let cmp = self.compile_pattern_cmp(subject_val, subject_kind, &arm.pattern)?;
            if first {
                self.emit(&format!("if ({}) {{", cmp));
                first = false;
            } else {
                self.emit(&format!("}} else if ({}) {{", cmp));
            }
            self.indent += 1;
            let saved_vars = self.variables.clone();

            if let Some(guard) = &arm.guard {
                let (guard_val, _) = self.compile_expr(guard)?;
                self.emit(&format!("if ({}) {{", guard_val));
                self.indent += 1;
            }

            let (body_val, body_kind) = self.compile_expr(&arm.body)?;
            result_kind = body_kind.clone();
            self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));

            if arm.guard.is_some() {
                self.indent -= 1;
                self.emit("}");
            }

            self.variables = saved_vars;
            self.indent -= 1;
        }

        if !first && !closed {
            self.emit("}");
        }

        // Close any extra else blocks opened for guarded variable-binding arms
        for _ in 0..extra_else_depth {
            self.indent -= 1;
            self.emit("}");
        }

        Ok((self.coerce_from_i64_expr(&result_tmp, &result_kind), result_kind))
    }

    fn compile_pattern_cmp(&mut self, subject: &str, _kind: &ValKind, pattern: &Pattern) -> Result<String, CCodeGenError> {
        match pattern {
            Pattern::IntLit(n) => Ok(format!("({} == {}LL)", subject, n)),
            Pattern::BoolLit(b) => Ok(format!("({} == {})", subject, if *b { 1 } else { 0 })),
            Pattern::StringLit(s) => {
                let str_lit = self.compile_string_literal(s);
                Ok(format!("(ore_str_eq({}, {}) != 0)", subject, str_lit))
            }
            Pattern::Range(start, end) => {
                Ok(format!("({} >= {}LL && {} <= {}LL)", subject, start, subject, end))
            }
            Pattern::Or(alternatives) => {
                let cmps: Vec<String> = alternatives.iter()
                    .map(|a| self.compile_pattern_cmp(subject, _kind, a))
                    .collect::<Result<_, _>>()?;
                Ok(format!("({})", cmps.join(" || ")))
            }
            Pattern::FloatLit(f) => {
                // Format float; ensure it has a decimal point for C
                let s = if f.fract() == 0.0 { format!("{}.0", f) } else { format!("{}", f) };
                Ok(format!("({} == {})", subject, s))
            }
            Pattern::Wildcard => Ok("1".to_string()),
            Pattern::Variant { name, .. } => Err(self.err(format!("unsupported pattern Variant({}) in compile_pattern_cmp", name))),
        }
    }

    fn compile_option_match(&mut self, subject_val: &str, arms: &[MatchArm]) -> Result<(String, ValKind), CCodeGenError> {
        self.compile_tagged_union_match(subject_val, arms, &[("None", 0), ("Some", 1)], |tag| tag == 1, "opt")
    }

    fn compile_result_match(&mut self, subject_val: &str, arms: &[MatchArm]) -> Result<(String, ValKind), CCodeGenError> {
        self.compile_tagged_union_match(subject_val, arms, &[("Ok", 0), ("Err", 1)], |_| true, "res")
    }

    fn compile_tagged_union_match(
        &mut self,
        subject_val: &str,
        arms: &[MatchArm],
        variant_tags: &[(&str, u8)],
        has_payload: impl Fn(u8) -> bool,
        _prefix: &str,
    ) -> Result<(String, ValKind), CCodeGenError> {
        let subject_tmp = self.tmp();
        self.emit(&format!("OreTaggedUnion {} = {};", subject_tmp, subject_val));
        let result_tmp = self.tmp();
        self.emit(&format!("int64_t {} = 0;", result_tmp));
        let mut result_kind = ValKind::Int;

        self.emit(&format!("switch ({}.tag) {{", subject_tmp));
        self.indent += 1;

        for arm in arms {
            match &arm.pattern {
                Pattern::Variant { name, bindings } => {
                    let vtag = variant_tags.iter()
                        .find(|(n, _)| *n == name.as_str())
                        .map(|(_, t)| *t)
                        .ok_or_else(|| self.err(format!("unknown variant '{}'", name)))?;

                    self.emit(&format!("case {}: {{", vtag));
                    self.indent += 1;
                    let saved_vars = self.variables.clone();
                    let saved_dynamic = self.dynamic_kind_tags.clone();

                    if has_payload(vtag) && !bindings.is_empty() {
                        self.emit(&format!("int64_t {} = {}.val;", bindings[0], subject_tmp));
                        let kind_var = format!("{}_kind", bindings[0]);
                        self.emit(&format!("int8_t {} = {}.kind;", kind_var, subject_tmp));
                        self.variables.insert(bindings[0].clone(), VarInfo {
                            c_name: bindings[0].clone(),
                            kind: ValKind::Int,
                            is_mutable: false,
                        });
                        self.dynamic_kind_tags.insert(bindings[0].clone());
                    }

                    let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                    result_kind = body_kind.clone();
                    self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
                    self.emit("break;");
                    self.variables = saved_vars;
                    self.dynamic_kind_tags = saved_dynamic;
                    self.indent -= 1;
                    self.emit("}");
                }
                Pattern::Wildcard => {
                    self.emit("default: {");
                    self.indent += 1;
                    let saved_vars = self.variables.clone();
                    let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                    result_kind = body_kind.clone();
                    self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
                    self.emit("break;");
                    self.variables = saved_vars;
                    self.indent -= 1;
                    self.emit("}");
                }
                _ => return Err(self.err("unsupported pattern in tagged union match")),
            }
        }

        self.indent -= 1;
        self.emit("}");

        Ok((self.coerce_from_i64_expr(&result_tmp, &result_kind), result_kind))
    }

    pub(crate) fn compile_variant_construct(&mut self, variant_name: &str, fields: &[(String, Expr)]) -> Result<(String, ValKind), CCodeGenError> {
        let enum_name = self.variant_to_enum.get(variant_name)
            .ok_or_else(|| self.err(format!("unknown variant '{}'", variant_name)))?.clone();
        let struct_name = format!("struct ore_enum_{}", Self::mangle_name(&enum_name));
        let enum_info = self.enums.get(&enum_name).ok_or_else(|| self.err(format!("undefined enum '{}'", enum_name)))?;
        let variant = enum_info.variants.iter().find(|v| v.name == variant_name)
            .ok_or_else(|| self.err(format!("unknown variant '{}'", variant_name)))?;
        let tag = variant.tag;
        let field_names = variant.field_names.clone();
        let field_kinds = variant.field_kinds.clone();

        let tmp = self.tmp();
        self.emit(&format!("{} {};", struct_name, tmp));
        self.emit(&format!("{}.tag = {};", tmp, tag));

        if !fields.is_empty() {
            let payload_name = format!("ore_payload_{}_{}", Self::mangle_name(&enum_name), variant_name);
            let payload_tmp = self.tmp();
            self.emit(&format!("struct {}* {} = (struct {}*){}.data;", payload_name, payload_tmp, payload_name, tmp));
            for (name, expr) in fields {
                let idx = field_names.iter().position(|n| n == name)
                    .ok_or_else(|| self.err(format!("unknown field '{}' on variant '{}'", name, variant_name)))?;
                let (val, val_kind) = self.compile_expr(expr)?;
                let fname = &field_names[idx];
                let target_kind = &field_kinds[idx];
                let coerced = self.coerce_expr(&val, &val_kind, target_kind);
                self.emit(&format!("{}->{} = {};", payload_tmp, fname, coerced));
            }
        }

        Ok((tmp, ValKind::Enum(enum_name)))
    }

    pub(crate) fn compile_option_method(&mut self, val: &str, method: &str, args: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        let tmp = self.tmp();
        self.emit(&format!("OreTaggedUnion {} = {};", tmp, val));
        match method {
            "unwrap_or" => {
                let (default_val, default_kind) = self.compile_expr(&args[0])?;
                let result = self.tmp();
                let c_type = self.kind_to_c_type_str(&default_kind);
                self.emit(&format!("{} {} = ({}.tag == 1) ? {} : {};",
                    c_type, result,
                    tmp,
                    self.coerce_from_i64_expr(&format!("{}.val", tmp), &default_kind),
                    default_val));
                Ok((result, default_kind))
            }
            "unwrap" => Ok((format!("{}.val", tmp), ValKind::Int)),
            "is_some" => Ok((format!("({}.tag == 1)", tmp), ValKind::Bool)),
            "is_none" => Ok((format!("({}.tag == 0)", tmp), ValKind::Bool)),
            "map" => {
                let (fn_ptr, _) = self.compile_lambda_arg(&args[0])?;
                let fn_name = fn_ptr.trim_start_matches("(void*)&");
                let result = self.tmp();
                self.emit(&format!("OreTaggedUnion {};", result));
                self.emit(&format!("if ({}.tag == 1) {{", tmp));
                self.indent += 1;
                self.emit(&format!("int64_t __mapped = {}({}.val);", fn_name, tmp));
                self.emit(&format!("{} = (OreTaggedUnion){{1, 0, __mapped}};", result));
                self.indent -= 1;
                self.emit("} else {");
                self.indent += 1;
                self.emit(&format!("{} = (OreTaggedUnion){{0, 0, 0}};", result));
                self.indent -= 1;
                self.emit("}");
                Ok((result, ValKind::Option))
            }
            _ => Err(self.err(format!("unknown Option method '{}'", method))),
        }
    }

    pub(crate) fn compile_result_method(&mut self, val: &str, method: &str, args: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        let tmp = self.tmp();
        self.emit(&format!("OreTaggedUnion {} = {};", tmp, val));
        match method {
            "unwrap_or" => {
                let (default_val, default_kind) = self.compile_expr(&args[0])?;
                let result = self.tmp();
                let c_type = self.kind_to_c_type_str(&default_kind);
                self.emit(&format!("{} {} = ({}.tag == 0) ? {} : {};",
                    c_type, result,
                    tmp,
                    self.coerce_from_i64_expr(&format!("{}.val", tmp), &default_kind),
                    default_val));
                Ok((result, default_kind))
            }
            "unwrap" => Ok((format!("{}.val", tmp), ValKind::Int)),
            "is_ok" => Ok((format!("({}.tag == 0)", tmp), ValKind::Bool)),
            "is_err" => Ok((format!("({}.tag == 1)", tmp), ValKind::Bool)),
            "map" => {
                let (fn_ptr, _) = self.compile_lambda_arg(&args[0])?;
                let fn_name = fn_ptr.trim_start_matches("(void*)&");
                let result = self.tmp();
                self.emit(&format!("OreTaggedUnion {};", result));
                self.emit(&format!("if ({}.tag == 0) {{", tmp));
                self.indent += 1;
                self.emit(&format!("int64_t __mapped = {}({}.val);", fn_name, tmp));
                self.emit(&format!("{} = (OreTaggedUnion){{0, 0, __mapped}};", result));
                self.indent -= 1;
                self.emit("} else {");
                self.indent += 1;
                self.emit(&format!("{} = {};", result, tmp));
                self.indent -= 1;
                self.emit("}");
                Ok((result, ValKind::Result))
            }
            _ => Err(self.err(format!("unknown Result method '{}'", method))),
        }
    }

    pub(crate) fn compile_record_construct(&mut self, type_name: &str, fields: &[(String, Expr)]) -> Result<(String, ValKind), CCodeGenError> {
        let struct_name = format!("struct ore_rec_{}", Self::mangle_name(type_name));
        let info = self.records.get(type_name).ok_or_else(|| self.err(format!("undefined type '{}'", type_name)))?;
        let field_names = info.field_names.clone();
        let field_kinds = info.field_kinds.clone();

        let tmp = self.tmp();
        self.emit(&format!("{} {};", struct_name, tmp));
        for (name, expr) in fields {
            let idx = field_names.iter().position(|n| n == name)
                .ok_or_else(|| self.err(format!("unknown field '{}' on type '{}'", name, type_name)))?;
            let (val, val_kind) = self.compile_expr(expr)?;
            let fname = &field_names[idx];
            let target_kind = &field_kinds[idx];
            let coerced = self.coerce_expr(&val, &val_kind, target_kind);
            self.emit(&format!("{}.{} = {};", tmp, fname, coerced));
        }

        Ok((tmp, ValKind::Record(type_name.to_string())))
    }

    pub(crate) fn compile_field_access(&mut self, object: &Expr, field: &str) -> Result<(String, ValKind), CCodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr(object)?;
        let type_name = match &obj_kind {
            ValKind::Record(name) => name.clone(),
            _ => {
                // Fallback: infer record type by searching all records for this field
                self.infer_record_type_by_field(field)
                    .ok_or_else(|| self.err(format!("field access '{}' on non-record type {:?}", field, obj_kind)))?
            }
        };
        let info = self.records.get(&type_name).ok_or_else(|| self.err(format!("undefined type '{}'", type_name)))?;
        let idx = info.field_names.iter().position(|n| n == field)
            .ok_or_else(|| self.err(format!("unknown field '{}' on type '{}'", field, type_name)))?;
        let field_kind = info.field_kinds[idx].clone();
        // When the object kind was not Record, we need to cast from i64 to the struct
        let obj_expr = if matches!(&obj_kind, ValKind::Record(_)) {
            obj_val
        } else {
            let c_type = format!("struct ore_rec_{}", Self::mangle_name(&type_name));
            format!("(*({c_type}*)(intptr_t)({obj_val}))")
        };
        Ok((format!("{}.{}", obj_expr, field), field_kind))
    }

    /// Check if a MapLit's entries represent an anonymous record literal.
    /// All keys must be bare Ident expressions, and the field set must match
    /// exactly one registered record type. Returns (type_name, fields) if so.
    pub(crate) fn try_as_record_fields(&self, entries: &[(Expr, Expr)]) -> Option<(String, Vec<(String, Expr)>)> {
        if entries.is_empty() {
            return None;
        }
        let mut field_names = Vec::new();
        let mut fields = Vec::new();
        for (k, v) in entries {
            if let Expr::Ident(name) = k {
                field_names.push(name.as_str());
                fields.push((name.clone(), v.clone()));
            } else {
                return None;
            }
        }
        // Find a record type whose fields match exactly
        let mut matches = Vec::new();
        for (name, info) in &self.records {
            if info.field_names.len() == field_names.len()
                && field_names.iter().all(|f| info.field_names.iter().any(|n| n == f))
            {
                matches.push(name.clone());
            }
        }
        if matches.len() == 1 {
            Some((matches.into_iter().next().unwrap(), fields))
        } else {
            None
        }
    }

    /// Search all registered records for one that has the given field name.
    /// Returns Some(type_name) if exactly one record has it, or the first match if multiple do.
    fn infer_record_type_by_field(&self, field: &str) -> Option<String> {
        let mut matches = Vec::new();
        for (name, info) in &self.records {
            if info.field_names.iter().any(|f| f == field) {
                matches.push(name.clone());
            }
        }
        matches.first().cloned()
    }

    /// Infer enum type from match arms containing variant patterns.
    /// Returns Some(enum_name) if any arm has a Variant pattern with a known enum variant.
    fn infer_enum_from_arms(&self, arms: &[MatchArm]) -> Option<String> {
        for arm in arms {
            if let Pattern::Variant { name, .. } = &arm.pattern {
                if let Some(enum_name) = self.variant_to_enum.get(name) {
                    return Some(enum_name.clone());
                }
            }
        }
        None
    }
}
