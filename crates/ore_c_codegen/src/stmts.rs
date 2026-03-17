use super::*;

impl CCodeGen {
    /// Compile a statement. Returns (Option<last_expr_str>, kind).
    pub(crate) fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(std::option::Option<String>, ValKind), CCodeGenError> {
        match stmt {
            Stmt::Let { name, mutable, value } => {
                let (val, kind) = self.compile_expr(value)?;
                let c_type = self.kind_to_c_type_str(&kind);
                let c_name = Self::mangle_var_name(name);
                self.emit(&format!("{} {} = {};", c_type, c_name, val));
                self.variables.insert(name.clone(), VarInfo {
                    c_name: c_name.clone(),
                    kind: kind.clone(),
                    is_mutable: *mutable,
                });
                self.track_variable_kinds(name, &kind);
                Ok((None, ValKind::Void))
            }
            Stmt::LetDestructure { names, value } => {
                let (val, vk) = self.compile_expr(value)?;
                let elem_kind = match &vk {
                    ValKind::List(Some(ek)) => ek.as_ref().clone(),
                    _ => ValKind::Int,
                };
                for (i, name) in names.iter().enumerate() {
                    let raw = format!("ore_list_get({}, {})", val, i);
                    let typed = self.coerce_from_i64_expr(&raw, &elem_kind);
                    let c_type = self.kind_to_c_type_str(&elem_kind);
                    self.emit(&format!("{} {} = {};", c_type, name, typed));
                    self.variables.insert(name.clone(), VarInfo {
                        c_name: name.clone(),
                        kind: elem_kind.clone(),
                        is_mutable: false,
                    });
                }
                Ok((None, ValKind::Void))
            }
            Stmt::Assign { name, value } => {
                let var = self.variables.get(name).ok_or_else(|| self.err(format!("undefined variable '{}'", name)))?;
                if !var.is_mutable {
                    return Err(self.err(format!("cannot assign to immutable variable '{}'", name)));
                }
                let c_name = var.c_name.clone();
                let (val, kind) = self.compile_expr(value)?;
                self.emit(&format!("{} = {};", c_name, val));
                self.track_variable_kinds(name, &kind);
                Ok((None, ValKind::Void))
            }
            Stmt::IndexAssign { object, index, value } => {
                let (obj_val, obj_kind) = self.compile_expr(object)?;
                let (idx_val, idx_kind) = self.compile_expr(index)?;
                let (val, val_kind) = self.compile_expr(value)?;
                match obj_kind {
                    ValKind::List(_) => {
                        let i64_val = self.value_to_i64_expr(&val, &val_kind);
                        self.emit(&format!("ore_list_set({}, {}, {});", obj_val, idx_val, i64_val));
                    }
                    ValKind::Map(_) => {
                        let key = if idx_kind == ValKind::Str { idx_val } else { self.value_to_str_expr(&idx_val, &idx_kind) };
                        let i64_val = self.value_to_i64_expr(&val, &val_kind);
                        self.emit(&format!("ore_map_set({}, {}, {});", obj_val, key, i64_val));
                    }
                    _ => return Err(self.err("index assignment only supported on lists and maps")),
                }
                Ok((None, ValKind::Void))
            }
            Stmt::FieldAssign { object, field, value } => {
                let (obj_val, obj_kind) = self.compile_expr(object)?;
                let (val, _) = self.compile_expr(value)?;
                if let ValKind::Record(ref _name) = obj_kind {
                    self.emit(&format!("{}.{} = {};", obj_val, field, val));
                } else {
                    return Err(self.err(format!("field assignment not supported on {:?}", obj_kind)));
                }
                Ok((None, ValKind::Void))
            }
            Stmt::Expr(expr) => {
                let (val, kind) = self.compile_expr(expr)?;
                let is_call = matches!(expr, Expr::Call { .. } | Expr::MethodCall { .. });
                if kind == ValKind::Void {
                    // Void expression: emit as standalone statement if it's a call
                    if val != "0" && !val.is_empty() {
                        self.emit(&format!("{};", val));
                    }
                    Ok((None, ValKind::Void))
                } else if is_call && val.contains('(') {
                    // Non-void call used as expression statement: emit for side effects,
                    // then use the result as the last expression value.
                    // Methods like remove_at return a value but also have side effects.
                    let tmp = self.tmp();
                    let c_type = self.kind_to_c_type_str(&kind);
                    self.emit(&format!("{} {} = {};", c_type, tmp, val));
                    Ok((Some(tmp), kind))
                } else {
                    Ok((Some(val), kind))
                }
            }
            Stmt::Return(Some(expr)) => {
                let (val, _kind) = self.compile_expr(expr)?;
                self.emit(&format!("return {};", val));
                Ok((None, ValKind::Void))
            }
            Stmt::Return(None) => {
                self.emit("return;");
                Ok((None, ValKind::Void))
            }
            Stmt::ForIn { var, start, end, step, body } => {
                self.compile_for_in(var, start, end, step.as_ref(), body)?;
                Ok((None, ValKind::Void))
            }
            Stmt::ForEach { var, iterable, body } => {
                self.compile_for_each(var, iterable, body)?;
                Ok((None, ValKind::Void))
            }
            Stmt::ForEachKV { key_var, val_var, iterable, body } => {
                self.compile_for_each_kv(key_var, val_var, iterable, body)?;
                Ok((None, ValKind::Void))
            }
            Stmt::While { cond, body } => {
                self.compile_while(cond, body)?;
                Ok((None, ValKind::Void))
            }
            Stmt::Loop { body } => {
                self.compile_loop(body)?;
                Ok((None, ValKind::Void))
            }
            Stmt::Break => {
                if let Some(label) = self.break_labels.last().cloned() {
                    self.emit(&format!("goto {};", label));
                } else {
                    return Err(self.err("break outside of loop"));
                }
                Ok((None, ValKind::Void))
            }
            Stmt::Continue => {
                if let Some(label) = self.continue_labels.last().cloned() {
                    self.emit(&format!("goto {};", label));
                } else {
                    return Err(self.err("continue outside of loop"));
                }
                Ok((None, ValKind::Void))
            }
            Stmt::Spawn(expr) => {
                if let Expr::Call { func: callee, args } = expr {
                    let name = match callee.as_ref() {
                        Expr::Ident(n) => n.clone(),
                        _ => return Err(self.err("spawn requires a named function call")),
                    };
                    let c_fn_name = Self::mangle_fn_name(&name);
                    if args.is_empty() {
                        self.emit(&format!("ore_spawn((void*)&{});", c_fn_name));
                    } else {
                        let mut arg_strs = Vec::new();
                        for a in args {
                            let (v, k) = self.compile_expr(a)?;
                            arg_strs.push(self.value_to_i64_expr(&v, &k));
                        }
                        let spawn_fn = match args.len() {
                            1 => "ore_spawn_with_arg",
                            2 => "ore_spawn_with_2args",
                            3 => "ore_spawn_with_3args",
                            n => return Err(self.err(format!("spawn supports at most 3 arguments, got {}", n))),
                        };
                        self.emit(&format!("{}((void*)&{}, {});", spawn_fn, c_fn_name, arg_strs.join(", ")));
                    }
                } else {
                    return Err(self.err("spawn requires a function call"));
                }
                Ok((None, ValKind::Void))
            }
            Stmt::LocalFn(fndef) => {
                // Save and restore state around local function compilation
                let saved_vars = self.variables.clone();
                let saved_lines = std::mem::take(&mut self.lines);
                let saved_indent = self.indent;

                self.declare_function(fndef)?;
                self.compile_function(fndef)?;

                // Move generated function to lambda_bodies (before main code)
                self.lambda_bodies.extend(std::mem::take(&mut self.lines));
                self.lines = saved_lines;
                self.indent = saved_indent;
                self.variables = saved_vars;

                Ok((None, ValKind::Void))
            }
        }
    }

