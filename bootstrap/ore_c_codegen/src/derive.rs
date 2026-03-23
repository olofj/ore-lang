//! Code generation for `deriving(...)` clauses on type and enum definitions.
//!
//! Supported derived traits:
//!   - `Debug`     — generates `debug() -> Str` (detailed string representation)
//!   - `Eq`        — generates `eq(other) -> Bool` (field-by-field equality)
//!   - `Clone`     — generates `clone() -> Self` (field-by-field copy)
//!   - `Serialize` — generates `toJson() -> Str` (JSON serialization)

use super::*;

/// Known derivable trait names.
const KNOWN_TRAITS: &[&str] = &["Debug", "Eq", "Clone", "Serialize"];

impl CCodeGen {
    /// Validate that all trait names in a deriving clause are known.
    pub(crate) fn validate_deriving(traits: &[String]) -> Result<(), CCodeGenError> {
        for t in traits {
            if !KNOWN_TRAITS.contains(&t.as_str()) {
                return Err(CCodeGenError {
                    msg: format!(
                        "unknown derived trait '{}'. Supported: {}",
                        t,
                        KNOWN_TRAITS.join(", ")
                    ),
                    line: None,
                });
            }
        }
        Ok(())
    }

    /// Generate and register all derived methods for record types.
    /// Called after `register_record` during type registration.
    pub(crate) fn generate_record_derives(
        &mut self,
        td: &TypeDef,
    ) -> Result<Vec<FnDef>, CCodeGenError> {
        if td.deriving.is_empty() {
            return Ok(Vec::new());
        }
        Self::validate_deriving(&td.deriving)?;

        let info = self.records.get(&td.name).cloned()
            .ok_or_else(|| self.err(format!("record '{}' not registered", td.name)))?;

        let mut fns = Vec::new();

        for trait_name in &td.deriving {
            match trait_name.as_str() {
                "Debug" => fns.push(self.gen_record_debug(&td.name, &info)),
                "Eq" => fns.push(self.gen_record_eq(&td.name, &info)),
                "Clone" => fns.push(self.gen_record_clone(&td.name, &info)),
                "Serialize" => fns.push(self.gen_record_serialize(&td.name, &info)),
                _ => {}
            }
        }

        Ok(fns)
    }

    /// Generate and register all derived methods for enum types.
    pub(crate) fn generate_enum_derives(
        &mut self,
        ed: &EnumDef,
    ) -> Result<Vec<FnDef>, CCodeGenError> {
        if ed.deriving.is_empty() {
            return Ok(Vec::new());
        }
        Self::validate_deriving(&ed.deriving)?;

        let info = self.enums.get(&ed.name).cloned()
            .ok_or_else(|| self.err(format!("enum '{}' not registered", ed.name)))?;

        let mut fns = Vec::new();

        for trait_name in &ed.deriving {
            match trait_name.as_str() {
                "Debug" => fns.push(self.gen_enum_debug(&ed.name, &info)),
                "Eq" => fns.push(self.gen_enum_eq(&ed.name, &info)),
                "Clone" => fns.push(self.gen_enum_clone(&ed.name)),
                "Serialize" => fns.push(self.gen_enum_serialize(&ed.name, &info)),
                _ => {}
            }
        }

        Ok(fns)
    }

    // ── Record Debug ──

