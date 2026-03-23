use super::*;

impl CCodeGen {
    /// Map a ValKind to its C type string.
    pub(crate) fn kind_to_c_type(&self, kind: &ValKind) -> &'static str {
        match kind {
            ValKind::Int | ValKind::Void => "int64_t",
            ValKind::Float => "double",
            ValKind::Bool => "int8_t",
            ValKind::Str | ValKind::List(_) | ValKind::Map(_) | ValKind::Channel => "void*",
            ValKind::Record(name) => {
                // Records are passed by value as their struct type
                // We'll use void* for now since we pass by pointer in C
                // Actually, records in LLVM codegen are passed by value (struct copy)
                // In C we need the actual struct name - but we can't return &'static str
                // for dynamic names. Let's use void* for pointer-based passing.
                let _ = name;
                "void*"
            }
            ValKind::Enum(_) => "void*",
            ValKind::Option | ValKind::Result => "OreTaggedUnion",
        }
    }

    /// Map a ValKind to its C type string, returning a String for dynamic names.
    pub(crate) fn kind_to_c_type_str(&self, kind: &ValKind) -> String {
        match kind {
            ValKind::Int | ValKind::Void => "int64_t".to_string(),
            ValKind::Float => "double".to_string(),
            ValKind::Bool => "int8_t".to_string(),
            ValKind::Str | ValKind::List(_) | ValKind::Map(_) | ValKind::Channel => "void*".to_string(),
            ValKind::Record(name) => format!("struct ore_rec_{}", Self::mangle_name(name)),
            ValKind::Enum(name) => format!("struct ore_enum_{}", Self::mangle_name(name)),
            // Note: These always use "struct" prefix since C requires it for struct types
            ValKind::Option | ValKind::Result => "OreTaggedUnion".to_string(),
        }
    }

    pub(crate) fn valkind_to_tag(kind: &ValKind) -> u8 {
        match kind {
            ValKind::Int => 0,
            ValKind::Float => 1,
            ValKind::Bool => 2,
            ValKind::Str => 3,
            ValKind::Void => 4,
            ValKind::Record(_) => 5,
            ValKind::Enum(_) => 6,
            ValKind::Option => 7,
            ValKind::Result => 8,
            ValKind::List(_) => 9,
            ValKind::Map(_) => 10,
            ValKind::Channel => 11,
        }
    }

    pub(crate) fn type_expr_to_kind(&self, ty: &TypeExpr) -> ValKind {
        match ty {
            TypeExpr::Named(n) => match n.as_str() {
                "Int" => ValKind::Int,
                "Float" => ValKind::Float,
                "Bool" => ValKind::Bool,
                "Str" => ValKind::Str,
                "Option" => ValKind::Option,
                "Result" => ValKind::Result,
                "List" => ValKind::List(None),
                "Map" => ValKind::Map(None),
                "Channel" => ValKind::Channel,
                other => {
                    if self.records.contains_key(other) {
                        ValKind::Record(other.to_string())
                    } else if self.enums.contains_key(other) {
                        ValKind::Enum(other.to_string())
                    } else {
                        ValKind::Int
                    }
                }
            },
            TypeExpr::Fn { .. } => ValKind::Int,
            TypeExpr::Generic(name, args) => {
                match name.as_str() {
                    "List" => {
                        if let Some(elem_ty) = args.first() {
                            ValKind::list_of(self.type_expr_to_kind(elem_ty))
                        } else {
                            ValKind::List(None)
                        }
                    }
                    "Map" => {
                        if args.len() >= 2 {
                            ValKind::map_of(self.type_expr_to_kind(&args[1]))
                        } else {
                            ValKind::Map(None)
                        }
                    }
                    "Option" => ValKind::Option,
                    "Result" => ValKind::Result,
                    other => {
                        if self.records.contains_key(other) {
                            ValKind::Record(other.to_string())
                        } else {
                            ValKind::Int
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn kind_size_bytes(&self, kind: &ValKind) -> u64 {
        match kind {
            ValKind::Bool => 1,
            ValKind::Int | ValKind::Float | ValKind::Str | ValKind::List(_)
            | ValKind::Map(_) | ValKind::Channel | ValKind::Void => 8,
            ValKind::Record(name) => {
                if let Some(info) = self.records.get(name) {
                    info.field_kinds.iter().map(|k| self.kind_size_bytes(k)).sum()
                } else { 8 }
            }
            ValKind::Enum(name) => {
                if let Some(info) = self.enums.get(name) {
                    1 + info.num_data_i64s as u64 * 8
                } else { 8 }
            }
            ValKind::Option | ValKind::Result => 10, // tag + kind + i64
        }
    }

    /// Infer the return kind of a method call by name.
    fn infer_method_return_kind(&self, method: &str) -> ValKind {
        match method {
            "to_upper" | "to_lower" | "trim" | "substr" | "replace"
            | "join" | "to_str" | "repeat" | "reverse" => ValKind::Str,
            "len" | "count" | "to_int" | "sum" | "min" | "max"
            | "index_of" | "pop" | "first" | "last" => ValKind::Int,
            "to_float" => ValKind::Float,
            "contains" | "starts_with" | "ends_with"
            | "is_empty" | "is_some" | "is_none" | "is_ok" | "is_err" => ValKind::Bool,
            "split" | "keys" | "values" | "entries"
            | "map" | "filter" | "take" | "drop" | "sort" | "flatten" => ValKind::List(None),
            _ => ValKind::Int,
        }
    }

    /// Infer the result kind of a binary operation.
    fn infer_binop_kind(&self, op: &BinOp, left: &Expr, right: &Expr) -> ValKind {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                let lk = self.infer_expr_kind(left);
                let rk = self.infer_expr_kind(right);
                if lk == ValKind::Str || rk == ValKind::Str {
                    ValKind::Str // string concatenation
                } else if lk == ValKind::Float || rk == ValKind::Float {
                    ValKind::Float
                } else {
                    ValKind::Int
                }
            }
            BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq
            | BinOp::Eq | BinOp::NotEq | BinOp::And | BinOp::Or => ValKind::Bool,
            BinOp::Pipe => self.infer_expr_kind(right),
        }
    }

    /// Infer expression kind without compilation (lightweight).
    pub(crate) fn infer_expr_kind(&self, expr: &Expr) -> ValKind {
        match expr {
            Expr::StringLit(_) | Expr::StringInterp(_) => ValKind::Str,
            Expr::IntLit(_) => ValKind::Int,
            Expr::FloatLit(_) => ValKind::Float,
            Expr::BoolLit(_) => ValKind::Bool,
            Expr::TupleLit(_) => ValKind::List(None),
            Expr::ListLit(_) | Expr::ListComp { .. } => ValKind::List(None),
            Expr::MapLit(_) => ValKind::Map(None),
            Expr::Ident(name) => {
                self.variables.get(name).map(|v| v.kind.clone()).unwrap_or(ValKind::Int)
            }
            Expr::MethodCall { method, .. } => self.infer_method_return_kind(method.as_str()),
            Expr::BinOp { op, left, right } => self.infer_binop_kind(op, left, right),
            Expr::IfElse { then_block, .. } => {
                if let Some(last) = then_block.stmts.last() {
                    if let Stmt::Expr(e) = &last.stmt {
                        return self.infer_expr_kind(e);
                    }
                }
                ValKind::Int
            }
            Expr::OptionSome(_) => ValKind::Option,
            Expr::OptionNone => ValKind::Option,
            Expr::ResultOk(_) => ValKind::Result,
            Expr::ResultErr(_) => ValKind::Result,
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = func.as_ref() {
                    if let Some(fn_info) = self.functions.get(name) {
                        return fn_info.ret_kind.clone();
                    }
                }
                ValKind::Int
            }
            Expr::RecordConstruct { type_name, .. } => {
                if self.variant_to_enum.contains_key(type_name) {
                    if let Some(enum_name) = self.variant_to_enum.get(type_name) {
                        ValKind::Enum(enum_name.clone())
                    } else {
                        ValKind::Int
                    }
                } else if self.records.contains_key(type_name) {
                    ValKind::Record(type_name.clone())
                } else {
                    ValKind::Int
                }
            }
            Expr::Index { object, .. } => {
                let obj_kind = self.infer_expr_kind(object);
                match obj_kind {
                    ValKind::List(Some(ek)) => *ek,
                    ValKind::Map(Some(vk)) => *vk,
                    _ => ValKind::Int,
                }
            }
            Expr::FieldAccess { object, field } => {
                let obj_kind = self.infer_expr_kind(object);
                if let ValKind::Record(ref name) = obj_kind {
                    if let Some(info) = self.records.get(name) {
                        for (i, fname) in info.field_names.iter().enumerate() {
                            if fname == field {
                                return info.field_kinds[i].clone();
                            }
                        }
                    }
                }
                ValKind::Int
            }
            Expr::UnaryMinus(inner) | Expr::UnaryNot(inner) => self.infer_expr_kind(inner),
            _ => ValKind::Int,
        }
    }

    /// Get the enriched kind for a variable, checking element/value kind maps.
    pub(crate) fn get_var_kind(&self, name: &str) -> ValKind {
        let base_kind = self.variables.get(name).map(|v| v.kind.clone()).unwrap_or(ValKind::Int);
        match &base_kind {
            ValKind::List(_) => {
                if let Some(ek) = self.list_element_kinds.get(name) {
                    ValKind::list_of(ek.clone())
                } else {
                    base_kind
                }
            }
            ValKind::Map(_) => {
                if let Some(vk) = self.map_value_kinds.get(name) {
                    ValKind::map_of(vk.clone())
                } else {
                    base_kind
                }
            }
            _ => base_kind,
        }
    }

    /// Generate a C expression to convert a value to i64 for storage.
    pub(crate) fn value_to_i64_expr(&self, expr: &str, kind: &ValKind) -> String {
        match kind {
            ValKind::Float => format!("*(int64_t*)&(double){{{}}}", expr),
            ValKind::Bool => format!("(int64_t)({})", expr),
            ValKind::Str | ValKind::List(_) | ValKind::Map(_) | ValKind::Channel => {
                format!("(int64_t)(intptr_t)({})", expr)
            }
            ValKind::Enum(name) => {
                // Heap-allocate the enum struct, store value, return pointer as i64
                // Use statement expression to materialize rvalues (e.g. function returns)
                // into an lvalue before taking its address
                let c_type = format!("struct ore_enum_{}", Self::mangle_name(name));
                format!("({{ {c_type} __v2i = {expr}; (int64_t)(intptr_t)memcpy(malloc(sizeof({c_type})), &__v2i, sizeof({c_type})); }})")
            }
            ValKind::Record(name) => {
                let c_type = format!("struct ore_rec_{}", Self::mangle_name(name));
                format!("({{ {c_type} __v2i = {expr}; (int64_t)(intptr_t)memcpy(malloc(sizeof({c_type})), &__v2i, sizeof({c_type})); }})")
            }
            ValKind::Option | ValKind::Result => {
                // Tagged union: copy to heap
                format!("({{ OreTaggedUnion __v2i = {expr}; (int64_t)(intptr_t)memcpy(malloc(sizeof(OreTaggedUnion)), &__v2i, sizeof(OreTaggedUnion)); }})")
            }
            _ => expr.to_string(),
        }
    }

    /// Generate a C expression to convert from i64 back to the correct type.
    pub(crate) fn coerce_from_i64_expr(&self, expr: &str, kind: &ValKind) -> String {
        match kind {
            ValKind::Str | ValKind::List(_) | ValKind::Map(_) | ValKind::Channel => {
                format!("(void*)(intptr_t)({})", expr)
            }
            ValKind::Float => format!("*(double*)&(int64_t){{{}}}", expr),
            ValKind::Bool => format!("(int8_t)(({}) != 0)", expr),
            ValKind::Enum(name) => {
                // Dereference heap pointer back to enum struct
                let c_type = format!("struct ore_enum_{}", Self::mangle_name(name));
                format!("*({c_type}*)(intptr_t)({expr})")
            }
            ValKind::Record(name) => {
                let c_type = format!("struct ore_rec_{}", Self::mangle_name(name));
                format!("*({c_type}*)(intptr_t)({expr})")
            }
            ValKind::Option | ValKind::Result => {
                format!("*(OreTaggedUnion*)(intptr_t)({})", expr)
            }
            _ => expr.to_string(),
        }
    }

    /// Coerce a compiled argument expression to match a function parameter's expected type.
    pub(crate) fn coerce_arg_to_param(&self, expr: &str, arg_kind: &ValKind, param_kind: &ValKind) -> String {
        if arg_kind == param_kind {
            return expr.to_string();
        }
        let arg_c = self.kind_to_c_type_str(arg_kind);
        let param_c = self.kind_to_c_type_str(param_kind);
        if arg_c == param_c {
            return expr.to_string();
        }
        if param_kind == &ValKind::Int && expr.starts_with("(void*)&") {
            return format!("(int64_t)(intptr_t){}", expr);
        }
        let as_i64 = self.value_to_i64_expr(expr, arg_kind);
        self.coerce_from_i64_expr(&as_i64, param_kind)
    }

    /// Coerce a compiled expression from one ValKind to another.
    pub(crate) fn coerce_expr(&self, expr: &str, from: &ValKind, to: &ValKind) -> String {
        if from == to {
            return expr.to_string();
        }
        let from_is_i64 = matches!(from, ValKind::Int | ValKind::Void);
        let to_is_struct = matches!(to, ValKind::Record(_) | ValKind::Enum(_) | ValKind::Option | ValKind::Result);
        if from_is_i64 && to_is_struct {
            return self.coerce_from_i64_expr(expr, to);
        }
        let from_is_struct = matches!(from, ValKind::Record(_) | ValKind::Enum(_) | ValKind::Option | ValKind::Result);
        let to_is_i64 = matches!(to, ValKind::Int | ValKind::Void);
        if from_is_struct && to_is_i64 {
            return self.value_to_i64_expr(expr, from);
        }
        expr.to_string()
    }

    /// Generate a safe C identifier suffix for a ValKind (for monomorphized function names).
    pub(crate) fn kind_to_suffix(kind: &ValKind) -> String {
        match kind {
            ValKind::Int => "Int".to_string(),
            ValKind::Float => "Float".to_string(),
            ValKind::Bool => "Bool".to_string(),
            ValKind::Str => "Str".to_string(),
            ValKind::Void => "Void".to_string(),
            ValKind::List(Some(ek)) => format!("List_{}", Self::kind_to_suffix(ek)),
            ValKind::List(None) => "List".to_string(),
            ValKind::Map(Some(vk)) => format!("Map_{}", Self::kind_to_suffix(vk)),
            ValKind::Map(None) => "Map".to_string(),
            ValKind::Option => "Option".to_string(),
            ValKind::Result => "Result".to_string(),
            ValKind::Channel => "Channel".to_string(),
            ValKind::Record(n) => format!("Rec_{}", n),
            ValKind::Enum(n) => format!("Enum_{}", n),
        }
    }

    /// Map a ValKind back to an Ore type name string (for generic monomorphization).
    pub(crate) fn kind_to_type_name<'a>(&self, kind: &'a ValKind) -> &'a str {
        match kind {
            ValKind::Int => "Int",
            ValKind::Float => "Float",
            ValKind::Bool => "Bool",
            ValKind::Str => "Str",
            ValKind::Void => "Int",
            ValKind::List(_) => "List",
            ValKind::Map(_) => "Map",
            ValKind::Option => "Option",
            ValKind::Result => "Result",
            ValKind::Channel => "Channel",
            ValKind::Record(name) => name.as_str(),
            ValKind::Enum(name) => name.as_str(),
        }
    }
}