    fn compile_for_in(&mut self, var: &str, start_expr: &Expr, end_expr: &Expr, step_expr: std::option::Option<&Expr>, body: &Block) -> Result<(), CCodeGenError> {
        let (start, _) = self.compile_expr(start_expr)?;
        let (end, _) = self.compile_expr(end_expr)?;
        let step = if let Some(se) = step_expr {
            let (s, _) = self.compile_expr(se)?;
            s
        } else {
            "1".to_string()
        };

        let end_tmp = self.tmp();
        let step_tmp = self.tmp();
        self.emit(&format!("int64_t {} = {};", end_tmp, end));
        self.emit(&format!("int64_t {} = {};", step_tmp, step));

        let break_label = self.label("for_end");
        let continue_label = self.label("for_inc");
        self.break_labels.push(break_label.clone());
        self.continue_labels.push(continue_label.clone());

        self.emit(&format!("for (int64_t {} = {}; {} < {}; {} += {}) {{", var, start, var, end_tmp, var, step_tmp));
        self.indent += 1;
        self.variables.insert(var.to_string(), VarInfo { c_name: var.to_string(), kind: ValKind::Int, is_mutable: false });

        self.compile_block_stmts(body)?;

        self.emit(&format!("{}:;", continue_label));
        self.indent -= 1;
        self.emit("}");
        self.emit(&format!("{}:;", break_label));

        self.break_labels.pop();
        self.continue_labels.pop();
        Ok(())
    }

