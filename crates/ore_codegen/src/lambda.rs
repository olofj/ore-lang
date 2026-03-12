use super::*;
use inkwell::values::{FunctionValue, PointerValue};
use inkwell::types::BasicType;

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
        Expr::BinOp { left, right, .. } => {
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
            // Lambda params introduce new bindings; they shadow outer names
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
        Expr::FieldAccess { object, .. } => {
            collect_free_vars(object, bound, free, seen);
        }
        Expr::MethodCall { object, args, .. } => {
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
        Expr::OptionalChain { object, .. } => {
            collect_free_vars(object, bound, free, seen);
        }
        Expr::OptionalMethodCall { object, args, .. } => {
            collect_free_vars(object, bound, free, seen);
            for arg in args {
                collect_free_vars(arg, bound, free, seen);
            }
        }
        Expr::Assert { cond, .. } => {
            collect_free_vars(cond, bound, free, seen);
        }
        Expr::AssertEq { left, right, .. } | Expr::AssertNe { left, right, .. } => {
            collect_free_vars(left, bound, free, seen);
            collect_free_vars(right, bound, free, seen);
        }
        // Literals and constants have no free variables
        Expr::IntLit(_) | Expr::FloatLit(_) | Expr::BoolLit(_) | Expr::StringLit(_)
        | Expr::Break | Expr::OptionNone => {}
    }
}

