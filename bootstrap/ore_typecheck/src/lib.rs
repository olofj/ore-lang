use ore_parser::ast::*;
use ore_types::Type;
use std::collections::HashMap;

/// Compute Levenshtein edit distance between two strings (case-insensitive).
fn edit_distance(a: &str, b: &str) -> usize {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in 0..=a.len() {
        dp[i][0] = i;
    }
    for j in 0..=b.len() {
        dp[0][j] = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[a.len()][b.len()]
}

/// Find the closest match to `name` from `candidates`.
/// Returns `Some(suggestion)` if within a reasonable edit distance threshold.
fn suggest_similar<'a>(name: &str, candidates: impl Iterator<Item = &'a str>) -> Option<String> {
    let threshold = match name.len() {
        0..=2 => 1,
        3..=5 => 2,
        _ => 3,
    };
    let mut best: Option<(usize, String)> = None;
    for candidate in candidates {
        if candidate == name {
            continue;
        }
        let dist = edit_distance(name, candidate);
        if dist <= threshold {
            if best.as_ref().map_or(true, |(d, _)| dist < *d) {
                best = Some((dist, candidate.to_string()));
            }
        }
    }
    best.map(|(_, s)| s)
}

/// Pipeline method names resolved at codegen time, not type-check time.
const PIPELINE_METHODS: &[&str] = &[
    "map", "filter", "each", "reduce", "fold", "scan", "join",
    "sort", "sort_by", "sort_by_key", "reverse", "unique", "dedup",
    "take", "skip", "take_while", "drop_while", "step",
    "flatten", "flat_map", "zip", "zip_with", "enumerate",
    "window", "chunks", "intersperse", "partition", "group_by",
    "count_by", "frequencies", "to_map", "first", "last",
    "sum", "product", "min", "max", "average", "any", "all",
    "find", "find_index", "index_of", "contains",
    "push", "pop", "insert", "remove_at", "clear", "set", "get", "get_or",
    "len", "is_empty", "slice",
    "to_upper", "to_lower", "trim", "trim_start", "trim_end",
    "split", "replace", "starts_with", "ends_with", "repeat",
    "capitalize", "count", "strip_prefix", "strip_suffix",
    "substr", "chars", "char_at", "pad_left", "pad_right", "words", "lines",
    "to_int", "to_float", "to_str", "parse_int", "parse_float",
    "abs", "floor", "ceil", "round", "sqrt", "pow", "clamp",
    "unwrap_or", "typeof", "merge", "keys", "values", "entries",
    "par_map", "par_each", "send", "recv", "tap",
    "map_with_index", "each_with_index",
    "__range",
];

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

    fn lookup(&self, name: &str) -> Option<&(Type, bool)> {
        self.vars.get(name).or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    fn insert(&mut self, name: String, ty: Type, mutable: bool) {
        self.vars.insert(name, (ty, mutable));
    }

    /// Collect all variable names visible in this scope (for suggestions).
    fn all_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.vars.keys().map(|s| s.as_str()).collect();
        if let Some(ref parent) = self.parent {
            names.extend(parent.all_names());
        }
        names
    }

    /// Create a child scope, taking ownership of the parent.
    fn push_scope(parent: &mut Env) -> Env {
        Env {
            vars: HashMap::new(),
            parent: Some(Box::new(std::mem::replace(parent, Env::new()))),
        }
    }

    /// Restore the parent scope from a child.
    fn pop_scope(child: Env, parent: &mut Env) {
        *parent = *child.parent.unwrap();
    }
}

pub struct TypeChecker {
    functions: HashMap<String, (Vec<Type>, Type)>,
    /// Minimum number of required parameters (without defaults) per function
    fn_required_params: HashMap<String, usize>,
    records: HashMap<String, RecordDef>,
    enums: HashMap<String, EnumDef>,
    variant_to_enum: HashMap<String, String>,
    traits: HashMap<String, TraitInfo>,
    errors: Vec<TypeError>,
    current_line: usize,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            functions: HashMap::new(),
            fn_required_params: HashMap::new(),
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

