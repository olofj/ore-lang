use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;

impl<'ctx> CodeGen<'ctx> {
    /// Convert an i8 value to a bool (i1) by comparing != 0.
    pub(crate) fn i8_to_bool(&self, val: IntValue<'ctx>) -> Result<IntValue<'ctx>, CodeGenError> {
        Ok(bld!(self.builder.build_int_compare(
            IntPredicate::NE, val,
            self.context.i8_type().const_int(0, false), "tobool"
        ))?)
    }

    /// Normalize a value to i1 bool: if it's an integer wider than 1 bit, compare != 0.
    pub(crate) fn normalize_to_bool(&self, val: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, CodeGenError> {
        let iv = val.into_int_value();
        if iv.get_type().get_bit_width() > 1 {
            Ok(bld!(self.builder.build_int_compare(
                IntPredicate::NE, iv,
                iv.get_type().const_int(0, false), "tobool"
            ))?)
        } else {
            Ok(iv)
        }
    }

    /// Generate a helpful "unknown method" error with available methods listed.
    pub(crate) fn unknown_method_error(type_name: &str, method: &str, available: &[&str]) -> CodeGenError {
        let mut msg = format!("unknown {} method '{}'. Available: {}", type_name, method, available.join(", "));
        // Add did-you-mean suggestion
        let best = Self::find_similar(method, available);
        if let Some(suggestion) = best {
            msg = format!("unknown {} method '{}'; did you mean '{}'?", type_name, method, suggestion);
        }
        CodeGenError { line: None, msg }
    }

    /// Get the current insert block, returning a proper error instead of panicking.
    pub(crate) fn current_block(&self) -> Result<inkwell::basic_block::BasicBlock<'ctx>, CodeGenError> {
        self.builder.get_insert_block().ok_or_else(|| {
            self.err("LLVM builder has no current insert block")
        })
    }

    /// Get the current function from the builder's insert block.
    pub(crate) fn current_fn(&self) -> Result<FunctionValue<'ctx>, CodeGenError> {
        self.current_block()?.get_parent().ok_or_else(|| {
            self.err("current block has no parent function")
        })
    }

