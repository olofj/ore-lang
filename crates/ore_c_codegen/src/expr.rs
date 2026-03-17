use super::*;

impl CCodeGen {
    /// Enrich a function's return kind using tracked element/value kind info.
    ///
    /// If the base return kind is `List(None)` or `Map(None)`, upgrades it to
    /// a typed variant when we have tracked element/value kind for the function.
    fn enrich_return_kind(&self, fn_name: &str, base_kind: ValKind) -> ValKind {
        if base_kind.is_list() {
            if let Some(ek) = self.fn_return_list_elem_kind.get(fn_name) {
                return ValKind::list_of(ek.clone());
            }
        } else if base_kind.is_map() {
            if let Some(vk) = self.fn_return_map_val_kind.get(fn_name) {
                return ValKind::map_of(vk.clone());
            }
        }
        base_kind
    }

    /// Compile an expression, returning (c_expr_string, ValKind).
    pub(crate) fn compile_expr(&mut self, expr: &Expr) -> Result<(String, ValKind), CCodeGenError> {
        match expr {
            Expr::IntLit(n) => {
                Ok((format!("((int64_t){}LL)", n), ValKind::Int))
            }
            Expr::FloatLit(f) => {
                // Use exact representation
                if f.is_infinite() {
                    if f.is_sign_positive() {
                        Ok(("(1.0/0.0)".to_string(), ValKind::Float))
                    } else {
                        Ok(("(-1.0/0.0)".to_string(), ValKind::Float))
                    }
                } else if f.is_nan() {
                    Ok(("(0.0/0.0)".to_string(), ValKind::Float))
                } else {
                    Ok((format!("{:.17}", f), ValKind::Float))
                }
            }
            Expr::BoolLit(b) => {
                Ok((format!("((int8_t){})", if *b { 1 } else { 0 }), ValKind::Bool))
            }
            Expr::StringLit(s) => {
                let c_expr = self.compile_string_literal(s);
                Ok((c_expr, ValKind::Str))
            }
            Expr::StringInterp(parts) => {
                self.compile_string_interp(parts)
            }
            Expr::Ident(name) => {
                // Check if it's a zero-arg enum variant
                if !self.variables.contains_key(name) && self.variant_to_enum.contains_key(name) {
                    return self.compile_variant_construct(name, &[]);
                }
                // Check if it's a function reference
                if !self.variables.contains_key(name)
                    && self.functions.contains_key(name) {
                        return Ok((format!("(void*)&{}", Self::mangle_fn_name(name)), ValKind::Int));
                    }
                let kind = self.get_var_kind(name);
                // Use the mangled C name from VarInfo if available
                let c_name = self.variables.get(name)
                    .map(|v| v.c_name.clone())
                    .unwrap_or_else(|| name.clone());
                Ok((c_name, kind))
            }
            Expr::BinOp { op, left, right } => {
                if *op == BinOp::Pipe {
                    return self.compile_pipeline(left, right);
                }
                if *op == BinOp::And || *op == BinOp::Or {
                    return self.compile_short_circuit(*op, left, right);
                }
                let (l_expr, lk) = self.compile_expr(left)?;
                let (r_expr, rk) = self.compile_expr(right)?;

                // List concatenation
                if lk.is_list() && *op == BinOp::Add {
                    let result = format!("ore_list_concat({}, {})", l_expr, r_expr);
                    let elem = match (&rk, &lk) {
                        (ValKind::List(Some(ek)), _) | (_, ValKind::List(Some(ek))) => Some(ek.clone()),
                        _ => None,
                    };
                    return Ok((result, ValKind::List(elem)));
                }

                // String repetition
                if lk == ValKind::Str && *op == BinOp::Mul {
                    return Ok((format!("ore_str_repeat({}, {})", l_expr, r_expr), ValKind::Str));
                }

                // String operations
                if lk == ValKind::Str && rk == ValKind::Str {
                    return self.compile_str_binop(*op, &l_expr, &r_expr);
                }

                // Float promotion
                let (l_c, r_c, is_float) = if lk == ValKind::Float || rk == ValKind::Float {
                    let l = if lk == ValKind::Int { format!("(double)({})", l_expr) } else { l_expr.clone() };
                    let r = if rk == ValKind::Int { format!("(double)({})", r_expr) } else { r_expr.clone() };
                    (l, r, true)
                } else {
                    (l_expr.clone(), r_expr.clone(), false)
                };

                let result_kind = match op {
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt
                    | BinOp::LtEq | BinOp::GtEq | BinOp::And | BinOp::Or => ValKind::Bool,
                    _ => if is_float { ValKind::Float } else { lk.clone() },
                };

                let c_op = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => {
                        if !is_float {
                            // Pre-compute both operands as statements BEFORE any containing expression
                            let l_tmp = self.tmp();
                            let r_tmp = self.tmp();
                            self.emit(&format!("int64_t {} = {};", l_tmp, l_c));
                            self.emit(&format!("int64_t {} = {};", r_tmp, r_c));
                            self.emit(&format!("if ({} == 0) ore_div_by_zero();", r_tmp));
                            return Ok((format!("({} / {})", l_tmp, r_tmp), result_kind));
                        }
                        "/"
                    }
                    BinOp::Mod => {
                        if !is_float {
                            let l_tmp = self.tmp();
                            let r_tmp = self.tmp();
                            self.emit(&format!("int64_t {} = {};", l_tmp, l_c));
                            self.emit(&format!("int64_t {} = {};", r_tmp, r_c));
                            self.emit(&format!("if ({} == 0) ore_div_by_zero();", r_tmp));
                            return Ok((format!("({} % {})", l_tmp, r_tmp), result_kind));
                        }
                        // For floats, use fmod
                        return Ok((format!("fmod({}, {})", l_c, r_c), ValKind::Float));
                    }
                    BinOp::Eq => "==",
                    BinOp::NotEq => "!=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::LtEq => "<=",
                    BinOp::GtEq => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                    BinOp::Pipe => unreachable!(),
                };

                // Bool comparisons: normalize bool to int8_t result
                if result_kind == ValKind::Bool && (lk == ValKind::Bool || rk == ValKind::Bool) {
                    match op {
                        BinOp::And => return Ok((format!("(({}) && ({}))", l_c, r_c), ValKind::Bool)),
                        BinOp::Or => return Ok((format!("(({}) || ({}))", l_c, r_c), ValKind::Bool)),
                        _ => {}
                    }
                }

                Ok((format!("({} {} {})", l_c, c_op, r_c), result_kind))
            }
            Expr::UnaryMinus(inner) => {
                let (val, kind) = self.compile_expr(inner)?;
                Ok((format!("(-({}))", val), kind))
            }
            Expr::UnaryNot(inner) => {
                let (val, _kind) = self.compile_expr(inner)?;
                Ok((format!("(!({}))", val), ValKind::Bool))
            }
            Expr::Print(inner) => {
                let (val, kind) = self.compile_expr(inner)?;
                self.compile_print_expr(&val, &kind, inner)?;
                Ok(("0".to_string(), ValKind::Void))
            }
            Expr::Sleep(inner) => {
                let (val, _) = self.compile_expr(inner)?;
                self.emit(&format!("ore_sleep({});", val));
                Ok(("0".to_string(), ValKind::Void))
            }
            Expr::Assert { cond, message } => {
                let (cond_val, _) = self.compile_expr(cond)?;
                let msg_str = message.as_deref().unwrap_or("assertion failed");
                let msg_c = self.compile_string_literal(msg_str);
                self.emit(&format!("ore_assert({}, {}, {});", cond_val, msg_c, self.current_line));
                Ok(("0".to_string(), ValKind::Void))
            }
            Expr::AssertEq { left, right, message } => {
                self.compile_assert_cmp(left, right, message.as_deref(), "eq")
            }
            Expr::AssertNe { left, right, message } => {
                self.compile_assert_cmp(left, right, message.as_deref(), "ne")
            }
            Expr::Call { func: callee, args } => {
                let name = match callee.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(self.err("only named function calls supported")),
                };
                // Try builtins first
                if let Some(result) = self.compile_builtin_call(&name, args)? {
                    return Ok(result);
                }
                // Variant construction
                if self.variant_to_enum.contains_key(&name) {
                    return self.compile_variant_construct(&name, &[]);
                }
                // Regular function call
                if let Some(fn_info) = self.functions.get(&name).cloned() {
                    let c_fn_name = Self::mangle_fn_name(&name);
                    let mut arg_strs = Vec::new();
                    for (i, arg) in args.iter().enumerate() {
                        let (a, _ak) = self.compile_expr(arg)?;
                        // Cast function pointers to int64_t when parameter expects Int
                        let param_kind = fn_info.param_kinds.get(i);
                        if param_kind == Some(&ValKind::Int) && a.starts_with("(void*)&") {
                            arg_strs.push(format!("(int64_t)(intptr_t){}", a));
                        } else {
                            arg_strs.push(a);
                        }
                    }
                    // Fill default args
                    if let Some(defaults) = self.fn_defaults.get(&name).cloned() {
                        let num_args = arg_strs.len();
                        for default_expr in defaults.iter().skip(num_args).flatten() {
                            let (a, _) = self.compile_expr(default_expr)?;
                            arg_strs.push(a);
                        }
                    }
                    let call_str = format!("{}({})", c_fn_name, arg_strs.join(", "));
                    let ret_kind = self.enrich_return_kind(&name, fn_info.ret_kind.clone());
                    return Ok((call_str, ret_kind));
                }
                // Generic function instantiation
                if let Some(generic_fn) = self.generic_fns.get(&name).cloned() {
                    let mut arg_kinds = Vec::new();
                    let mut arg_strs = Vec::new();
                    for arg in args {
                        let (a, ak) = self.compile_expr(arg)?;
                        arg_strs.push(a);
                        arg_kinds.push(ak);
                    }
                    return self.instantiate_generic(&name, &generic_fn, &arg_kinds, &arg_strs);
                }
                // Variable holding a function pointer
                if self.variables.contains_key(&name) {
                    let mut arg_strs = Vec::new();
                    for arg in args {
                        let (a, _) = self.compile_expr(arg)?;
                        arg_strs.push(a);
                    }
                    let call_str = format!("((int64_t(*)({})){})({})",
                        vec!["int64_t"; args.len()].join(", "),
                        name,
                        arg_strs.join(", "));
                    return Ok((call_str, ValKind::Int));
                }
                Err(self.err(format!("undefined function '{}'", name)))
            }
            Expr::IfElse { cond, then_block, else_block } => {
                self.compile_if_else(cond, then_block, else_block.as_ref())
            }
            Expr::ColonMatch { cond, then_expr, else_expr } => {
                let (cond_val, _) = self.compile_expr(cond)?;
                let (then_val, then_kind) = self.compile_expr(then_expr)?;
                let result_tmp = self.tmp();
                let c_type = self.kind_to_c_type_str(&then_kind);
                self.emit(&format!("{} {};", c_type, result_tmp));
                self.emit(&format!("if ({}) {{", cond_val));
                self.indent += 1;
                self.emit(&format!("{} = {};", result_tmp, then_val));
                self.indent -= 1;
                if let Some(e) = else_expr {
                    let (else_val, _) = self.compile_expr(e)?;
                    self.emit("} else {");
                    self.indent += 1;
                    self.emit(&format!("{} = {};", result_tmp, else_val));
                    self.indent -= 1;
                } else {
                    self.emit("} else {");
                    self.indent += 1;
                    self.emit(&format!("{} = 0;", result_tmp));
                    self.indent -= 1;
                }
                self.emit("}");
                Ok((result_tmp, then_kind))
            }
            Expr::RecordConstruct { type_name, fields } => {
                if self.variant_to_enum.contains_key(type_name) {
                    return self.compile_variant_construct(type_name, fields);
                }
                self.compile_record_construct(type_name, fields)
            }
            Expr::Match { subject, arms } => {
                self.compile_match(subject, arms)
            }
            Expr::FieldAccess { object, field } => {
                self.compile_field_access(object, field)
            }
            Expr::MethodCall { object, method, args } => {
                self.compile_method_call(object, method, args)
            }
            Expr::ListLit(elements) => {
                self.compile_list_lit(elements)
            }
            Expr::ListComp { expr: body, var, iterable, cond } => {
                self.compile_list_comp(body, var, iterable, cond.as_deref())
            }
            Expr::MapLit(entries) => {
                self.compile_map_lit(entries)
            }
            Expr::Index { object, index } => {
                self.compile_index(object, index)
            }
            Expr::OptionNone => {
                Ok(("((OreTaggedUnion){0, 0, 0})".to_string(), ValKind::Option))
            }
            Expr::OptionSome(inner) => {
                let (val, kind) = self.compile_expr(inner)?;
                let kind_tag = Self::valkind_to_tag(&kind);
                let i64_val = self.value_to_i64_expr(&val, &kind);
                Ok((format!("((OreTaggedUnion){{1, {}, {}}})", kind_tag, i64_val), ValKind::Option))
            }
            Expr::ResultOk(inner) => {
                let (val, kind) = self.compile_expr(inner)?;
                let kind_tag = Self::valkind_to_tag(&kind);
                let i64_val = self.value_to_i64_expr(&val, &kind);
                Ok((format!("((OreTaggedUnion){{0, {}, {}}})", kind_tag, i64_val), ValKind::Result))
            }
            Expr::ResultErr(inner) => {
                let (val, kind) = self.compile_expr(inner)?;
                let kind_tag = Self::valkind_to_tag(&kind);
                let i64_val = self.value_to_i64_expr(&val, &kind);
                Ok((format!("((OreTaggedUnion){{1, {}, {}}})", kind_tag, i64_val), ValKind::Result))
            }
            Expr::Try(inner) => {
                let (val, kind) = self.compile_expr(inner)?;
                let tmp = self.tmp();
                self.emit(&format!("OreTaggedUnion {} = {};", tmp, val));
                if kind == ValKind::Result {
                    // If Err, propagate
                    self.emit(&format!("if ({}.tag == 1) return {};", tmp, tmp));
                    Ok((format!("{}.val", tmp), ValKind::Int))
                } else {
                    // Option: if None, propagate None
                    self.emit(&format!("if ({}.tag == 0) return {};", tmp, tmp));
                    Ok((format!("{}.val", tmp), ValKind::Int))
                }
            }
            Expr::Lambda { params, body } => {
                let (expr, kind) = self.compile_lambda(params, body, None)?;
                // If it's a closure expression, extract just the function pointer
                // (captures are only usable through compile_lambda_arg_with_kinds for list methods)
                let (fn_ptr, _env_ptr) = Self::parse_closure_expr(&expr);
                Ok((fn_ptr, kind))
            }
            Expr::BlockExpr(block) => {
                self.compile_block_stmts(block).map(|(v, k)| {
                    (v.unwrap_or_else(|| "0".to_string()), k)
                })
            }
            Expr::OptionalChain { object, field: _ } => {
                let (val, _) = self.compile_expr(object)?;
                let tmp = self.tmp();
                let result = self.tmp();
                self.emit(&format!("OreTaggedUnion {} = {};", tmp, val));
                self.emit(&format!("OreTaggedUnion {};", result));
                self.emit(&format!("if ({}.tag == 1) {{", tmp));
                self.indent += 1;
                self.emit(&format!("{} = (OreTaggedUnion){{1, {}.kind, {}.val}};", result, tmp, tmp));
                self.indent -= 1;
                self.emit("} else {");
                self.indent += 1;
                self.emit(&format!("{} = (OreTaggedUnion){{0, 0, 0}};", result));
                self.indent -= 1;
                self.emit("}");
                Ok((result, ValKind::Option))
            }
            Expr::OptionalMethodCall { object, method: _, args: _ } => {
                // Simplified: just pass through
                let (val, _) = self.compile_expr(object)?;
                Ok((val, ValKind::Option))
            }
            Expr::Break => {
                if let Some(label) = self.break_labels.last().cloned() {
                    self.emit(&format!("goto {};", label));
                } else {
                    return Err(self.err("break outside of loop"));
                }
                Ok(("0".to_string(), ValKind::Void))
            }
        }
    }

    pub(crate) fn compile_short_circuit(&mut self, op: BinOp, left: &Expr, right: &Expr) -> Result<(String, ValKind), CCodeGenError> {
        let result = self.tmp();
        let (lhs, _) = self.compile_expr(left)?;
        self.emit(&format!("int64_t {} = {};", result, if op == BinOp::And { 0 } else { 1 }));
        if op == BinOp::And {
            self.emit(&format!("if ({}) {{", lhs));
        } else {
            self.emit(&format!("if (!{}) {{", lhs));
        }
        self.indent += 1;
        let (rhs, _) = self.compile_expr(right)?;
        self.emit(&format!("{} = {};", result, rhs));
        self.indent -= 1;
        self.emit("}");
        Ok((result, ValKind::Bool))
    }

    pub(crate) fn compile_str_binop(&mut self, op: BinOp, l: &str, r: &str) -> Result<(String, ValKind), CCodeGenError> {
        match op {
            BinOp::Add => Ok((format!("ore_str_concat({}, {})", l, r), ValKind::Str)),
            BinOp::Eq => Ok((format!("(ore_str_eq({}, {}) != 0)", l, r), ValKind::Bool)),
            BinOp::NotEq => Ok((format!("(ore_str_eq({}, {}) == 0)", l, r), ValKind::Bool)),
            BinOp::Lt => Ok((format!("(ore_str_cmp({}, {}) < 0)", l, r), ValKind::Bool)),
            BinOp::Gt => Ok((format!("(ore_str_cmp({}, {}) > 0)", l, r), ValKind::Bool)),
            BinOp::LtEq => Ok((format!("(ore_str_cmp({}, {}) <= 0)", l, r), ValKind::Bool)),
            BinOp::GtEq => Ok((format!("(ore_str_cmp({}, {}) >= 0)", l, r), ValKind::Bool)),
            _ => Err(self.err(format!("unsupported string op {:?}", op))),
        }
    }

    pub(crate) fn compile_if_else(&mut self, cond: &Expr, then_block: &Block, else_block: std::option::Option<&Block>) -> Result<(String, ValKind), CCodeGenError> {
        let (cond_val, _) = self.compile_expr(cond)?;
        let result_tmp = self.tmp();

        // Infer the result kind from the then block to choose the right C type
        let inferred_kind = if let Some(last) = then_block.stmts.last() {
            if let Stmt::Expr(e) = &last.stmt {
                self.infer_expr_kind(e)
            } else if let Stmt::Return(_) = &last.stmt {
                ValKind::Void
            } else {
                ValKind::Int
            }
        } else {
            ValKind::Int
        };

        // Use the actual C type for the result variable to avoid type mismatches
        let use_native_type = !matches!(inferred_kind, ValKind::Int | ValKind::Void);

        if use_native_type {
            let c_type = self.kind_to_c_type_str(&inferred_kind);
            if matches!(inferred_kind, ValKind::Option | ValKind::Result | ValKind::Record(_) | ValKind::Enum(_)) {
                self.emit(&format!("{} {} = {{}};", c_type, result_tmp));
            } else {
                self.emit(&format!("{} {} = 0;", c_type, result_tmp));
            }
        } else {
            self.emit(&format!("int64_t {} = 0;", result_tmp));
        }

        self.emit(&format!("if ({}) {{", cond_val));
        self.indent += 1;
        let saved_vars = self.variables.clone();
        let (then_val, then_kind) = self.compile_block_stmts(then_block)?;
        if let Some(ref tv) = then_val {
            if use_native_type {
                self.emit(&format!("{} = {};", result_tmp, tv));
            } else {
                self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(tv, &then_kind)));
            }
        }
        self.variables = saved_vars;
        self.indent -= 1;

        if let Some(eb) = else_block {
            self.emit("} else {");
            self.indent += 1;
            let saved_vars = self.variables.clone();
            let (else_val, else_kind) = self.compile_block_stmts(eb)?;
            if let Some(ref ev) = else_val {
                if use_native_type {
                    self.emit(&format!("{} = {};", result_tmp, ev));
                } else {
                    self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(ev, &else_kind)));
                }
            }
            self.variables = saved_vars;
            self.indent -= 1;
        }
        self.emit("}");

        if use_native_type {
            Ok((result_tmp, inferred_kind))
        } else {
            Ok((result_tmp, ValKind::Int))
        }
    }

    pub(crate) fn compile_pipeline(&mut self, arg: &Expr, func_expr: &Expr) -> Result<(String, ValKind), CCodeGenError> {
        match func_expr {
            Expr::Ident(name) => {
                self.compile_pipe_to_named(arg, name, &[])
            }
            Expr::Call { func, args } => {
                let name = match func.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(self.err("pipeline target must be a function")),
                };
                self.compile_pipe_to_named(arg, &name, args)
            }
            Expr::Lambda { params, body } => {
                let (arg_val, _) = self.compile_expr(arg)?;
                let (raw, ret_kind) = self.compile_lambda(params, body, None)?;
                let (fn_ptr, env_ptr) = Self::parse_closure_expr(&raw);
                let fn_name = fn_ptr.trim_start_matches("(void*)&");
                let call = if env_ptr == "NULL" {
                    format!("{}({})", fn_name, arg_val)
                } else {
                    format!("{}({}, {})", fn_name, env_ptr, arg_val)
                };
                Ok((call, ret_kind))
            }
            _ => Err(self.err("unsupported pipeline target")),
        }
    }

    fn compile_pipe_to_named(&mut self, arg: &Expr, name: &str, extra_args: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        if self.functions.contains_key(name) {
            let c_fn_name = Self::mangle_fn_name(name);
            let (arg_val, _) = self.compile_expr(arg)?;
            let fn_info = self.functions[name].clone();
            let mut arg_strs = vec![arg_val];
            for a in extra_args {
                let (v, _) = self.compile_expr(a)?;
                arg_strs.push(v);
            }
            // Fill defaults
            if let Some(defaults) = self.fn_defaults.get(name).cloned() {
                let num_args = arg_strs.len();
                for default_expr in defaults.iter().skip(num_args).flatten() {
                    let (a, _) = self.compile_expr(default_expr)?;
                    arg_strs.push(a);
                }
            }
            let call = format!("{}({})", c_fn_name, arg_strs.join(", "));
            let ret_kind = self.enrich_return_kind(name, fn_info.ret_kind.clone());
            Ok((call, ret_kind))
        } else if let Some(generic_fn) = self.generic_fns.get(name).cloned() {
            // Generic function instantiation for pipeline
            let (arg_val, arg_kind) = self.compile_expr(arg)?;
            let mut arg_strs = vec![arg_val];
            let mut arg_kinds = vec![arg_kind];
            for a in extra_args {
                let (v, k) = self.compile_expr(a)?;
                arg_strs.push(v);
                arg_kinds.push(k);
            }
            self.instantiate_generic(name, &generic_fn, &arg_kinds, &arg_strs)
        } else {
            // Try as method call
            let method_call = Expr::MethodCall {
                object: Box::new(arg.clone()),
                method: name.to_string(),
                args: extra_args.to_vec(),
            };
            self.compile_expr(&method_call)
        }
    }

    /// Instantiate a generic function with concrete types inferred from arguments.
    /// Returns (call_expression, return_kind).
    fn instantiate_generic(
        &mut self,
        name: &str,
        generic_fn: &FnDef,
        arg_kinds: &[ValKind],
        arg_strs: &[String],
    ) -> Result<(String, ValKind), CCodeGenError> {
        // Build type param -> kind mapping from arguments
        let type_param_names: Vec<String> = generic_fn.type_params.iter().map(|tp| tp.name.clone()).collect();
        let mut type_map: HashMap<String, ValKind> = HashMap::new();
        for (i, p) in generic_fn.params.iter().enumerate() {
            if let Some(ak) = arg_kinds.get(i) {
                if let TypeExpr::Named(ref tn) = p.ty {
                    if type_param_names.contains(tn) {
                        type_map.insert(tn.clone(), ak.clone());
                    }
                }
            }
        }
        // Generate mangled name based on concrete types
        let type_suffix: Vec<String> = type_param_names.iter()
            .map(|tp| type_map.get(tp).map(Self::kind_to_suffix).unwrap_or_else(|| "Int".to_string()))
            .collect();
        let mono_name = format!("{}__{}", name, type_suffix.join("_"));
        // Check if already instantiated
        if !self.functions.contains_key(&mono_name) {
            // Recursive type substitution
            fn subst(te: &TypeExpr, tm: &HashMap<String, ValKind>, cg: &CCodeGen) -> TypeExpr {
                match te {
                    TypeExpr::Named(ref tn) => {
                        if let Some(kind) = tm.get(tn) {
                            TypeExpr::Named(cg.kind_to_type_name(kind).to_string())
                        } else {
                            te.clone()
                        }
                    }
                    TypeExpr::Generic(name, args) => {
                        let new_args: Vec<TypeExpr> = args.iter().map(|a| subst(a, tm, cg)).collect();
                        TypeExpr::Generic(name.clone(), new_args)
                    }
                    TypeExpr::Fn { params, ret } => {
                        TypeExpr::Fn {
                            params: params.iter().map(|p| subst(p, tm, cg)).collect(),
                            ret: Box::new(subst(ret, tm, cg)),
                        }
                    }
                }
            }
            let mono_params: Vec<Param> = generic_fn.params.iter().map(|p| {
                let ty = subst(&p.ty, &type_map, self);
                Param { name: p.name.clone(), ty, default: p.default.clone() }
            }).collect();
            let mono_ret = generic_fn.ret_type.as_ref().map(|rt| subst(rt, &type_map, self));
            let mono_fn = FnDef {
                name: mono_name.clone(),
                type_params: vec![],
                params: mono_params,
                ret_type: mono_ret,
                body: generic_fn.body.clone(),
            };
            // Save and restore state around function compilation
            let saved_vars = self.variables.clone();
            let saved_dynamic = self.dynamic_kind_tags.clone();
            let saved_lines = std::mem::take(&mut self.lines);
            let saved_indent = self.indent;
            self.declare_function(&mono_fn)?;
            self.compile_function(&mono_fn)?;
            self.lambda_bodies.extend(std::mem::take(&mut self.lines));
            self.lines = saved_lines;
            self.indent = saved_indent;
            self.variables = saved_vars;
            self.dynamic_kind_tags = saved_dynamic;
        }
        let fn_info = self.functions.get(&mono_name).cloned()
            .ok_or_else(|| self.err(format!("failed to instantiate generic '{}'", name)))?;
        let call_str = format!("{}({})", mono_name, arg_strs.join(", "));
        Ok((call_str, fn_info.ret_kind))
    }

    pub(crate) fn compile_print_expr(&mut self, val: &str, kind: &ValKind, _inner: &Expr) -> Result<(), CCodeGenError> {
        // Check for dynamic kind tag (from Result/Option match bindings)
        if let Expr::Ident(name) = _inner {
            if self.dynamic_kind_tags.contains(name) {
                let kind_var = format!("{}_kind", name);
                self.emit(&format!("{{ void* __dstr = ore_dynamic_to_str({}, {}); ore_str_print(__dstr); ore_str_release(__dstr); }}", val, kind_var));
                return Ok(());
            }
        }
        match kind {
            ValKind::Str => { self.emit(&format!("ore_str_print({});", val)); }
            ValKind::Int => { self.emit(&format!("ore_print_int({});", val)); }
            ValKind::Float => { self.emit(&format!("ore_print_float({});", val)); }
            ValKind::Bool => { self.emit(&format!("ore_print_bool({});", val)); }
            ValKind::List(Some(ref ek)) if **ek == ValKind::Str => {
                self.emit(&format!("ore_list_print_str({});", val));
            }
            ValKind::List(Some(ref ek)) if **ek == ValKind::Float => {
                self.emit(&format!("ore_list_print_float({});", val));
            }
            ValKind::List(Some(ref ek)) if **ek == ValKind::Bool => {
                self.emit(&format!("ore_list_print_bool({});", val));
            }
            ValKind::List(_) => { self.emit(&format!("ore_list_print({});", val)); }
            ValKind::Map(Some(ref vk)) if **vk == ValKind::Str => {
                self.emit(&format!("ore_map_print_str({});", val));
            }
            ValKind::Map(_) => { self.emit(&format!("ore_map_print({});", val)); }
            ValKind::Record(ref name) => {
                let name = name.clone();
                let s = self.record_to_str_expr(val, &name);
                self.emit(&format!("ore_str_print({});", s));
            }
            ValKind::Enum(ref name) => {
                let name = name.clone();
                let s = self.enum_to_str_expr(val, &name);
                self.emit(&format!("ore_str_print({});", s));
            }
            _ => { self.emit(&format!("ore_print_int({});", val)); }
        }
        Ok(())
    }

    fn compile_assert_cmp(&mut self, left: &Expr, right: &Expr, message: std::option::Option<&str>, op: &str) -> Result<(String, ValKind), CCodeGenError> {
        let (left_val, left_kind) = self.compile_expr(left)?;
        let (right_val, right_kind) = self.compile_expr(right)?;
        let default_msg = format!("assert_{} failed", op);
        let msg_str = message.unwrap_or(&default_msg);
        let msg_c = self.compile_string_literal(msg_str);
        let fn_name = match (&left_kind, &right_kind) {
            (ValKind::Float, _) | (_, ValKind::Float) if op == "eq" => "ore_assert_eq_float".to_string(),
            (ValKind::Str, _) | (_, ValKind::Str) => format!("ore_assert_{}_str", op),
            _ => format!("ore_assert_{}_int", op),
        };
        self.emit(&format!("{}({}, {}, {}, {});", fn_name, left_val, right_val, msg_c, self.current_line));
        Ok(("0".to_string(), ValKind::Void))
    }

    pub(crate) fn compile_string_interp(&mut self, parts: &[StringPart]) -> Result<(String, ValKind), CCodeGenError> {
        let mut result: std::option::Option<String> = None;
        for part in parts {
            let part_expr = match part {
                StringPart::Lit(s) => self.compile_string_literal(s),
                StringPart::Expr(expr) => {
                    let (val, kind) = self.compile_expr(expr)?;
                    // Check for dynamic kind tag (from Result/Option match bindings)
                    if let Expr::Ident(name) = expr {
                        if self.dynamic_kind_tags.contains(name) {
                            let kind_var = format!("{}_kind", name);
                            format!("ore_dynamic_to_str({}, {})", val, kind_var)
                        } else {
                            self.value_to_str_expr(&val, &kind)
                        }
                    } else {
                        self.value_to_str_expr(&val, &kind)
                    }
                }
            };
            result = Some(match result {
                None => part_expr,
                Some(acc) => {
                    let tmp = self.tmp();
                    self.emit(&format!("void* {} = ore_str_concat({}, {});", tmp, acc, part_expr));
                    tmp
                }
            });
        }
        Ok((result.unwrap_or_else(|| "ore_str_new(\"\", 0)".to_string()), ValKind::Str))
    }

    pub(crate) fn value_to_str_expr(&mut self, val: &str, kind: &ValKind) -> String {
        match kind {
            ValKind::Str => val.to_string(),
            ValKind::Int => format!("ore_int_to_str({})", val),
            ValKind::Float => format!("ore_float_to_str({})", val),
            ValKind::Bool => format!("ore_bool_to_str({})", val),
            ValKind::Record(ref name) => {
                let name = name.clone();
                self.record_to_str_expr(val, &name)
            }
            ValKind::Enum(ref name) => {
                let name = name.clone();
                self.enum_to_str_expr(val, &name)
            }
            _ => format!("ore_int_to_str({})", val),
        }
    }

    /// Emit the concatenation loop for "label: value" field pairs.
    /// `result` is the tmp var accumulating the string, `obj_prefix` is the
    /// expression prefix for accessing fields (e.g. "val" or "payload_tmp"),
    /// and `indent` is the C indentation prefix for emitted lines.
    fn emit_fields_to_str(
        &mut self,
        result: &str,
        obj_prefix: &str,
        field_names: &[String],
        field_kinds: &[ValKind],
        indent: &str,
    ) {
        for (i, (fname, fkind)) in field_names.iter().zip(field_kinds.iter()).enumerate() {
            if i > 0 {
                let comma = self.compile_string_literal(", ");
                let t = self.tmp();
                self.emit(&format!("{}void* {} = ore_str_concat({}, {});", indent, t, result, comma));
                self.emit(&format!("{}{} = {};", indent, result, t));
            }
            let label = format!("{}: ", fname);
            let label_str = self.compile_string_literal(&label);
            let t = self.tmp();
            self.emit(&format!("{}void* {} = ore_str_concat({}, {});", indent, t, result, label_str));
            self.emit(&format!("{}{} = {};", indent, result, t));
            let field_expr = format!("{}.{}", obj_prefix, fname);
            let fval_str = self.value_to_str_expr(&field_expr, fkind);
            let t = self.tmp();
            self.emit(&format!("{}void* {} = ore_str_concat({}, {});", indent, t, result, fval_str));
            self.emit(&format!("{}{} = {};", indent, result, t));
        }
    }

    /// Generate code that converts a record value to a display string.
    /// Returns a C expression (tmp variable) holding the result string.
    fn record_to_str_expr(&mut self, val: &str, type_name: &str) -> String {
        let info = self.records.get(type_name).cloned();
        let result = self.tmp();
        if let Some(info) = info {
            let header = format!("{}(", type_name);
            let header_str = self.compile_string_literal(&header);
            self.emit(&format!("void* {} = {};", result, header_str));
            self.emit_fields_to_str(&result, val, &info.field_names, &info.field_kinds, "");
            let close = self.compile_string_literal(")");
            let t = self.tmp();
            self.emit(&format!("void* {} = ore_str_concat({}, {});", t, result, close));
            self.emit(&format!("{} = {};", result, t));
        } else {
            let fallback = self.compile_string_literal(&format!("{}(...)", type_name));
            self.emit(&format!("void* {} = {};", result, fallback));
        }
        result
    }

    /// Generate code that converts an enum value to a display string.
    /// Returns a C expression (tmp variable) holding the result string.
    fn enum_to_str_expr(&mut self, val: &str, type_name: &str) -> String {
        let info = self.enums.get(type_name).cloned();
        let result = self.tmp();
        self.emit(&format!("void* {};", result));
        if let Some(info) = info {
            self.emit(&format!("switch ({}.tag) {{", val));
            for v in &info.variants {
                self.emit(&format!("case {}: {{", v.tag));
                if v.field_names.is_empty() {
                    let name_str = self.compile_string_literal(&v.name);
                    self.emit(&format!("    {} = {};", result, name_str));
                } else {
                    let payload_type = format!("struct ore_payload_{}_{}", Self::mangle_name(type_name), v.name);
                    let payload_tmp = self.tmp();
                    self.emit(&format!("    {} {}; memcpy(&{}, {}.data, sizeof({}));",
                        payload_type, payload_tmp, payload_tmp, val, payload_type));
                    let header = format!("{}(", v.name);
                    let header_str = self.compile_string_literal(&header);
                    self.emit(&format!("    {} = {};", result, header_str));
                    self.emit_fields_to_str(&result, &payload_tmp, &v.field_names, &v.field_kinds, "    ");
                    let close = self.compile_string_literal(")");
                    let t = self.tmp();
                    self.emit(&format!("    void* {} = ore_str_concat({}, {});", t, result, close));
                    self.emit(&format!("    {} = {};", result, t));
                }
                self.emit("    break;");
                self.emit("}");
            }
            let fallback = self.compile_string_literal(&format!("{}(?)", type_name));
            self.emit(&format!("default: {} = {}; break;", result, fallback));
            self.emit("}");
        } else {
            let fallback = self.compile_string_literal(&format!("{}(?)", type_name));
            self.emit(&format!("{} = {};", result, fallback));
        }
        result
    }
}
