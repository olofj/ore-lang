use super::*;

impl CCodeGen {
    pub(crate) fn compile_map_method(&mut self, map_val: &str, method: &str, args: &[Expr], val_kind: &ValKind) -> Result<(String, ValKind), CCodeGenError> {
        match method {
            "set" => {
                let (key, _) = self.compile_expr(&args[0])?;
                let (val, vk) = self.compile_expr(&args[1])?;
                let i64_val = self.value_to_i64_expr(&val, &vk);
                self.emit(&format!("ore_map_set({}, {}, {});", map_val, key, i64_val));
                Ok((map_val.to_string(), ValKind::map_of(vk)))
            }
            "get" => {
                let (key, _) = self.compile_expr(&args[0])?;
                let raw = format!("ore_map_get({}, {})", map_val, key);
                Ok((self.coerce_from_i64_expr(&raw, val_kind), val_kind.clone()))
            }
            "contains" => {
                let (key, _) = self.compile_expr(&args[0])?;
                Ok((format!("(ore_map_contains({}, {}) != 0)", map_val, key), ValKind::Bool))
            }
            "len" => Ok((format!("ore_map_len({})", map_val), ValKind::Int)),
            "remove" => {
                let (key, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_map_remove({}, {})", map_val, key), ValKind::Int))
            }
            "keys" => Ok((format!("ore_map_keys({})", map_val), ValKind::list_of(ValKind::Str))),
            "values" => Ok((format!("ore_map_values({})", map_val), ValKind::list_of(val_kind.clone()))),
            "entries" => Ok((format!("ore_map_entries({})", map_val), ValKind::list_of(ValKind::List(None)))),
            "merge" => {
                let (other, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_map_merge({}, {})", map_val, other), ValKind::Map(Some(Box::new(val_kind.clone())))))
            }
            "clear" => {
                self.emit(&format!("ore_map_clear({});", map_val));
                Ok((map_val.to_string(), ValKind::Map(Some(Box::new(val_kind.clone())))))
            }
            "get_or" => {
                let (key, _) = self.compile_expr(&args[0])?;
                let (default, _) = self.compile_expr(&args[1])?;
                let default_i64 = self.value_to_i64_expr(&default, val_kind);
                let raw = format!("ore_map_get_or({}, {}, {})", map_val, key, default_i64);
                Ok((self.coerce_from_i64_expr(&raw, val_kind), val_kind.clone()))
            }
            "each" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg(&args[0])?;
                self.emit(&format!("ore_map_each({}, {}, {});", map_val, fn_ptr, env_ptr));
                Ok(("0".to_string(), ValKind::Void))
            }
            "map" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg(&args[0])?;
                Ok((format!("ore_map_map_values({}, {}, {})", map_val, fn_ptr, env_ptr), ValKind::Map(Some(Box::new(val_kind.clone())))))
            }
            "filter" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg(&args[0])?;
                Ok((format!("ore_map_filter({}, {}, {})", map_val, fn_ptr, env_ptr), ValKind::Map(Some(Box::new(val_kind.clone())))))
            }
            _ => Err(self.err(format!("unknown Map method '{}'", method))),
        }
    }

    pub(crate) fn compile_map_lit(&mut self, entries: &[(Expr, Expr)]) -> Result<(String, ValKind), CCodeGenError> {
        let tmp = self.tmp();
        self.emit(&format!("void* {} = ore_map_new();", tmp));

        let mut first_val_kind = None;
        for (key, value) in entries {
            let (key_val, key_kind) = self.compile_expr(key)?;
            let key_str = if key_kind == ValKind::Str { key_val } else { self.value_to_str_expr(&key_val, &key_kind) };
            let (val, vk) = self.compile_expr(value)?;
            if first_val_kind.is_none() {
                first_val_kind = Some(vk.clone());
            }
            let kind_tag = Self::valkind_to_tag(&vk);
            let i64_val = self.value_to_i64_expr(&val, &vk);
            self.emit(&format!("ore_map_set_typed({}, {}, {}, {});", tmp, key_str, i64_val, kind_tag));
        }

        Ok((tmp, ValKind::Map(first_val_kind.map(Box::new))))
    }
}
