#![allow(dead_code)]

use ore_parser::ast::*;
use ore_types::Type;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeError {
    pub msg: String,
    pub line: usize,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(f, "line {}: {}", self.line, self.msg)
        } else {
            write!(f, "{}", self.msg)
        }
    }
}

/// Information about a record type
#[derive(Debug, Clone)]
struct RecordDef {
    fields: Vec<(String, Type)>,
}

/// Information about an enum type
#[derive(Debug, Clone)]
struct EnumDef {
    variants: Vec<(String, Vec<(String, Type)>)>,
}

/// Information about a trait
#[derive(Debug, Clone)]
struct TraitInfo {
    methods: Vec<(String, Vec<Type>, Option<Type>)>,
}

/// Type checking environment
struct Env {
    /// Variable types in scope: name -> (type, mutable)
    vars: HashMap<String, (Type, bool)>,
    /// Parent scope for lookup
    parent: Option<Box<Env>>,
}

impl Env {
    fn new() -> Self {
        Env {
            vars: HashMap::new(),
            parent: None,
        }
    }

    fn child(parent: Env) -> Self {
        Env {
            vars: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    fn lookup(&self, name: &str) -> Option<&(Type, bool)> {
        self.vars.get(name).or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    fn insert(&mut self, name: String, ty: Type, mutable: bool) {
        self.vars.insert(name, (ty, mutable));
    }
}

pub struct TypeChecker {
    functions: HashMap<String, (Vec<Type>, Type)>,
    records: HashMap<String, RecordDef>,
    enums: HashMap<String, EnumDef>,
    variant_to_enum: HashMap<String, String>,
    traits: HashMap<String, TraitInfo>,
    errors: Vec<TypeError>,
    current_line: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            functions: HashMap::new(),
            records: HashMap::new(),
            enums: HashMap::new(),
            variant_to_enum: HashMap::new(),
            traits: HashMap::new(),
            errors: Vec::new(),
            current_line: 0,
        }
    }

    fn err(&mut self, msg: impl Into<String>) {
        self.errors.push(TypeError { msg: msg.into(), line: self.current_line });
    }

    fn resolve_type_expr(&self, te: &TypeExpr) -> Type {
        match te {
            TypeExpr::Named(n) => match n.as_str() {
                "Int" => Type::Int,
                "Float" => Type::Float,
                "Bool" => Type::Bool,
                "Str" => Type::Str,
                other => {
                    if self.records.contains_key(other) {
                        Type::Record(other.to_string())
                    } else if self.enums.contains_key(other) {
                        Type::Enum(other.to_string())
                    } else {
                        Type::Any
                    }
                }
            },
            TypeExpr::Fn { .. } => {
                // Function types aren't tracked in the type checker yet
                Type::Any
            }
            TypeExpr::Generic(name, args) => {
                let resolved: Vec<Type> = args.iter().map(|a| self.resolve_type_expr(a)).collect();
                match name.as_str() {
                    "List" => Type::List(Box::new(resolved.first().cloned().unwrap_or(Type::Any))),
                    "Map" => Type::Map(
                        Box::new(resolved.first().cloned().unwrap_or(Type::Str)),
                        Box::new(resolved.get(1).cloned().unwrap_or(Type::Any)),
                    ),
                    "Option" => Type::Option(Box::new(resolved.first().cloned().unwrap_or(Type::Any))),
                    "Result" => Type::Result(
                        Box::new(resolved.first().cloned().unwrap_or(Type::Any)),
                        Box::new(resolved.get(1).cloned().unwrap_or(Type::Any)),
                    ),
                    _ => Type::Any,
                }
            }
        }
    }

    /// Register all top-level definitions before checking bodies
    fn register_items(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::FnDef(fndef) => {
                    let params: Vec<Type> = fndef.params.iter().map(|p| self.resolve_type_expr(&p.ty)).collect();
                    let ret = fndef.ret_type.as_ref().map(|t| self.resolve_type_expr(t)).unwrap_or(Type::Unit);
                    self.functions.insert(fndef.name.clone(), (params, ret));
                }
                Item::TypeDef(td) => {
                    let fields: Vec<(String, Type)> = td.fields.iter()
                        .map(|f| (f.name.clone(), self.resolve_type_expr(&f.ty)))
                        .collect();
                    self.records.insert(td.name.clone(), RecordDef { fields });
                }
                Item::EnumDef(ed) => {
                    let variants: Vec<(String, Vec<(String, Type)>)> = ed.variants.iter()
                        .map(|v| {
                            let fields: Vec<(String, Type)> = v.fields.iter()
                                .map(|f| (f.name.clone(), self.resolve_type_expr(&f.ty)))
                                .collect();
                            (v.name.clone(), fields)
                        })
                        .collect();
                    for (vname, _) in &variants {
                        self.variant_to_enum.insert(vname.clone(), ed.name.clone());
                    }
                    self.enums.insert(ed.name.clone(), EnumDef { variants });
                }
                Item::TraitDef(td) => {
                    let methods: Vec<(String, Vec<Type>, Option<Type>)> = td.methods.iter()
                        .map(|m| {
                            let params: Vec<Type> = m.params.iter().map(|p| self.resolve_type_expr(&p.ty)).collect();
                            let ret = m.ret_type.as_ref().map(|t| self.resolve_type_expr(t));
                            (m.name.clone(), params, ret)
                        })
                        .collect();
                    self.traits.insert(td.name.clone(), TraitInfo { methods });
                }
                Item::ImplBlock { .. } | Item::ImplTrait { .. } | Item::Use { .. } | Item::TestDef { .. } => {}
            }
        }
    }

    fn check_items(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::FnDef(fndef) => self.check_fn(fndef),
                Item::TestDef { body, .. } => {
                    let mut env = Env::new();
                    self.check_block(body, &mut env, &Type::Unit);
                }
                Item::ImplBlock { methods, .. } => {
                    for m in methods {
                        self.check_fn(m);
                    }
                }
                Item::ImplTrait { trait_name, type_name, methods } => {
                    self.check_impl_trait(trait_name, type_name, methods);
                    for m in methods {
                        self.check_fn(m);
                    }
                }
                _ => {}
            }
        }
    }

    /// Validate that an impl block implements all required trait methods
    fn check_impl_trait(&mut self, trait_name: &str, type_name: &str, methods: &[FnDef]) {
        let trait_info = match self.traits.get(trait_name) {
            Some(info) => info.clone(),
            None => {
                self.err(format!("unknown trait '{}'", trait_name));
                return;
            }
        };

        // Check that the type exists
        if !self.records.contains_key(type_name)
            && !self.enums.contains_key(type_name)
            && !matches!(type_name, "Int" | "Float" | "Bool" | "Str")
        {
            self.err(format!("unknown type '{}' in impl block", type_name));
        }

        // Check that all required methods are implemented
        for (req_name, req_params, req_ret) in &trait_info.methods {
            if let Some(impl_method) = methods.iter().find(|m| m.name == *req_name) {
                // Check parameter count (impl has self as first param, trait sig also has self)
                let impl_param_count = impl_method.params.len();
                let trait_param_count = req_params.len();
                if impl_param_count != trait_param_count {
                    self.err(format!(
                        "method '{}' in impl {} for {} has {} params, trait requires {}",
                        req_name, trait_name, type_name, impl_param_count, trait_param_count
                    ));
                }

                // Check return type matches
                if let Some(req_ret_ty) = req_ret {
                    if let Some(impl_ret) = &impl_method.ret_type {
                        let impl_ret_ty = self.resolve_type_expr(impl_ret);
                        if !req_ret_ty.compatible_with(&impl_ret_ty) {
                            self.err(format!(
                                "method '{}' in impl {} for {}: return type mismatch, expected {}, got {}",
                                req_name, trait_name, type_name, req_ret_ty, impl_ret_ty
                            ));
                        }
                    }
                }
            } else {
                self.err(format!(
                    "impl {} for {} is missing method '{}'",
                    trait_name, type_name, req_name
                ));
            }
        }

        // Warn about extra methods not in the trait
        for m in methods {
            if !trait_info.methods.iter().any(|(name, _, _)| name == &m.name) {
                self.err(format!(
                    "method '{}' in impl {} for {} is not defined in trait {}",
                    m.name, trait_name, type_name, trait_name
                ));
            }
        }
    }

    fn check_fn(&mut self, fndef: &FnDef) {
        let mut env = Env::new();
        for p in &fndef.params {
            let ty = self.resolve_type_expr(&p.ty);
            env.insert(p.name.clone(), ty, false);
        }
        let ret_ty = fndef.ret_type.as_ref().map(|t| self.resolve_type_expr(t)).unwrap_or(Type::Unit);
        let body_ty = self.check_block(&fndef.body, &mut env, &ret_ty);
        // Check return type mismatch (skip for main and for Any/Unit)
        if fndef.name != "main" && ret_ty != Type::Unit && ret_ty != Type::Any
            && body_ty != Type::Any && !body_ty.compatible_with(&ret_ty)
        {
            self.err(format!(
                "function '{}' declared to return {}, but body returns {}",
                fndef.name, ret_ty, body_ty
            ));
        }
    }

    fn check_block(&mut self, block: &Block, env: &mut Env, ret_ty: &Type) -> Type {
        let mut last = Type::Unit;
        for spanned in &block.stmts {
            self.current_line = spanned.line;
            last = self.check_stmt(&spanned.stmt, env, ret_ty);
        }
        last
    }

    fn check_stmt(&mut self, stmt: &Stmt, env: &mut Env, ret_ty: &Type) -> Type {
        match stmt {
            Stmt::Let { name, mutable, value } => {
                let ty = self.infer_expr(value, env);
                env.insert(name.clone(), ty.clone(), *mutable);
                ty
            }
            Stmt::Assign { name, value } => {
                let val_ty = self.infer_expr(value, env);
                if let Some((var_ty, mutable)) = env.lookup(name).cloned() {
                    if !mutable {
                        self.err(format!("cannot assign to immutable variable '{}'", name));
                    }
                    if !var_ty.compatible_with(&val_ty) {
                        self.err(format!(
                            "type mismatch in assignment to '{}': expected {}, got {}",
                            name, var_ty, val_ty
                        ));
                    }
                } else {
                    self.err(format!("undefined variable '{}'", name));
                }
                Type::Unit
            }
            Stmt::IndexAssign { object, index, value } => {
                self.infer_expr(object, env);
                self.infer_expr(index, env);
                self.infer_expr(value, env);
                Type::Unit
            }
            Stmt::FieldAssign { object, field, value } => {
                let obj_ty = self.infer_expr(object, env);
                self.infer_expr(value, env);
                if let Type::Record(name) = &obj_ty {
                    if let Some(rd) = self.records.get(name) {
                        if !rd.fields.iter().any(|(f, _)| f == field) {
                            self.err(format!("type '{}' has no field '{}'", name, field));
                        }
                    }
                }
                Type::Unit
            }
            Stmt::Expr(e) => self.infer_expr(e, env),
            Stmt::Return(Some(e)) => {
                let ty = self.infer_expr(e, env);
                if !ret_ty.compatible_with(&ty) {
                    self.err(format!("return type mismatch: expected {}, got {}", ret_ty, ty));
                }
                ty
            }
            Stmt::Return(None) => {
                if *ret_ty != Type::Unit && *ret_ty != Type::Any {
                    self.err(format!("missing return value, expected {}", ret_ty));
                }
                Type::Unit
            }
            Stmt::ForIn { var, start, end, body } => {
                self.check_int_expr(start, env, "for-in start");
                self.check_int_expr(end, env, "for-in end");
                let mut child = Env::child(std::mem::replace(env, Env::new()));
                child.insert(var.clone(), Type::Int, false);
                self.check_block(body, &mut child, ret_ty);
                *env = *child.parent.unwrap();
                Type::Unit
            }
            Stmt::While { cond, body } => {
                let ct = self.infer_expr(cond, env);
                if ct != Type::Bool && ct != Type::Any {
                    self.err(format!("while condition must be Bool, got {}", ct));
                }
                let mut child = Env::child(std::mem::replace(env, Env::new()));
                self.check_block(body, &mut child, ret_ty);
                *env = *child.parent.unwrap();
                Type::Unit
            }
            Stmt::ForEach { var, iterable, body } => {
                let iter_ty = self.infer_expr(iterable, env);
                let elem_ty = match &iter_ty {
                    Type::List(elem) => *elem.clone(),
                    _ => Type::Any,
                };
                let mut child = Env::child(std::mem::replace(env, Env::new()));
                child.insert(var.clone(), elem_ty, false);
                self.check_block(body, &mut child, ret_ty);
                *env = *child.parent.unwrap();
                Type::Unit
            }
            Stmt::ForEachKV { key_var, val_var, iterable, body } => {
                let _iter_ty = self.infer_expr(iterable, env);
                let mut child = Env::child(std::mem::replace(env, Env::new()));
                child.insert(key_var.clone(), Type::Str, false);
                child.insert(val_var.clone(), Type::Any, false);
                self.check_block(body, &mut child, ret_ty);
                *env = *child.parent.unwrap();
                Type::Unit
            }
            Stmt::Loop { body } => {
                let mut child = Env::child(std::mem::replace(env, Env::new()));
                self.check_block(body, &mut child, ret_ty);
                *env = *child.parent.unwrap();
                Type::Unit
            }
            Stmt::Break => Type::Unit,
            Stmt::Continue => Type::Unit,
            Stmt::Spawn(e) => {
                // Check that spawn doesn't pass mutable variables
                if let Expr::Call { args, .. } = e {
                    for arg in args {
                        if let Expr::Ident(name) = arg {
                            if let Some((_, true)) = env.lookup(name) {
                                self.err(format!("cannot send mutable variable '{}' to spawned task", name));
                            }
                        }
                    }
                }
                self.infer_expr(e, env);
                Type::Int
            }
        }
    }

    fn check_int_expr(&mut self, expr: &Expr, env: &mut Env, ctx: &str) {
        let ty = self.infer_expr(expr, env);
        if ty != Type::Int && ty != Type::Any {
            self.err(format!("{} must be Int, got {}", ctx, ty));
        }
    }

    fn infer_expr(&mut self, expr: &Expr, env: &mut Env) -> Type {
        match expr {
            Expr::IntLit(_) => Type::Int,
            Expr::FloatLit(_) => Type::Float,
            Expr::BoolLit(_) => Type::Bool,
            Expr::StringLit(_) => Type::Str,
            Expr::StringInterp(_) => Type::Str,

            Expr::Ident(name) => {
                if let Some((ty, _)) = env.lookup(name) {
                    ty.clone()
                } else if self.functions.contains_key(name) {
                    // Function reference
                    let (params, ret) = self.functions[name].clone();
                    Type::Fn { params, ret: Box::new(ret) }
                } else if self.variant_to_enum.contains_key(name) {
                    // Enum variant used as a value (no-arg variant)
                    let enum_name = self.variant_to_enum[name].clone();
                    Type::Enum(enum_name)
                } else {
                    // Don't error on unknown idents since builtins are registered at codegen level
                    Type::Any
                }
            }

            Expr::BinOp { op, left, right } => {
                let lt = self.infer_expr(left, env);
                let rt = self.infer_expr(right, env);

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        // String + String = String
                        if *op == BinOp::Add && lt == Type::Str && rt == Type::Str {
                            return Type::Str;
                        }
                        // Number ops
                        if lt == Type::Float || rt == Type::Float {
                            Type::Float
                        } else if lt == Type::Int || lt == Type::Any {
                            lt.clone()
                        } else {
                            rt.clone()
                        }
                    }
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                        Type::Bool
                    }
                    BinOp::And | BinOp::Or => Type::Bool,
                    BinOp::Pipe => {
                        // a | f desugars to f(a), so result is f's return type
                        if let Type::Fn { ret, .. } = &rt {
                            *ret.clone()
                        } else {
                            Type::Any
                        }
                    }
                }
            }

            Expr::UnaryMinus(inner) => self.infer_expr(inner, env),
            Expr::UnaryNot(_) => Type::Bool,

            Expr::Call { func, args } => {
                // Check args
                for a in args {
                    self.infer_expr(a, env);
                }

                match func.as_ref() {
                    Expr::Ident(name) => {
                        // Built-in functions
                        match name.as_str() {
                            "print" | "readln" => return Type::Str,
                            "abs" | "min" | "max" => return Type::Int,
                            "int" | "len" => return Type::Int,
                            "float" => return Type::Float,
                            "str" => return Type::Str,
                            "file_read" => return Type::Str,
                            "file_write" | "file_append" => return Type::Bool,
                            "file_exists" => return Type::Bool,
                            "time_now" | "time_ms" | "rand_int" => return Type::Int,
                            "exit" => return Type::Unit,
                            "type_of" | "env_get" => return Type::Str,
                            "env_set" => return Type::Unit,
                            "json_parse" => return Type::Map(Box::new(Type::Str), Box::new(Type::Any)),
                            "json_stringify" => return Type::Str,
                            "channel" => return Type::Channel,
                            _ => {}
                        }

                        // User-defined functions
                        if let Some((params, ret)) = self.functions.get(name).cloned() {
                            // Allow args.len() == params.len() - 1 for pipeline-style calls
                            // (the pipe operator prepends the first argument at codegen time)
                            if args.len() != params.len() && args.len() + 1 != params.len() {
                                self.err(format!(
                                    "function '{}' expects {} args, got {}",
                                    name, params.len(), args.len()
                                ));
                            } else {
                                // Check argument types
                                let arg_types: Vec<Type> = args.iter().map(|a| self.infer_expr(a, env)).collect();
                                for (i, (arg_ty, param_ty)) in arg_types.iter().zip(params.iter()).enumerate() {
                                    if !arg_ty.compatible_with(param_ty) {
                                        self.err(format!(
                                            "argument {} of '{}' expects {}, got {}",
                                            i + 1, name, param_ty, arg_ty
                                        ));
                                    }
                                }
                            }
                            return ret;
                        }

                        // Enum variant construction
                        if let Some(enum_name) = self.variant_to_enum.get(name).cloned() {
                            return Type::Enum(enum_name);
                        }

                        // Record construction handled separately
                        Type::Any
                    }
                    _ => {
                        // Calling a lambda/closure
                        let func_ty = self.infer_expr(func, env);
                        if let Type::Fn { ret, .. } = func_ty {
                            *ret
                        } else {
                            Type::Any
                        }
                    }
                }
            }

            Expr::Print(inner) => {
                self.infer_expr(inner, env);
                Type::Unit
            }

            Expr::IfElse { cond, then_block, else_block } => {
                let ct = self.infer_expr(cond, env);
                if ct != Type::Bool && ct != Type::Any {
                    self.err(format!("if condition must be Bool, got {}", ct));
                }
                let mut then_env = Env::child(std::mem::replace(env, Env::new()));
                let then_ty = self.check_block(then_block, &mut then_env, &Type::Any);
                *env = *then_env.parent.unwrap();

                if let Some(else_blk) = else_block {
                    let mut else_env = Env::child(std::mem::replace(env, Env::new()));
                    let _else_ty = self.check_block(else_blk, &mut else_env, &Type::Any);
                    *env = *else_env.parent.unwrap();
                }
                then_ty
            }

            Expr::ColonMatch { cond, then_expr, else_expr } => {
                let ct = self.infer_expr(cond, env);
                if ct != Type::Bool && ct != Type::Any {
                    self.err(format!("colon-match condition must be Bool, got {}", ct));
                }
                let then_ty = self.infer_expr(then_expr, env);
                if let Some(else_e) = else_expr {
                    self.infer_expr(else_e, env);
                }
                then_ty
            }

            Expr::Match { subject, arms } => {
                let subj_ty = self.infer_expr(subject, env);
                let mut result_ty = Type::Any;
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.infer_expr(guard, env);
                    }
                    let arm_ty = self.infer_expr(&arm.body, env);
                    if result_ty == Type::Any {
                        result_ty = arm_ty;
                    }
                }
                // Exhaustiveness check for enum types
                if let Type::Enum(enum_name) = &subj_ty {
                    self.check_match_exhaustiveness(enum_name, arms);
                }
                result_ty
            }

            Expr::BlockExpr(block) => {
                let mut child = Env::child(std::mem::replace(env, Env::new()));
                let mut last_ty = Type::Unit;
                for spanned in &block.stmts {
                    last_ty = self.check_stmt(&spanned.stmt, &mut child, &Type::Any);
                }
                *env = *child.parent.unwrap();
                last_ty
            }
            Expr::Lambda { params, body } => {
                let mut child = Env::child(std::mem::replace(env, Env::new()));
                let param_types: Vec<Type> = params.iter().map(|_| Type::Any).collect();
                for (name, ty) in params.iter().zip(param_types.iter()) {
                    child.insert(name.clone(), ty.clone(), false);
                }
                let ret = self.infer_expr(body, &mut child);
                *env = *child.parent.unwrap();
                Type::Fn {
                    params: param_types,
                    ret: Box::new(ret),
                }
            }

            Expr::RecordConstruct { type_name, fields } => {
                // Check field expressions
                for (_f, val) in fields {
                    self.infer_expr(val, env);
                }

                if let Some(rd) = self.records.get(type_name).cloned() {
                    // Record construction
                    for (fname, _fty) in &rd.fields {
                        if !fields.iter().any(|(f, _)| f == fname) {
                            self.err(format!("missing field '{}' in {} construction", fname, type_name));
                        }
                    }
                    for (f, _) in fields {
                        if !rd.fields.iter().any(|(fname, _)| fname == f) {
                            self.err(format!("unknown field '{}' in type '{}'", f, type_name));
                        }
                    }
                    Type::Record(type_name.clone())
                } else if let Some(enum_name) = self.variant_to_enum.get(type_name).cloned() {
                    // Enum variant construction: Circle(radius: 5.0)
                    Type::Enum(enum_name)
                } else {
                    self.err(format!("unknown type '{}'", type_name));
                    Type::Any
                }
            }

            Expr::FieldAccess { object, field } => {
                let obj_ty = self.infer_expr(object, env);
                match &obj_ty {
                    Type::Record(name) => {
                        if let Some(rd) = self.records.get(name) {
                            if let Some((_, fty)) = rd.fields.iter().find(|(f, _)| f == field) {
                                return fty.clone();
                            } else {
                                self.err(format!("type '{}' has no field '{}'", name, field));
                            }
                        }
                        Type::Any
                    }
                    _ => Type::Any,
                }
            }

            Expr::MethodCall { object, method, args } => {
                let obj_ty = self.infer_expr(object, env);
                for a in args {
                    self.infer_expr(a, env);
                }
                // Enforce mut scope confinement: cannot send mutable variable through channel
                if method == "send" && obj_ty == Type::Channel {
                    for a in args {
                        if let Expr::Ident(name) = a {
                            if let Some((_, true)) = env.lookup(name) {
                                self.err(format!("cannot send mutable variable '{}' through channel", name));
                            }
                        }
                    }
                }
                self.infer_method_return(&obj_ty, method, args.len())
            }

            Expr::ListLit(elems) => {
                let mut elem_ty = Type::Any;
                for e in elems {
                    let t = self.infer_expr(e, env);
                    if elem_ty == Type::Any {
                        elem_ty = t;
                    }
                }
                Type::List(Box::new(elem_ty))
            }

            Expr::MapLit(entries) => {
                let mut key_ty = Type::Any;
                let mut val_ty = Type::Any;
                for (k, v) in entries {
                    let kt = self.infer_expr(k, env);
                    let vt = self.infer_expr(v, env);
                    if key_ty == Type::Any { key_ty = kt; }
                    if val_ty == Type::Any { val_ty = vt; }
                }
                Type::Map(Box::new(key_ty), Box::new(val_ty))
            }

            Expr::Index { object, index } => {
                let obj_ty = self.infer_expr(object, env);
                self.infer_expr(index, env);
                match &obj_ty {
                    Type::List(elem) => *elem.clone(),
                    Type::Map(_, v) => *v.clone(),
                    _ => Type::Any,
                }
            }

            Expr::Break => Type::Unit,
            Expr::OptionNone => Type::Option(Box::new(Type::Any)),
            Expr::OptionSome(inner) => {
                let t = self.infer_expr(inner, env);
                Type::Option(Box::new(t))
            }
            Expr::ResultOk(inner) => {
                let t = self.infer_expr(inner, env);
                Type::Result(Box::new(t), Box::new(Type::Any))
            }
            Expr::ResultErr(inner) => {
                let t = self.infer_expr(inner, env);
                Type::Result(Box::new(Type::Any), Box::new(t))
            }
            Expr::Try(inner) => {
                self.infer_expr(inner, env);
                Type::Any
            }
            Expr::Sleep(_) => Type::Unit,
            Expr::OptionalChain { object, .. } => {
                self.infer_expr(object, env);
                Type::Option(Box::new(Type::Any))
            }
            Expr::OptionalMethodCall { object, args, .. } => {
                self.infer_expr(object, env);
                for arg in args {
                    self.infer_expr(arg, env);
                }
                Type::Option(Box::new(Type::Any))
            }
            Expr::Assert { cond, .. } => {
                let ct = self.infer_expr(cond, env);
                if ct != Type::Bool && ct != Type::Any {
                    self.err(format!("assert condition must be Bool, got {}", ct));
                }
                Type::Unit
            }
            Expr::AssertEq { left, right, .. } | Expr::AssertNe { left, right, .. } => {
                self.infer_expr(left, env);
                self.infer_expr(right, env);
                Type::Unit
            }
        }
    }

    fn check_match_exhaustiveness(&mut self, enum_name: &str, arms: &[MatchArm]) {
        let enum_def = match self.enums.get(enum_name) {
            Some(ed) => ed.clone(),
            None => return,
        };

        // Check if there's a wildcard or catch-all pattern
        let has_wildcard = arms.iter().any(|arm| {
            self.pattern_is_wildcard(&arm.pattern)
        });
        if has_wildcard {
            return;
        }

        // Collect covered variant names
        let covered: std::collections::HashSet<String> = arms.iter()
            .flat_map(|arm| self.pattern_variant_names(&arm.pattern))
            .collect();

        // Find uncovered variants
        let missing: Vec<&str> = enum_def.variants.iter()
            .filter(|(name, _)| !covered.contains(name))
            .map(|(name, _)| name.as_str())
            .collect();

        if !missing.is_empty() {
            self.err(format!(
                "non-exhaustive match on {}: missing variant(s) {}",
                enum_name,
                missing.join(", ")
            ));
        }
    }

    fn pattern_is_wildcard(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Wildcard => true,
            Pattern::Variant { name, .. } => {
                // A bare variable binding (not a known variant) acts as wildcard
                !self.variant_to_enum.contains_key(name)
            }
            Pattern::Or(pats) => pats.iter().any(|p| self.pattern_is_wildcard(p)),
            _ => false,
        }
    }

    fn pattern_variant_names(&self, pattern: &Pattern) -> Vec<String> {
        match pattern {
            Pattern::Variant { name, .. } => {
                if self.variant_to_enum.contains_key(name) {
                    vec![name.clone()]
                } else {
                    vec![] // Variable binding, not a variant
                }
            }
            Pattern::Or(pats) => pats.iter()
                .flat_map(|p| self.pattern_variant_names(p))
                .collect(),
            _ => vec![],
        }
    }

    fn infer_method_return(&self, obj_ty: &Type, method: &str, _arg_count: usize) -> Type {
        match obj_ty {
            Type::Str => match method {
                "len" | "to_int" => Type::Int,
                "to_float" => Type::Float,
                "contains" | "starts_with" | "ends_with" => Type::Bool,
                "trim" | "to_upper" | "to_lower" | "replace" | "substr" => Type::Str,
                "split" => Type::List(Box::new(Type::Str)),
                _ => Type::Any,
            },
            Type::List(elem) => match method {
                "len" => Type::Int,
                "push" => Type::Unit,
                "get" => *elem.clone(),
                "contains" => Type::Bool,
                "map" | "filter" => obj_ty.clone(),
                "each" => Type::Unit,
                "reduce" => Type::Any,
                "find" => *elem.clone(),
                "join" => Type::Str,
                "sort" | "reverse" => obj_ty.clone(),
                _ => Type::Any,
            },
            Type::Map(_, v) => match method {
                "len" => Type::Int,
                "contains" => Type::Bool,
                "get" => *v.clone(),
                "keys" => Type::List(Box::new(Type::Str)),
                "values" => Type::List(Box::new(*v.clone())),
                "set" | "remove" => Type::Unit,
                _ => Type::Any,
            },
            _ => {
                // to_str is available on all types
                if method == "to_str" {
                    Type::Str
                } else {
                    Type::Any
                }
            }
        }
    }
}

/// Type-check a program, returning a list of errors (empty = OK).
pub fn typecheck(program: &Program) -> Result<(), Vec<TypeError>> {
    let mut checker = TypeChecker::new();
    checker.register_items(program);
    checker.check_items(program);
    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
    }
}