    fn gen_record_debug(&mut self, type_name: &str, info: &RecordInfo) -> FnDef {
        // Emit a C function: void* TypeName_debug(struct ore_rec_TypeName self) {
        //   void* result = ore_str_new("TypeName(", ...);
        //   // for each field: result = ore_str_concat(result, "field: "); result = ore_str_concat(result, field_to_str);
        //   result = ore_str_concat(result, ")");
        //   return result;
        // }
        let fn_name = format!("{}_debug", type_name);
        let mangled_type = Self::mangle_name(type_name);
        let struct_type = format!("struct ore_rec_{}", mangled_type);

        let mut body_lines = Vec::new();
        let header = format!("{}(", type_name);
        body_lines.push(format!(
            "    void* ore_result = ore_str_new(\"{}\", {});",
            Self::escape_c_string(&header),
            header.len()
        ));

        for (i, (fname, fkind)) in info.field_names.iter().zip(info.field_kinds.iter()).enumerate() {
            if i > 0 {
                body_lines.push(format!(
                    "    ore_result = ore_str_concat(ore_result, ore_str_new(\", \", 2));"
                ));
            }
            let label = format!("{}: ", fname);
            body_lines.push(format!(
                "    ore_result = ore_str_concat(ore_result, ore_str_new(\"{}\", {}));",
                Self::escape_c_string(&label),
                label.len()
            ));
            let field_expr = format!("ore_self.{}", fname);
            let to_str = self.field_to_str_c_expr(&field_expr, fkind);
            body_lines.push(format!(
                "    ore_result = ore_str_concat(ore_result, {});",
                to_str
            ));
        }

        body_lines.push("    ore_result = ore_str_concat(ore_result, ore_str_new(\")\", 1));".to_string());
        body_lines.push("    return ore_result;".to_string());

        self.emit_derived_c_function(
            &fn_name,
            "void*",
            &format!("{} ore_self", struct_type),
            &body_lines,
        );

        // Register in functions map
        self.functions.insert(fn_name.clone(), FnInfo {
            ret_kind: ValKind::Str,
            param_kinds: vec![ValKind::Record(type_name.to_string())],
            context: vec![],
        });

        self.make_stub_fndef(&fn_name, type_name, &[], Some(TypeExpr::Named("Str".to_string())))
    }

    // ── Record Eq ──

    fn gen_record_eq(&mut self, type_name: &str, info: &RecordInfo) -> FnDef {
        let fn_name = format!("{}_eq", type_name);
        let mangled_type = Self::mangle_name(type_name);
        let struct_type = format!("struct ore_rec_{}", mangled_type);

        let mut body_lines = Vec::new();

        if info.field_names.is_empty() {
            body_lines.push("    return 1;".to_string());
        } else {
            for (fname, fkind) in info.field_names.iter().zip(info.field_kinds.iter()) {
                let cmp = self.field_eq_c_expr(
                    &format!("ore_self.{}", fname),
                    &format!("ore_other.{}", fname),
                    fkind,
                );
                body_lines.push(format!("    if (!{}) return 0;", cmp));
            }
            body_lines.push("    return 1;".to_string());
        }

        self.emit_derived_c_function(
            &fn_name,
            "int8_t",
            &format!("{} ore_self, {} ore_other", struct_type, struct_type),
            &body_lines,
        );

        self.functions.insert(fn_name.clone(), FnInfo {
            ret_kind: ValKind::Bool,
            param_kinds: vec![
                ValKind::Record(type_name.to_string()),
                ValKind::Record(type_name.to_string()),
            ],
            context: vec![],
        });

        self.make_stub_fndef(
            &fn_name,
            type_name,
            &[("other", type_name)],
            Some(TypeExpr::Named("Bool".to_string())),
        )
    }

    // ── Record Clone ──

    fn gen_record_clone(&mut self, type_name: &str, info: &RecordInfo) -> FnDef {
        let fn_name = format!("{}_clone", type_name);
        let mangled_type = Self::mangle_name(type_name);
        let struct_type = format!("struct ore_rec_{}", mangled_type);

        let mut body_lines = Vec::new();
        body_lines.push(format!("    {} ore_copy;", struct_type));

        for (fname, fkind) in info.field_names.iter().zip(info.field_kinds.iter()) {
            let clone_expr = self.field_clone_c_expr(&format!("ore_self.{}", fname), fkind);
            body_lines.push(format!("    ore_copy.{} = {};", fname, clone_expr));
        }

        body_lines.push("    return ore_copy;".to_string());

        self.emit_derived_c_function(
            &fn_name,
            &struct_type,
            &format!("{} ore_self", struct_type),
            &body_lines,
        );

        self.functions.insert(fn_name.clone(), FnInfo {
            ret_kind: ValKind::Record(type_name.to_string()),
            param_kinds: vec![ValKind::Record(type_name.to_string())],
            context: vec![],
        });

        self.make_stub_fndef(
            &fn_name,
            type_name,
            &[],
            Some(TypeExpr::Named(type_name.to_string())),
        )
    }

