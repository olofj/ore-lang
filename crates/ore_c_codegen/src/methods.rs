use super::*;

impl CCodeGen {
    pub(crate) fn compile_method_call(&mut self, object: &Expr, method: &str, args: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr(object)?;

        if obj_kind.is_list() {
            let elem_kind = obj_kind.list_elem_kind().cloned()
                .or_else(|| {
                    // Try to infer from variable tracking or function return tracking
                    if let Expr::Ident(ref name) = object {
                        self.list_element_kinds.get(name).cloned()
                    } else if let Expr::Call { func, .. } = object {
                        if let Expr::Ident(ref fn_name) = func.as_ref() {
                            self.fn_return_list_elem_kind.get(fn_name).cloned()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(ValKind::Int);
            let result = self.compile_list_method(&obj_val, method, args, &elem_kind)?;
            // Track push kind
            if method == "push" {
                if let Expr::Ident(var_name) = object {
                    if let ValKind::List(Some(ref ek)) = result.1 {
                        let should_update = if ek.as_ref() == &ValKind::Int {
                            !self.list_element_kinds.contains_key(var_name)
                        } else {
                            true
                        };
                        if should_update {
                            self.list_element_kinds.insert(var_name.clone(), ek.as_ref().clone());
                        }
                    }
                }
            }
            return Ok(result);
        }

        if obj_kind == ValKind::Str {
            return self.compile_str_method(&obj_val, method, args);
        }

        if obj_kind.is_map() {
            let map_vk = obj_kind.map_val_kind().cloned().unwrap_or(ValKind::Int);
            let result = self.compile_map_method(&obj_val, method, args, &map_vk)?;
            if method == "set" {
                if let Expr::Ident(var_name) = object {
                    if let ValKind::Map(Some(ref vk)) = result.1 {
                        self.map_value_kinds.insert(var_name.clone(), vk.as_ref().clone());
                    }
                }
            }
            return Ok(result);
        }

        if obj_kind == ValKind::Option {
            return self.compile_option_method(&obj_val, method, args);
        }
        if obj_kind == ValKind::Result {
            return self.compile_result_method(&obj_val, method, args);
        }
        if obj_kind == ValKind::Channel {
            return self.compile_channel_method(&obj_val, method, args);
        }

        // to_str on any type
        if method == "to_str" {
            let result = self.value_to_str_expr(&obj_val, &obj_kind);
            return Ok((result, ValKind::Str));
        }

        // Bool.to_int
        if obj_kind == ValKind::Bool && method == "to_int" {
            return Ok((format!("(int64_t)({})", obj_val), ValKind::Int));
        }

        // Int methods — but first check if this looks like a map/list method
        // (common when list elements are untyped and default to Int)
        if obj_kind == ValKind::Int {
            // Map-like methods: the Int is likely an opaque pointer to a map
            let map_methods = ["set", "get", "contains", "remove", "keys", "values",
                               "len", "merge", "clear", "each", "entries", "get_or",
                               "map_values", "filter"];
            if map_methods.contains(&method) {
                let map_val = format!("(void*)(intptr_t)({})", obj_val);
                let map_vk = ValKind::Int;
                let result = self.compile_map_method(&map_val, method, args, &map_vk)?;
                return Ok(result);
            }
            // List-like methods when the Int is actually a list pointer
            let list_methods = ["push", "pop", "sort", "reverse", "map", "filter",
                                "each", "find", "fold", "any", "all", "join",
                                "slice", "take", "skip", "zip", "enumerate",
                                "flat_map", "count", "sum", "product", "first", "last"];
            if list_methods.contains(&method) {
                let list_val = format!("(void*)(intptr_t)({})", obj_val);
                // Try to infer element kind from variable or function tracking
                let elem_kind = if let Expr::Ident(ref name) = object {
                    self.list_element_kinds.get(name).cloned()
                } else if let Expr::Call { func, .. } = object {
                    if let Expr::Ident(ref fn_name) = func.as_ref() {
                        self.fn_return_list_elem_kind.get(fn_name).cloned()
                    } else {
                        None
                    }
                } else {
                    None
                }.unwrap_or(ValKind::Int);
                return self.compile_list_method(&list_val, method, args, &elem_kind);
            }
            return self.compile_int_method(&obj_val, method, args);
        }

        // Float methods
        if obj_kind == ValKind::Float {
            return self.compile_float_method(&obj_val, method, args);
        }

        // Record method call
        if let ValKind::Record(ref type_name) = obj_kind {
            let mangled_name = format!("{}_{}", type_name, method);
            if let Some(fn_info) = self.functions.get(&mangled_name).cloned() {
                let c_fn_name = Self::mangle_fn_name(&mangled_name);
                let mut arg_strs = vec![obj_val];
                for a in args {
                    let (v, _) = self.compile_expr(a)?;
                    arg_strs.push(v);
                }
                let call = format!("{}({})", c_fn_name, arg_strs.join(", "));
                return Ok((call, fn_info.ret_kind.clone()));
            }
        }

        Err(self.err(format!("unknown method '{}' on {:?}", method, obj_kind)))
    }

    fn compile_int_method(&mut self, val: &str, method: &str, args: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        match method {
            "to_float" => Ok((format!("(double)({})", val), ValKind::Float)),
            "to_str" => Ok((format!("ore_int_to_str({})", val), ValKind::Str)),
            // When Int is actually an opaque Str pointer (from untyped list/map), to_int parses it
            "to_int" => Ok((format!("ore_str_to_int((void*)(intptr_t)({}))", val), ValKind::Int)),
            "abs" => Ok((format!("(({0}) < 0 ? -({0}) : ({0}))", val), ValKind::Int)),
            "pow" => {
                let (exp, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_int_pow({}, {})", val, exp), ValKind::Int))
            }
            "max" => {
                let (other, _) = self.compile_expr(&args[0])?;
                Ok((format!("(({0}) > ({1}) ? ({0}) : ({1}))", val, other), ValKind::Int))
            }
            "min" => {
                let (other, _) = self.compile_expr(&args[0])?;
                Ok((format!("(({0}) < ({1}) ? ({0}) : ({1}))", val, other), ValKind::Int))
            }
            "clamp" => {
                let (lo, _) = self.compile_expr(&args[0])?;
                let (hi, _) = self.compile_expr(&args[1])?;
                let tmp = self.tmp();
                let v = val;
                self.emit(&format!("int64_t {tmp} = ({v} < {lo}) ? {lo} : (({v} > {hi}) ? {hi} : {v});"));
                Ok((tmp, ValKind::Int))
            }
            _ => Err(self.err(format!("unknown Int method '{}'", method))),
        }
    }

    fn compile_float_method(&mut self, val: &str, method: &str, args: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        match method {
            "to_int" => Ok((format!("(int64_t)({})", val), ValKind::Int)),
            "to_str" => Ok((format!("ore_float_to_str({})", val), ValKind::Str)),
            "round" => Ok((format!("ore_math_round({})", val), ValKind::Float)),
            "floor" => Ok((format!("ore_math_floor({})", val), ValKind::Float)),
            "ceil" => Ok((format!("ore_math_ceil({})", val), ValKind::Float)),
            "abs" => Ok((format!("ore_math_abs({})", val), ValKind::Float)),
            "sqrt" => Ok((format!("ore_math_sqrt({})", val), ValKind::Float)),
            "pow" => {
                let (exp, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_math_pow({}, {})", val, exp), ValKind::Float))
            }
            "max" => {
                let (other, _) = self.compile_expr(&args[0])?;
                Ok((format!("(({0}) > ({1}) ? ({0}) : ({1}))", val, other), ValKind::Float))
            }
            "min" => {
                let (other, _) = self.compile_expr(&args[0])?;
                Ok((format!("(({0}) < ({1}) ? ({0}) : ({1}))", val, other), ValKind::Float))
            }
            "format" => {
                let (dec, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_float_format({}, {})", val, dec), ValKind::Str))
            }
            "clamp" => {
                let (lo, _) = self.compile_expr(&args[0])?;
                let (hi, _) = self.compile_expr(&args[1])?;
                let tmp = self.tmp();
                let v = val;
                self.emit(&format!("double {tmp} = ({v} < {lo}) ? {lo} : (({v} > {hi}) ? {hi} : {v});"));
                Ok((tmp, ValKind::Float))
            }
            _ => Err(self.err(format!("unknown Float method '{}'", method))),
        }
    }

    fn compile_channel_method(&mut self, val: &str, method: &str, args: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        match method {
            "send" => {
                let (arg, kind) = self.compile_expr(&args[0])?;
                let i64_val = self.value_to_i64_expr(&arg, &kind);
                self.emit(&format!("ore_channel_send({}, {});", val, i64_val));
                Ok(("0".to_string(), ValKind::Void))
            }
            "recv" => {
                Ok((format!("ore_channel_recv({})", val), ValKind::Int))
            }
            _ => Err(self.err(format!("unknown Channel method '{}'", method))),
        }
    }

    pub(crate) fn compile_index(&mut self, object: &Expr, index: &Expr) -> Result<(String, ValKind), CCodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr(object)?;
        let (idx_val, _) = self.compile_expr(index)?;

        match obj_kind {
            ValKind::List(ref ek) => {
                let elem_kind = ek.as_ref().map(|k| k.as_ref().clone()).unwrap_or(ValKind::Int);
                let raw = format!("ore_list_get({}, {})", obj_val, idx_val);
                let typed = self.coerce_from_i64_expr(&raw, &elem_kind);
                Ok((typed, elem_kind))
            }
            ValKind::Map(_) => {
                let val_kind = obj_kind.map_val_kind().cloned().unwrap_or(ValKind::Int);
                let key = if matches!(self.infer_expr_kind(index), ValKind::Str) {
                    idx_val
                } else {
                    format!("ore_int_to_str({})", idx_val)
                };
                let raw = format!("ore_map_get({}, {})", obj_val, key);
                let typed = self.coerce_from_i64_expr(&raw, &val_kind);
                Ok((typed, val_kind))
            }
            _ => Err(self.err("indexing only supported on lists and maps")),
        }
    }
}
