use super::*;

impl CCodeGen {
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
                if !self.variables.contains_key(name) {
                    if self.functions.contains_key(name) {
                        return Ok((format!("(void*)&{}", name), ValKind::Int));
                    }
                }
                let kind = self.get_var_kind(name);
                Ok((name.clone(), kind))
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
                            // Need div-by-zero check for integers
                            let tmp = self.tmp();
                            self.emit(&format!("int64_t {} = {};", tmp, r_c));
                            self.emit(&format!("if ({} == 0) ore_div_by_zero();", tmp));
                            return Ok((format!("({} / {})", l_c, tmp), result_kind));
                        }
                        "/"
                    }
                    BinOp::Mod => {
                        if !is_float {
                            let tmp = self.tmp();
                            self.emit(&format!("int64_t {} = {};", tmp, r_c));
                            self.emit(&format!("if ({} == 0) ore_div_by_zero();", tmp));
                            return Ok((format!("({} % {})", l_c, tmp), result_kind));
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
                    let mut arg_strs = Vec::new();
                    for arg in args {
                        let (a, _) = self.compile_expr(arg)?;
                        arg_strs.push(a);
                    }
                    // Fill default args
                    if let Some(defaults) = self.fn_defaults.get(&name).cloned() {
                        let num_args = arg_strs.len();
                        for default_expr in defaults.iter().skip(num_args).flatten() {
                            let (a, _) = self.compile_expr(default_expr)?;
                            arg_strs.push(a);
                        }
                    }
                    let call_str = format!("{}({})", name, arg_strs.join(", "));
                    let ret_kind = if fn_info.ret_kind.is_list() {
                        if let Some(ek) = self.fn_return_list_elem_kind.get(&name) {
                            ValKind::list_of(ek.clone())
                        } else {
                            fn_info.ret_kind.clone()
                        }
                    } else if fn_info.ret_kind.is_map() {
                        if let Some(vk) = self.fn_return_map_val_kind.get(&name) {
                            ValKind::map_of(vk.clone())
                        } else {
                            fn_info.ret_kind.clone()
                        }
                    } else {
                        fn_info.ret_kind.clone()
                    };
                    return Ok((call_str, ret_kind));
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
                self.compile_lambda(params, body, None)
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

        // Use int64_t as a common type for the result
        self.emit(&format!("int64_t {} = 0;", result_tmp));
        self.emit(&format!("if ({}) {{", cond_val));
        self.indent += 1;
        let (then_val, then_kind) = self.compile_block_stmts(then_block)?;
        if let Some(ref tv) = then_val {
            self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(tv, &then_kind)));
        }
        self.indent -= 1;

        if let Some(eb) = else_block {
            self.emit("} else {");
            self.indent += 1;
            let (else_val, else_kind) = self.compile_block_stmts(eb)?;
            if let Some(ref ev) = else_val {
                self.emit(&format!("{} = {};", result_tmp, self.value_to_i64_expr(ev, &else_kind)));
            }
            self.indent -= 1;
        }
        self.emit("}");

        Ok((result_tmp, ValKind::Int))
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
            _ => Err(self.err("unsupported pipeline target")),
        }
    }

    fn compile_pipe_to_named(&mut self, arg: &Expr, name: &str, extra_args: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        if self.functions.contains_key(name) {
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
            let call = format!("{}({})", name, arg_strs.join(", "));
            Ok((call, fn_info.ret_kind.clone()))
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

    pub(crate) fn compile_print_expr(&mut self, val: &str, kind: &ValKind, _inner: &Expr) -> Result<(), CCodeGenError> {
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
                    self.value_to_str_expr(&val, &kind)
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

    pub(crate) fn value_to_str_expr(&self, val: &str, kind: &ValKind) -> String {
        match kind {
            ValKind::Str => val.to_string(),
            ValKind::Int => format!("ore_int_to_str({})", val),
            ValKind::Float => format!("ore_float_to_str({})", val),
            ValKind::Bool => format!("ore_bool_to_str({})", val),
            _ => format!("ore_int_to_str({})", val),
        }
    }
}
