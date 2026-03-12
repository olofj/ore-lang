use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue};

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_map_method(
        &mut self,
        map_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "set" => {
                self.check_arity("set", args, 2)?;
                let key = self.compile_map_key(&args[0], func)?;
                let (val, val_kind) = self.compile_expr_with_kind(&args[1], func)?;
                let i64_val = self.value_to_i64(val)?;
                let rt = self.rt("ore_map_set")?;
                bld!(self.builder.build_call(rt, &[map_val.into(), key.into(), i64_val.into()], ""))?;
                // Track value kind for later retrieval
                self.last_map_val_kind = Some(val_kind);
                Ok((map_val, ValKind::Map))
            }
            "get" => {
                self.check_arity("get", args, 1)?;
                let key = self.compile_map_key(&args[0], func)?;
                let i64_val = self.call_rt("ore_map_get", &[map_val.into(), key.into()], "mget")?;

                // Determine value kind from map tracking
                // Check if the map object is a variable with a tracked value kind
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                match &val_kind {
                    ValKind::Str => {
                        // Convert i64 back to pointer
                        let ptr = self.i64_to_ptr(i64_val.into_int_value())?;
                        Ok((ptr.into(), ValKind::Str))
                    }
                    ValKind::List(_) => {
                        let ptr = self.i64_to_ptr(i64_val.into_int_value())?;
                        Ok((ptr.into(), val_kind))
                    }
                    _ => Ok((i64_val, val_kind))
                }
            }
            "contains" => {
                self.check_arity("contains", args, 1)?;
                let key = self.compile_map_key(&args[0], func)?;
                let i8_val = self.call_rt("ore_map_contains", &[map_val.into(), key.into()], "mcontains")?.into_int_value();
                let bool_val = self.i8_to_bool(i8_val)?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "len" => {
                let val = self.call_rt("ore_map_len", &[map_val.into()], "mlen")?;
                Ok((val, ValKind::Int))
            }
            "remove" => {
                self.check_arity("remove", args, 1)?;
                let key = self.compile_map_key(&args[0], func)?;
                let val = self.call_rt("ore_map_remove", &[map_val.into(), key.into()], "mremove")?;
                Ok((val, ValKind::Int))
            }
            "keys" => {
                let val = self.call_rt("ore_map_keys", &[map_val.into()], "mkeys")?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok((val, ValKind::list_of(ValKind::Str)))
            }
            "values" => {
                let val = self.call_rt("ore_map_values", &[map_val.into()], "mvalues")?;
                // Track the value kind from the map
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                self.last_list_elem_kind = Some(val_kind.clone());
                Ok((val, ValKind::list_of(val_kind)))
            }
            "merge" => {
                self.check_arity("merge", args, 1)?;
                let other = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_map_merge", &[map_val.into(), other.into()], "mmerge")?;
                Ok((val, ValKind::Map))
            }
            "clear" => {
                let rt = self.rt("ore_map_clear")?;
                bld!(self.builder.build_call(rt, &[map_val.into()], ""))?;
                Ok((map_val, ValKind::Map))
            }
            "each" => {
                self.check_arity("map.each()", args, 1)?;
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                let kinds = [ValKind::Str, val_kind];
                let (lambda_fn, env_ptr) = self.resolve_lambda_arg(&args[0], &kinds, "map.each()", false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let rt = self.rt("ore_map_each")?;
                bld!(self.builder.build_call(rt, &[map_val.into(), fn_ptr.into(), env_ptr.into()], ""))?;
                Ok(self.void_result())
            }
            "map" => {
                self.check_arity("map.map()", args, 1)?;
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                let kinds = [ValKind::Str, val_kind];
                let (lambda_fn, env_ptr) = self.resolve_lambda_arg(&args[0], &kinds, "map.map()", false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let val = self.call_rt("ore_map_map_values", &[map_val.into(), fn_ptr.into(), env_ptr.into()], "mmap")?;
                Ok((val, ValKind::Map))
            }
            "filter" => {
                self.check_arity("map.filter()", args, 1)?;
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                let kinds = [ValKind::Str, val_kind];
                let (lambda_fn, env_ptr) = self.resolve_lambda_arg(&args[0], &kinds, "map.filter()", false)?;
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let val = self.call_rt("ore_map_filter", &[map_val.into(), fn_ptr.into(), env_ptr.into()], "mfilter")?;
                Ok((val, ValKind::Map))
            }
            "get_or" => {
                self.check_arity("get_or", args, 2)?;
                let key = self.compile_map_key(&args[0], func)?;
                let (default_val, default_kind) = self.compile_expr_with_kind(&args[1], func)?;
                let default_i64 = match default_kind {
                    ValKind::Str | ValKind::List(_) | ValKind::Map => {
                        self.ptr_to_i64(default_val.into_pointer_value())?
                    }
                    _ => default_val.into_int_value(),
                };
                let i64_val = self.call_rt("ore_map_get_or", &[map_val.into(), key.into(), default_i64.into()], "mgetor")?;
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                match &val_kind {
                    ValKind::Str => {
                        let ptr = self.i64_to_ptr(i64_val.into_int_value())?;
                        Ok((ptr.into(), ValKind::Str))
                    }
                    ValKind::List(_) => {
                        let ptr = self.i64_to_ptr(i64_val.into_int_value())?;
                        Ok((ptr.into(), val_kind))
                    }
                    _ => Ok((i64_val, val_kind))
                }
            }
            "entries" => {
                let val = self.call_rt("ore_map_entries", &[map_val.into()], "mentries")?;
                self.last_list_elem_kind = Some(ValKind::List(None));
                Ok((val, ValKind::list_of(ValKind::List(None))))
            }
            _ => Err(Self::unknown_method_error("Map", method, &[
                "get", "set", "contains", "len", "remove", "keys", "values",
                "merge", "clear", "each", "map", "filter", "get_or", "entries",
            ])),
        }
    }

    pub(crate) fn compile_map_lit(
        &mut self,
        entries: &[(Expr, Expr)],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let map_new = self.rt("ore_map_new")?;
        let map_set_typed = self.rt("ore_map_set_typed")?;

        let map_result = bld!(self.builder.build_call(map_new, &[], "map"))?;
        let map_ptr = self.call_result_to_value(map_result)?.into_pointer_value();

        let mut first_val_kind = None;
        for (key, value) in entries {
            let (raw_key, key_kind) = self.compile_expr_with_kind(key, func)?;
            // Map keys must be strings — convert non-string keys automatically
            let key_val = if key_kind == ValKind::Str {
                raw_key
            } else {
                self.value_to_str(raw_key, key_kind)?.into()
            };
            let (val, val_kind) = self.compile_expr_with_kind(value, func)?;
            if first_val_kind.is_none() {
                first_val_kind = Some(val_kind.clone());
            }
            // Compute kind tag for runtime type tracking
            let kind_tag = self.valkind_to_tag(&val_kind);
            let kind_const = self.context.i8_type().const_int(kind_tag as u64, false);
            let i64_val = self.value_to_i64(val)?;
            bld!(self.builder.build_call(
                map_set_typed,
                &[map_ptr.into(), key_val.into(), i64_val.into(), kind_const.into()],
                ""
            ))?;
        }

        self.last_map_val_kind = first_val_kind;
        Ok((map_ptr.into(), ValKind::Map))
    }

    /// Compile a map key expression, converting non-string keys to strings.
    pub(crate) fn compile_map_key(
        &mut self,
        expr: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let (val, kind) = self.compile_expr_with_kind(expr, func)?;
        if kind == ValKind::Str {
            Ok(val)
        } else {
            Ok(self.value_to_str(val, kind)?.into())
        }
    }

}