    // ── Record Serialize (toJson) ──

    fn gen_record_serialize(&mut self, type_name: &str, info: &RecordInfo) -> FnDef {
        let fn_name = format!("{}_toJson", type_name);
        let mangled_type = Self::mangle_name(type_name);
        let struct_type = format!("struct ore_rec_{}", mangled_type);

        let mut body_lines = Vec::new();
        body_lines.push("    void* ore_result = ore_str_new(\"{\", 1);".to_string());

        for (i, (fname, fkind)) in info.field_names.iter().zip(info.field_kinds.iter()).enumerate() {
            if i > 0 {
                body_lines.push(
                    "    ore_result = ore_str_concat(ore_result, ore_str_new(\", \", 2));"
                        .to_string(),
                );
            }
            // "fieldName": (with actual quote characters)
            let key_with_colon = format!("\"{}\"", fname);
            let key_with_colon = format!("{}: ", key_with_colon);
            let escaped = Self::escape_c_string(&key_with_colon);
            body_lines.push(format!(
                "    ore_result = ore_str_concat(ore_result, ore_str_new(\"{}\", {}));",
                escaped,
                key_with_colon.len()
            ));
            let field_expr = format!("ore_self.{}", fname);
            let json_val = self.field_to_json_c_expr(&field_expr, fkind);
            body_lines.push(format!(
                "    ore_result = ore_str_concat(ore_result, {});",
                json_val
            ));
        }

        body_lines.push(
            "    ore_result = ore_str_concat(ore_result, ore_str_new(\"}\", 1));".to_string(),
        );
        body_lines.push("    return ore_result;".to_string());

        self.emit_derived_c_function(
            &fn_name,
            "void*",
            &format!("{} ore_self", struct_type),
            &body_lines,
        );

        self.functions.insert(fn_name.clone(), FnInfo {
            ret_kind: ValKind::Str,
            param_kinds: vec![ValKind::Record(type_name.to_string())],
            context: vec![],
        });

        self.make_stub_fndef(&fn_name, type_name, &[], Some(TypeExpr::Named("Str".to_string())))
    }

    // ── Enum Debug ──