    fn compile_for_each(&mut self, var: &str, iterable: &Expr, body: &Block) -> Result<(), CCodeGenError> {
        let (val, kind) = self.compile_expr(iterable)?;

        if kind.is_map() {
            let keys_tmp = self.tmp();
            self.emit(&format!("void* {} = ore_map_keys({});", keys_tmp, val));
            return self.compile_for_each_list(var, &keys_tmp, ValKind::Str, body);
        }

        let elem_kind = kind.list_elem_kind().cloned().unwrap_or(ValKind::Int);
        self.compile_for_each_list(var, &val, elem_kind, body)
    }

    fn compile_for_each_list(&mut self, var: &str, list_val: &str, elem_kind: ValKind, body: &Block) -> Result<(), CCodeGenError> {
        let len_tmp = self.tmp();
        let idx = self.tmp();
        let c_type = self.kind_to_c_type_str(&elem_kind);

        self.emit(&format!("int64_t {} = ore_list_len({});", len_tmp, list_val));

        let break_label = self.label("foreach_end");
        let continue_label = self.label("foreach_inc");
        self.break_labels.push(break_label.clone());
        self.continue_labels.push(continue_label.clone());

        self.emit(&format!("for (int64_t {} = 0; {} < {}; {}++) {{", idx, idx, len_tmp, idx));
        self.indent += 1;

        let raw = format!("ore_list_get({}, {})", list_val, idx);
        let typed = self.coerce_from_i64_expr(&raw, &elem_kind);
        self.emit(&format!("{} {} = {};", c_type, var, typed));
        self.variables.insert(var.to_string(), VarInfo { c_name: var.to_string(), kind: elem_kind, is_mutable: false });

        self.compile_block_stmts(body)?;

        self.emit(&format!("{}:;", continue_label));
        self.indent -= 1;
        self.emit("}");
        self.emit(&format!("{}:;", break_label));

        self.break_labels.pop();
        self.continue_labels.pop();
        Ok(())
    }

