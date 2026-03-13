use super::*;
use inkwell::values::{BasicValueEnum, FunctionValue};

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_map_method(
        &mut self,
        map_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
        val_kind: &ValKind,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "set" => {
                self.check_arity("set", args, 2)?;
                let key = self.compile_map_key(&args[0], func)?;
                let (val, val_kind) = self.compile_expr_with_kind(&args[1], func)?;
                let i64_val = self.value_to_i64(val)?;
                self.call_rt("ore_map_set", &[map_val.into(), key.into(), i64_val.into()], "")?;
                Ok((map_val, ValKind::map_of(val_kind)))
            }
            "get" => {
                self.check_arity("get", args, 1)?;
                let key = self.compile_map_key(&args[0], func)?;
                let i64_val = self.call_rt("ore_map_get", &[map_val.into(), key.into()], "mget")?;

                // Determine value kind from map tracking
                // Check if the map object is a variable with a tracked value kind
                self.unwrap_map_value(i64_val, val_kind)
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
                Ok((val, ValKind::list_of(ValKind::Str)))
            }
            "values" => {
                let val = self.call_rt("ore_map_values", &[map_val.into()], "mvalues")?;
                // Track the value kind from the map
                let val_kind = val_kind.clone();
                Ok((val, ValKind::list_of(val_kind)))
            }
            "merge" => {
                self.check_arity("merge", args, 1)?;
                let other = self.compile_expr(&args[0], func)?;
                let val = self.call_rt("ore_map_merge", &[map_val.into(), other.into()], "mmerge")?;
                Ok((val, ValKind::Map(Some(Box::new(val_kind.clone())))))
            }
            "clear" => {
                self.call_rt("ore_map_clear", &[map_val.into()], "")?;
                Ok((map_val, ValKind::Map(Some(Box::new(val_kind.clone())))))
            }
            "each" => {
                self.check_arity("map.each()", args, 1)?;
                let val_kind = val_kind.clone();
                let kinds = [ValKind::Str, val_kind];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, "map.each()")?;

                self.call_rt("ore_map_each", &[map_val.into(), fn_ptr.into(), env_ptr.into()], "")?;
                Ok(self.void_result())
            }
            "map" => {
                self.check_arity("map.map()", args, 1)?;
                let kinds = [ValKind::Str, val_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, "map.map()")?;

                let val = self.call_rt("ore_map_map_values", &[map_val.into(), fn_ptr.into(), env_ptr.into()], "mmap")?;
                Ok((val, ValKind::Map(Some(Box::new(val_kind.clone())))))
            }
            "filter" => {
                self.check_arity("map.filter()", args, 1)?;
                let kinds = [ValKind::Str, val_kind.clone()];
                let (fn_ptr, env_ptr, _) = self.resolve_lambda_arg(&args[0], &kinds, "map.filter()")?;

                let val = self.call_rt("ore_map_filter", &[map_val.into(), fn_ptr.into(), env_ptr.into()], "mfilter")?;
                Ok((val, ValKind::Map(Some(Box::new(val_kind.clone())))))
            }
            "get_or" => {
                self.check_arity("get_or", args, 2)?;
                let key = self.compile_map_key(&args[0], func)?;
                let (default_val, _default_kind) = self.compile_expr_with_kind(&args[1], func)?;
                let default_i64 = self.value_to_i64(default_val)?;
                let i64_val = self.call_rt("ore_map_get_or", &[map_val.into(), key.into(), default_i64.into()], "mgetor")?;
                self.unwrap_map_value(i64_val, val_kind)
            }
            "entries" => {
                let val = self.call_rt("ore_map_entries", &[map_val.into()], "mentries")?;
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
        let map_ptr = self.call_rt("ore_map_new", &[], "map")?.into_pointer_value();

        let mut first_val_kind = None;
        for (key, value) in entries {
            let key_val = self.compile_map_key(key, func)?;
            let (val, val_kind) = self.compile_expr_with_kind(value, func)?;
            if first_val_kind.is_none() {
                first_val_kind = Some(val_kind.clone());
            }
            // Compute kind tag for runtime type tracking
            let kind_tag = self.valkind_to_tag(&val_kind);
            let kind_const = self.context.i8_type().const_int(kind_tag as u64, false);
            let i64_val = self.value_to_i64(val)?;
            self.call_rt("ore_map_set_typed", &[map_ptr.into(), key_val.into(), i64_val.into(), kind_const.into()], "")?;
        }

        Ok((map_ptr.into(), ValKind::Map(first_val_kind.map(Box::new))))
    }

    /// Convert a raw i64 map value back to the correct type based on tracked value kind.
    fn unwrap_map_value(
        &mut self,
        i64_val: BasicValueEnum<'ctx>,
        val_kind: &ValKind,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let typed = self.coerce_from_i64(i64_val, val_kind)?;
        Ok((typed, val_kind.clone()))
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
