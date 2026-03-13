use super::*;
use std::collections::HashSet;

/// Walk an expression tree and collect identifiers that are not in `bound`.
fn find_free_vars(expr: &Expr, bound: &HashSet<String>) -> Vec<String> {
    let mut free = Vec::new();
    let mut seen = HashSet::new();
    collect_free_vars(expr, bound, &mut free, &mut seen);
    free
}

fn collect_free_vars(expr: &Expr, bound: &HashSet<String>, free: &mut Vec<String>, seen: &mut HashSet<String>) {
    match expr {
        Expr::Ident(name) => {
            if !bound.contains(name) && !seen.contains(name) {
                seen.insert(name.clone());
                free.push(name.clone());
            }
        }
        Expr::BinOp { left, right, .. }
        | Expr::AssertEq { left, right, .. } | Expr::AssertNe { left, right, .. } => {
            collect_free_vars(left, bound, free, seen);
            collect_free_vars(right, bound, free, seen);
        }
        Expr::UnaryMinus(inner) | Expr::UnaryNot(inner) | Expr::Print(inner)
        | Expr::OptionSome(inner) | Expr::ResultOk(inner) | Expr::ResultErr(inner)
        | Expr::Try(inner) | Expr::Sleep(inner) => {
            collect_free_vars(inner, bound, free, seen);
        }
        Expr::Call { func, args } => {
            collect_free_vars(func, bound, free, seen);
            for arg in args {
                collect_free_vars(arg, bound, free, seen);
            }
        }
        Expr::Lambda { params, body } => {
            let mut inner_bound = bound.clone();
            for p in params {
                inner_bound.insert(p.clone());
            }
            collect_free_vars(body, &inner_bound, free, seen);
        }
        Expr::IfElse { cond, then_block, else_block } => {
            collect_free_vars(cond, bound, free, seen);
            for stmt in then_block.iter_stmts() {
                collect_free_vars_stmt(stmt, bound, free, seen);
            }
            if let Some(eb) = else_block {
                for stmt in eb.iter_stmts() {
                    collect_free_vars_stmt(stmt, bound, free, seen);
                }
            }
        }
        Expr::ColonMatch { cond, then_expr, else_expr } => {
            collect_free_vars(cond, bound, free, seen);
            collect_free_vars(then_expr, bound, free, seen);
            if let Some(e) = else_expr {
                collect_free_vars(e, bound, free, seen);
            }
        }
        Expr::Match { subject, arms } => {
            collect_free_vars(subject, bound, free, seen);
            for arm in arms {
                if let Some(g) = &arm.guard {
                    collect_free_vars(g, bound, free, seen);
                }
                collect_free_vars(&arm.body, bound, free, seen);
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let StringPart::Expr(e) = part {
                    collect_free_vars(e, bound, free, seen);
                }
            }
        }
        Expr::BlockExpr(block) => {
            for s in block.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Expr::RecordConstruct { fields, .. } => {
            for (_, e) in fields {
                collect_free_vars(e, bound, free, seen);
            }
        }
        Expr::FieldAccess { object, .. } | Expr::OptionalChain { object, .. } => {
            collect_free_vars(object, bound, free, seen);
        }
        Expr::MethodCall { object, args, .. } | Expr::OptionalMethodCall { object, args, .. } => {
            collect_free_vars(object, bound, free, seen);
            for arg in args {
                collect_free_vars(arg, bound, free, seen);
            }
        }
        Expr::ListLit(elements) => {
            for e in elements {
                collect_free_vars(e, bound, free, seen);
            }
        }
        Expr::ListComp { expr, var, iterable, cond } => {
            collect_free_vars(iterable, bound, free, seen);
            let mut inner_bound = bound.clone();
            inner_bound.insert(var.clone());
            collect_free_vars(expr, &inner_bound, free, seen);
            if let Some(c) = cond {
                collect_free_vars(c, &inner_bound, free, seen);
            }
        }
        Expr::MapLit(entries) => {
            for (k, v) in entries {
                collect_free_vars(k, bound, free, seen);
                collect_free_vars(v, bound, free, seen);
            }
        }
        Expr::Index { object, index } => {
            collect_free_vars(object, bound, free, seen);
            collect_free_vars(index, bound, free, seen);
        }
        Expr::Assert { cond, .. } => {
            collect_free_vars(cond, bound, free, seen);
        }
        Expr::IntLit(_) | Expr::FloatLit(_) | Expr::BoolLit(_) | Expr::StringLit(_)
        | Expr::Break | Expr::OptionNone => {}
    }
}

fn collect_free_vars_stmt(stmt: &Stmt, bound: &HashSet<String>, free: &mut Vec<String>, seen: &mut HashSet<String>) {
    match stmt {
        Stmt::Let { value, .. } | Stmt::LetDestructure { value, .. }
        | Stmt::Assign { value, .. } => collect_free_vars(value, bound, free, seen),
        Stmt::Expr(e) | Stmt::Spawn(e) | Stmt::Return(Some(e)) => collect_free_vars(e, bound, free, seen),
        Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        Stmt::ForIn { start, end, body, .. } => {
            collect_free_vars(start, bound, free, seen);
            collect_free_vars(end, bound, free, seen);
            for s in body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::While { cond, body } => {
            collect_free_vars(cond, bound, free, seen);
            for s in body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::ForEach { iterable, body, .. } | Stmt::ForEachKV { iterable, body, .. } => {
            collect_free_vars(iterable, bound, free, seen);
            for s in body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::Loop { body } => {
            for s in body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::IndexAssign { object, index, value } => {
            collect_free_vars(object, bound, free, seen);
            collect_free_vars(index, bound, free, seen);
            collect_free_vars(value, bound, free, seen);
        }
        Stmt::FieldAssign { object, field: _, value } => {
            collect_free_vars(object, bound, free, seen);
            collect_free_vars(value, bound, free, seen);
        }
        Stmt::LocalFn(fndef) => {
            for s in fndef.body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
    }
}

impl CCodeGen {
    /// Compile a lambda expression, emitting the function body to lambda_bodies.
    /// Returns (C expression for function pointer, ValKind).
    /// Supports closures: detects free variables, creates capture struct, passes env_ptr.
    pub(crate) fn compile_lambda(
        &mut self,
        params: &[String],
        body: &Expr,
        param_kinds: std::option::Option<&[ValKind]>,
    ) -> Result<(String, ValKind), CCodeGenError> {
        let name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Detect free variables (captures)
        let bound: HashSet<String> = params.iter().cloned().collect();
        let free_vars = find_free_vars(body, &bound);

        // Filter to only variables that exist in the current scope
        let mut capture_names = Vec::new();
        let mut capture_kinds = Vec::new();
        for fv in &free_vars {
            if let Some(v) = self.variables.get(fv) {
                capture_names.push(fv.clone());
                capture_kinds.push(v.kind.clone());
            }
        }
        let has_captures = !capture_names.is_empty();

        // Build function signature
        let mut param_strs = Vec::new();
        if has_captures {
            param_strs.push("void* __env".to_string());
        }
        for p in params {
            param_strs.push(format!("int64_t {}", p));
        }
        let params_str = if param_strs.is_empty() { "void".to_string() } else { param_strs.join(", ") };

        // Emit capture struct definition if needed
        let captures_struct_name = if has_captures {
            let struct_name = format!("__captures_{}", name);
            let mut struct_def = format!("struct {} {{\n", struct_name);
            for (cap_name, cap_kind) in capture_names.iter().zip(capture_kinds.iter()) {
                let c_type = self.kind_to_c_type_str(cap_kind);
                struct_def.push_str(&format!("    {} {};\n", c_type, cap_name));
            }
            struct_def.push_str("};");
            self.top_level.push(struct_def);
            self.top_level.push(String::new());
            Some(struct_name)
        } else {
            None
        };

        // Save state
        let saved_vars = self.variables.clone();
        let saved_lines = std::mem::take(&mut self.lines);
        let saved_indent = self.indent;

        // Set up lambda body compilation
        self.variables.clear();
        self.indent = 1;

        // Extract captured variables from env_ptr at the start of the lambda body
        if has_captures {
            let struct_name = captures_struct_name.as_ref().unwrap();
            self.emit(&format!("struct {}* __cap = (struct {}*)__env;", struct_name, struct_name));
            for (cap_name, cap_kind) in capture_names.iter().zip(capture_kinds.iter()) {
                let c_type = self.kind_to_c_type_str(cap_kind);
                self.emit(&format!("{} {} = __cap->{};", c_type, cap_name, cap_name));
                self.variables.insert(cap_name.clone(), VarInfo {
                    c_name: cap_name.clone(),
                    kind: cap_kind.clone(),
                    is_mutable: false,
                });
            }
        }

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
            ret_kind: ret_kind.clone(),
            param_kinds: vec![ValKind::Int; params.len()],
        });

        // Store capture info for later use when building call sites
        if has_captures {
            self.lambda_captures.insert(name.clone(), capture_names.iter().zip(capture_kinds.iter()).map(|(n, k)| (n.clone(), k.clone())).collect());
        }

        // Build captures struct in the calling scope if needed, and return env_ptr expression
        if has_captures {
            let struct_name = captures_struct_name.as_ref().unwrap();
            let cap_var = self.tmp();
            self.emit(&format!("struct {} {};", struct_name, cap_var));
            for (cap_name, cap_kind) in capture_names.iter().zip(capture_kinds.iter()) {
                let c_name = self.variables.get(cap_name)
                    .map(|v| v.c_name.clone())
                    .unwrap_or_else(|| cap_name.clone());
                let _ = cap_kind;
                self.emit(&format!("{}.{} = {};", cap_var, cap_name, c_name));
            }
            // Store the captures variable name associated with this lambda
            // The fn_ptr expression includes env info
            Ok((format!("__closure_{}|{}", name, cap_var), ret_kind))
        } else {
            Ok((format!("(void*)&{}", name), ret_kind))
        }
    }

    /// Parse a closure expression string into (fn_ptr, env_ptr) parts.
    /// Closure format: "__closure_LAMBDA_NAME|CAP_VAR"
    /// Regular function: "(void*)&FUNC_NAME"
    pub(crate) fn parse_closure_expr(expr: &str) -> (String, String) {
        if let Some(rest) = expr.strip_prefix("__closure_") {
            if let Some(idx) = rest.find('|') {
                let lambda_name = &rest[..idx];
                let cap_var = &rest[idx+1..];
                (format!("(void*)&{}", lambda_name), format!("(void*)&{}", cap_var))
            } else {
                (expr.to_string(), "NULL".to_string())
            }
        } else {
            (expr.to_string(), "NULL".to_string())
        }
    }
}
