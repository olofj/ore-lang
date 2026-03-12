use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;

impl<'ctx> CodeGen<'ctx> {
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
            ValKind::Int => Ok(val.into_int_value()),
            ValKind::Bool => {
                bld!(self.builder.build_int_z_extend(
                    val.into_int_value(), self.context.i64_type(), "btoi64"
                ))
            }
            ValKind::Float => {
                bld!(self.builder.build_bit_cast(val, self.context.i64_type(), "ftoi64")).map(|v| v.into_int_value())
            }
            ValKind::Str | ValKind::List(_) | ValKind::Map => {
                self.ptr_to_i64(val.into_pointer_value())
            }
            ValKind::Void => Ok(self.context.i64_type().const_int(0, false)),
            _ => Ok(val.into_int_value()),
        }
    }

    pub(crate) fn coerce_from_i64(&mut self, val: BasicValueEnum<'ctx>, kind: &ValKind) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match kind {
            ValKind::Str | ValKind::List(_) | ValKind::Map => {
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
    /// For enums/records, the i64 is a pointer to heap-allocated data that needs to be loaded.
    pub(crate) fn list_elem_from_i64(&mut self, raw: BasicValueEnum<'ctx>, kind: &ValKind) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match kind {
            ValKind::Enum(name) => {
                let et = self.enums[name].enum_type;
                let ptr = self.i64_to_ptr(raw.into_int_value())?;
                let val = bld!(self.builder.build_load(et, ptr, "load_enum"))?;
                Ok(val)
            }
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let ptr = self.i64_to_ptr(raw.into_int_value())?;
                let val = bld!(self.builder.build_load(st, ptr, "load_rec"))?;
                Ok(val)
            }
            ValKind::Str | ValKind::List(_) | ValKind::Map => {
                let ptr = self.i64_to_ptr(raw.into_int_value())?;
                Ok(ptr.into())
            }
            ValKind::Float => {
                let f = bld!(self.builder.build_bit_cast(raw, self.context.f64_type(), "i2f"))?;
                Ok(f)
            }
            ValKind::Bool => {
                let b = bld!(self.builder.build_int_compare(
                    IntPredicate::NE, raw.into_int_value(),
                    self.context.i64_type().const_int(0, false), "i2b"
                ))?;
                Ok(b.into())
            }
            _ => Ok(raw),
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
            _ => Ok(val.into_int_value()),
        }
    }

    pub(crate) fn resolve_function(&self, name: &str) -> Result<(FunctionValue<'ctx>, ValKind), CodeGenError> {
        if let Some((f, k)) = self.functions.get(name) {
            return Ok((*f, k.clone()));
        }
        if let Some(f) = self.module.get_function(name) {
            return Ok((f, ValKind::Void));
        }
        // Suggest similar function names
        let mut msg = format!("undefined function '{}'", name);
        let candidates: Vec<&str> = self.functions.keys().map(|s| s.as_str()).collect();
        if let Some(suggestion) = Self::find_similar(name, &candidates) {
            msg.push_str(&format!("; did you mean '{}'?", suggestion));
        }
        Err(CodeGenError { line: None, msg })
    }

    /// Resolve a lambda or function reference argument into (FunctionValue, fn_ptr, env_ptr).
    /// `param_kinds` specifies the types passed to the lambda parameters.
    /// If `track_return_kind` is true, sets `self.last_lambda_return_kind` from named function refs.
    pub(crate) fn resolve_list_lambda_arg(
        &mut self,
        arg: &Expr,
        param_kinds: &[ValKind],
        method_name: &str,
        func: FunctionValue<'ctx>,
        track_return_kind: bool,
    ) -> Result<(FunctionValue<'ctx>, PointerValue<'ctx>), CodeGenError> {
        let lambda_fn = match arg {
            Expr::Lambda { params, body } => {
                self.compile_lambda_with_kinds(params, body, func, Some(param_kinds))?
            }
            Expr::Ident(name) => {
                let (f, ret_kind) = self.resolve_function(name)?;
                if track_return_kind {
                    self.last_lambda_return_kind = Some(ret_kind);
                }
                f
            }
            _ => return Err(self.err(format!("{} argument must be a function", method_name))),
        };
        let lambda_name = Self::get_lambda_name(lambda_fn);
        let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
            self.build_captures_struct(&lambda_name)?
        } else {
            self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
        };
        Ok((lambda_fn, env_ptr))
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
            if d <= 2 && d > 0
                && (best.is_none() || d < best.unwrap().1) {
                    best = Some((c, d));
                }
        }
        best.map(|(s, _)| s)
    }

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

    /// Map a ValKind back to a TypeExpr for monomorphization substitution.
    pub(crate) fn valkind_to_type_expr(kind: &ValKind) -> TypeExpr {
        match kind {
            ValKind::Int => TypeExpr::Named("Int".to_string()),
            ValKind::Float => TypeExpr::Named("Float".to_string()),
            ValKind::Bool => TypeExpr::Named("Bool".to_string()),
            ValKind::Str => TypeExpr::Named("Str".to_string()),
            ValKind::Void => TypeExpr::Named("Void".to_string()),
            ValKind::Record(name) => TypeExpr::Named(name.clone()),
            ValKind::Enum(name) => TypeExpr::Named(name.clone()),
            ValKind::Option => TypeExpr::Named("Option".to_string()),
            ValKind::Result => TypeExpr::Named("Result".to_string()),
            ValKind::List(_) => TypeExpr::Named("List".to_string()),
            ValKind::Map => TypeExpr::Named("Map".to_string()),
            ValKind::Channel => TypeExpr::Named("Channel".to_string()),
        }
    }

    /// Create a mangled name for a monomorphized function.
    pub(crate) fn mangle_generic_name(base: &str, concrete_kinds: &[ValKind]) -> String {
        let mut name = base.to_string();
        for k in concrete_kinds {
            name.push('$');
            match k {
                ValKind::Int => name.push_str("Int"),
                ValKind::Float => name.push_str("Float"),
                ValKind::Bool => name.push_str("Bool"),
                ValKind::Str => name.push_str("Str"),
                ValKind::Void => name.push_str("Void"),
                ValKind::Record(n) => name.push_str(n),
                ValKind::Enum(n) => name.push_str(n),
                ValKind::Option => name.push_str("Option"),
                ValKind::Result => name.push_str("Result"),
                ValKind::List(_) => name.push_str("List"),
                ValKind::Map => name.push_str("Map"),
                ValKind::Channel => name.push_str("Channel"),
            }
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
        _current_fn: FunctionValue<'ctx>,
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
            body: generic_fn.body.clone(),
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

}