    /// Look up a runtime function by name, returning a proper error instead of panicking.
    pub(crate) fn rt(&self, name: &str) -> Result<FunctionValue<'ctx>, CodeGenError> {
        self.module.get_function(name).ok_or_else(|| CodeGenError {
            line: None,
            msg: format!("runtime function '{}' not declared", name),
        })
    }

    pub(crate) fn coerce_to_i64(&mut self, val: BasicValueEnum<'ctx>, kind: &ValKind) -> Result<IntValue<'ctx>, CodeGenError> {
        match kind {
            ValKind::Bool => {
                bld!(self.builder.build_int_z_extend(
                    val.into_int_value(), self.context.i64_type(), "btoi64"
                ))
            }
            ValKind::Float => {
                bld!(self.builder.build_bit_cast(val, self.context.i64_type(), "ftoi64")).map(|v| v.into_int_value())
            }
            ValKind::Str | ValKind::List(_) | ValKind::Map(_) => {
                self.ptr_to_i64(val.into_pointer_value())
            }
            ValKind::Void => Ok(self.context.i64_type().const_int(0, false)),
            _ => Ok(val.into_int_value()),
        }
    }

    pub(crate) fn coerce_from_i64(&mut self, val: BasicValueEnum<'ctx>, kind: &ValKind) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match kind {
            ValKind::Str | ValKind::List(_) | ValKind::Map(_) => {
                let ptr = self.i64_to_ptr(val.into_int_value())?;
                Ok(ptr.into())
            }
            ValKind::Bool => {
                let cmp = bld!(self.builder.build_int_compare(
                    IntPredicate::NE, val.into_int_value(),
                    self.context.i64_type().const_int(0, false), "i64tobool"
                ))?;
                Ok(cmp.into())
            }
            ValKind::Float => {
                let f = bld!(self.builder.build_bit_cast(val, self.context.f64_type(), "i64tof64"))?;
                Ok(f)
            }
            _ => Ok(val),
        }
    }

    /// Convert a raw i64 list element back to the correct type.
    /// Extends coerce_from_i64 with Enum/Record support (heap-allocated, needs load through pointer).
    pub(crate) fn list_elem_from_i64(&mut self, raw: BasicValueEnum<'ctx>, kind: &ValKind) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match kind {
            ValKind::Enum(name) => {
                let et = self.enums[name].enum_type;
                let ptr = self.i64_to_ptr(raw.into_int_value())?;
                Ok(bld!(self.builder.build_load(et, ptr, "load_enum"))?)
            }
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let ptr = self.i64_to_ptr(raw.into_int_value())?;
                Ok(bld!(self.builder.build_load(st, ptr, "load_rec"))?)
            }
            _ => self.coerce_from_i64(raw, kind),
        }
    }

    /// Convert any BasicValueEnum to i64 for storage in Option/Result payloads.
    pub(crate) fn value_to_i64(&mut self, val: BasicValueEnum<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, CodeGenError> {
        match val {
            BasicValueEnum::IntValue(v) => {
                if v.get_type().get_bit_width() < 64 {
                    Ok(bld!(self.builder.build_int_z_extend(v, self.context.i64_type(), "zext"))?)
                } else {
                    Ok(v)
                }
            }
            BasicValueEnum::FloatValue(v) => {
                Ok(bld!(self.builder.build_bit_cast(v, self.context.i64_type(), "f2i"))?.into_int_value())
            }
            BasicValueEnum::PointerValue(v) => {
                self.ptr_to_i64(v)
            }
            BasicValueEnum::StructValue(v) => {
                // Heap-allocate the struct and return its pointer as i64.
                // This handles enums and records stored in Option/Result payloads.
                let ty = v.get_type();
                let heap_ptr = bld!(self.builder.build_malloc(ty, "heap_struct"))?;
                bld!(self.builder.build_store(heap_ptr, v))?;
                self.ptr_to_i64(heap_ptr)
            }
            _ => Ok(val.into_int_value()),
        }
    }

    /// Convert a value to i64 for storage in a list, heap-allocating enums/records.
    pub(crate) fn val_to_list_i64(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: &ValKind,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match kind {
            ValKind::Enum(name) => {
                let et = self.enums[name].enum_type;
                let heap_ptr = bld!(self.builder.build_malloc(et, "heap_enum"))?;
                bld!(self.builder.build_store(heap_ptr, val))?;
                Ok(self.ptr_to_i64(heap_ptr)?.into())
            }
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let heap_ptr = bld!(self.builder.build_malloc(st, "heap_rec"))?;
                bld!(self.builder.build_store(heap_ptr, val))?;
                Ok(self.ptr_to_i64(heap_ptr)?.into())
            }
            ValKind::Float => {
                Ok(bld!(self.builder.build_bit_cast(val, self.context.i64_type(), "f2i"))?)
            }
            ValKind::Bool => {
                Ok(bld!(self.builder.build_int_z_extend(val.into_int_value(), self.context.i64_type(), "b2i"))?.into())
            }
            ValKind::Str | ValKind::List(_) | ValKind::Map(_) => {
                Ok(self.ptr_to_i64(val.into_pointer_value())?.into())
            }
            _ => Ok(val),
        }
    }

    pub(crate) fn resolve_function(&self, name: &str) -> Result<(FunctionValue<'ctx>, ValKind), CodeGenError> {
        if let Some((f, k)) = self.functions.get(name) {
            return Ok((*f, k.clone()));
        }
        if let Some(f) = self.module.get_function(name) {
            return Ok((f, ValKind::Void));
        }
        Err(self.undefined_fn_error(name))
    }

    /// Build an "undefined X 'name'" error with a "did you mean?" suggestion.
    pub(crate) fn undefined_var_error(&self, name: &str) -> CodeGenError {
        self.undefined_error("variable", name, &self.variables.keys().map(|s| s.as_str()).collect::<Vec<_>>())
    }

    pub(crate) fn undefined_fn_error(&self, name: &str) -> CodeGenError {
        self.undefined_error("function", name, &self.functions.keys().map(|s| s.as_str()).collect::<Vec<_>>())
    }

    fn undefined_error(&self, kind: &str, name: &str, candidates: &[&str]) -> CodeGenError {
        let mut msg = format!("undefined {} '{}'", kind, name);
        if let Some(suggestion) = Self::find_similar(name, candidates) {
            msg.push_str(&format!("; did you mean '{}'?", suggestion));
        }
        CodeGenError { line: None, msg }
    }

    /// Return (i64_zero, ValKind::Void) — used as the return value for void expressions.
    pub(crate) fn void_result(&self) -> (BasicValueEnum<'ctx>, ValKind) {
        (self.context.i64_type().const_int(0, false).into(), ValKind::Void)
    }

    /// Build a null-terminated C string as an LLVM global constant.
    /// Returns a pointer to the string data.
    pub(crate) fn build_c_string_global(&mut self, s: &str, name: &str) -> Result<PointerValue<'ctx>, CodeGenError> {
        let bytes: Vec<u8> = s.bytes().chain(std::iter::once(0)).collect();
        let i8_type = self.context.i8_type();
        let arr_type = i8_type.array_type(bytes.len() as u32);
        let global = self.module.add_global(arr_type, None, name);
        global.set_initializer(&i8_type.const_array(
            &bytes.iter().map(|&b| i8_type.const_int(b as u64, false)).collect::<Vec<_>>(),
        ));
        global.set_constant(true);
        bld!(self.builder.build_pointer_cast(
            global.as_pointer_value(), self.ptr_type(), "cstr_ptr"
        ))
    }

    /// Convert a value to f64, accepting both Float and Int kinds.
    pub(crate) fn coerce_to_float(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: &ValKind,
        context: &str,
    ) -> Result<inkwell::values::FloatValue<'ctx>, CodeGenError> {
        match kind {
            ValKind::Float => Ok(val.into_float_value()),
            ValKind::Int => bld!(self.builder.build_signed_int_to_float(
                val.into_int_value(), self.context.f64_type(), "itof"
            )),
            _ => Err(self.err(format!("{} requires numeric argument", context))),
        }
    }

    /// Resolve a lambda or function reference argument, returning (fn_ptr, env_ptr, return_kind).
    /// `param_kinds` specifies the types passed to the lambda parameters.
    pub(crate) fn resolve_lambda_arg(
        &mut self,
        arg: &Expr,
        param_kinds: &[ValKind],
        method_name: &str,
    ) -> Result<(PointerValue<'ctx>, PointerValue<'ctx>, ValKind), CodeGenError> {
        let (lambda_fn, ret_kind) = match arg {
            Expr::Lambda { params, body } => {
                self.compile_lambda_with_kinds(params, body, Some(param_kinds))?
            }
            Expr::Ident(name) => {
                self.resolve_function(name)?
            }
            _ => return Err(self.err(format!("{} argument must be a function", method_name))),
        };
        let lambda_name = Self::get_lambda_name(lambda_fn);
        let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
            self.build_captures_struct(&lambda_name)?
        } else {
            self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
        };
        let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
        Ok((fn_ptr, env_ptr, ret_kind))
    }

    /// Extract the name of a lambda function as a String.
    pub(crate) fn get_lambda_name(lambda_fn: FunctionValue<'ctx>) -> String {
        lambda_fn.get_name().to_str().unwrap_or("__unknown_lambda").to_string()
    }

    /// Build an alloca in the entry block of `func`, preserving the current insert position.
    pub(crate) fn build_entry_alloca(
        &self,
        func: FunctionValue<'ctx>,
        ty: impl inkwell::types::BasicType<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let entry = func.get_first_basic_block().ok_or_else(|| self.err("function has no entry block"))?;
        let current = self.current_block()?;
        if let Some(first_instr) = entry.get_first_instruction() {
            self.builder.position_before(&first_instr);
        } else {
            self.builder.position_at_end(entry);
        }
        let alloca = bld!(self.builder.build_alloca(ty, name))?;
        self.builder.position_at_end(current);
        Ok(alloca)
    }

    /// Convert a pointer value to i64.
    pub(crate) fn ptr_to_i64(&self, ptr: PointerValue<'ctx>) -> Result<IntValue<'ctx>, CodeGenError> {
        Ok(bld!(self.builder.build_ptr_to_int(ptr, self.context.i64_type(), "p2i"))?)
    }

    /// Convert an i64 value to a pointer.
    pub(crate) fn i64_to_ptr(&self, val: IntValue<'ctx>) -> Result<PointerValue<'ctx>, CodeGenError> {
        Ok(bld!(self.builder.build_int_to_ptr(val, self.ptr_type(), "i2p"))?)
    }

    /// Find the most similar string from candidates (edit distance <= 2).
    pub(crate) fn find_similar<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
        let mut best: Option<(&str, usize)> = None;
        for &c in candidates {
            let d = Self::edit_distance(name, c);
            if d > 0 && d <= 2 && best.as_ref().is_none_or(|(_, bd)| d < *bd) {
                best = Some((c, d));
            }
        }
        best.map(|(s, _)| s)
    }

    #[allow(clippy::needless_range_loop)]
    pub(crate) fn edit_distance(a: &str, b: &str) -> usize {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        let (m, n) = (a.len(), b.len());
        if m == 0 { return n; }
        if n == 0 { return m; }
        let mut dp = vec![vec![0usize; n + 1]; m + 1];
        for i in 0..=m { dp[i][0] = i; }
        for j in 0..=n { dp[0][j] = j; }
        for i in 1..=m {
            for j in 1..=n {
                let cost = if a[i-1] == b[j-1] { 0 } else { 1 };
                dp[i][j] = (dp[i-1][j] + 1).min(dp[i][j-1] + 1).min(dp[i-1][j-1] + cost);
            }
        }
        dp[m][n]
    }

    /// Map a ValKind to its string name for mangling and type expression conversion.
    pub(crate) fn valkind_to_name(kind: &ValKind) -> String {
        match kind {
            ValKind::Int => "Int".to_string(),
            ValKind::Float => "Float".to_string(),
            ValKind::Bool => "Bool".to_string(),
            ValKind::Str => "Str".to_string(),
            ValKind::Void => "Void".to_string(),
            ValKind::Record(name) | ValKind::Enum(name) => name.clone(),
            ValKind::Option => "Option".to_string(),
            ValKind::Result => "Result".to_string(),
            ValKind::List(_) => "List".to_string(),
            ValKind::Map(_) => "Map".to_string(),
            ValKind::Channel => "Channel".to_string(),
        }
    }

    /// Map a ValKind back to a TypeExpr for monomorphization substitution.
    pub(crate) fn valkind_to_type_expr(kind: &ValKind) -> TypeExpr {
        TypeExpr::Named(Self::valkind_to_name(kind))
    }

    /// Create a mangled name for a monomorphized function.
    pub(crate) fn mangle_generic_name(base: &str, concrete_kinds: &[ValKind]) -> String {
        let mut name = base.to_string();
        for k in concrete_kinds {
            name.push('$');
            name.push_str(&Self::valkind_to_name(k));
        }
        name
    }

    /// Substitute type parameters in a TypeExpr.
    pub(crate) fn substitute_type_expr(ty: &TypeExpr, subst: &HashMap<String, TypeExpr>) -> TypeExpr {
        match ty {
            TypeExpr::Named(name) => {
                if let Some(replacement) = subst.get(name) {
                    replacement.clone()
                } else {
                    ty.clone()
                }
            }
            TypeExpr::Generic(name, args) => {
                let new_args: Vec<TypeExpr> = args.iter()
                    .map(|a| Self::substitute_type_expr(a, subst))
                    .collect();
                TypeExpr::Generic(name.clone(), new_args)
            }
            TypeExpr::Fn { params, ret } => {
                let new_params: Vec<TypeExpr> = params.iter()
                    .map(|p| Self::substitute_type_expr(p, subst))
                    .collect();
                let new_ret = Box::new(Self::substitute_type_expr(ret, subst));
                TypeExpr::Fn { params: new_params, ret: new_ret }
            }
        }
    }

    /// Monomorphize a generic function for the given concrete argument kinds.
    /// Returns the (FunctionValue, return ValKind) for the specialized version.
    pub(crate) fn monomorphize(
        &mut self,
        generic_name: &str,
        arg_kinds: &[ValKind],
    ) -> Result<(FunctionValue<'ctx>, ValKind), CodeGenError> {
        let mangled = Self::mangle_generic_name(generic_name, arg_kinds);

        // Already monomorphized?
        if let Some((f, k)) = self.functions.get(&mangled) {
            return Ok((*f, k.clone()));
        }

        let generic_fn = self.generic_fns.get(generic_name).cloned().ok_or_else(|| {
            self.err(format!("no generic function '{}'", generic_name))
        })?;

        // Build substitution map: type_param_name -> concrete TypeExpr
        let mut subst = HashMap::new();
        for (i, tp) in generic_fn.type_params.iter().enumerate() {
            // Match type params to arg kinds based on param positions
            // Find which argument position uses this type param
            let concrete = if let Some(kind) = self.find_concrete_for_type_param(&tp.name, &generic_fn.params, arg_kinds) {
                kind
            } else if i < arg_kinds.len() {
                // Fallback: positional mapping
                Self::valkind_to_type_expr(&arg_kinds[i])
            } else {
                TypeExpr::Named("Int".to_string()) // default fallback
            };
            subst.insert(tp.name.clone(), concrete);
        }

        // Create specialized FnDef
        let specialized_params: Vec<Param> = generic_fn.params.iter().map(|p| {
            Param {
                name: p.name.clone(),
                ty: Self::substitute_type_expr(&p.ty, &subst),
                default: p.default.clone(),
            }
        }).collect();

        let specialized_ret = generic_fn.ret_type.as_ref().map(|t| Self::substitute_type_expr(t, &subst));

        let specialized_fn = FnDef {
            name: mangled.clone(),
            type_params: vec![], // No longer generic
            params: specialized_params,
            ret_type: specialized_ret,
            body: generic_fn.body,
        };

        // Declare and compile the specialized function
        self.declare_function(&specialized_fn)?;

        // Save/restore state around compilation
        let saved_bb = self.builder.get_insert_block();
        let saved_vars = std::mem::take(&mut self.variables);
        let saved_break = self.break_target.take();
        let saved_continue = self.continue_target.take();

        self.compile_function(&specialized_fn)?;

        self.variables = saved_vars;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        // Restore builder position to where we were
        if let Some(bb) = saved_bb {
            self.builder.position_at_end(bb);
        }

        let (f, k) = self.functions.get(&mangled).ok_or_else(|| {
            self.err(format!("monomorphized function '{}' not found after compilation", mangled))
        })?.clone();
        Ok((f, k))
    }

    /// Find the concrete TypeExpr for a type parameter by scanning param declarations.
    pub(crate) fn find_concrete_for_type_param(
        &self,
        type_param: &str,
        params: &[Param],
        arg_kinds: &[ValKind],
    ) -> Option<TypeExpr> {
        for (i, param) in params.iter().enumerate() {
            if i >= arg_kinds.len() { break; }
            if let TypeExpr::Named(name) = &param.ty {
                if name == type_param {
                    return Some(Self::valkind_to_type_expr(&arg_kinds[i]));
                }
            }
        }
        None
    }

    pub(crate) fn call_result_to_value(
        &self,
        result: inkwell::values::CallSiteValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match result.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(val) => Ok(val),
            inkwell::values::ValueKind::Instruction(_) => {
                Ok(self.context.i64_type().const_int(0, false).into())
            }
        }
    }

    /// Call a unary f64→f64 LLVM intrinsic (e.g. llvm.floor.f64, llvm.sqrt.f64).
    pub(crate) fn call_f64_intrinsic(
        &self,
        intrinsic: &str,
        arg: BasicValueEnum<'ctx>,
        label: &str,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let f64_type = self.context.f64_type();
        let func = self.module.get_function(intrinsic).unwrap_or_else(|| {
            self.module.add_function(intrinsic, f64_type.fn_type(&[f64_type.into()], false), None)
        });
        let result = bld!(self.builder.build_call(func, &[arg.into()], label))?;
        self.call_result_to_value(result)
    }

    /// Load the tag (index 0, i8) from a tagged union (enum, Option, Result).
    pub(crate) fn load_tag(
        &self,
        struct_type: inkwell::types::StructType<'ctx>,
        alloca: inkwell::values::PointerValue<'ctx>,
    ) -> Result<IntValue<'ctx>, CodeGenError> {
        let tag_ptr = bld!(self.builder.build_struct_gep(struct_type, alloca, 0, "tag_ptr"))?;
        Ok(bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value())
    }

    /// Load the value (index 2, i64) from a tagged union (Option, Result).
    pub(crate) fn load_tagged_value(
        &self,
        struct_type: inkwell::types::StructType<'ctx>,
        alloca: inkwell::values::PointerValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let val_ptr = bld!(self.builder.build_struct_gep(struct_type, alloca, 2, "val_ptr"))?;
        Ok(bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "val"))?)
    }

    /// Look up a runtime function, call it, and extract the return value.
    pub(crate) fn call_rt(
        &self,
        name: &str,
        args: &[inkwell::values::BasicMetadataValueEnum<'ctx>],
        label: &str,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let rt = self.rt(name)?;
        let result = bld!(self.builder.build_call(rt, args, label))?;
        self.call_result_to_value(result)
    }

    /// Save break/continue targets and set new ones for a loop body.
    /// Returns the saved targets to pass to `restore_loop_targets()`.
    pub(crate) fn set_loop_targets(
        &mut self,
        end_bb: inkwell::basic_block::BasicBlock<'ctx>,
        inc_bb: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> (Option<inkwell::basic_block::BasicBlock<'ctx>>, Option<inkwell::basic_block::BasicBlock<'ctx>>) {
        let saved = (self.break_target, self.continue_target);
        self.break_target = Some(end_bb);
        self.continue_target = Some(inc_bb);
        saved
    }

    /// Restore break/continue targets after a loop body, and add a branch
    /// to `fallthrough_bb` if the current block has no terminator.
    pub(crate) fn restore_loop_targets(
        &mut self,
        saved: (Option<inkwell::basic_block::BasicBlock<'ctx>>, Option<inkwell::basic_block::BasicBlock<'ctx>>),
        fallthrough_bb: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> Result<(), CodeGenError> {
        self.break_target = saved.0;
        self.continue_target = saved.1;
        if self.current_block()?.get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(fallthrough_bb))?;
        }
        Ok(())
    }


    /// Normalize a bool IntValue to i8, handling any bit width (i1, i8, i64).
    pub(crate) fn bool_to_i8(&mut self, int_val: IntValue<'ctx>) -> Result<IntValue<'ctx>, CodeGenError> {
        let bw = int_val.get_type().get_bit_width();
        if bw < 8 {
            Ok(bld!(self.builder.build_int_z_extend(int_val, self.context.i8_type(), "zext"))?)
        } else if bw > 8 {
            Ok(bld!(self.builder.build_int_truncate(int_val, self.context.i8_type(), "trunc"))?)
        } else {
            Ok(int_val)
        }
    }

}