    fn gen_enum_debug(&mut self, type_name: &str, info: &EnumInfo) -> FnDef {
        let fn_name = format!("{}_debug", type_name);
        let mangled_type = Self::mangle_name(type_name);
        let struct_type = format!("struct ore_enum_{}", mangled_type);

        let mut body_lines = Vec::new();
        body_lines.push("    switch (ore_self.tag) {".to_string());

        for v in &info.variants {
            body_lines.push(format!("    case {}:", v.tag));
            if v.field_names.is_empty() {
                let name_str = format!("ore_str_new(\"{}\", {})", v.name, v.name.len());
                body_lines.push(format!("        return {};", name_str));
            } else {
                let payload_type = format!(
                    "struct ore_payload_{}_{}",
                    Self::mangle_name(type_name),
                    v.name
                );
                body_lines.push(format!(
                    "        {{ {} ore_pl; memcpy(&ore_pl, ore_self.data, sizeof(ore_pl));",
                    payload_type
                ));
                let header = format!("{}(", v.name);
                body_lines.push(format!(
                    "        void* ore_result = ore_str_new(\"{}\", {});",
                    Self::escape_c_string(&header),
                    header.len()
                ));
                for (i, (fname, fkind)) in
                    v.field_names.iter().zip(v.field_kinds.iter()).enumerate()
                {
                    if i > 0 {
                        body_lines.push(
                            "        ore_result = ore_str_concat(ore_result, ore_str_new(\", \", 2));"
                                .to_string(),
                        );
                    }
                    let label = format!("{}: ", fname);
                    body_lines.push(format!(
                        "        ore_result = ore_str_concat(ore_result, ore_str_new(\"{}\", {}));",
                        Self::escape_c_string(&label),
                        label.len()
                    ));
                    let field_expr = format!("ore_pl.{}", fname);
                    let to_str = self.field_to_str_c_expr(&field_expr, fkind);
                    body_lines.push(format!(
                        "        ore_result = ore_str_concat(ore_result, {});",
                        to_str
                    ));
                }
                body_lines.push(
                    "        ore_result = ore_str_concat(ore_result, ore_str_new(\")\", 1));"
                        .to_string(),
                );
                body_lines.push("        return ore_result; }".to_string());
            }
        }

        body_lines.push(format!(
            "    default: return ore_str_new(\"{}(?)\", {});",
            type_name,
            type_name.len() + 3
        ));
        body_lines.push("    }".to_string());

        self.emit_derived_c_function(
            &fn_name,
            "void*",
            &format!("{} ore_self", struct_type),
            &body_lines,
        );

        self.functions.insert(fn_name.clone(), FnInfo {
            ret_kind: ValKind::Str,
            param_kinds: vec![ValKind::Enum(type_name.to_string())],
            context: vec![],
        });

        self.make_stub_fndef(&fn_name, type_name, &[], Some(TypeExpr::Named("Str".to_string())))
    }

    // ── Enum Eq ──

    fn gen_enum_eq(&mut self, type_name: &str, info: &EnumInfo) -> FnDef {
        let fn_name = format!("{}_eq", type_name);
        let mangled_type = Self::mangle_name(type_name);
        let struct_type = format!("struct ore_enum_{}", mangled_type);

        let mut body_lines = Vec::new();
        body_lines.push("    if (ore_self.tag != ore_other.tag) return 0;".to_string());

        // For variants with payloads, compare field by field
        let has_payloads = info.variants.iter().any(|v| !v.field_names.is_empty());
        if has_payloads {
            body_lines.push("    switch (ore_self.tag) {".to_string());
            for v in &info.variants {
                if v.field_names.is_empty() {
                    continue;
                }
                body_lines.push(format!("    case {}:", v.tag));
                let payload_type = format!(
                    "struct ore_payload_{}_{}",
                    Self::mangle_name(type_name),
                    v.name
                );
                body_lines.push(format!(
                    "        {{ {} ore_pl_a; memcpy(&ore_pl_a, ore_self.data, sizeof(ore_pl_a));",
                    payload_type
                ));
                body_lines.push(format!(
                    "        {} ore_pl_b; memcpy(&ore_pl_b, ore_other.data, sizeof(ore_pl_b));",
                    payload_type
                ));
                for (fname, fkind) in v.field_names.iter().zip(v.field_kinds.iter()) {
                    let cmp = self.field_eq_c_expr(
                        &format!("ore_pl_a.{}", fname),
                        &format!("ore_pl_b.{}", fname),
                        fkind,
                    );
                    body_lines.push(format!("        if (!{}) return 0;", cmp));
                }
                body_lines.push("        return 1; }".to_string());
            }
            body_lines.push("    }".to_string());
        }

        body_lines.push("    return 1;".to_string());

        self.emit_derived_c_function(
            &fn_name,
            "int8_t",
            &format!("{} ore_self, {} ore_other", struct_type, struct_type),
            &body_lines,
        );

        self.functions.insert(fn_name.clone(), FnInfo {
            ret_kind: ValKind::Bool,
            param_kinds: vec![
                ValKind::Enum(type_name.to_string()),
                ValKind::Enum(type_name.to_string()),
            ],
            context: vec![],
        });

        self.make_stub_fndef(
            &fn_name,
            type_name,
            &[("other", type_name)],
            Some(TypeExpr::Named("Bool".to_string())),
        )
    }