    fn compile_for_each_kv(&mut self, key_var: &str, val_var: &str, iterable: &Expr, body: &Block) -> Result<(), CCodeGenError> {
        let (val, kind) = self.compile_expr(iterable)?;

        if kind.is_map() {
            let val_kind = kind.map_val_kind().cloned().unwrap_or(ValKind::Int);
            let keys_tmp = self.tmp();
            let len_tmp = self.tmp();
            let idx = self.tmp();
            self.emit(&format!("void* {} = ore_map_keys({});", keys_tmp, val));
            self.emit(&format!("int64_t {} = ore_list_len({});", len_tmp, keys_tmp));

            let break_label = self.label("forkv_end");
            let continue_label = self.label("forkv_inc");
            self.break_labels.push(break_label.clone());
            self.continue_labels.push(continue_label.clone());

            self.emit(&format!("for (int64_t {} = 0; {} < {}; {}++) {{", idx, idx, len_tmp, idx));
            self.indent += 1;

            self.emit(&format!("void* {} = (void*)(intptr_t)ore_list_get({}, {});", key_var, keys_tmp, idx));
            self.variables.insert(key_var.to_string(), VarInfo { c_name: key_var.to_string(), kind: ValKind::Str, is_mutable: false });

            let val_c_type = self.kind_to_c_type_str(&val_kind);
            let raw_val = format!("ore_map_get({}, {})", val, key_var);
            let typed_val = self.coerce_from_i64_expr(&raw_val, &val_kind);
            self.emit(&format!("{} {} = {};", val_c_type, val_var, typed_val));
            self.variables.insert(val_var.to_string(), VarInfo { c_name: val_var.to_string(), kind: val_kind, is_mutable: false });

            self.compile_block_stmts(body)?;

            self.emit(&format!("{}:;", continue_label));
            self.indent -= 1;
            self.emit("}");
            self.emit(&format!("{}:;", break_label));

            self.break_labels.pop();
            self.continue_labels.pop();
            return Ok(());
        }

        // List enumeration: for i, x in list
        let elem_kind = kind.list_elem_kind().cloned().unwrap_or(ValKind::Int);
        let len_tmp = self.tmp();
        let c_type = self.kind_to_c_type_str(&elem_kind);

        self.emit(&format!("int64_t {} = ore_list_len({});", len_tmp, val));

        let break_label = self.label("forenum_end");
        let continue_label = self.label("forenum_inc");
        self.break_labels.push(break_label.clone());
        self.continue_labels.push(continue_label.clone());

        self.emit(&format!("for (int64_t {} = 0; {} < {}; {}++) {{", key_var, key_var, len_tmp, key_var));
        self.indent += 1;

        self.variables.insert(key_var.to_string(), VarInfo { c_name: key_var.to_string(), kind: ValKind::Int, is_mutable: false });

        let raw = format!("ore_list_get({}, {})", val, key_var);
        let typed = self.coerce_from_i64_expr(&raw, &elem_kind);
        self.emit(&format!("{} {} = {};", c_type, val_var, typed));
        self.variables.insert(val_var.to_string(), VarInfo { c_name: val_var.to_string(), kind: elem_kind, is_mutable: false });

        self.compile_block_stmts(body)?;

        self.emit(&format!("{}:;", continue_label));
        self.indent -= 1;
        self.emit("}");
        self.emit(&format!("{}:;", break_label));

        self.break_labels.pop();
        self.continue_labels.pop();
        Ok(())
    }

    fn compile_while(&mut self, cond: &Expr, body: &Block) -> Result<(), CCodeGenError> {
        let break_label = self.label("while_end");
        let continue_label = self.label("while_cond");
        self.break_labels.push(break_label.clone());
        self.continue_labels.push(continue_label.clone());

        // Use for(;;) with condition re-evaluation each iteration.
        // This is necessary because compile_expr may emit setup code
        // (e.g. short-circuit evaluation for `and`/`or`) that must run
        // every iteration, not just once before the loop.
        self.emit(&format!("{}:;", continue_label));
        self.emit("for (;;) {");
        self.indent += 1;

        // Evaluate condition at the top of each iteration
        let (cond_val, _) = self.compile_expr(cond)?;
        self.emit(&format!("if (!({cond_val})) goto {break_label};"));

        self.compile_block_stmts(body)?;

        self.indent -= 1;
        self.emit("}");
        self.emit(&format!("{}:;", break_label));

        self.break_labels.pop();
        self.continue_labels.pop();
        Ok(())
    }

    fn compile_loop(&mut self, body: &Block) -> Result<(), CCodeGenError> {
        let break_label = self.label("loop_end");
        let continue_label = self.label("loop_body");
        self.break_labels.push(break_label.clone());
        self.continue_labels.push(continue_label.clone());

        self.emit(&format!("{}:;", continue_label));
        self.emit("for (;;) {");
        self.indent += 1;

        self.compile_block_stmts(body)?;

        self.indent -= 1;
        self.emit("}");
        self.emit(&format!("{}:;", break_label));

        self.break_labels.pop();
        self.continue_labels.pop();
        Ok(())
    }
}
