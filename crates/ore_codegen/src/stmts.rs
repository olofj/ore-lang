use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::IntPredicate;

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_stmt(
        &mut self,
        stmt: &Stmt,
        func: FunctionValue<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenError> {
        match stmt {
            Stmt::Let { name, mutable, value } => {
                self.last_list_elem_kind = None;
                self.last_map_val_kind = None;
                let (val, kind) = self.compile_expr_with_kind(value, func)?;
                let ty = val.get_type();
                let alloca = self.build_entry_alloca(func, ty, name)?;
                bld!(self.builder.build_store(alloca, val))?;
                self.variables.insert(name.clone(), (alloca, ty, kind.clone(), *mutable));
                // Track element kind for typed lists
                if let ValKind::List(Some(ref ek)) = kind {
                    self.list_element_kinds.insert(name.clone(), *ek.clone());
                } else if kind.is_list() {
                    if let Some(ek) = self.last_list_elem_kind.take() {
                        self.list_element_kinds.insert(name.clone(), ek);
                    }
                }
                // Track value kind for typed maps
                if kind == ValKind::Map {
                    if let Some(vk) = self.last_map_val_kind.take() {
                        self.map_value_kinds.insert(name.clone(), vk);
                    }
                }
                Ok(None)
            }
            Stmt::LetDestructure { names, value } => {
                self.last_list_elem_kind = None;
                let (val, vk) = self.compile_expr_with_kind(value, func)?;
                let list_ptr = val.into_pointer_value();
                let elem_kind = match &vk {
                    ValKind::List(Some(ek)) => *ek.clone(),
                    _ => self.last_list_elem_kind.clone().unwrap_or(ValKind::Int),
                };
                let list_get_fn = self.rt("ore_list_get")?;
                let i64_type = self.context.i64_type();

                for (i, name) in names.iter().enumerate() {
                    let idx = i64_type.const_int(i as u64, false);
                    let result = bld!(self.builder.build_call(list_get_fn, &[list_ptr.into(), idx.into()], "destr"))?;
                    let raw_val = self.call_result_to_value(result)?;

                    match &elem_kind {
                        ValKind::Str => {
                            let ptr = self.i64_to_ptr(raw_val.into_int_value())?;
                            let pt = self.context.ptr_type(inkwell::AddressSpace::default());
                            let alloca = bld!(self.builder.build_alloca(pt, name))?;
                            bld!(self.builder.build_store(alloca, ptr))?;
                            self.variables.insert(name.clone(), (alloca, pt.into(), ValKind::Str, false));
                        }
                        _ => {
                            let alloca = bld!(self.builder.build_alloca(i64_type, name))?;
                            bld!(self.builder.build_store(alloca, raw_val))?;
                            self.variables.insert(name.clone(), (alloca, i64_type.into(), elem_kind.clone(), false));
                        }
                    }
                }
                Ok(None)
            }
            Stmt::Assign { name, value } => {
                let (val, kind) = self.compile_expr_with_kind(value, func)?;
                let (ptr, _, _, is_mut) = self.variables.get(name).ok_or_else(|| {
                    let mut msg = format!("undefined variable '{}'", name);
                    let candidates: Vec<&str> = self.variables.keys().map(|s| s.as_str()).collect();
                    if let Some(suggestion) = Self::find_similar(name, &candidates) {
                        msg.push_str(&format!("; did you mean '{}'?", suggestion));
                    }
                    CodeGenError { line: None, msg }
                })?;
                if !is_mut {
                    return Err(CodeGenError {
                        line: Some(self.current_line), msg: format!("cannot assign to immutable variable '{}'", name),
                    });
                }
                bld!(self.builder.build_store(*ptr, val))?;
                // Update element kind tracking for lists and maps on reassignment
                if let ValKind::List(Some(ref ek)) = kind {
                    self.list_element_kinds.insert(name.clone(), *ek.clone());
                } else if kind.is_list() {
                    if let Some(ek) = self.last_list_elem_kind.clone() {
                        self.list_element_kinds.insert(name.clone(), ek);
                    }
                }
                if kind == ValKind::Map {
                    if let Some(vk) = self.last_map_val_kind.clone() {
                        self.map_value_kinds.insert(name.clone(), vk);
                    }
                }
                Ok(None)
            }
            Stmt::IndexAssign { object, index, value } => {
                let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
                let idx_val = self.compile_expr(index, func)?;
                let (val, _val_kind) = self.compile_expr_with_kind(value, func)?;
                match obj_kind {
                    ValKind::List(_) => {
                        let rt = self.rt("ore_list_set")?;
                        bld!(self.builder.build_call(rt, &[obj_val.into(), idx_val.into(), val.into()], ""))?;
                    }
                    ValKind::Map => {
                        // Convert non-string keys to strings for map
                        let map_key = if idx_val.is_pointer_value() {
                            idx_val
                        } else {
                            self.value_to_str(idx_val, ValKind::Int)?.into()
                        };
                        let rt = self.rt("ore_map_set")?;
                        bld!(self.builder.build_call(rt, &[obj_val.into(), map_key.into(), val.into()], ""))?;
                    }
                    _ => return Err(self.err("index assignment only supported on lists and maps")),
                }
                Ok(None)
            }
            Stmt::FieldAssign { object, field, value } => {
                let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
                let (val, _val_kind) = self.compile_expr_with_kind(value, func)?;
                match obj_kind {
                    ValKind::Record(ref name) => {
                        let rec_info = self.records.get(name).ok_or_else(|| CodeGenError {
                            line: Some(self.current_line), msg: format!("undefined record type '{}'", name),
                        })?;
                        let field_idx = rec_info.field_names.iter().position(|f| f == field).ok_or_else(|| CodeGenError {
                            line: Some(self.current_line), msg: format!("unknown field '{}' on record '{}'", field, name),
                        })?;
                        let struct_type = rec_info.struct_type;
                        let field_ptr = bld!(self.builder.build_struct_gep(
                            struct_type, obj_val.into_pointer_value(), field_idx as u32, &format!("fld_{}", field)
                        ))?;
                        bld!(self.builder.build_store(field_ptr, val))?;
                    }
                    _ => return Err(self.err(format!("field assignment not supported on {:?}", obj_kind))),
                }
                Ok(None)
            }
            Stmt::Expr(expr) => {
                let (val, _kind) = self.compile_expr_with_kind(expr, func)?;
                Ok(Some(val))
            }
            Stmt::Return(Some(expr)) => {
                let (val, _kind) = self.compile_expr_with_kind(expr, func)?;
                bld!(self.builder.build_return(Some(&val)))?;
                Ok(None)
            }
            Stmt::Return(None) => {
                bld!(self.builder.build_return(None))?;
                Ok(None)
            }
            Stmt::ForIn { var, start, end, step, body } => {
                self.compile_for_in(var, start, end, step.as_ref(), body, func)?;
                Ok(None)
            }
            Stmt::ForEach { var, iterable, body } => {
                self.compile_for_each(var, iterable, body, func)?;
                Ok(None)
            }
            Stmt::ForEachKV { key_var, val_var, iterable, body } => {
                self.compile_for_each_kv(key_var, val_var, iterable, body, func)?;
                Ok(None)
            }
            Stmt::While { cond, body } => {
                self.compile_while(cond, body, func)?;
                Ok(None)
            }
            Stmt::Loop { body } => {
                self.compile_loop(body, func)?;
                Ok(None)
            }
            Stmt::Break => {
                if let Some(target) = self.break_target {
                    bld!(self.builder.build_unconditional_branch(target))?;
                } else {
                    return Err(self.err("break outside of loop"));
                }
                Ok(None)
            }
            Stmt::Continue => {
                if let Some(target) = self.continue_target {
                    bld!(self.builder.build_unconditional_branch(target))?;
                } else {
                    return Err(self.err("continue outside of loop"));
                }
                Ok(None)
            }
            Stmt::Spawn(expr) => {
                match expr {
                    Expr::Call { func: callee, args } => {
                        let name = match callee.as_ref() {
                            Expr::Ident(n) => n.clone(),
                            _ => return Err(self.err("spawn requires a named function call")),
                        };
                        let (target_fn, _) = self.resolve_function(&name)?;
                        let fn_ptr = target_fn.as_global_value().as_pointer_value();

                        if args.is_empty() {
                            let ore_spawn = self.rt("ore_spawn")?;
                            bld!(self.builder.build_call(ore_spawn, &[fn_ptr.into()], ""))?;
                        } else {
                            let mut i64_args: Vec<BasicValueEnum> = vec![fn_ptr.into()];
                            for arg in args {
                                let arg_val = self.compile_expr(arg, func)?;
                                let i64_val = self.value_to_i64(arg_val)?;
                                i64_args.push(i64_val.into());
                            }
                            let spawn_fn_name = match args.len() {
                                1 => "ore_spawn_with_arg",
                                2 => "ore_spawn_with_2args",
                                3 => "ore_spawn_with_3args",
                                n => return Err(self.err(format!("spawn supports at most 3 arguments, got {}", n))),
                            };
                            let ore_spawn = self.rt(spawn_fn_name)?;
                            let call_args: Vec<_> = i64_args.iter().map(|a| (*a).into()).collect();
                            bld!(self.builder.build_call(ore_spawn, &call_args, ""))?;
                        }
                        Ok(None)
                    }
                    _ => Err(self.err("spawn requires a function call")),
                }
            }
            Stmt::LocalFn(fndef) => {
                // Compile local function with mangled name
                let mangled = if let Ok(parent) = func.get_name().to_str() {
                    format!("{}__{}", parent, fndef.name)
                } else {
                    fndef.name.clone()
                };
                let mut mangled_fndef = fndef.clone();
                let original_name = fndef.name.clone();
                mangled_fndef.name = mangled.clone();

                // Save parent function state
                let saved_vars = self.variables.clone();
                let saved_insert_block = self.builder.get_insert_block();
                let saved_list_elem_kind = self.last_list_elem_kind.clone();
                let saved_break = self.break_target;
                let saved_continue = self.continue_target;

                self.declare_function(&mangled_fndef)?;
                self.compile_function(&mangled_fndef)?;

                // Restore parent function state
                self.variables = saved_vars;
                if let Some(block) = saved_insert_block {
                    self.builder.position_at_end(block);
                }
                self.last_list_elem_kind = saved_list_elem_kind;
                self.break_target = saved_break;
                self.continue_target = saved_continue;

                // Also register under the original name so calls resolve
                if let Some(f) = self.module.get_function(&mangled) {
                    let ret_kind = match fndef.ret_type.as_ref() {
                        Some(te) => self.type_expr_to_kind(te),
                        None => ValKind::Void,
                    };
                    self.functions.insert(original_name, (f, ret_kind));
                }
                Ok(None)
            }
        }
    }

    /// Pre-scan a block for map.set() calls to infer value kinds before compilation.
    /// This allows map[key] indexing to work correctly even when .set() appears in
    /// a later branch (e.g., else block) that is compiled after the indexing code.
    pub(crate) fn prescan_map_value_kinds(&mut self, block: &Block) {
        for spanned in &block.stmts {
            self.prescan_stmt_for_map_kinds(&spanned.stmt);
        }
    }

    pub(crate) fn prescan_stmt_for_map_kinds(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) | Stmt::Let { value: expr, .. } | Stmt::Assign { value: expr, .. } => {
                self.prescan_expr_for_map_kinds(expr);
            }
            Stmt::While { body, .. } | Stmt::Loop { body, .. } => {
                self.prescan_map_value_kinds(body);
            }
            Stmt::ForIn { body, .. } | Stmt::ForEach { body, .. } | Stmt::ForEachKV { body, .. } => {
                self.prescan_map_value_kinds(body);
            }
            _ => {}
        }
    }

    pub(crate) fn prescan_expr_for_map_kinds(&mut self, expr: &Expr) {
        match expr {
            Expr::MethodCall { object, method, args } => {
                if method == "set" && args.len() == 2 {
                    if let Expr::Ident(map_name) = object.as_ref() {
                        let val_kind = self.infer_expr_kind(&args[1]);
                        if val_kind != ValKind::Int || !self.map_value_kinds.contains_key(map_name) {
                            self.map_value_kinds.insert(map_name.clone(), val_kind);
                        }
                    }
                }
                self.prescan_expr_for_map_kinds(object);
                for arg in args { self.prescan_expr_for_map_kinds(arg); }
            }
            Expr::IfElse { cond, then_block, else_block } => {
                self.prescan_expr_for_map_kinds(cond);
                self.prescan_map_value_kinds(then_block);
                if let Some(eb) = else_block {
                    self.prescan_map_value_kinds(eb);
                }
            }
            Expr::Match { arms, .. } => {
                for arm in arms {
                    self.prescan_expr_for_map_kinds(&arm.body);
                }
            }
            Expr::BlockExpr(block) => {
                self.prescan_map_value_kinds(block);
            }
            _ => {}
        }
    }

    pub(crate) fn compile_for_in(
        &mut self,
        var: &str,
        start_expr: &Expr,
        end_expr: &Expr,
        step_expr: Option<&Expr>,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();
        let start_val = self.compile_expr(start_expr, func)?.into_int_value();
        let end_val = self.compile_expr(end_expr, func)?.into_int_value();
        let step_val = if let Some(se) = step_expr {
            self.compile_expr(se, func)?.into_int_value()
        } else {
            i64_type.const_int(1, false)
        };

        // Alloca for loop variable
        let var_alloca = bld!(self.builder.build_alloca(i64_type, var))?;
        bld!(self.builder.build_store(var_alloca, start_val))?;
        self.variables.insert(var.to_string(), (var_alloca, i64_type.into(), ValKind::Int, false));

        let cond_bb = self.context.append_basic_block(func, "for_cond");
        let body_bb = self.context.append_basic_block(func, "for_body");
        let inc_bb = self.context.append_basic_block(func, "for_inc");
        let end_bb = self.context.append_basic_block(func, "for_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        // Condition: i < end
        self.builder.position_at_end(cond_bb);
        let current = bld!(self.builder.build_load(i64_type, var_alloca, var))?.into_int_value();
        let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, current, end_val, "for_cmp"))?;
        bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

        // Body
        self.builder.position_at_end(body_bb);
        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(inc_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        if self.current_block()?.get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(inc_bb))?;
        }

        // Increment by step value
        self.builder.position_at_end(inc_bb);
        let current = bld!(self.builder.build_load(i64_type, var_alloca, var))?.into_int_value();
        let next = bld!(self.builder.build_int_add(current, step_val, "inc"))?;
        bld!(self.builder.build_store(var_alloca, next))?;
        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    pub(crate) fn compile_for_each(
        &mut self,
        var: &str,
        iterable: &Expr,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        // Check if the iterable is a map — if so, iterate over its keys
        let is_map = if let Expr::Ident(name) = iterable {
            self.map_value_kinds.contains_key(name)
        } else {
            false
        };

        if is_map {
            // For maps: get keys list and iterate over it
            let map_ptr = self.compile_expr(iterable, func)?.into_pointer_value();
            let keys_fn = self.rt("ore_map_keys")?;
            let keys_result = bld!(self.builder.build_call(keys_fn, &[map_ptr.into()], "keys"))?;
            let list_ptr = self.call_result_to_value(keys_result)?.into_pointer_value();
            return self.compile_for_each_over_list(var, list_ptr, ValKind::Str, body, func);
        }

        // Determine element kind from the iterable
        let elem_kind = if let Expr::Ident(name) = iterable {
            self.list_element_kinds.get(name).cloned().unwrap_or(ValKind::Int)
        } else {
            ValKind::Int
        };

        let (list_val, list_kind) = self.compile_expr_with_kind(iterable, func)?;
        // Priority: list_element_kinds (most up-to-date) > last_list_elem_kind > ValKind variant > default
        let final_elem_kind = if elem_kind != ValKind::Int {
            // list_element_kinds had a non-default value — use it (most up-to-date)
            elem_kind
        } else {
            self.last_list_elem_kind.clone()
                .or_else(|| match &list_kind {
                    ValKind::List(Some(ek)) => Some(*ek.clone()),
                    _ => None,
                })
                .unwrap_or(elem_kind)
        };
        let list_ptr = list_val.into_pointer_value();

        self.compile_for_each_over_list(var, list_ptr, final_elem_kind, body, func)
    }

    pub(crate) fn compile_for_each_over_list(
        &mut self,
        var: &str,
        list_ptr: PointerValue<'ctx>,
        elem_kind: ValKind,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();

        // Get list length
        let list_len_fn = self.rt("ore_list_len")?;
        let len_result = bld!(self.builder.build_call(list_len_fn, &[list_ptr.into()], "len"))?;
        let len_val = self.call_result_to_value(len_result)?.into_int_value();

        // Index variable
        let idx_alloca = bld!(self.builder.build_alloca(i64_type, "idx"))?;
        bld!(self.builder.build_store(idx_alloca, i64_type.const_int(0, false)))?;

        // Element variable — use appropriate type based on element kind
        let (elem_alloca, elem_ty): (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>) = match &elem_kind {
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let alloca = bld!(self.builder.build_alloca(st, var))?;
                (alloca, st.into())
            }
            ValKind::Enum(name) => {
                let et = self.enums[name].enum_type;
                let alloca = bld!(self.builder.build_alloca(et, var))?;
                (alloca, et.into())
            }
            ValKind::Str => {
                let pt = self.context.ptr_type(inkwell::AddressSpace::default());
                let alloca = bld!(self.builder.build_alloca(pt, var))?;
                (alloca, pt.into())
            }
            ValKind::Float => {
                let f64_type = self.context.f64_type();
                let alloca = bld!(self.builder.build_alloca(f64_type, var))?;
                (alloca, f64_type.into())
            }
            _ => {
                let alloca = bld!(self.builder.build_alloca(i64_type, var))?;
                (alloca, i64_type.into())
            }
        };
        self.variables.insert(var.to_string(), (elem_alloca, elem_ty, elem_kind.clone(), false));

        let cond_bb = self.context.append_basic_block(func, "foreach_cond");
        let body_bb = self.context.append_basic_block(func, "foreach_body");
        let inc_bb = self.context.append_basic_block(func, "foreach_inc");
        let end_bb = self.context.append_basic_block(func, "foreach_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        // Condition: idx < len
        self.builder.position_at_end(cond_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, idx, len_val, "foreach_cmp"))?;
        bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

        // Body: load element, execute body
        self.builder.position_at_end(body_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let list_get_fn = self.rt("ore_list_get")?;
        let elem_result = bld!(self.builder.build_call(list_get_fn, &[list_ptr.into(), idx.into()], "elem"))?;
        let raw_val = self.call_result_to_value(elem_result)?;
        // Convert raw i64 from list back to the correct type
        let typed_val = self.list_elem_from_i64(raw_val, &elem_kind)?;
        bld!(self.builder.build_store(elem_alloca, typed_val))?;

        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(inc_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        if self.current_block()?.get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(inc_bb))?;
        }

        // Increment index
        self.builder.position_at_end(inc_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let next = bld!(self.builder.build_int_add(idx, i64_type.const_int(1, false), "inc"))?;
        bld!(self.builder.build_store(idx_alloca, next))?;
        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    pub(crate) fn compile_for_each_kv(
        &mut self,
        key_var: &str,
        val_var: &str,
        iterable: &Expr,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        // Detect if iterable is a map or a list
        let is_map = if let Expr::Ident(name) = iterable {
            self.map_value_kinds.contains_key(name)
        } else {
            false
        };

        if is_map {
            return self.compile_for_each_kv_map(key_var, val_var, iterable, body, func);
        }

        // List enumeration: for i, x in list
        let elem_kind = if let Expr::Ident(name) = iterable {
            self.list_element_kinds.get(name).cloned().unwrap_or(ValKind::Int)
        } else {
            ValKind::Int
        };

        let (list_val, list_kind) = self.compile_expr_with_kind(iterable, func)?;
        let final_elem_kind = if elem_kind != ValKind::Int {
            elem_kind
        } else {
            self.last_list_elem_kind.clone()
                .or_else(|| match &list_kind {
                    ValKind::List(Some(ek)) => Some(*ek.clone()),
                    _ => None,
                })
                .unwrap_or(elem_kind)
        };
        let elem_kind = final_elem_kind;
        let list_ptr = list_val.into_pointer_value();

        let list_len_fn = self.rt("ore_list_len")?;
        let len_result = bld!(self.builder.build_call(list_len_fn, &[list_ptr.into()], "len"))?;
        let len_val = self.call_result_to_value(len_result)?.into_int_value();

        // Index variable (exposed as key_var)
        let idx_alloca = bld!(self.builder.build_alloca(i64_type, key_var))?;
        bld!(self.builder.build_store(idx_alloca, i64_type.const_int(0, false)))?;
        self.variables.insert(key_var.to_string(), (idx_alloca, i64_type.into(), ValKind::Int, false));

        // Element variable
        let (elem_alloca, elem_ty): (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>) = match &elem_kind {
            ValKind::Str => {
                let alloca = bld!(self.builder.build_alloca(ptr_type, val_var))?;
                (alloca, ptr_type.into())
            }
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let alloca = bld!(self.builder.build_alloca(st, val_var))?;
                (alloca, st.into())
            }
            _ => {
                let alloca = bld!(self.builder.build_alloca(i64_type, val_var))?;
                (alloca, i64_type.into())
            }
        };
        self.variables.insert(val_var.to_string(), (elem_alloca, elem_ty, elem_kind.clone(), false));

        let cond_bb = self.context.append_basic_block(func, "forenum_cond");
        let body_bb = self.context.append_basic_block(func, "forenum_body");
        let inc_bb = self.context.append_basic_block(func, "forenum_inc");
        let end_bb = self.context.append_basic_block(func, "forenum_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(cond_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, idx, len_val, "forenum_cmp"))?;
        bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

        self.builder.position_at_end(body_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let list_get_fn = self.rt("ore_list_get")?;
        let elem_result = bld!(self.builder.build_call(list_get_fn, &[list_ptr.into(), idx.into()], "elem"))?;
        let raw_val = self.call_result_to_value(elem_result)?;
        match &elem_kind {
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let p = self.i64_to_ptr(raw_val.into_int_value())?;
                let sv = bld!(self.builder.build_load(st, p, "rec_elem"))?;
                bld!(self.builder.build_store(elem_alloca, sv))?;
            }
            ValKind::Str => {
                let p = self.i64_to_ptr(raw_val.into_int_value())?;
                bld!(self.builder.build_store(elem_alloca, p))?;
            }
            _ => {
                bld!(self.builder.build_store(elem_alloca, raw_val))?;
            }
        }

        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(inc_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        if self.current_block()?.get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(inc_bb))?;
        }

        self.builder.position_at_end(inc_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let next = bld!(self.builder.build_int_add(idx, i64_type.const_int(1, false), "inc"))?;
        bld!(self.builder.build_store(idx_alloca, next))?;
        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    pub(crate) fn compile_for_each_kv_map(
        &mut self,
        key_var: &str,
        val_var: &str,
        iterable: &Expr,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        let val_kind = if let Expr::Ident(name) = iterable {
            self.map_value_kinds.get(name).cloned().unwrap_or(ValKind::Int)
        } else {
            ValKind::Int
        };

        let map_ptr = self.compile_expr(iterable, func)?.into_pointer_value();

        let keys_fn = self.rt("ore_map_keys")?;
        let keys_result = bld!(self.builder.build_call(keys_fn, &[map_ptr.into()], "keys"))?;
        let keys_list = self.call_result_to_value(keys_result)?.into_pointer_value();

        let list_len_fn = self.rt("ore_list_len")?;
        let len_result = bld!(self.builder.build_call(list_len_fn, &[keys_list.into()], "len"))?;
        let len_val = self.call_result_to_value(len_result)?.into_int_value();

        let idx_alloca = bld!(self.builder.build_alloca(i64_type, "idx"))?;
        bld!(self.builder.build_store(idx_alloca, i64_type.const_int(0, false)))?;

        let key_alloca = bld!(self.builder.build_alloca(ptr_type, key_var))?;
        self.variables.insert(key_var.to_string(), (key_alloca, ptr_type.into(), ValKind::Str, false));

        let (val_alloca, val_ty): (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>) = match &val_kind {
            ValKind::Str => {
                let alloca = bld!(self.builder.build_alloca(ptr_type, val_var))?;
                (alloca, ptr_type.into())
            }
            _ => {
                let alloca = bld!(self.builder.build_alloca(i64_type, val_var))?;
                (alloca, i64_type.into())
            }
        };
        self.variables.insert(val_var.to_string(), (val_alloca, val_ty, val_kind.clone(), false));

        let cond_bb = self.context.append_basic_block(func, "forkv_cond");
        let body_bb = self.context.append_basic_block(func, "forkv_body");
        let inc_bb = self.context.append_basic_block(func, "forkv_inc");
        let end_bb = self.context.append_basic_block(func, "forkv_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(cond_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, idx, len_val, "forkv_cmp"))?;
        bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

        self.builder.position_at_end(body_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();

        let list_get_fn = self.rt("ore_list_get")?;
        let key_result = bld!(self.builder.build_call(list_get_fn, &[keys_list.into(), idx.into()], "key_raw"))?;
        let key_raw = self.call_result_to_value(key_result)?.into_int_value();
        let key_ptr = self.i64_to_ptr(key_raw)?;
        bld!(self.builder.build_store(key_alloca, key_ptr))?;

        let map_get_fn = self.rt("ore_map_get")?;
        let val_result = bld!(self.builder.build_call(map_get_fn, &[map_ptr.into(), key_ptr.into()], "val_raw"))?;
        let val_raw = self.call_result_to_value(val_result)?;
        match &val_kind {
            ValKind::Str => {
                let val_ptr = self.i64_to_ptr(val_raw.into_int_value())?;
                bld!(self.builder.build_store(val_alloca, val_ptr))?;
            }
            _ => {
                bld!(self.builder.build_store(val_alloca, val_raw))?;
            }
        }

        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(inc_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        if self.current_block()?.get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(inc_bb))?;
        }

        self.builder.position_at_end(inc_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let next = bld!(self.builder.build_int_add(idx, i64_type.const_int(1, false), "inc"))?;
        bld!(self.builder.build_store(idx_alloca, next))?;
        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    pub(crate) fn compile_while(
        &mut self,
        cond_expr: &Expr,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let cond_bb = self.context.append_basic_block(func, "while_cond");
        let body_bb = self.context.append_basic_block(func, "while_body");
        let end_bb = self.context.append_basic_block(func, "while_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(cond_bb);
        let cond_val = self.compile_expr(cond_expr, func)?.into_int_value();
        bld!(self.builder.build_conditional_branch(cond_val, body_bb, end_bb))?;

        self.builder.position_at_end(body_bb);
        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(cond_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;
        if self.current_block()?.get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(cond_bb))?;
        }

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    pub(crate) fn compile_loop(
        &mut self,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let body_bb = self.context.append_basic_block(func, "loop_body");
        let end_bb = self.context.append_basic_block(func, "loop_end");

        bld!(self.builder.build_unconditional_branch(body_bb))?;

        self.builder.position_at_end(body_bb);
        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(body_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;
        if self.current_block()?.get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(body_bb))?;
        }

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    pub(crate) fn compile_if_else_with_kind(
        &mut self,
        cond: &Expr,
        then_block: &Block,
        else_block: Option<&Block>,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let cond_val = self.compile_expr(cond, func)?;
        let cond_int = match cond_val {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(self.err("condition must be a boolean")),
        };

        let i64_type = self.context.i64_type();
        let then_bb = self.context.append_basic_block(func, "then");
        let else_bb = self.context.append_basic_block(func, "else");
        let merge_bb = self.context.append_basic_block(func, "merge");

        bld!(self.builder.build_conditional_branch(cond_int, then_bb, else_bb))?;

        // Compile then branch
        self.builder.position_at_end(then_bb);
        let (then_val, then_kind) = self.compile_block_stmts_with_kind(then_block, func)?;
        let then_val = then_val.unwrap_or_else(|| i64_type.const_int(0, false).into());
        if self.current_block()?.get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(merge_bb))?;
        }
        let then_end_bb = self.current_block()?;

        // Compile else branch
        self.builder.position_at_end(else_bb);
        let (else_val, _else_kind) = if let Some(eb) = else_block {
            let (v, k) = self.compile_block_stmts_with_kind(eb, func)?;
            (v.unwrap_or_else(|| i64_type.const_int(0, false).into()), k)
        } else {
            (i64_type.const_int(0, false).into(), ValKind::Int)
        };

        if self.current_block()?.get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(merge_bb))?;
        }
        let else_end_bb = self.current_block()?;

        self.builder.position_at_end(merge_bb);

        // If types match, use phi node directly
        if then_val.get_type() == else_val.get_type() {
            let phi = bld!(self.builder.build_phi(then_val.get_type(), "ifval"))?;
            phi.add_incoming(&[(&then_val, then_end_bb), (&else_val, else_end_bb)]);
            return Ok((phi.as_basic_value(), then_kind));
        }

        // Types differ — use an alloca-based approach with i64 coercion.
        // We need to rebuild with the alloca in the entry block.
        // Remove the merge block contents and recompile.
        // Actually, we need to insert stores before the branches. Use a different strategy:
        // Build an alloca in the entry block, then patch stores into then/else before their terminators.
        let result_alloca = self.build_entry_alloca(func, i64_type, "if_result")?;

        // Insert store in then block before its terminator
        if let Some(term) = then_end_bb.get_terminator() {
            self.builder.position_before(&term);
        } else {
            self.builder.position_at_end(then_end_bb);
        }
        let then_i64 = self.coerce_to_i64(then_val, &then_kind)?;
        bld!(self.builder.build_store(result_alloca, then_i64))?;

        // Insert store in else block before its terminator
        if let Some(term) = else_end_bb.get_terminator() {
            self.builder.position_before(&term);
        } else {
            self.builder.position_at_end(else_end_bb);
        }
        let else_i64 = self.coerce_to_i64(else_val, &_else_kind)?;
        bld!(self.builder.build_store(result_alloca, else_i64))?;

        self.builder.position_at_end(merge_bb);
        let result = bld!(self.builder.build_load(i64_type, result_alloca, "ifval"))?;
        Ok((result, then_kind))
    }

    pub(crate) fn compile_colon_match_with_kind(
        &mut self,
        cond: &Expr,
        then_expr: &Expr,
        else_expr: Option<&Expr>,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let cond_val = self.compile_expr(cond, func)?;
        let cond_int = match cond_val {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(self.err("condition must be a boolean")),
        };

        let then_bb = self.context.append_basic_block(func, "cthen");
        let else_bb = self.context.append_basic_block(func, "celse");
        let merge_bb = self.context.append_basic_block(func, "cmerge");

        bld!(self.builder.build_conditional_branch(cond_int, then_bb, else_bb))?;

        self.builder.position_at_end(then_bb);
        let (then_val, then_kind) = self.compile_expr_with_kind(then_expr, func)?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;
        let then_end_bb = self.current_block()?;

        self.builder.position_at_end(else_bb);
        let (else_val, _) = if let Some(e) = else_expr {
            self.compile_expr_with_kind(e, func)?
        } else {
            (self.context.i64_type().const_int(0, false).into(), ValKind::Int)
        };
        bld!(self.builder.build_unconditional_branch(merge_bb))?;
        let else_end_bb = self.current_block()?;

        self.builder.position_at_end(merge_bb);
        let phi = bld!(self.builder.build_phi(then_val.get_type(), "cval"))?;
        phi.add_incoming(&[(&then_val, then_end_bb), (&else_val, else_end_bb)]);
        Ok((phi.as_basic_value(), then_kind))
    }

}
