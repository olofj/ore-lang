use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;

/// Loop control structure returned by `build_indexed_loop`.
struct IndexedLoop<'ctx> {
    idx_alloca: PointerValue<'ctx>,
    inc_bb: inkwell::basic_block::BasicBlock<'ctx>,
    end_bb: inkwell::basic_block::BasicBlock<'ctx>,
    cond_bb: inkwell::basic_block::BasicBlock<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    /// Build the skeleton of an indexed loop: idx alloca, cond/body/inc/end blocks,
    /// and branch to the condition block. Returns the loop structure; the caller
    /// should position at body_bb and emit body code, then call `finish_indexed_loop`.
    fn build_indexed_loop(
        &mut self,
        len_val: IntValue<'ctx>,
        prefix: &str,
        func: FunctionValue<'ctx>,
    ) -> Result<IndexedLoop<'ctx>, CodeGenError> {
        let i64_type = self.context.i64_type();
        let idx_alloca = bld!(self.builder.build_alloca(i64_type, "idx"))?;
        bld!(self.builder.build_store(idx_alloca, i64_type.const_int(0, false)))?;

        let cond_bb = self.context.append_basic_block(func, &format!("{prefix}_cond"));
        let body_bb = self.context.append_basic_block(func, &format!("{prefix}_body"));
        let inc_bb = self.context.append_basic_block(func, &format!("{prefix}_inc"));
        let end_bb = self.context.append_basic_block(func, &format!("{prefix}_end"));

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(cond_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, idx, len_val, &format!("{prefix}_cmp")))?;
        bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

        self.builder.position_at_end(body_bb);
        Ok(IndexedLoop { idx_alloca, inc_bb, end_bb, cond_bb })
    }

    /// Load the current index from the loop alloca.
    fn loop_index(&self, lp: &IndexedLoop<'ctx>) -> Result<IntValue<'ctx>, CodeGenError> {
        let i64_type = self.context.i64_type();
        Ok(bld!(self.builder.build_load(i64_type, lp.idx_alloca, "idx"))?.into_int_value())
    }

    /// Emit the increment block and position at end_bb. Must be called after the
    /// body block code has been emitted (including compile_block_stmts).
    fn finish_indexed_loop(&mut self, lp: &IndexedLoop<'ctx>) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();
        self.builder.position_at_end(lp.inc_bb);
        let idx = bld!(self.builder.build_load(i64_type, lp.idx_alloca, "idx"))?.into_int_value();
        let next = bld!(self.builder.build_int_add(idx, i64_type.const_int(1, false), "inc"))?;
        bld!(self.builder.build_store(lp.idx_alloca, next))?;
        bld!(self.builder.build_unconditional_branch(lp.cond_bb))?;
        self.builder.position_at_end(lp.end_bb);
        Ok(())
    }
    /// Track list element kind and map value kind for a variable binding.
    fn track_variable_kinds(&mut self, name: &str, kind: &ValKind) {
        if let ValKind::List(Some(ref ek)) = kind {
            self.list_element_kinds.insert(name.to_string(), ek.as_ref().clone());
        }
        if let ValKind::Map(Some(ref vk)) = kind {
            self.map_value_kinds.insert(name.to_string(), vk.as_ref().clone());
        }
        // Keep VarInfo.kind in sync so lambda captures and other direct reads see updated kinds
        if kind.is_list() || kind.is_map() {
            if let Some(var) = self.variables.get_mut(name) {
                var.kind = kind.clone();
            }
        }
    }

    pub(crate) fn compile_stmt(
        &mut self,
        stmt: &Stmt,
        func: FunctionValue<'ctx>,
    ) -> Result<(Option<BasicValueEnum<'ctx>>, ValKind), CodeGenError> {
        match stmt {
            Stmt::Let { name, mutable, value, .. } => {
                self.compile_let(name, *mutable, value, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::LetDestructure { names, value } => {
                self.compile_let_destructure(names, value, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::Assign { name, value } => {
                self.compile_assign(name, value, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::AssignIfUnset { name, value } => {
                // For now, treat as a regular assign in LLVM codegen
                // (the C codegen has the conditional logic)
                self.compile_assign(name, value, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::IndexAssign { object, index, value } => {
                self.compile_index_assign(object, index, value, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::FieldAssign { object, field, value } => {
                self.compile_field_assign(object, field, value, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::Expr(expr) => {
                let (val, kind) = self.compile_expr_with_kind(expr, func)?;
                Ok((Some(val), kind))
            }
            Stmt::Return(Some(expr)) => {
                let (val, _kind) = self.compile_expr_with_kind(expr, func)?;
                bld!(self.builder.build_return(Some(&val)))?;
                Ok((None, ValKind::Void))
            }
            Stmt::Return(None) => {
                bld!(self.builder.build_return(None))?;
                Ok((None, ValKind::Void))
            }
            Stmt::ForIn { var, start, end, step, body } => {
                self.compile_for_in(var, start, end, step.as_ref(), body, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::ForEach { var, iterable, body } => {
                self.compile_for_each(var, iterable, body, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::ForEachKV { key_var, val_var, iterable, body } => {
                self.compile_for_each_kv(key_var, val_var, iterable, body, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::While { cond, body } => {
                self.compile_while(cond, body, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::Loop { body } => {
                self.compile_loop(body, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::Break => {
                if let Some(target) = self.break_target {
                    bld!(self.builder.build_unconditional_branch(target))?;
                } else {
                    return Err(self.err("break outside of loop"));
                }
                Ok((None, ValKind::Void))
            }
            Stmt::Continue => {
                if let Some(target) = self.continue_target {
                    bld!(self.builder.build_unconditional_branch(target))?;
                } else {
                    return Err(self.err("continue outside of loop"));
                }
                Ok((None, ValKind::Void))
            }
            Stmt::Spawn(expr) => {
                self.compile_spawn(expr, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::LocalFn(fndef) => {
                self.compile_local_fn(fndef, func)?;
                Ok((None, ValKind::Void))
            }
            Stmt::WithBlock { expr: _expr, body } => {
                // Context injection: compile the body in the current scope.
                // The with-expression is evaluated but context propagation
                // to callees is handled by the C backend; the LLVM backend
                // compiles the body directly for now.
                self.compile_block_stmts(body, func)
            }
        }
    }

    fn compile_let(
        &mut self,
        name: &str,
        mutable: bool,
        value: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let (val, kind) = self.compile_expr_with_kind(value, func)?;
        let ty = val.get_type();
        let alloca = self.build_entry_alloca(func, ty, name)?;
        bld!(self.builder.build_store(alloca, val))?;
        self.variables.insert(name.to_string(), VarInfo { ptr: alloca, ty, kind: kind.clone(), is_mutable: mutable });
        self.track_variable_kinds(name, &kind);
        Ok(())
    }

    fn compile_let_destructure(
        &mut self,
        names: &[String],
        value: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let (val, vk) = self.compile_expr_with_kind(value, func)?;
        let list_ptr = val.into_pointer_value();
        let elem_kind = match &vk {
            ValKind::List(Some(ek)) => ek.as_ref().clone(),
            _ => ValKind::Int,
        };
        let list_get_fn = self.rt("ore_list_get")?;
        let i64_type = self.context.i64_type();

        for (i, name) in names.iter().enumerate() {
            let idx = i64_type.const_int(i as u64, false);
            let result = bld!(self.builder.build_call(list_get_fn, &[list_ptr.into(), idx.into()], "destr"))?;
            let raw_val = self.call_result_to_value(result)?;
            let typed_val = self.list_elem_from_i64(raw_val, &elem_kind)?;
            let (alloca, ty) = self.alloca_for_kind(name, &elem_kind)?;
            bld!(self.builder.build_store(alloca, typed_val))?;
            self.variables.insert(name.clone(), VarInfo { ptr: alloca, ty, kind: elem_kind.clone(), is_mutable: false });
        }
        Ok(())
    }

    fn compile_assign(
        &mut self,
        name: &str,
        value: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let (val, kind) = self.compile_expr_with_kind(value, func)?;
        let var_info = self.variables.get(name).ok_or_else(|| self.undefined_var_error(name))?;
        if !var_info.is_mutable {
            return Err(self.err(format!("cannot assign to immutable variable '{}'", name)));
        }
        bld!(self.builder.build_store(var_info.ptr, val))?;
        self.track_variable_kinds(name, &kind);
        Ok(())
    }

    fn compile_index_assign(
        &mut self,
        object: &Expr,
        index: &Expr,
        value: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        let idx_val = self.compile_expr(index, func)?;
        let (val, _val_kind) = self.compile_expr_with_kind(value, func)?;
        match obj_kind {
            ValKind::List(_) => {
                self.call_rt("ore_list_set", &[obj_val.into(), idx_val.into(), val.into()], "")?;
            }
            ValKind::Map(_) => {
                let map_key = if idx_val.is_pointer_value() {
                    idx_val
                } else {
                    self.value_to_str(idx_val, ValKind::Int)?.into()
                };
                self.call_rt("ore_map_set", &[obj_val.into(), map_key.into(), val.into()], "")?;
            }
            _ => return Err(self.err("index assignment only supported on lists and maps")),
        }
        Ok(())
    }

    fn compile_field_assign(
        &mut self,
        object: &Expr,
        field: &str,
        value: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        let (val, _val_kind) = self.compile_expr_with_kind(value, func)?;
        match obj_kind {
            ValKind::Record(ref name) => {
                let rec_info = self.records.get(name).ok_or_else(|| self.err(format!("undefined record type '{}'", name)))?;
                let field_idx = rec_info.field_names.iter().position(|f| f == field).ok_or_else(|| self.err(format!("unknown field '{}' on record '{}'", field, name)))?;
                let struct_type = rec_info.struct_type;
                let field_ptr = bld!(self.builder.build_struct_gep(
                    struct_type, obj_val.into_pointer_value(), field_idx as u32, &format!("fld_{}", field)
                ))?;
                bld!(self.builder.build_store(field_ptr, val))?;
            }
            _ => return Err(self.err(format!("field assignment not supported on {:?}", obj_kind))),
        }
        Ok(())
    }

    fn compile_spawn(
        &mut self,
        expr: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        match expr {
            Expr::Call { func: callee, args } => {
                let name = match callee.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(self.err("spawn requires a named function call")),
                };
                let (target_fn, _) = self.resolve_function(&name)?;
                let fn_ptr = target_fn.as_global_value().as_pointer_value();

                if args.is_empty() {
                    self.call_rt("ore_spawn", &[fn_ptr.into()], "")?;
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
                    let call_args: Vec<_> = i64_args.iter().map(|a| (*a).into()).collect();
                    self.call_rt(spawn_fn_name, &call_args, "")?;
                }
                Ok(())
            }
            _ => Err(self.err("spawn requires a function call")),
        }
    }

    fn compile_local_fn(
        &mut self,
        fndef: &FnDef,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let mangled = if let Ok(parent) = func.get_name().to_str() {
            format!("{}__{}", parent, fndef.name)
        } else {
            fndef.name.clone()
        };
        let mut mangled_fndef = fndef.clone();
        let original_name = fndef.name.clone();
        mangled_fndef.name = mangled.clone();

        let saved_vars = self.variables.clone();
        let saved_insert_block = self.builder.get_insert_block();
        let saved_break = self.break_target;
        let saved_continue = self.continue_target;

        self.declare_function(&mangled_fndef)?;
        self.compile_function(&mangled_fndef)?;

        self.variables = saved_vars;
        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        if let Some(f) = self.module.get_function(&mangled) {
            let ret_kind = match fndef.ret_type.as_ref() {
                Some(te) => self.type_expr_to_kind(te),
                None => ValKind::Void,
            };
            self.functions.insert(original_name, (f, ret_kind));
        }
        Ok(())
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
            Stmt::Expr(expr) | Stmt::Let { value: expr, .. } | Stmt::Assign { value: expr, .. } | Stmt::AssignIfUnset { value: expr, .. } => {
                self.prescan_expr_for_map_kinds(expr);
            }
            Stmt::While { body, .. } | Stmt::Loop { body, .. }
            | Stmt::ForIn { body, .. } | Stmt::ForEach { body, .. } | Stmt::ForEachKV { body, .. } => {
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
        self.variables.insert(var.to_string(), VarInfo { ptr: var_alloca, ty: i64_type.into(), kind: ValKind::Int, is_mutable: false });

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
        let saved = self.set_loop_targets(end_bb, inc_bb);
        self.compile_block_stmts(body, func)?;
        self.restore_loop_targets(saved, inc_bb)?;

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
        let (val, kind) = self.compile_expr_with_kind(iterable, func)?;
        if kind.is_map() {
            let map_ptr = val.into_pointer_value();
            let list_ptr = self.call_rt("ore_map_keys", &[map_ptr.into()], "keys")?.into_pointer_value();
            return self.compile_for_each_over_list(var, list_ptr, ValKind::Str, body, func, None);
        }
        let elem_kind = kind.list_elem_kind().cloned().unwrap_or(ValKind::Int);
        let list_ptr = val.into_pointer_value();
        self.compile_for_each_over_list(var, list_ptr, elem_kind, body, func, None)
    }

    /// Allocate a local variable with the correct LLVM type for a given ValKind.
    /// Returns (alloca pointer, LLVM type).
    pub(crate) fn alloca_for_kind(
        &mut self,
        name: &str,
        kind: &ValKind,
    ) -> Result<(PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>), CodeGenError> {
        let ty = self.kind_to_llvm_type(kind);
        let alloca = bld!(self.builder.build_alloca(ty, name))?;
        Ok((alloca, ty))
    }

    /// Compile a for-each loop over a list. If `index_var` is Some, also exposes the
    /// loop index as a variable with that name.
    pub(crate) fn compile_for_each_over_list(
        &mut self,
        var: &str,
        list_ptr: PointerValue<'ctx>,
        elem_kind: ValKind,
        body: &Block,
        func: FunctionValue<'ctx>,
        index_var: Option<&str>,
    ) -> Result<(), CodeGenError> {
        let len_val = self.call_rt("ore_list_len", &[list_ptr.into()], "len")?.into_int_value();

        let (elem_alloca, elem_ty) = self.alloca_for_kind(var, &elem_kind)?;
        self.variables.insert(var.to_string(), VarInfo { ptr: elem_alloca, ty: elem_ty, kind: elem_kind.clone(), is_mutable: false });

        let label = if index_var.is_some() { "forenum" } else { "foreach" };
        let lp = self.build_indexed_loop(len_val, label, func)?;

        if let Some(idx_name) = index_var {
            let i64_type = self.context.i64_type();
            self.variables.insert(idx_name.to_string(), VarInfo { ptr: lp.idx_alloca, ty: i64_type.into(), kind: ValKind::Int, is_mutable: false });
        }

        let idx = self.loop_index(&lp)?;
        let raw_val = self.call_rt("ore_list_get", &[list_ptr.into(), idx.into()], "elem")?;
        let typed_val = self.list_elem_from_i64(raw_val, &elem_kind)?;
        bld!(self.builder.build_store(elem_alloca, typed_val))?;

        let saved = self.set_loop_targets(lp.end_bb, lp.inc_bb);
        self.compile_block_stmts(body, func)?;
        self.restore_loop_targets(saved, lp.inc_bb)?;

        self.finish_indexed_loop(&lp)?;
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
        let (val, kind) = self.compile_expr_with_kind(iterable, func)?;
        if kind.is_map() {
            let val_kind = kind.map_val_kind().cloned().unwrap_or(ValKind::Int);
            let map_ptr = val.into_pointer_value();
            return self.compile_for_each_kv_map(key_var, val_var, map_ptr, &val_kind, body, func);
        }
        // List enumeration: for i, x in list
        let elem_kind = kind.list_elem_kind().cloned().unwrap_or(ValKind::Int);
        let list_ptr = val.into_pointer_value();
        self.compile_for_each_over_list(val_var, list_ptr, elem_kind, body, func, Some(key_var))
    }

    fn compile_for_each_kv_map(
        &mut self,
        key_var: &str,
        val_var: &str,
        map_ptr: PointerValue<'ctx>,
        val_kind: &ValKind,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let keys_list = self.call_rt("ore_map_keys", &[map_ptr.into()], "keys")?.into_pointer_value();
        let len_val = self.call_rt("ore_list_len", &[keys_list.into()], "len")?.into_int_value();

        let key_alloca = bld!(self.builder.build_alloca(ptr_type, key_var))?;
        self.variables.insert(key_var.to_string(), VarInfo { ptr: key_alloca, ty: ptr_type.into(), kind: ValKind::Str, is_mutable: false });

        let (val_alloca, val_ty) = self.alloca_for_kind(val_var, val_kind)?;
        self.variables.insert(val_var.to_string(), VarInfo { ptr: val_alloca, ty: val_ty, kind: val_kind.clone(), is_mutable: false });

        let lp = self.build_indexed_loop(len_val, "forkv", func)?;
        let idx = self.loop_index(&lp)?;

        let key_raw = self.call_rt("ore_list_get", &[keys_list.into(), idx.into()], "key_raw")?.into_int_value();
        let key_ptr = self.i64_to_ptr(key_raw)?;
        bld!(self.builder.build_store(key_alloca, key_ptr))?;

        let val_raw = self.call_rt("ore_map_get", &[map_ptr.into(), key_ptr.into()], "val_raw")?;
        let val_typed = self.coerce_from_i64(val_raw, val_kind)?;
        bld!(self.builder.build_store(val_alloca, val_typed))?;

        let saved = self.set_loop_targets(lp.end_bb, lp.inc_bb);
        self.compile_block_stmts(body, func)?;
        self.restore_loop_targets(saved, lp.inc_bb)?;

        self.finish_indexed_loop(&lp)?;
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
        let saved = self.set_loop_targets(end_bb, cond_bb);
        self.compile_block_stmts(body, func)?;
        self.restore_loop_targets(saved, cond_bb)?;

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
        let saved = self.set_loop_targets(end_bb, body_bb);
        self.compile_block_stmts(body, func)?;
        self.restore_loop_targets(saved, body_bb)?;

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
        let (else_val, else_kind) = if let Some(eb) = else_block {
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
        let else_i64 = self.coerce_to_i64(else_val, &else_kind)?;
        bld!(self.builder.build_store(result_alloca, else_i64))?;

        self.builder.position_at_end(merge_bb);
        let result = bld!(self.builder.build_load(i64_type, result_alloca, "ifval"))?;
        // Value was coerced to i64, so kind is Int (original kind info is lost)
        Ok((result, ValKind::Int))
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