    // ── Enum Clone ──

    fn gen_enum_clone(&mut self, type_name: &str) -> FnDef {
        // Enums are value types (small struct), so clone is just a copy
        let fn_name = format!("{}_clone", type_name);
        let mangled_type = Self::mangle_name(type_name);
        let struct_type = format!("struct ore_enum_{}", mangled_type);

        let body_lines = vec!["    return ore_self;".to_string()];

        self.emit_derived_c_function(
            &fn_name,
            &struct_type,
            &format!("{} ore_self", struct_type),
            &body_lines,
        );

        self.functions.insert(fn_name.clone(), FnInfo {
            ret_kind: ValKind::Enum(type_name.to_string()),
            param_kinds: vec![ValKind::Enum(type_name.to_string())],
            context: vec![],
        });

        self.make_stub_fndef(
            &fn_name,
            type_name,
            &[],
            Some(TypeExpr::Named(type_name.to_string())),
        )
    }

    // ── Enum Serialize (toJson) ──

    fn gen_enum_serialize(&mut self, type_name: &str, info: &EnumInfo) -> FnDef {
        let fn_name = format!("{}_toJson", type_name);
        let mangled_type = Self::mangle_name(type_name);
        let struct_type = format!("struct ore_enum_{}", mangled_type);

        let mut body_lines = Vec::new();
        body_lines.push("    switch (ore_self.tag) {".to_string());

        for v in &info.variants {
            body_lines.push(format!("    case {}:", v.tag));
            if v.field_names.is_empty() {
                // Simple variant: {"type": "VariantName"}
                let json = format!("{{\"type\": \"{}\"}}", v.name);
                let escaped = Self::escape_c_string(&json);
                body_lines.push(format!(
                    "        return ore_str_new(\"{}\", {});",
                    escaped,
                    json.len()
                ));
            } else {
                let payload_type = format!(
                    "struct ore_payload_{}_{}",
                    Self::mangle_name(type_name),
                    v.name
                );
                body_lines.push(format!(
                    "        {{ {} ore_pl; memcpy(&ore_pl, ore_self.data, sizeof(ore_pl));",
                    payload_type
                ));
                // {"type": "VariantName", "field1": val1, ...}
                let type_key = format!("{{\"type\": \"{}\", ", v.name);
                let escaped_key = Self::escape_c_string(&type_key);
                body_lines.push(format!(
                    "        void* ore_result = ore_str_new(\"{}\", {});",
                    escaped_key,
                    type_key.len()
                ));
                for (i, (fname, fkind)) in
                    v.field_names.iter().zip(v.field_kinds.iter()).enumerate()
                {
                    if i > 0 {
                        body_lines.push(
                            "        ore_result = ore_str_concat(ore_result, ore_str_new(\", \", 2));"
                                .to_string(),
                        );
                    }
                    let key_with_colon = format!("\"{}\": ", fname);
                    let escaped = Self::escape_c_string(&key_with_colon);
                    body_lines.push(format!(
                        "        ore_result = ore_str_concat(ore_result, ore_str_new(\"{}\", {}));",
                        escaped,
                        key_with_colon.len()
                    ));
                    let field_expr = format!("ore_pl.{}", fname);
                    let json_val = self.field_to_json_c_expr(&field_expr, fkind);
                    body_lines.push(format!(
                        "        ore_result = ore_str_concat(ore_result, {});",
                        json_val
                    ));
                }
                body_lines.push(
                    "        ore_result = ore_str_concat(ore_result, ore_str_new(\"}\", 1));"
                        .to_string(),
                );
                body_lines.push("        return ore_result; }".to_string());
            }
        }

        body_lines.push(
            "    default: return ore_str_new(\"null\", 4);".to_string()
        );
        body_lines.push("    }".to_string());

        self.emit_derived_c_function(
            &fn_name,
            "void*",
            &format!("{} ore_self", struct_type),
            &body_lines,
        );

        self.functions.insert(fn_name.clone(), FnInfo {
            ret_kind: ValKind::Str,
            param_kinds: vec![ValKind::Enum(type_name.to_string())],
            context: vec![],
        });

        self.make_stub_fndef(&fn_name, type_name, &[], Some(TypeExpr::Named("Str".to_string())))
    }