fn collect_free_vars_stmt(stmt: &Stmt, bound: &HashSet<String>, free: &mut Vec<String>, seen: &mut HashSet<String>) {
    match stmt {
        Stmt::Let { value, .. } => collect_free_vars(value, bound, free, seen),
        Stmt::LetDestructure { value, .. } => collect_free_vars(value, bound, free, seen),
        Stmt::Assign { value, .. } => collect_free_vars(value, bound, free, seen),
        Stmt::Expr(e) => collect_free_vars(e, bound, free, seen),
        Stmt::Return(Some(e)) => collect_free_vars(e, bound, free, seen),
        Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        Stmt::Spawn(e) => collect_free_vars(e, bound, free, seen),
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
            // Collect free vars from the local function's body
            for s in fndef.body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
    }
}

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_lambda(
        &mut self,
        params: &[String],
        body: &Expr,
        _parent_fn: FunctionValue<'ctx>,
    ) -> Result<FunctionValue<'ctx>, CodeGenError> {
        self.compile_lambda_with_kinds(params, body, _parent_fn, None)
    }

    pub(crate) fn compile_lambda_with_kinds(
        &mut self,
        params: &[String],
        body: &Expr,
        _parent_fn: FunctionValue<'ctx>,
        param_kinds: Option<&[ValKind]>,
    ) -> Result<FunctionValue<'ctx>, CodeGenError> {
        let name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        let i64_type = self.context.i64_type();
        let ptr_type = self.ptr_type();

        // Detect free variables (captures) — identifiers in body not in params
        let bound: HashSet<String> = params.iter().cloned().collect();
        let free_vars = find_free_vars(body, &bound);

        // Filter to only variables that exist in the current scope
        let mut capture_names = Vec::new();
        let mut capture_types = Vec::new();
        let mut capture_kinds = Vec::new();
        for fv in &free_vars {
            if let Some((_ptr, ty, kind, _)) = self.variables.get(fv) {
                capture_names.push(fv.clone());
                capture_types.push(*ty);
                capture_kinds.push(kind.clone());
            }
        }
        let has_captures = !capture_names.is_empty();

        // Build function signature: if captures, first param is env_ptr
        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = Vec::new();
        if has_captures {
            param_types.push(ptr_type.into()); // env_ptr
        }
        for _ in params {
            param_types.push(i64_type.into());
        }
        let fn_type = i64_type.fn_type(&param_types, false);
        let lambda_fn = self.module.add_function(&name, fn_type, None);

        // Build the captures struct type and store CaptureInfo if needed
        let captures_struct_type = if has_captures {
            let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> = capture_types.clone();
            let st = self.context.struct_type(&field_types, false);
            self.lambda_captures.insert(name.clone(), CaptureInfo {
                struct_type: st,
                names: capture_names.clone(),
                types: capture_types.clone(),
                kinds: capture_kinds.clone(),
            });
            Some(st)
        } else {
            None
        };

        // Save current state
        let saved_vars = self.variables.clone();
        let saved_block = self.builder.get_insert_block();

        // Build lambda body
        let entry = self.context.append_basic_block(lambda_fn, "entry");
        self.builder.position_at_end(entry);
        self.variables.clear();

        // If we have captures, extract them from the env_ptr (first param)
        if has_captures {
            let env_ptr = lambda_fn.get_nth_param(0).unwrap().into_pointer_value();
            let st = captures_struct_type.unwrap();
            for (i, cap_name) in capture_names.iter().enumerate() {
                let field_ptr = bld!(self.builder.build_struct_gep(
                    st, env_ptr, i as u32, &format!("cap_{}", cap_name)
                ))?;
                let field_ty = capture_types[i];
                let val = bld!(self.builder.build_load(field_ty, field_ptr, cap_name))?;
                let alloca = bld!(self.builder.build_alloca(field_ty, cap_name))?;
                bld!(self.builder.build_store(alloca, val))?;
                self.variables.insert(cap_name.clone(), (alloca, field_ty, capture_kinds[i].clone(), false));
            }
        }

        // Bind lambda parameters (offset by 1 if captures exist)
        let param_offset: u32 = if has_captures { 1 } else { 0 };
        for (i, param_name) in params.iter().enumerate() {
            let val = lambda_fn.get_nth_param(i as u32 + param_offset).unwrap();
            let kind = param_kinds.and_then(|k| k.get(i).cloned()).unwrap_or(ValKind::Int);
            // For pointer-based types (Str, List, Map), convert i64 param to pointer
            match &kind {
                ValKind::Str | ValKind::List | ValKind::Map => {
                    let ptr_ty = self.ptr_type();
                    let ptr_val = bld!(self.builder.build_int_to_ptr(
                        val.into_int_value(), ptr_ty, &format!("{}_ptr", param_name)
                    ))?;
                    let alloca = bld!(self.builder.build_alloca(ptr_ty, param_name))?;
                    bld!(self.builder.build_store(alloca, ptr_val))?;
                    self.variables.insert(param_name.clone(), (alloca, ptr_ty.as_basic_type_enum(), kind, false));
                }
                ValKind::Float => {
                    // Float list elements are stored as i64 (bit pattern); bitcast to f64
                    let f64_ty = self.context.f64_type();
                    let f_val = bld!(self.builder.build_bit_cast(val, f64_ty, &format!("{}_f", param_name)))?;
                    let alloca = bld!(self.builder.build_alloca(f64_ty, param_name))?;
                    bld!(self.builder.build_store(alloca, f_val))?;
                    self.variables.insert(param_name.clone(), (alloca, f64_ty.as_basic_type_enum(), kind, false));
                }
                _ => {
                    let ty = val.get_type();
                    let alloca = bld!(self.builder.build_alloca(ty, param_name))?;
                    bld!(self.builder.build_store(alloca, val))?;
                    self.variables.insert(param_name.clone(), (alloca, ty, kind, false));
                }
            }
        }

        let (result, kind) = self.compile_expr_with_kind(body, lambda_fn)?;
        let return_kind = kind.clone();

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            // Coerce result to i64 if needed (e.g. bool i1 from comparisons, ptr from Str)
            let ret_val = match kind {
                ValKind::Bool => {
                    bld!(self.builder.build_int_z_extend(
                        result.into_int_value(),
                        self.context.i64_type(),
                        "bool_to_i64"
                    ))?.into()
                }
                ValKind::Str | ValKind::List | ValKind::Map if result.is_pointer_value() => {
                    bld!(self.builder.build_ptr_to_int(
                        result.into_pointer_value(),
                        self.context.i64_type(),
                        "ptr_to_i64"
                    ))?.into()
                }
                ValKind::Float if result.is_float_value() => {
                    bld!(self.builder.build_bit_cast(result, self.context.i64_type(), "f64_to_i64"))?.into()
                }
                _ => result,
            };
            bld!(self.builder.build_return(Some(&ret_val)))?;
        }

        // Restore state
        self.variables = saved_vars;
        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }

        self.last_lambda_return_kind = Some(return_kind);
        self.functions.insert(name, (lambda_fn, ValKind::Int));
        Ok(lambda_fn)
    }

    /// Build the captures struct on the stack and fill it with current variable values.
    /// Returns a pointer to the alloca'd struct.
    pub(crate) fn build_captures_struct(
        &mut self,
        lambda_name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let cap_info = self.lambda_captures.get(lambda_name).ok_or_else(|| CodeGenError {
            line: Some(self.current_line), msg: format!("no capture info for lambda '{}'", lambda_name),
        })?;
        let struct_type = cap_info.struct_type;
        let names = cap_info.names.clone();
        let types = cap_info.types.clone();

        let alloca = bld!(self.builder.build_alloca(struct_type, "captures"))?;

        for (i, cap_name) in names.iter().enumerate() {
            let (var_ptr, var_ty, _kind, _) = self.variables.get(cap_name).ok_or_else(|| CodeGenError {
                line: Some(self.current_line), msg: format!("captured variable '{}' not found in scope", cap_name),
            })?;
            let val = bld!(self.builder.build_load(*var_ty, *var_ptr, cap_name))?;
            let field_ptr = bld!(self.builder.build_struct_gep(
                struct_type, alloca, i as u32, &format!("cap_store_{}", cap_name)
            ))?;
            // If types don't exactly match (e.g. i64 vs i64), just store directly
            let _ = types[i]; // ensure we have the type
            bld!(self.builder.build_store(field_ptr, val))?;
        }

        Ok(alloca)
    }

}
