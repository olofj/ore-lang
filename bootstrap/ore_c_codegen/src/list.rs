use super::*;

impl CCodeGen {
    pub(crate) fn compile_list_method(&mut self, list_val: &str, method: &str, args: &[Expr], elem_kind: &ValKind) -> Result<(String, ValKind), CCodeGenError> {
        match method {
            "len" => Ok((format!("ore_list_len({})", list_val), ValKind::Int)),
            "is_empty" => Ok((format!("(ore_list_len({}) == 0)", list_val), ValKind::Bool)),
            "clear" => {
                self.emit(&format!("ore_list_clear({});", list_val));
                Ok((list_val.to_string(), ValKind::list_of(elem_kind.clone())))
            }
            "push" => {
                let (arg, arg_kind) = self.compile_expr(&args[0])?;
                let i64_val = self.value_to_i64_expr(&arg, &arg_kind);
                // Check for dynamic kind tag (from untyped list element access)
                if let Some(kind_var) = self.dynamic_kind_exprs.get(&arg).cloned() {
                    self.emit(&format!("ore_list_push_typed({}, {}, {});", list_val, i64_val, kind_var));
                } else {
                    let tag = Self::valkind_to_tag(&arg_kind);
                    if tag == 0 {
                        // KIND_INT — use untyped push (no kinds-array overhead)
                        self.emit(&format!("ore_list_push({}, {});", list_val, i64_val));
                    } else {
                        self.emit(&format!("ore_list_push_typed({}, {}, {});", list_val, i64_val, tag));
                    }
                }
                Ok((list_val.to_string(), ValKind::list_of(arg_kind)))
            }
            "pop" => {
                let raw = format!("ore_list_pop({})", list_val);
                Ok((self.coerce_from_i64_expr(&raw, elem_kind), elem_kind.clone()))
            }
            "get" => {
                let (idx, _) = self.compile_expr(&args[0])?;
                let raw = format!("ore_list_get({}, {})", list_val, idx);
                Ok((self.coerce_from_i64_expr(&raw, elem_kind), elem_kind.clone()))
            }
            "set" => {
                let (idx, _) = self.compile_expr(&args[0])?;
                let (val, _) = self.compile_expr(&args[1])?;
                let i64_val = self.value_to_i64_expr(&val, elem_kind);
                self.emit(&format!("ore_list_set({}, {}, {});", list_val, idx, i64_val));
                Ok((list_val.to_string(), ValKind::list_of(elem_kind.clone())))
            }
            "get_or" => {
                let (idx, _) = self.compile_expr(&args[0])?;
                let (default, _) = self.compile_expr(&args[1])?;
                let default_i64 = self.value_to_i64_expr(&default, elem_kind);
                let raw = format!("ore_list_get_or({}, {}, {})", list_val, idx, default_i64);
                Ok((self.coerce_from_i64_expr(&raw, elem_kind), elem_kind.clone()))
            }
            "insert" => {
                let (idx, _) = self.compile_expr(&args[0])?;
                let (val, _) = self.compile_expr(&args[1])?;
                let i64_val = self.value_to_i64_expr(&val, elem_kind);
                self.emit(&format!("ore_list_insert({}, {}, {});", list_val, idx, i64_val));
                Ok((list_val.to_string(), ValKind::list_of(elem_kind.clone())))
            }
            "remove_at" => {
                let (idx, _) = self.compile_expr(&args[0])?;
                let raw = format!("ore_list_remove_at({}, {})", list_val, idx);
                Ok((self.coerce_from_i64_expr(&raw, elem_kind), elem_kind.clone()))
            }
            "map" | "filter" | "flat_map" | "take_while" | "drop_while" => {
                let (fn_ptr, env_ptr, ret_kind) = self.compile_lambda_arg_full(&args[0], &[elem_kind.clone()])?;
                let rt = format!("ore_list_{}", method);
                let result = format!("{}({}, {}, {})", rt, list_val, fn_ptr, env_ptr);
                let result_elem = if method == "flat_map" {
                    // flat_map's lambda returns a list; the result element is the inner type
                    ret_kind.list_elem_kind().cloned().unwrap_or(ret_kind)
                } else if method == "map" {
                    ret_kind
                } else {
                    elem_kind.clone()
                };
                Ok((result, ValKind::list_of(result_elem)))
            }
            "each" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                self.emit(&format!("ore_list_each({}, {}, {});", list_val, fn_ptr, env_ptr));
                Ok(("0".to_string(), ValKind::Void))
            }
            "tap" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                Ok((format!("ore_list_tap({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::list_of(elem_kind.clone())))
            }
            "find_index" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                Ok((format!("ore_list_find_index({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::Int))
            }
            "find" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                let raw = format!("ore_list_find({}, {}, {}, 0)", list_val, fn_ptr, env_ptr);
                Ok((self.coerce_from_i64_expr(&raw, elem_kind), elem_kind.clone()))
            }
            "fold" => {
                let (init, _) = self.compile_expr(&args[0])?;
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[1], &[ValKind::Int, elem_kind.clone()])?;
                Ok((format!("ore_list_fold({}, {}, {}, {})", list_val, init, fn_ptr, env_ptr), ValKind::Int))
            }
            "reduce" => {
                if args.len() == 1 {
                    let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[ValKind::Int, elem_kind.clone()])?;
                    Ok((format!("ore_list_reduce1({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::Int))
                } else {
                    let (init, _) = self.compile_expr(&args[0])?;
                    let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[1], &[ValKind::Int, elem_kind.clone()])?;
                    Ok((format!("ore_list_fold({}, {}, {}, {})", list_val, init, fn_ptr, env_ptr), ValKind::Int))
                }
            }
            "scan" => {
                let (init, _) = self.compile_expr(&args[0])?;
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[1], &[ValKind::Int, elem_kind.clone()])?;
                Ok((format!("ore_list_scan({}, {}, {}, {})", list_val, init, fn_ptr, env_ptr), ValKind::list_of(ValKind::Int)))
            }
            "any" | "all" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                Ok((format!("(ore_list_{}({}, {}, {}) != 0)", method, list_val, fn_ptr, env_ptr), ValKind::Bool))
            }
            "count" => {
                if args.is_empty() {
                    return Ok((format!("ore_list_len({})", list_val), ValKind::Int));
                }
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                Ok((format!("ore_list_count({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::Int))
            }
            "sort" => {
                if args.is_empty() {
                    let rt = match elem_kind {
                        ValKind::Str => "ore_list_sort_str",
                        ValKind::Float => "ore_list_sort_float",
                        _ => "ore_list_sort",
                    };
                    return Ok((format!("{}({})", rt, list_val), ValKind::list_of(elem_kind.clone())));
                }
                let ek = elem_kind.clone();
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[ek.clone(), ek])?;
                Ok((format!("ore_list_sort_by({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::list_of(elem_kind.clone())))
            }
            "sort_by" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                Ok((format!("ore_list_sort_by_key({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::list_of(elem_kind.clone())))
            }
            "reverse" => Ok((format!("ore_list_reverse_new({})", list_val), ValKind::list_of(elem_kind.clone()))),
            "contains" => {
                let (val, _) = self.compile_expr(&args[0])?;
                let rt = if matches!(elem_kind, ValKind::Str) { "ore_list_contains_str" } else { "ore_list_contains" };
                Ok((format!("({rt}({list_val}, {val}) != 0)"), ValKind::Bool))
            }
            "join" => {
                let (sep, _) = self.compile_expr(&args[0])?;
                let rt = match elem_kind {
                    ValKind::Str => "ore_list_join_str",
                    ValKind::Float => "ore_list_join_float",
                    ValKind::Int => "ore_list_join_int",
                    _ => "ore_list_join",
                };
                Ok((format!("{}({}, {})", rt, list_val, sep), ValKind::Str))
            }
            "take" | "skip" | "step" => {
                let (n, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_list_{}({}, {})", method, list_val, n), ValKind::list_of(elem_kind.clone())))
            }
            "sum" | "product" => {
                let (rt, kind) = if matches!(elem_kind, ValKind::Float) {
                    (format!("ore_list_{}_float", method), ValKind::Float)
                } else {
                    (format!("ore_list_{}", method), ValKind::Int)
                };
                Ok((format!("{}({})", rt, list_val), kind))
            }
            "average" => {
                let rt = if matches!(elem_kind, ValKind::Float) { "ore_list_average_float" } else { "ore_list_average" };
                Ok((format!("{}({})", rt, list_val), ValKind::Float))
            }
            "min" | "max" => {
                let (rt, kind) = match elem_kind {
                    ValKind::Float => (format!("ore_list_{}_float", method), ValKind::Float),
                    ValKind::Str => (format!("ore_list_{}_str", method), ValKind::Str),
                    _ => (format!("ore_list_{}", method), ValKind::Int),
                };
                Ok((format!("{}({})", rt, list_val), kind))
            }
            "first" => Ok((self.coerce_from_i64_expr(&format!("ore_list_get({}, 0)", list_val), elem_kind), elem_kind.clone())),
            "last" => Ok((self.coerce_from_i64_expr(&format!("ore_list_get({}, -1)", list_val), elem_kind), elem_kind.clone())),
            "zip" => {
                let (other, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_list_zip({}, {})", list_val, other), ValKind::list_of(ValKind::List(None))))
            }
            "enumerate" => Ok((format!("ore_list_enumerate({})", list_val), ValKind::list_of(ValKind::List(None)))),
            "slice" => {
                let (start, _) = self.compile_expr(&args[0])?;
                let (end, _) = self.compile_expr(&args[1])?;
                Ok((format!("ore_list_slice({}, {}, {})", list_val, start, end), ValKind::list_of(elem_kind.clone())))
            }
            "index_of" => {
                let (val, _) = self.compile_expr(&args[0])?;
                let rt = if matches!(elem_kind, ValKind::Str) { "ore_list_index_of_str" } else { "ore_list_index_of" };
                Ok((format!("{}({}, {})", rt, list_val, val), ValKind::Int))
            }
            "unique" => {
                let rt = if matches!(elem_kind, ValKind::Str) { "ore_list_unique_str" } else { "ore_list_unique" };
                Ok((format!("{}({})", rt, list_val), ValKind::list_of(elem_kind.clone())))
            }
            "flatten" => Ok((format!("ore_list_flatten({})", list_val), ValKind::List(None))),
            "window" | "chunks" => {
                let (n, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_list_{}({}, {})", method, list_val, n), ValKind::list_of(ValKind::List(None))))
            }
            "count_by" | "group_by" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                let rt = if method == "count_by" { "ore_list_count_by" } else { "ore_list_group_by" };
                let val_kind = if method == "count_by" { ValKind::Int } else { ValKind::list_of(elem_kind.clone()) };
                Ok((format!("{}({}, {}, {})", rt, list_val, fn_ptr, env_ptr), ValKind::map_of(val_kind)))
            }
            "to_map" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                Ok((format!("ore_list_to_map({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::map_of(elem_kind.clone())))
            }
            "partition" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                Ok((format!("ore_list_partition({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::list_of(ValKind::list_of(elem_kind.clone()))))
            }
            "zip_with" => {
                let (other, _) = self.compile_expr(&args[0])?;
                let ek = elem_kind.clone();
                let (fn_ptr, env_ptr, ret_kind) = self.compile_lambda_arg_full(&args[1], &[ek.clone(), ek])?;
                Ok((format!("ore_list_zip_with({}, {}, {}, {})", list_val, other, fn_ptr, env_ptr), ValKind::list_of(ret_kind)))
            }
            "unique_by" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                Ok((format!("ore_list_unique_by({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::list_of(elem_kind.clone())))
            }
            "min_by" | "max_by" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                let rt = if method == "min_by" { "ore_list_min_by" } else { "ore_list_max_by" };
                let raw = format!("{}({}, {}, {})", rt, list_val, fn_ptr, env_ptr);
                Ok((self.coerce_from_i64_expr(&raw, elem_kind), elem_kind.clone()))
            }
            "map_with_index" => {
                let (fn_ptr, env_ptr, ret_kind) = self.compile_lambda_arg_full(&args[0], &[ValKind::Int, elem_kind.clone()])?;
                Ok((format!("ore_list_map_with_index({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::list_of(ret_kind)))
            }
            "each_with_index" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[ValKind::Int, elem_kind.clone()])?;
                self.emit(&format!("ore_list_each_with_index({}, {}, {});", list_val, fn_ptr, env_ptr));
                Ok(("0".to_string(), ValKind::Void))
            }
            "par_map" => {
                let (fn_ptr, env_ptr, ret_kind) = self.compile_lambda_arg_full(&args[0], &[elem_kind.clone()])?;
                Ok((format!("ore_list_par_map({}, {}, {})", list_val, fn_ptr, env_ptr), ValKind::list_of(ret_kind)))
            }
            "par_each" => {
                let (fn_ptr, env_ptr) = self.compile_lambda_arg_with_kinds(&args[0], &[elem_kind.clone()])?;
                self.emit(&format!("ore_list_par_each({}, {}, {});", list_val, fn_ptr, env_ptr));
                Ok(("0".to_string(), ValKind::Void))
            }
            "intersperse" => {
                let (sep, sep_kind) = self.compile_expr(&args[0])?;
                let sep_i64 = self.value_to_i64_expr(&sep, &sep_kind);
                Ok((format!("ore_list_intersperse({}, {})", list_val, sep_i64), ValKind::list_of(elem_kind.clone())))
            }
            "dedup" => Ok((format!("ore_list_dedup({})", list_val), ValKind::list_of(elem_kind.clone()))),
            "frequencies" => {
                let kind_tag = Self::valkind_to_tag(elem_kind);
                Ok((format!("ore_list_frequencies({}, {})", list_val, kind_tag), ValKind::map_of(ValKind::Int)))
            }
            _ => Err(self.err(format!("unknown List method '{}'", method))),
        }
    }

    pub(crate) fn compile_list_lit(&mut self, elements: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        let tmp = self.tmp();
        self.emit(&format!("void* {} = ore_list_new();", tmp));

        let mut elem_kind: std::option::Option<ValKind> = None;
        for elem in elements {
            let (val, kind) = self.compile_expr(elem)?;
            let i64_val = self.value_to_i64_expr(&val, &kind);
            // Check for dynamic kind tag (from untyped list element access)
            if let Some(kind_var) = self.dynamic_kind_exprs.get(&val).cloned() {
                self.emit(&format!("ore_list_push_typed({}, {}, {});", tmp, i64_val, kind_var));
            } else {
                let tag = Self::valkind_to_tag(&kind);
                if tag == 0 {
                    self.emit(&format!("ore_list_push({}, {});", tmp, i64_val));
                } else {
                    self.emit(&format!("ore_list_push_typed({}, {}, {});", tmp, i64_val, tag));
                }
            }
            elem_kind = Some(kind);
        }

        Ok((tmp, ValKind::List(elem_kind.map(Box::new))))
    }

    pub(crate) fn compile_list_comp(&mut self, body: &Expr, var: &str, iterable: &Expr, cond: std::option::Option<&Expr>) -> Result<(String, ValKind), CCodeGenError> {
        let result_tmp = self.tmp();
        self.emit(&format!("void* {} = ore_list_new();", result_tmp));

        let (iter_val, iter_kind) = self.compile_expr(iterable)?;
        let elem_kind = iter_kind.list_elem_kind().cloned().unwrap_or(ValKind::Int);

        let len_tmp = self.tmp();
        let idx = self.tmp();
        self.emit(&format!("int64_t {} = ore_list_len({});", len_tmp, iter_val));

        let c_type = self.kind_to_c_type_str(&elem_kind);
        self.emit(&format!("for (int64_t {} = 0; {} < {}; {}++) {{", idx, idx, len_tmp, idx));
        self.indent += 1;

        let raw = format!("ore_list_get({}, {})", iter_val, idx);
        let typed = self.coerce_from_i64_expr(&raw, &elem_kind);
        self.emit(&format!("{} {} = {};", c_type, var, typed));
        self.variables.insert(var.to_string(), VarInfo { c_name: var.to_string(), kind: elem_kind, is_mutable: false });

        if let Some(c) = cond {
            let (cond_val, _) = self.compile_expr(c)?;
            self.emit(&format!("if (!({cond_val})) continue;"));
        }

        let (body_val, body_kind) = self.compile_expr(body)?;
        let push_val = self.value_to_i64_expr(&body_val, &body_kind);
        if let Some(kind_var) = self.dynamic_kind_exprs.get(&body_val).cloned() {
            self.emit(&format!("ore_list_push_typed({}, {}, {});", result_tmp, push_val, kind_var));
        } else {
            let tag = Self::valkind_to_tag(&body_kind);
            if tag == 0 {
                self.emit(&format!("ore_list_push({}, {});", result_tmp, push_val));
            } else {
                self.emit(&format!("ore_list_push_typed({}, {}, {});", result_tmp, push_val, tag));
            }
        }

        self.indent -= 1;
        self.emit("}");

        Ok((result_tmp, ValKind::list_of(body_kind)))
    }

    /// Compile a lambda or function reference argument for higher-order list methods.
    /// Returns (fn_ptr_expr, env_ptr_expr).
    pub(crate) fn compile_lambda_arg(&mut self, arg: &Expr) -> Result<(String, String), CCodeGenError> {
        self.compile_lambda_arg_with_kinds(arg, &[ValKind::Int])
    }

    /// Compile a lambda arg with known parameter kinds.
    /// Returns (fn_ptr_expr, env_ptr_expr) — for closures, env_ptr points to captures struct.
    pub(crate) fn compile_lambda_arg_with_kinds(&mut self, arg: &Expr, param_kinds: &[ValKind]) -> Result<(String, String), CCodeGenError> {
        let (fn_ptr, env_ptr, _ret_kind) = self.compile_lambda_arg_full(arg, param_kinds)?;
        Ok((fn_ptr, env_ptr))
    }

    /// Compile a lambda arg and return (fn_ptr, env_ptr, ret_kind).
    pub(crate) fn compile_lambda_arg_full(&mut self, arg: &Expr, param_kinds: &[ValKind]) -> Result<(String, String, ValKind), CCodeGenError> {
        match arg {
            Expr::Lambda { params, body } => {
                let (fn_expr, ret_kind) = self.compile_lambda(params, body, Some(param_kinds))?;
                let (fn_ptr, env_ptr) = Self::parse_closure_expr(&fn_expr);
                Ok((fn_ptr, env_ptr, ret_kind))
            }
            Expr::Ident(name) => {
                let ret_kind = self.functions.get(name)
                    .map(|fi| fi.ret_kind.clone())
                    .unwrap_or(ValKind::Int);
                Ok((format!("(void*)&{}", Self::mangle_fn_name(name)), "NULL".to_string(), ret_kind))
            }
            _ => Err(self.err("expected a function or lambda")),
        }
    }
}