    // ── Helpers ──

    /// Emit a raw C function directly into the lambda_bodies buffer
    /// (which gets placed before main functions in the output).
    fn emit_derived_c_function(
        &mut self,
        fn_name: &str,
        ret_type: &str,
        params: &str,
        body_lines: &[String],
    ) {
        let c_fn_name = Self::mangle_fn_name(fn_name);
        self.forward_decls
            .push(format!("{} {}({});", ret_type, c_fn_name, params));
        self.lambda_bodies
            .push(format!("{} {}({}) {{", ret_type, c_fn_name, params));
        for line in body_lines {
            self.lambda_bodies.push(line.clone());
        }
        self.lambda_bodies.push("}".to_string());
        self.lambda_bodies.push(String::new());
        self.compiled_functions.insert(fn_name.to_string());
    }

    /// Create a stub FnDef (for the declare/compile pipeline — body is never used
    /// since we emit the C directly).
    fn make_stub_fndef(
        &self,
        fn_name: &str,
        type_name: &str,
        extra_params: &[(&str, &str)],
        ret_type: Option<TypeExpr>,
    ) -> FnDef {
        let mut params = vec![Param {
            name: "self".to_string(),
            ty: TypeExpr::Named(type_name.to_string()),
            default: None,
        }];
        for (name, ty) in extra_params {
            params.push(Param {
                name: name.to_string(),
                ty: TypeExpr::Named(ty.to_string()),
                default: None,
            });
        }
        FnDef {
            name: fn_name.to_string(),
            type_params: vec![],
            params,
            ret_type,
            context: vec![],
            body: Block { stmts: vec![] },
        }
    }

    /// Generate a C expression that converts a field value to a string.
    fn field_to_str_c_expr(&self, expr: &str, kind: &ValKind) -> String {
        match kind {
            ValKind::Str => expr.to_string(),
            ValKind::Int => format!("ore_int_to_str({})", expr),
            ValKind::Float => format!("ore_float_to_str({})", expr),
            ValKind::Bool => format!("ore_bool_to_str({})", expr),
            _ => format!("ore_int_to_str((int64_t)({}))", expr),
        }
    }

    /// Generate a C expression that compares two field values for equality.
    fn field_eq_c_expr(&self, a: &str, b: &str, kind: &ValKind) -> String {
        match kind {
            ValKind::Str => format!("ore_str_eq({}, {})", a, b),
            ValKind::Float => format!("(({}) == ({}))", a, b),
            _ => format!("(({}) == ({}))", a, b),
        }
    }

    /// Generate a C expression that clones a field value.
    /// Strings are reference-counted, so a shallow copy is sufficient.
    fn field_clone_c_expr(&self, expr: &str, _kind: &ValKind) -> String {
        // All primitive types and RC'd strings are safe to copy by value
        expr.to_string()
    }

