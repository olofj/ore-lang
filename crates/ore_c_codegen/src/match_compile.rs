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

        self.emit(&format!("switch ({}.tag) {{", subject_tmp));
        self.indent += 1;

        for arm in arms {
            match &arm.pattern {
                Pattern::Variant { name, bindings } => {
                    let variant = variants.iter().find(|v| v.0 == *name).ok_or_else(|| self.err(format!("unknown variant '{}'", name)))?;
                    let (_, vtag, ref field_names, ref field_kinds) = variant;

                    self.emit(&format!("case {}: {{", vtag));
                    self.indent += 1;

                    // Extract fields
                    if !bindings.is_empty() {
                        let payload_name = format!("ore_payload_{}_{}", Self::mangle_name(enum_name), name);
                        let payload_tmp = self.tmp();
                        self.emit(&format!("struct {}* {} = (struct {}*){}.data;",
                            payload_name, payload_tmp, payload_name, subject_tmp));

                        for (i, binding) in bindings.iter().enumerate() {
                            let fkind = &field_kinds[i];
                            let c_type = self.kind_to_c_type_str(fkind);
                            let fname = &field_names[i];
                            self.emit(&format!("{} {} = {}->{};", c_type, binding, payload_tmp, fname));
                            self.variables.insert(binding.clone(), VarInfo {
                                c_name: binding.clone(),
                                kind: fkind.clone(),
                                is_mutable: false,
                            });
                        }
                    }

                    // Guard
                    if let Some(guard) = &arm.guard {
                        let (guard_val, _) = self.compile_expr(guard)?;
                        self.emit(&format!("if (!({})) break;", guard_val));
                    }

                    let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                    result_kind = body_kind.clone();
                    self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
                    self.emit("break;");
                    self.indent -= 1;
                    self.emit("}");
                }
                Pattern::Wildcard => {
                    self.emit("default: {");
                    self.indent += 1;
                    let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                    result_kind = body_kind.clone();
                    self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
                    self.emit("break;");
                    self.indent -= 1;
                    self.emit("}");
                }
                _ => return Err(self.err("unsupported pattern in enum match")),
            }
        }

        self.indent -= 1;
        self.emit("}");

        Ok((self.coerce_from_i64_expr(&result_tmp, &result_kind), result_kind))
    }

    fn compile_literal_match(&mut self, subject_val: &str, subject_kind: &ValKind, arms: &[MatchArm]) -> Result<(String, ValKind), CCodeGenError> {
        let result_tmp = self.tmp();
        self.emit(&format!("int64_t {} = 0;", result_tmp));
        let mut result_kind = ValKind::Int;
        let mut first = true;
        let mut closed = false;

        for arm in arms {
            let is_wildcard = matches!(&arm.pattern, Pattern::Wildcard);
            let is_var_binding = matches!(&arm.pattern, Pattern::Variant { name, bindings }
                if bindings.is_empty() && !self.variant_to_enum.contains_key(name));

            if is_wildcard || is_var_binding {
                if first {
                    self.emit("{");
                } else {
                    self.emit("} else {");
                }
                self.indent += 1;

                // Bind variable if named
                if let Pattern::Variant { name, .. } = &arm.pattern {
                    let c_type = self.kind_to_c_type_str(subject_kind);
                    self.emit(&format!("{} {} = {};", c_type, name, subject_val));
                    self.variables.insert(name.clone(), VarInfo {
                        c_name: name.clone(),
                        kind: subject_kind.clone(),
                        is_mutable: false,
                    });
                }

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

                self.indent -= 1;
                self.emit("}");
                closed = true;
                break;
            }

            let cmp = self.compile_pattern_cmp(subject_val, subject_kind, &arm.pattern)?;
            if first {
                self.emit(&format!("if ({}) {{", cmp));
                first = false;
            } else {
                self.emit(&format!("}} else if ({}) {{", cmp));
            }
            self.indent += 1;

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

            self.indent -= 1;
        }

        if !first && !closed {
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
            _ => Err(self.err("unsupported pattern")),
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
                    self.indent -= 1;
                    self.emit("}");
                }
                Pattern::Wildcard => {
                    self.emit("default: {");
                    self.indent += 1;
                    let (body_val, body_kind) = self.compile_expr(&arm.body)?;
                    result_kind = body_kind.clone();
                    self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(&body_val, &body_kind)));
                    self.emit("break;");
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
                let (val, _) = self.compile_expr(expr)?;
                let fname = &field_names[idx];
                self.emit(&format!("{}->{} = {};", payload_tmp, fname, val));
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

        let tmp = self.tmp();
        self.emit(&format!("{} {};", struct_name, tmp));
        for (name, expr) in fields {
            let idx = field_names.iter().position(|n| n == name)
                .ok_or_else(|| self.err(format!("unknown field '{}' on type '{}'", name, type_name)))?;
            let (val, _) = self.compile_expr(expr)?;
            let fname = &field_names[idx];
            self.emit(&format!("{}.{} = {};", tmp, fname, val));
        }

        Ok((tmp, ValKind::Record(type_name.to_string())))
    }

    pub(crate) fn compile_field_access(&mut self, object: &Expr, field: &str) -> Result<(String, ValKind), CCodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr(object)?;
        let type_name = match &obj_kind {
            ValKind::Record(name) => name.clone(),
            _ => return Err(self.err("field access on non-record type")),
        };
        let info = self.records.get(&type_name).ok_or_else(|| self.err(format!("undefined type '{}'", type_name)))?;
        let idx = info.field_names.iter().position(|n| n == field)
            .ok_or_else(|| self.err(format!("unknown field '{}' on type '{}'", field, type_name)))?;
        let field_kind = info.field_kinds[idx].clone();
        Ok((format!("{}.{}", obj_val, field), field_kind))
    }
}
