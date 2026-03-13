use super::*;

impl CCodeGen {
    /// Compile a lambda expression, emitting the function body to lambda_bodies.
    /// Returns (C expression for function pointer, ValKind).
    pub(crate) fn compile_lambda(
        &mut self,
        params: &[String],
        body: &Expr,
        param_kinds: std::option::Option<&[ValKind]>,
    ) -> Result<(String, ValKind), CCodeGenError> {
        let name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Build function signature: all params are i64 (runtime convention)
        let mut param_strs = Vec::new();
        for p in params {
            param_strs.push(format!("int64_t {}", p));
        }
        let params_str = if param_strs.is_empty() { "void".to_string() } else { param_strs.join(", ") };

        // Save state
        let saved_vars = self.variables.clone();
        let saved_lines = std::mem::take(&mut self.lines);
        let saved_indent = self.indent;

        // Set up lambda body compilation
        self.variables.clear();
        self.indent = 1;

        for (i, p) in params.iter().enumerate() {
            let kind = param_kinds.and_then(|k| k.get(i).cloned()).unwrap_or(ValKind::Int);

            // For pointer-based types, emit a conversion from i64 to the correct type
            match &kind {
                ValKind::Str | ValKind::List(_) | ValKind::Map(_) => {
                    let typed_name = format!("{}_typed", p);
                    self.emit(&format!("void* {} = (void*)(intptr_t){};", typed_name, p));
                    self.variables.insert(p.clone(), VarInfo {
                        c_name: typed_name,
                        kind,
                        is_mutable: false,
                    });
                }
                ValKind::Float => {
                    let typed_name = format!("{}_typed", p);
                    self.emit(&format!("double {} = *(double*)&{};", typed_name, p));
                    self.variables.insert(p.clone(), VarInfo {
                        c_name: typed_name,
                        kind,
                        is_mutable: false,
                    });
                }
                _ => {
                    self.variables.insert(p.clone(), VarInfo {
                        c_name: p.clone(),
                        kind,
                        is_mutable: false,
                    });
                }
            }
        }

        let (result, ret_kind) = self.compile_expr(body)?;

        // Convert return value to i64
        let ret_val = self.value_to_i64_expr(&result, &ret_kind);
        self.emit(&format!("return {};", ret_val));

        // Collect lambda body
        let body_lines = std::mem::take(&mut self.lines);

        // Restore state
        self.lines = saved_lines;
        self.indent = saved_indent;
        self.variables = saved_vars;

        // Emit lambda function to lambda_bodies
        self.lambda_bodies.push(format!("int64_t {}({}) {{", name, params_str));
        self.lambda_bodies.extend(body_lines);
        self.lambda_bodies.push("}".to_string());
        self.lambda_bodies.push(String::new());

        // Also add forward declaration
        self.forward_decls.push(format!("int64_t {}({});", name, params_str));

        // Register in functions
        self.functions.insert(name.clone(), FnInfo {
            ret_kind: ret_kind,
            param_kinds: vec![ValKind::Int; params.len()],
        });

        Ok((format!("(void*)&{}", name), ValKind::Int))
    }
}