    /// Generate a C expression that converts a field value to a JSON string.
    fn field_to_json_c_expr(&self, expr: &str, kind: &ValKind) -> String {
        match kind {
            ValKind::Str => {
                // Wrap in quotes: "\"value\""
                format!(
                    "ore_str_concat(ore_str_new(\"\\\"\", 1), ore_str_concat({}, ore_str_new(\"\\\"\", 1)))",
                    expr
                )
            }
            ValKind::Int => format!("ore_int_to_str({})", expr),
            ValKind::Float => format!("ore_float_to_str({})", expr),
            ValKind::Bool => format!(
                "(({}) ? ore_str_new(\"true\", 4) : ore_str_new(\"false\", 5))",
                expr
            ),
            _ => format!("ore_int_to_str((int64_t)({}))", expr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_known_traits() {
        assert!(CCodeGen::validate_deriving(&["Debug".into(), "Eq".into()]).is_ok());
        assert!(CCodeGen::validate_deriving(&["Clone".into(), "Serialize".into()]).is_ok());
    }

    #[test]
    fn validate_unknown_trait() {
        let result = CCodeGen::validate_deriving(&["Unknown".into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().msg.contains("Unknown"));
    }

    #[test]
    fn record_debug_generates_function() {
        let mut cg = CCodeGen::new();
        let td = TypeDef {
            name: "Point".into(),
            type_params: vec![],
            fields: vec![
                FieldDef { name: "x".into(), ty: TypeExpr::Named("Int".into()) },
                FieldDef { name: "y".into(), ty: TypeExpr::Named("Int".into()) },
            ],
            deriving: vec!["Debug".into()],
        };
        cg.register_record(&td).unwrap();
        let fns = cg.generate_record_derives(&td).unwrap();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "Point_debug");
        assert!(cg.functions.contains_key("Point_debug"));
    }

    #[test]
    fn record_eq_generates_function() {
        let mut cg = CCodeGen::new();
        let td = TypeDef {
            name: "Point".into(),
            type_params: vec![],
            fields: vec![
                FieldDef { name: "x".into(), ty: TypeExpr::Named("Int".into()) },
                FieldDef { name: "y".into(), ty: TypeExpr::Named("Int".into()) },
            ],
            deriving: vec!["Eq".into()],
        };
        cg.register_record(&td).unwrap();
        let fns = cg.generate_record_derives(&td).unwrap();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "Point_eq");
        assert!(cg.functions.contains_key("Point_eq"));
        // Should have 2 params (self + other)
        assert_eq!(fns[0].params.len(), 2);
    }

    #[test]
    fn record_all_derives() {
        let mut cg = CCodeGen::new();
        let td = TypeDef {
            name: "User".into(),
            type_params: vec![],
            fields: vec![
                FieldDef { name: "name".into(), ty: TypeExpr::Named("Str".into()) },
                FieldDef { name: "age".into(), ty: TypeExpr::Named("Int".into()) },
            ],
            deriving: vec!["Debug".into(), "Eq".into(), "Clone".into(), "Serialize".into()],
        };
        cg.register_record(&td).unwrap();
        let fns = cg.generate_record_derives(&td).unwrap();
        assert_eq!(fns.len(), 4);
        assert!(cg.functions.contains_key("User_debug"));
        assert!(cg.functions.contains_key("User_eq"));
        assert!(cg.functions.contains_key("User_clone"));
        assert!(cg.functions.contains_key("User_toJson"));
    }

    #[test]
    fn enum_debug_generates_function() {
        let mut cg = CCodeGen::new();
        let ed = EnumDef {
            name: "Color".into(),
            variants: vec![
                Variant { name: "Red".into(), fields: vec![] },
                Variant { name: "Green".into(), fields: vec![] },
            ],
            deriving: vec!["Debug".into()],
        };
        cg.register_enum(&ed).unwrap();
        let fns = cg.generate_enum_derives(&ed).unwrap();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "Color_debug");
    }

    #[test]
    fn no_deriving_produces_no_fns() {
        let mut cg = CCodeGen::new();
        let td = TypeDef {
            name: "Empty".into(),
            type_params: vec![],
            fields: vec![],
            deriving: vec![],
        };
        cg.register_record(&td).unwrap();
        let fns = cg.generate_record_derives(&td).unwrap();
        assert!(fns.is_empty());
    }
}
