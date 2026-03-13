use super::*;

impl CCodeGen {
    /// Compile a lambda expression, emitting the function body to lambda_bodies.
    /// Returns (C expression for function pointer, ValKind).
    pub(crate) fn compile_lambda(
        &mut self,
        params: &[String],
        body: &Expr,
        _param_kinds: std::option::Option<&[ValKind]>,
    ) -> Result<(String, ValKind), CCodeGenError> {
        let name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Build function signature: all params are i64
        let mut param_strs = Vec::new();
        // For now, no captures support in initial implementation
        // TODO: Add closure capture support
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

        for p in params {
            self.variables.insert(p.clone(), VarInfo {
                c_name: p.clone(),
                kind: ValKind::Int,
                is_mutable: false,
            });
        }

        let (result, _kind) = self.compile_expr(body)?;
        self.emit(&format!("return {};", result));

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
            ret_kind: ValKind::Int,
            param_kinds: vec![ValKind::Int; params.len()],
        });

        Ok((format!("(void*)&{}", name), ValKind::Int))
    }
}