    /// Collect all known type names (records, enums, builtins) for suggestions.
    fn all_type_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        names.extend(self.records.keys().map(|s| s.as_str()));
        names.extend(self.enums.keys().map(|s| s.as_str()));
        names.extend(["Int", "Float", "Bool", "Str"].iter());
        names
    }

    /// Collect all known function names for suggestions.
    fn all_function_names(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
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

    /// Register a function's parameter types, return type, and required param count.
    fn register_fn(&mut self, fndef: &FnDef) -> (Vec<Type>, Type) {
        let params: Vec<Type> = fndef.params.iter().map(|p| self.resolve_type_expr(&p.ty)).collect();
        let ret = fndef.ret_type.as_ref().map(|t| self.resolve_type_expr(t)).unwrap_or(Type::Unit);
        let required = fndef.params.iter().filter(|p| p.default.is_none()).count();
        self.functions.insert(fndef.name.clone(), (params.clone(), ret.clone()));
        if required < params.len() {
            self.fn_required_params.insert(fndef.name.clone(), required);
        }
        (params, ret)
    }

    /// Register all top-level definitions before checking bodies
    fn register_items(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::FnDef(fndef) => { self.register_fn(fndef); }
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
            let mut msg = format!("unknown type '{}' in impl block", type_name);
            if let Some(suggestion) = suggest_similar(type_name, self.all_type_names().into_iter()) {
                msg.push_str(&format!("; did you mean '{}'?", suggestion));
            }
            self.err(msg);
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
            Stmt::Let { name, mutable, type_annotation, value } => {
                let inferred = self.infer_expr(value, env);
                let ty = if let Some(te) = type_annotation {
                    let annotated = self.resolve_type_expr(te);
                    // Tuple-to-record auto-conversion: check field count and types match
                    if let (Type::Record(rec_name), Type::Tuple(tuple_types)) = (&annotated, &inferred) {
                        let fields = self.records.get(rec_name).map(|rd| rd.fields.clone());
                        if let Some(fields) = fields {
                            if tuple_types.len() != fields.len() {
                                self.err(format!(
                                    "tuple has {} elements but {} has {} fields",
                                    tuple_types.len(), rec_name, fields.len()
                                ));
                            } else {
                                for (i, ((fname, ftype), ttype)) in fields.iter().zip(tuple_types.iter()).enumerate() {
                                    if !ttype.compatible_with(ftype) {
                                        self.err(format!(
                                            "tuple element {} (type {}) is not compatible with field '{}' (type {}) of {}",
                                            i, ttype, fname, ftype, rec_name
                                        ));
                                    }
                                }
                            }
                        }
                    } else if !inferred.compatible_with(&annotated) {
                        self.err(format!(
                            "type mismatch: annotated type {} but value has type {}",
                            annotated, inferred
                        ));
                    }
                    annotated
                } else {
                    inferred
                };
                env.insert(name.clone(), ty.clone(), *mutable);
                ty
            }
            Stmt::LetDestructure { names, value } => {
                let ty = self.infer_expr(value, env);
                let elem_ty = match &ty {
                    Type::List(t) => (**t).clone(),
                    _ => {
                        self.err("destructuring requires a list value".to_string());
                        Type::Any
                    }
                };
                for name in names {
                    env.insert(name.clone(), elem_ty.clone(), false);
                }
                ty
            }
            Stmt::Assign { name, value } => {
                let val_ty = self.infer_expr(value, env);
                if let Some((var_ty, mutable)) = env.lookup(name).cloned() {
                    if !mutable {
                        self.err(format!("cannot assign to immutable variable '{}'; declare with 'mut' to allow assignment", name));
                    }
                    if !var_ty.compatible_with(&val_ty) {
                        self.err(format!(
                            "type mismatch in assignment to '{}': expected {}, got {}",
                            name, var_ty, val_ty
                        ));
                    }
                } else {
                    let mut msg = format!("undefined variable '{}'", name);
                    if let Some(suggestion) = suggest_similar(name, env.all_names().into_iter()) {
                        msg.push_str(&format!("; did you mean '{}'?", suggestion));
                    } else {
                        msg.push_str("; use ':=' to declare a new variable");
                    }
                    self.err(msg);
                }
                Type::Unit
            }
            Stmt::AssignIfUnset { name, value } => {
                let _val_ty = self.infer_expr(value, env);
                if let Some((_var_ty, mutable)) = env.lookup(name).cloned() {
                    if !mutable {
                        self.err(format!("cannot assign to immutable variable '{}'; declare with 'mut' to allow assignment", name));
                    }
                } else {
                    let mut msg = format!("undefined variable '{}'", name);
                    if let Some(suggestion) = suggest_similar(name, env.all_names().into_iter()) {
                        msg.push_str(&format!("; did you mean '{}'?", suggestion));
                    } else {
                        msg.push_str("; use ':=' to declare a new variable");
                    }
                    self.err(msg);
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
                            let field_names: Vec<&str> = rd.fields.iter().map(|(n, _)| n.as_str()).collect();
                            let mut msg = format!("type '{}' has no field '{}'", name, field);
                            if let Some(suggestion) = suggest_similar(field, field_names.into_iter()) {
                                msg.push_str(&format!("; did you mean '{}'?", suggestion));
                            }
                            self.err(msg);
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
            Stmt::ForIn { var, start, end, step, body } => {
                self.check_int_expr(start, env, "for-in start");
                self.check_int_expr(end, env, "for-in end");
                if let Some(s) = step {
                    self.check_int_expr(s, env, "for-in step");
                }
                let mut child = Env::push_scope(env);
                child.insert(var.clone(), Type::Int, false);
                self.check_block(body, &mut child, ret_ty);
                Env::pop_scope(child, env);
                Type::Unit
            }
            Stmt::While { cond, body } => {
                self.check_bool_expr(cond, env, "while");
                let mut child = Env::push_scope(env);
                self.check_block(body, &mut child, ret_ty);
                Env::pop_scope(child, env);
                Type::Unit
            }
            Stmt::ForEach { var, iterable, body } => {
                let iter_ty = self.infer_expr(iterable, env);
                let elem_ty = match &iter_ty {
                    Type::List(elem) => *elem.clone(),
                    _ => Type::Any,
                };
                let mut child = Env::push_scope(env);
                child.insert(var.clone(), elem_ty, false);
                self.check_block(body, &mut child, ret_ty);
                Env::pop_scope(child, env);
                Type::Unit
            }
            Stmt::ForEachKV { key_var, val_var, iterable, body } => {
                let _iter_ty = self.infer_expr(iterable, env);
                let mut child = Env::push_scope(env);
                child.insert(key_var.clone(), Type::Str, false);
                child.insert(val_var.clone(), Type::Any, false);
                self.check_block(body, &mut child, ret_ty);
                Env::pop_scope(child, env);
                Type::Unit
            }
            Stmt::Loop { body } => {
                let mut child = Env::push_scope(env);
                self.check_block(body, &mut child, ret_ty);
                Env::pop_scope(child, env);
                Type::Unit
            }
            Stmt::Break | Stmt::Continue => Type::Unit,
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
            Stmt::LocalFn(fndef) => {
                let (params, ret) = self.register_fn(fndef);
                let mut fn_env = Env::new();
                for (param, ty) in fndef.params.iter().zip(params.iter()) {
                    fn_env.insert(param.name.clone(), ty.clone(), false);
                }
                self.check_block(&fndef.body, &mut fn_env, &ret);
                Type::Unit
            }
            Stmt::WithBlock { expr, body } => {
                let ctx_ty = self.infer_expr(expr, env);
                // If the with-expression is a record, bind its fields as variables
                if let Type::Record(rec_name) = &ctx_ty {
                    if let Some(fields) = self.records.get(rec_name).map(|rd| rd.fields.clone()) {
                        for (fname, ftype) in &fields {
                            env.insert(fname.clone(), ftype.clone(), false);
                        }
                    }
                }
                self.check_block(body, env, ret_ty)
            }
        }
    }

    fn check_int_expr(&mut self, expr: &Expr, env: &mut Env, ctx: &str) {
        let ty = self.infer_expr(expr, env);
        if ty != Type::Int && ty != Type::Any {
            self.err(format!("{} must be Int, got {}", ctx, ty));
        }
    }

    fn check_bool_expr(&mut self, expr: &Expr, env: &mut Env, ctx: &str) {
        let ty = self.infer_expr(expr, env);
        if ty != Type::Bool && ty != Type::Any {
            self.err(format!("{} condition must be Bool, got {}", ctx, ty));
        }
    }

    fn infer_expr(&mut self, expr: &Expr, env: &mut Env) -> Type {
        match expr {
            Expr::IntLit(_) => Type::Int,
            Expr::FloatLit(_) => Type::Float,
            Expr::BoolLit(_) => Type::Bool,
            Expr::StringLit(_) | Expr::StringInterp(_) => Type::Str,

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
                            lt
                        } else {
                            rt
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
                            "print" | "readln" | "input" | "str" | "chr" | "file_read"
                            | "exec" | "type_of" | "env_get" => return Type::Str,
                            "abs" | "min" | "max" | "int" | "len" | "ord"
                            | "time_now" | "time_ms" | "rand_int" => return Type::Int,
                            "float" => return Type::Float,
                            "file_read_lines" | "args" => return Type::List(Box::new(Type::Str)),
                            "file_write" | "file_append" | "file_exists" => return Type::Bool,
                            "range" | "repeat" => return Type::List(Box::new(Type::Int)),
                            "exit" | "eprint" | "assert" | "assert_eq" | "assert_ne" | "env_set" => return Type::Unit,
                            "json_parse" => return Type::Map(Box::new(Type::Str), Box::new(Type::Any)),
                            "json_stringify" => return Type::Str,
                            "channel" => return Type::Channel,
                            "sqrt" | "sin" | "cos" | "tan" | "log" | "log10" | "exp" | "math_abs" | "math_floor" | "math_ceil" | "math_round"
                            | "pow" | "atan2" | "pi" | "euler" | "e" => return Type::Float,
                            _ => {}
                        }

                        // User-defined functions
                        if let Some((params, ret)) = self.functions.get(name).cloned() {
                            let min_required = self.fn_required_params.get(name).copied().unwrap_or(params.len());
                            // Allow args.len() == params.len() - 1 for pipeline-style calls
                            // (the pipe operator prepends the first argument at codegen time)
                            // Also allow fewer args when defaults exist (min_required..=params.len())
                            let valid = (args.len() >= min_required && args.len() <= params.len())
                                || (args.len() + 1 >= min_required && args.len() < params.len());
                            if !valid {
                                self.err(format!(
                                    "function '{}' expects {}{} args, got {}",
                                    name,
                                    if min_required < params.len() { format!("{}-", min_required) } else { String::new() },
                                    params.len(), args.len()
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

                        // Record construction
                        if self.records.contains_key(name) {
                            return Type::Record(name.clone());
                        }

                        // Check if it's a variable that holds a function
                        if let Some((ty, _)) = env.lookup(name) {
                            return ty.clone();
                        }

                        if !PIPELINE_METHODS.contains(&name.as_str()) && !name.starts_with("__") {
                            let mut msg = format!("undefined function '{}'", name);
                            // Collect candidates from user-defined functions, env vars, records, enum variants
                            let fn_names = self.all_function_names();
                            let var_names = env.all_names();
                            let all: Vec<&str> = fn_names.iter().copied()
                                .chain(var_names.iter().copied())
                                .chain(self.records.keys().map(|s| s.as_str()))
                                .chain(self.variant_to_enum.keys().map(|s| s.as_str()))
                                .collect();
                            if let Some(suggestion) = suggest_similar(name, all.into_iter()) {
                                msg.push_str(&format!("; did you mean '{}'?", suggestion));
                            }
                            self.err(msg);
                        }
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
                self.check_bool_expr(cond, env, "if");
                let mut then_env = Env::push_scope(env);
                let then_ty = self.check_block(then_block, &mut then_env, &Type::Any);
                Env::pop_scope(then_env, env);

                if let Some(else_blk) = else_block {
                    let mut else_env = Env::push_scope(env);
                    let _else_ty = self.check_block(else_blk, &mut else_env, &Type::Any);
                    Env::pop_scope(else_env, env);
                }
                then_ty
            }

            Expr::ColonMatch { cond, then_expr, else_expr } => {
                self.check_bool_expr(cond, env, "colon-match");
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
                let mut child = Env::push_scope(env);
                let mut last_ty = Type::Unit;
                for spanned in &block.stmts {
                    last_ty = self.check_stmt(&spanned.stmt, &mut child, &Type::Any);
                }
                Env::pop_scope(child, env);
                last_ty
            }
            Expr::Lambda { params, body } => {
                let mut child = Env::push_scope(env);
                let param_types: Vec<Type> = params.iter().map(|_| Type::Any).collect();
                for (name, ty) in params.iter().zip(param_types.iter()) {
                    child.insert(name.clone(), ty.clone(), false);
                }
                let ret = self.infer_expr(body, &mut child);
                Env::pop_scope(child, env);
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
                            let field_names: Vec<&str> = rd.fields.iter().map(|(n, _)| n.as_str()).collect();
                            let mut msg = format!("unknown field '{}' in type '{}'", f, type_name);
                            if let Some(suggestion) = suggest_similar(f, field_names.into_iter()) {
                                msg.push_str(&format!("; did you mean '{}'?", suggestion));
                            } else {
                                let available: Vec<&str> = rd.fields.iter().map(|(n, _)| n.as_str()).collect();
                                msg.push_str(&format!("; available fields: {}", available.join(", ")));
                            }
                            self.err(msg);
                        }
                    }
                    Type::Record(type_name.clone())
                } else if let Some(enum_name) = self.variant_to_enum.get(type_name).cloned() {
                    // Enum variant construction: Circle(radius: 5.0)
                    Type::Enum(enum_name)
                } else {
                    let mut msg = format!("unknown type '{}'", type_name);
                    if let Some(suggestion) = suggest_similar(type_name, self.all_type_names().into_iter()) {
                        msg.push_str(&format!("; did you mean '{}'?", suggestion));
                    }
                    self.err(msg);
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
                                let field_names: Vec<&str> = rd.fields.iter().map(|(n, _)| n.as_str()).collect();
                                let mut msg = format!("type '{}' has no field '{}'", name, field);
                                if let Some(suggestion) = suggest_similar(field, field_names.into_iter()) {
                                    msg.push_str(&format!("; did you mean '{}'?", suggestion));
                                }
                                self.err(msg);
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
                self.infer_method_return(&obj_ty, method)
            }

            Expr::TupleLit(elems) => {
                let types: Vec<Type> = elems.iter().map(|e| self.infer_expr(e, env)).collect();
                Type::Tuple(types)
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

            Expr::ListComp { expr, var, iterable, cond } => {
                let iter_ty = self.infer_expr(iterable, env);
                let elem_ty = match &iter_ty {
                    Type::List(t) => (**t).clone(),
                    _ => Type::Int, // range produces Int
                };
                env.insert(var.clone(), elem_ty, false);
                if let Some(c) = cond {
                    self.infer_expr(c, env);
                }
                let result_ty = self.infer_expr(expr, env);
                Type::List(Box::new(result_ty))
            }

            Expr::MapLit(entries) => {
                // Check if this is an anonymous record literal: all keys are bare identifiers
                let field_names: Option<Vec<&str>> = entries.iter().map(|(k, _)| {
                    if let Expr::Ident(name) = k { Some(name.as_str()) } else { None }
                }).collect();

                if let Some(names) = field_names {
                    if !names.is_empty() {
                        // Look for a record type whose fields match exactly
                        if let Some(record_name) = self.find_record_by_fields(&names) {
                            // Type-check field values
                            for (_k, v) in entries {
                                self.infer_expr(v, env);
                            }
                            return Type::Record(record_name);
                        }
                    }
                }

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
            Expr::Unwrap(inner) => {
                let t = self.infer_expr(inner, env);
                match t {
                    Type::Option(inner_ty) => *inner_ty,
                    _ => Type::Any,
                }
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
                self.check_bool_expr(cond, env, "assert");
                Type::Unit
            }
            Expr::AssertEq { left, right, .. } | Expr::AssertNe { left, right, .. } => {
                self.infer_expr(left, env);
                self.infer_expr(right, env);
                Type::Unit
            }
            Expr::Fork { input, branches } => {
                self.infer_expr(input, env);
                for b in branches {
                    self.infer_expr(b, env);
                }
                Type::List(Box::new(Type::Any))
            }
        }
    }

    /// Find a record type whose field names match the given names exactly.
    /// Returns the record type name if exactly one record matches.
    fn find_record_by_fields(&self, field_names: &[&str]) -> Option<String> {
        let mut matches = Vec::new();
        for (name, rd) in &self.records {
            let record_fields: Vec<&str> = rd.fields.iter().map(|(f, _)| f.as_str()).collect();
            if record_fields.len() == field_names.len()
                && field_names.iter().all(|f| record_fields.contains(f))
            {
                matches.push(name.clone());
            }
        }
        if matches.len() == 1 {
            Some(matches.into_iter().next().unwrap())
        } else {
            None
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

    fn infer_method_return(&self, obj_ty: &Type, method: &str) -> Type {
        match obj_ty {
            Type::Str => match method {
                "len" | "to_int" | "parse_int" | "count" => Type::Int,
                "to_float" | "parse_float" => Type::Float,
                "contains" | "starts_with" | "ends_with" | "is_empty" => Type::Bool,
                "trim" | "trim_start" | "trim_end" | "to_upper" | "to_lower" | "replace" | "substr" | "pad_left" | "pad_right" | "repeat" | "strip_prefix" | "strip_suffix" | "char_at" | "capitalize" => Type::Str,
                "split" | "words" | "lines" => Type::List(Box::new(Type::Str)),
                _ => Type::Any,
            },
            Type::List(elem) => match method {
                "len" | "sum" | "product" | "find_index" | "fold" => Type::Int,
                "average" => Type::Float,
                "push" | "set" | "each" | "each_with_index" | "map_with_index" => Type::Unit,
                "pop" | "get" | "get_or" | "find" | "min_by" | "max_by" => *elem.clone(),
                "contains" | "is_empty" => Type::Bool,
                "map" | "filter" | "sort" | "sort_by" | "reverse" | "window" | "chunks"
                | "take_while" | "drop_while" | "step" | "intersperse" | "dedup" => obj_ty.clone(),
                "partition" => Type::List(Box::new(obj_ty.clone())),
                "reduce" => Type::Any,
                "scan" => Type::List(Box::new(Type::Any)),
                "join" => Type::Str,
                "count_by" | "frequencies" => Type::Map(Box::new(Type::Str), Box::new(Type::Int)),
                "group_by" => Type::Map(Box::new(Type::Str), Box::new(obj_ty.clone())),
                "to_map" => Type::Map(Box::new(Type::Str), Box::new(Type::Any)),
                _ => Type::Any,
            },
            Type::Map(_, v) => match method {
                "len" => Type::Int,
                "contains" => Type::Bool,
                "get" | "get_or" => *v.clone(),
                "keys" => Type::List(Box::new(Type::Str)),
                "values" => Type::List(Box::new(*v.clone())),
                "set" | "remove" | "each" | "clear" => Type::Unit,
                "map" | "merge" | "filter" => obj_ty.clone(),
                "entries" => Type::List(Box::new(Type::List(Box::new(Type::Any)))),
                _ => Type::Any,
            },
            Type::Bool => match method {
                "to_str" => Type::Str,
                "to_int" => Type::Int,
                _ => Type::Any,
            },
            Type::Int => match method {
                "to_str" => Type::Str,
                "to_float" => Type::Float,
                "abs" | "pow" => Type::Int,
                _ => Type::Any,
            },
            Type::Float => match method {
                "to_str" => Type::Str,
                "to_int" => Type::Int,
                "abs" | "floor" | "ceil" | "round" | "min" | "max" | "pow" | "sqrt" => Type::Float,
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

#[cfg(test)]
mod tests {
    use super::*;
    use ore_lexer::lex;
    use ore_parser::parse;

    fn check(src: &str) -> Result<(), Vec<TypeError>> {
        let tokens = lex(src).expect("lex failed");
        let program = parse(tokens).expect("parse failed");
        typecheck(&program)
    }

    fn check_err(src: &str) -> Vec<TypeError> {
        check(src).expect_err("expected type error")
    }

    // --- Passing programs ---

    #[test]
    fn valid_simple_fn() {
        assert!(check("fn main\n  x := 42\n  print x\n").is_ok());
    }

    #[test]
    fn valid_typed_fn() {
        assert!(check("fn add a:Int b:Int -> Int\n  a + b\n\nfn main\n  print add(1, 2)\n").is_ok());
    }

    #[test]
    fn valid_bool_binding() {
        assert!(check("fn main\n  x := true\n  print x\n").is_ok());
    }

    #[test]
    fn valid_string_binding() {
        assert!(check("fn main\n  x := \"hello\"\n  print x\n").is_ok());
    }

    #[test]
    fn valid_mutable_variable() {
        assert!(check("fn main\n  mut x := 0\n  x = 1\n  print x\n").is_ok());
    }

    #[test]
    fn valid_if_else() {
        assert!(check("fn main\n  if true\n    print 1\n  else\n    print 2\n").is_ok());
    }

    #[test]
    fn valid_while_loop() {
        assert!(check("fn main\n  mut i := 0\n  while i < 10\n    i = i + 1\n").is_ok());
    }

    #[test]
    fn valid_for_loop() {
        assert!(check("fn main\n  for i in 0..10\n    print i\n").is_ok());
    }

    #[test]
    fn valid_list_literal() {
        assert!(check("fn main\n  xs := [1, 2, 3]\n  print xs\n").is_ok());
    }

    #[test]
    fn valid_record_def() {
        assert!(check("type Point { x:Int, y:Int }\n\nfn main\n  p := Point(x: 1, y: 2)\n  print p.x\n").is_ok());
    }

    // --- Error cases ---

    #[test]
    fn error_assign_to_immutable() {
        let errs = check_err("fn main\n  x := 0\n  x = 1\n");
        assert!(errs.iter().any(|e| e.msg.contains("immutable") || e.msg.contains("mutable") || e.msg.contains("mut")),
            "expected mutability error, got: {:?}", errs);
    }

    #[test]
    fn error_undefined_function() {
        let errs = check_err("fn main\n  foo()\n");
        assert!(errs.iter().any(|e| e.msg.contains("foo") || e.msg.contains("undefined") || e.msg.contains("unknown")),
            "expected undefined function error, got: {:?}", errs);
    }

    #[test]
    fn error_record_missing_field() {
        let errs = check_err("type Point { x:Int, y:Int }\n\nfn main\n  p := Point(x: 1)\n");
        assert!(errs.iter().any(|e| e.msg.to_lowercase().contains("field") || e.msg.to_lowercase().contains("missing")),
            "expected missing field error, got: {:?}", errs);
    }

    #[test]
    fn valid_recursion() {
        assert!(check("fn fib n:Int -> Int\n  if n < 2\n    n\n  else\n    fib(n - 1) + fib(n - 2)\n\nfn main\n  print fib(10)\n").is_ok());
    }

    #[test]
    fn valid_enum_def() {
        assert!(check("type Color\n  Red\n  Green\n  Blue\n\nfn main\n  c := Red\n  print c\n").is_ok());
    }

    #[test]
    fn valid_match() {
        assert!(check("fn main\n  x := 1\n  match x\n    1 -> print \"one\"\n    _ -> print \"other\"\n").is_ok());
    }

    #[test]
    fn valid_test_block() {
        assert!(check("fn main\n  print 1\n\ntest \"basic\"\n  assert true\n").is_ok());
    }

    #[test]
    fn valid_pipeline() {
        assert!(check("fn double x:Int -> Int\n  x * 2\n\nfn main\n  5 | double\n").is_ok());
    }

    #[test]
    fn valid_lambda() {
        assert!(check("fn main\n  f := (x => x + 1)\n  print f(5)\n").is_ok());
    }

    // --- Additional error cases ---

    #[test]
    fn error_fn_arg_type_mismatch() {
        let errs = check_err("fn add a:Int b:Int -> Int\n  a + b\n\nfn main\n  add(\"hello\", 42)\n");
        assert!(errs.iter().any(|e| e.msg.contains("argument") && e.msg.contains("expects")),
            "expected argument type mismatch error, got: {:?}", errs);
    }

    #[test]
    fn error_wrong_return_type() {
        let errs = check_err("fn greet -> Int\n  \"hello\"\n");
        assert!(errs.iter().any(|e| e.msg.contains("return")),
            "expected return type error, got: {:?}", errs);
    }

    #[test]
    fn error_enum_exhaustiveness() {
        let errs = check_err(
            "type Color\n  Red\n  Green\n  Blue\n\nfn main\n  c := Red\n  match c\n    Red -> print \"r\"\n    Green -> print \"g\"\n"
        );
        assert!(errs.iter().any(|e| e.msg.contains("exhaustive") || e.msg.contains("missing variant")),
            "expected exhaustiveness error, got: {:?}", errs);
    }

    #[test]
    fn error_arity_too_few() {
        let errs = check_err("fn add a:Int b:Int -> Int\n  a + b\n\nfn main\n  add()\n");
        assert!(errs.iter().any(|e| e.msg.contains("expects") && e.msg.contains("args")),
            "expected arity error, got: {:?}", errs);
    }

    #[test]
    fn error_arity_too_many() {
        let errs = check_err("fn add a:Int b:Int -> Int\n  a + b\n\nfn main\n  add(1, 2, 3)\n");
        assert!(errs.iter().any(|e| e.msg.contains("expects") && e.msg.contains("args")),
            "expected arity error, got: {:?}", errs);
    }

    #[test]
    fn error_undefined_variable_in_assign() {
        let errs = check_err("fn main\n  x = 42\n");
        assert!(errs.iter().any(|e| e.msg.contains("undefined") && e.msg.contains("x")),
            "expected undefined variable error, got: {:?}", errs);
    }

    #[test]
    fn error_unknown_type_in_record_construct() {
        let errs = check_err("fn main\n  p := Bogus(x: 1)\n");
        assert!(errs.iter().any(|e| e.msg.contains("unknown type") || e.msg.contains("undefined")),
            "expected unknown type error, got: {:?}", errs);
    }

    #[test]
    fn error_unknown_field_in_record_construct() {
        let errs = check_err("type Point { x:Int, y:Int }\n\nfn main\n  p := Point(x: 1, y: 2, z: 3)\n");
        assert!(errs.iter().any(|e| e.msg.contains("unknown field") && e.msg.contains("z")),
            "expected unknown field error, got: {:?}", errs);
    }

    #[test]
    fn error_field_access_nonexistent() {
        let errs = check_err("type Point { x:Int, y:Int }\n\nfn main\n  p := Point(x: 1, y: 2)\n  print p.z\n");
        assert!(errs.iter().any(|e| e.msg.contains("no field") && e.msg.contains("z")),
            "expected field access error, got: {:?}", errs);
    }

    #[test]
    fn error_field_assign_nonexistent() {
        let errs = check_err("type Point { x:Int, y:Int }\n\nfn main\n  mut p := Point(x: 1, y: 2)\n  p.z = 3\n");
        assert!(errs.iter().any(|e| e.msg.contains("no field") && e.msg.contains("z")),
            "expected field assign error, got: {:?}", errs);
    }

    #[test]
    fn error_return_type_mismatch_explicit() {
        let errs = check_err("fn foo -> Int\n  return \"hello\"\n");
        assert!(errs.iter().any(|e| e.msg.contains("return")),
            "expected return type mismatch error, got: {:?}", errs);
    }

    #[test]
    fn error_missing_return_value() {
        let errs = check_err("fn foo -> Int\n  return\n");
        assert!(errs.iter().any(|e| e.msg.contains("return") || e.msg.contains("missing")),
            "expected missing return value error, got: {:?}", errs);
    }

    #[test]
    fn error_assign_type_mismatch() {
        let errs = check_err("fn main\n  mut x := 42\n  x = \"hello\"\n");
        assert!(errs.iter().any(|e| e.msg.contains("type mismatch") && e.msg.contains("x")),
            "expected type mismatch in assignment error, got: {:?}", errs);
    }

    #[test]
    fn error_destructure_non_list() {
        let errs = check_err("fn main\n  [a, b] := 42\n");
        assert!(errs.iter().any(|e| e.msg.contains("destructuring") && e.msg.contains("list")),
            "expected destructuring error, got: {:?}", errs);
    }

    #[test]
    fn error_while_condition_not_bool() {
        let errs = check_err("fn main\n  while 42\n    print \"loop\"\n");
        assert!(errs.iter().any(|e| e.msg.contains("while") && e.msg.contains("Bool")),
            "expected while condition type error, got: {:?}", errs);
    }

    #[test]
    fn error_if_condition_not_bool() {
        let errs = check_err("fn main\n  if 42\n    print \"yes\"\n");
        assert!(errs.iter().any(|e| e.msg.contains("if") && e.msg.contains("Bool")),
            "expected if condition type error, got: {:?}", errs);
    }

    #[test]
    fn error_for_in_non_int_bounds() {
        let errs = check_err("fn main\n  for i in \"a\"..\"z\"\n    print i\n");
        assert!(errs.iter().any(|e| e.msg.contains("for-in") && e.msg.contains("Int")),
            "expected for-in type error, got: {:?}", errs);
    }

    // --- Anonymous record literal inference ---

    #[test]
    fn anon_record_literal_inferred_as_named_type() {
        // {x: 1, y: 2} should be inferred as Point when Point has fields x and y
        assert!(check("type Point { x:Int, y:Int }\n\nfn main\n  p := {x: 1, y: 2}\n  print p.x\n").is_ok());
    }

    #[test]
    fn anon_record_literal_field_access() {
        // Accessing fields on anonymous record literal should work
        assert!(check("type Person { name:Str, age:Int }\n\nfn main\n  p := {name: \"Alice\", age: 30}\n  print p.name\n").is_ok());
    }

    #[test]
    fn anon_record_literal_as_function_arg() {
        // Anonymous record literal passed to a function expecting the record type
        assert!(check(
            "type Point { x:Int, y:Int }\n\nfn show p:Point -> Str\n  \"point\"\n\nfn main\n  show({x: 1, y: 2})\n"
        ).is_ok());
    }

    // --- "Did you mean" suggestion tests ---

    #[test]
    fn suggestion_undefined_function_typo() {
        // "prnt" should suggest "print" isn't user-defined, but "greet" -> "great" should work
        let errs = check_err("fn greet\n  print \"hi\"\n\nfn main\n  gret()\n");
        assert!(errs.iter().any(|e| e.msg.contains("did you mean 'greet'")),
            "expected suggestion for 'gret', got: {:?}", errs);
    }

    #[test]
    fn suggestion_undefined_variable_typo() {
        let errs = check_err("fn main\n  count := 42\n  cont = 1\n");
        assert!(errs.iter().any(|e| e.msg.contains("did you mean 'count'")),
            "expected suggestion for 'cont', got: {:?}", errs);
    }

    #[test]
    fn suggestion_unknown_type_typo() {
        let errs = check_err("type Point { x:Int, y:Int }\n\nfn main\n  p := Piont(x: 1, y: 2)\n");
        assert!(errs.iter().any(|e| e.msg.contains("did you mean 'Point'")),
            "expected suggestion for 'Piont', got: {:?}", errs);
    }

    #[test]
    fn suggestion_unknown_field_typo() {
        let errs = check_err("type Point { x:Int, y:Int }\n\nfn main\n  p := Point(x: 1, z: 2)\n");
        let err = errs.iter().find(|e| e.msg.contains("unknown field 'z'")).unwrap();
        assert!(err.msg.contains("did you mean"),
            "expected did-you-mean suggestion for 'z', got: {:?}", err.msg);
    }

    #[test]
    fn suggestion_field_access_typo() {
        let errs = check_err("type Point { x:Int, y:Int }\n\nfn main\n  p := Point(x: 1, y: 2)\n  print p.z\n");
        let err = errs.iter().find(|e| e.msg.contains("no field 'z'")).unwrap();
        // z is distance 1 from both x and y, so we should get a suggestion
        assert!(err.msg.contains("did you mean"),
            "expected did-you-mean suggestion, got: {:?}", err.msg);
    }

    #[test]
    fn suggestion_undefined_var_hints_declare() {
        // When no similar name exists, suggest using ':='
        let errs = check_err("fn main\n  xyzzy = 42\n");
        assert!(errs.iter().any(|e| e.msg.contains("use ':=' to declare")),
            "expected ':=' hint, got: {:?}", errs);
    }

    #[test]
    fn suggestion_immutable_assign_hints_mut() {
        let errs = check_err("fn main\n  x := 0\n  x = 1\n");
        assert!(errs.iter().any(|e| e.msg.contains("mut")),
            "expected mut hint, got: {:?}", errs);
    }

    #[test]
    fn edit_distance_basic() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("Point", "Piont"), 2);
    }
}
