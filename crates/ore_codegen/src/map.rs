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
                if args.len() != 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "set takes 2 arguments (key, value)".into() });
                }
                let key = self.compile_map_key(&args[0], func)?;
                let (val, val_kind) = self.compile_expr_with_kind(&args[1], func)?;
                let i64_val = match val_kind {
                    ValKind::Int => val.into_int_value(),
                    ValKind::Bool => {
                        bld!(self.builder.build_int_z_extend(
                            val.into_int_value(), self.context.i64_type(), "bool_to_i64"
                        ))?
                    }
                    ValKind::Str | ValKind::List(_) | ValKind::Map => {
                        bld!(self.builder.build_ptr_to_int(
                            val.into_pointer_value(), self.context.i64_type(), "ptr_to_i64"
                        ))?
                    }
                    _ => val.into_int_value(),
                };
                let rt = self.rt("ore_map_set")?;
                bld!(self.builder.build_call(rt, &[map_val.into(), key.into(), i64_val.into()], ""))?;
                // Track value kind for later retrieval
                self.last_map_val_kind = Some(val_kind);
                Ok((map_val, ValKind::Map))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "get takes 1 argument (key)".into() });
                }
                let key = self.compile_map_key(&args[0], func)?;
                let rt = self.rt("ore_map_get")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), key.into()], "mget"))?;
                let i64_val = self.call_result_to_value(result)?;

                // Determine value kind from map tracking
                // Check if the map object is a variable with a tracked value kind
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                match &val_kind {
                    ValKind::Str => {
                        // Convert i64 back to pointer
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            i64_val.into_int_value(), self.ptr_type(), "i64_to_ptr"
                        ))?;
                        Ok((ptr.into(), ValKind::Str))
                    }
                    ValKind::List(_) => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            i64_val.into_int_value(), self.ptr_type(), "i64_to_ptr"
                        ))?;
                        Ok((ptr.into(), val_kind))
                    }
                    _ => Ok((i64_val, val_kind))
                }
            }
            "contains" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "contains takes 1 argument (key)".into() });
                }
                let key = self.compile_map_key(&args[0], func)?;
                let rt = self.rt("ore_map_contains")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), key.into()], "mcontains"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    i8_val,
                    self.context.i8_type().const_int(0, false),
                    "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "len" => {
                let rt = self.rt("ore_map_len")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into()], "mlen"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "remove" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "remove takes 1 argument (key)".into() });
                }
                let key = self.compile_map_key(&args[0], func)?;
                let rt = self.rt("ore_map_remove")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), key.into()], "mremove"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "keys" => {
                let rt = self.rt("ore_map_keys")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into()], "mkeys"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok((val, ValKind::list_of(ValKind::Str)))
            }
            "values" => {
                let rt = self.rt("ore_map_values")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into()], "mvalues"))?;
                let val = self.call_result_to_value(result)?;
                // Track the value kind from the map
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                self.last_list_elem_kind = Some(val_kind.clone());
                Ok((val, ValKind::list_of(val_kind)))
            }
            "merge" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "merge takes 1 argument (other map)".into() });
                }
                let other = self.compile_expr(&args[0], func)?;
                let rt = self.rt("ore_map_merge")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), other.into()], "mmerge"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Map))
            }
            "clear" => {
                let rt = self.rt("ore_map_clear")?;
                bld!(self.builder.build_call(rt, &[map_val.into()], ""))?;
                Ok((map_val, ValKind::Map))
            }
            "each" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "map.each() takes 1 argument (lambda)".into() });
                }
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        let kinds = vec![ValKind::Str, val_kind.clone()];
                        self.compile_lambda_with_kinds(params, body, func, Some(&kinds))?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: Some(self.current_line), msg: "map.each() requires a lambda".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let rt = self.rt("ore_map_each")?;
                bld!(self.builder.build_call(rt, &[map_val.into(), fn_ptr.into(), env_ptr.into()], ""))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            "map" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "map.map() takes 1 argument (lambda)".into() });
                }
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        let kinds = vec![ValKind::Str, val_kind.clone()];
                        self.compile_lambda_with_kinds(params, body, func, Some(&kinds))?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: Some(self.current_line), msg: "map.map() requires a lambda".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let rt = self.rt("ore_map_map_values")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), fn_ptr.into(), env_ptr.into()], "mmap"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Map))
            }
            "filter" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "map.filter() takes 1 argument (lambda)".into() });
                }
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        let kinds = vec![ValKind::Str, val_kind.clone()];
                        self.compile_lambda_with_kinds(params, body, func, Some(&kinds))?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: Some(self.current_line), msg: "map.filter() requires a lambda".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let rt = self.rt("ore_map_filter")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), fn_ptr.into(), env_ptr.into()], "mfilter"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Map))
            }
            "get_or" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: Some(self.current_line), msg: "get_or takes 2 arguments (key, default)".into() });
                }
                let key = self.compile_map_key(&args[0], func)?;
                let (default_val, default_kind) = self.compile_expr_with_kind(&args[1], func)?;
                let default_i64 = match default_kind {
                    ValKind::Str | ValKind::List(_) | ValKind::Map => {
                        bld!(self.builder.build_ptr_to_int(
                            default_val.into_pointer_value(), self.context.i64_type(), "def2i"
                        ))?
                    }
                    _ => default_val.into_int_value(),
                };
                let rt = self.rt("ore_map_get_or")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), key.into(), default_i64.into()], "mgetor"))?;
                let i64_val = self.call_result_to_value(result)?;
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                match &val_kind {
                    ValKind::Str => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            i64_val.into_int_value(), self.ptr_type(), "i64_to_ptr"
                        ))?;
                        Ok((ptr.into(), ValKind::Str))
                    }
                    ValKind::List(_) => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            i64_val.into_int_value(), self.ptr_type(), "i64_to_ptr"
                        ))?;
                        Ok((ptr.into(), val_kind))
                    }
                    _ => Ok((i64_val, val_kind))
                }
            }
            "entries" => {
                let rt = self.rt("ore_map_entries")?;
                let result = bld!(self.builder.build_call(rt, &[map_val.into()], "mentries"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::List(None));
                Ok((val, ValKind::list_of(ValKind::List(None))))
            }
            _ => Err(Self::unknown_method_error("Map", method, &[
                "get", "set", "contains", "len", "remove", "keys", "values",
                "merge", "clear", "each", "map", "filter", "get_or",
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
            // Convert value to i64 for storage
            let i64_val = match val_kind {
                ValKind::Int => val.into_int_value(),
                ValKind::Bool => {
                    bld!(self.builder.build_int_z_extend(
                        val.into_int_value(),
                        self.context.i64_type(),
                        "bool_to_i64"
                    ))?
                }
                ValKind::Str | ValKind::List(_) | ValKind::Map => {
                    bld!(self.builder.build_ptr_to_int(
                        val.into_pointer_value(),
                        self.context.i64_type(),
                        "ptr_to_i64"
                    ))?
                }
                _ => val.into_int_value(),
            };
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
