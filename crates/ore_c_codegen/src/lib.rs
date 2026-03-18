#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use ore_parser::ast::*;

mod expr;
mod stmts;
mod match_compile;
mod methods;
mod builtins;
mod str_methods;
mod list;
mod map;
mod lambda;
mod util;
mod runtime_decls;
mod type_resolution;
mod assembly;

/// Tracks semantic type of compiled values (mirrors ore_codegen::ValKind)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValKind {
    Int,
    Float,
    Bool,
    Str,
    Void,
    Record(String),
    Enum(String),
    Option,
    Result,
    List(std::option::Option<Box<ValKind>>),
    Map(std::option::Option<Box<ValKind>>),
    Channel,
}

impl ValKind {
    pub fn is_list(&self) -> bool { matches!(self, ValKind::List(_)) }
    pub fn is_map(&self) -> bool { matches!(self, ValKind::Map(_)) }

    pub fn list_elem_kind(&self) -> std::option::Option<&ValKind> {
        match self {
            ValKind::List(Some(k)) => Some(k),
            _ => None,
        }
    }

    pub fn map_val_kind(&self) -> std::option::Option<&ValKind> {
        match self {
            ValKind::Map(Some(k)) => Some(k),
            _ => None,
        }
    }

    pub fn list_of(kind: ValKind) -> ValKind {
        ValKind::List(Some(Box::new(kind)))
    }

    pub fn map_of(kind: ValKind) -> ValKind {
        ValKind::Map(Some(Box::new(kind)))
    }
}

/// Info about a record type
#[derive(Clone)]
struct RecordInfo {
    field_names: Vec<String>,
    field_kinds: Vec<ValKind>,
}

/// Info about a single enum variant
#[derive(Clone)]
struct VariantInfo {
    name: String,
    tag: u8,
    field_names: Vec<String>,
    field_kinds: Vec<ValKind>,
}

/// Info about an enum type
#[derive(Clone)]
struct EnumInfo {
    variants: Vec<VariantInfo>,
    /// Number of i64s in the data array: ceil(max_payload_size / 8)
    num_data_i64s: u32,
}

/// Info about a declared function
#[derive(Clone)]
struct FnInfo {
    ret_kind: ValKind,
    param_kinds: Vec<ValKind>,
}

/// Info about a local variable in the current function
#[derive(Clone)]
struct VarInfo {
    /// C variable name (may be mangled)
    c_name: String,
    kind: ValKind,
    is_mutable: bool,
}

#[derive(Debug, Default)]
pub struct CCodeGenError {
    pub msg: String,
    pub line: std::option::Option<usize>,
}

impl std::fmt::Display for CCodeGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line {
            write!(f, "line {}: {}", line, self.msg)
        } else {
            write!(f, "{}", self.msg)
        }
    }
}

pub struct CCodeGen {
    /// Emitted C code lines
    lines: Vec<String>,
    /// Current indentation level
    indent: u32,
    /// Local variables in the current function
    variables: HashMap<String, VarInfo>,
    /// Declared functions and their return types
    functions: HashMap<String, FnInfo>,
    /// Record type definitions
    records: HashMap<String, RecordInfo>,
    /// Enum type definitions
    enums: HashMap<String, EnumInfo>,
    /// Maps variant name -> enum name
    variant_to_enum: HashMap<String, String>,
    /// Counter for unique temporary variable names
    temp_counter: u32,
    /// Counter for string literal globals
    str_counter: u32,
    /// Counter for lambda functions
    lambda_counter: u32,
    /// Current source line for error reporting
    current_line: usize,
    /// Generic function definitions (not yet monomorphized)
    generic_fns: HashMap<String, FnDef>,
    /// Default parameter expressions per function
    fn_defaults: HashMap<String, Vec<std::option::Option<Expr>>>,
    /// Tracks element kind for functions returning List[T]
    fn_return_list_elem_kind: HashMap<String, ValKind>,
    /// Tracks value kind for functions returning Map[K, V]
    fn_return_map_val_kind: HashMap<String, ValKind>,
    /// Maps variable name -> element ValKind for typed lists
    list_element_kinds: HashMap<String, ValKind>,
    /// Maps variable name -> value ValKind for typed maps
    map_value_kinds: HashMap<String, ValKind>,
    /// Test function names in order
    pub test_names: Vec<String>,
    /// Forward declarations (function prototypes)
    forward_decls: Vec<String>,
    /// Top-level code (struct defs, globals)
    top_level: Vec<String>,
    /// Lambda function bodies (emitted before main functions)
    lambda_bodies: Vec<String>,
    /// Names of captured variables per lambda
    lambda_captures: HashMap<String, Vec<(String, ValKind)>>,
    /// Tracks variables that have dynamic kind tags (from Option/Result match)
    dynamic_kind_tags: HashSet<String>,
    /// Maps a C value expression to its runtime kind-tag variable (for untyped
    /// list element access).  Consumed by `value_to_str_expr` to emit
    /// `ore_dynamic_to_str` instead of the default `ore_int_to_str`.
    dynamic_kind_exprs: HashMap<String, String>,
    /// Break/continue label stack
    break_labels: Vec<String>,
    continue_labels: Vec<String>,
    /// Label counter for unique loop labels
    label_counter: u32,
    /// Tracks which function bodies have already been compiled (dedup guard)
    compiled_functions: HashSet<String>,
}

impl Default for CCodeGen {
    fn default() -> Self {
        Self::new()
    }
}

impl CCodeGen {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            indent: 0,
            variables: HashMap::new(),
            functions: HashMap::new(),
            records: HashMap::new(),
            enums: HashMap::new(),
            variant_to_enum: HashMap::new(),
            temp_counter: 0,
            str_counter: 0,
            lambda_counter: 0,
            current_line: 0,
            generic_fns: HashMap::new(),
            fn_defaults: HashMap::new(),
            fn_return_list_elem_kind: HashMap::new(),
            fn_return_map_val_kind: HashMap::new(),
            list_element_kinds: HashMap::new(),
            map_value_kinds: HashMap::new(),
            test_names: Vec::new(),
            forward_decls: Vec::new(),
            top_level: Vec::new(),
            lambda_bodies: Vec::new(),
            lambda_captures: HashMap::new(),
            dynamic_kind_tags: HashSet::new(),
            dynamic_kind_exprs: HashMap::new(),
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
            label_counter: 0,
            compiled_functions: HashSet::new(),
        }
    }

    /// Emit a line of C code at the current indentation level.
    fn emit(&mut self, line: &str) {
        let indent_str = "    ".repeat(self.indent as usize);
        self.lines.push(format!("{}{}", indent_str, line));
    }

    /// Emit a line with no indentation.
    fn emit_raw(&mut self, line: &str) {
        self.lines.push(line.to_string());
    }

    /// Generate a unique temporary variable name.
    fn tmp(&mut self) -> String {
        let name = format!("__tmp_{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }

    /// Generate a unique label name.
    fn label(&mut self, prefix: &str) -> String {
        let name = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        name
    }

    /// Create an error at the current source line.
    fn err(&self, msg: impl Into<String>) -> CCodeGenError {
        CCodeGenError { line: Some(self.current_line), msg: msg.into() }
    }

    fn check_arity(&self, name: &str, args: &[Expr], expected: usize) -> Result<(), CCodeGenError> {
        if args.len() != expected {
            Err(self.err(format!("{} takes {} argument(s)", name, expected)))
        } else {
            Ok(())
        }
    }

    /// Mangle a name for use as a C identifier.
    fn mangle_name(name: &str) -> String {
        name.replace("::", "__").replace("$", "_D_")
    }

    /// C reserved words that may conflict with Ore function names.
    const C_RESERVED: &'static [&'static str] = &[
        "auto", "break", "case", "char", "const", "continue", "default", "do",
        "double", "else", "enum", "extern", "float", "for", "goto", "if",
        "int", "long", "register", "return", "short", "signed", "sizeof",
        "static", "struct", "switch", "typedef", "union", "unsigned", "void",
        "volatile", "while", "abs", "malloc", "free", "printf", "scanf",
        "exit", "rand", "main",
    ];

    /// Mangle a function name to avoid C keyword/stdlib conflicts.
    fn mangle_fn_name(name: &str) -> String {
        if name == "main" {
            return name.to_string();
        }
        if Self::C_RESERVED.contains(&name) {
            format!("ore_fn_{}", name)
        } else {
            name.to_string()
        }
    }

    /// Mangle a variable/parameter name to avoid C keyword conflicts.
    fn mangle_var_name(name: &str) -> String {
        if Self::C_RESERVED.contains(&name) {
            format!("ore_v_{}", name)
        } else {
            name.to_string()
        }
    }

    /// Resolve Self type references in impl blocks.
    fn resolve_self_type(ty: &TypeExpr, type_name: &str) -> TypeExpr {
        match ty {
            TypeExpr::Named(n) if n == "Self" => TypeExpr::Named(type_name.to_string()),
            TypeExpr::Generic(n, args) => {
                let new_name = if n == "Self" { type_name.to_string() } else { n.clone() };
                let new_args = args.iter().map(|a| Self::resolve_self_type(a, type_name)).collect();
                TypeExpr::Generic(new_name, new_args)
            }
            TypeExpr::Fn { params, ret } => {
                let new_params = params.iter().map(|p| Self::resolve_self_type(p, type_name)).collect();
                let new_ret = Box::new(Self::resolve_self_type(ret, type_name));
                TypeExpr::Fn { params: new_params, ret: new_ret }
            }
            _ => ty.clone(),
        }
    }

    fn resolve_self_in_params(params: &[Param], type_name: &str) -> Vec<Param> {
        params.iter().map(|p| Param {
            name: p.name.clone(),
            ty: Self::resolve_self_type(&p.ty, type_name),
            default: p.default.clone(),
        }).collect()
    }

    fn impl_items(items: &[Item]) -> impl Iterator<Item = (&String, &Vec<FnDef>)> {
        items.iter().filter_map(|item| match item {
            Item::ImplBlock { type_name, methods }
            | Item::ImplTrait { type_name, methods, .. } => Some((type_name, methods)),
            _ => None,
        })
    }

    fn mangle_impl_method(type_name: &str, method: &FnDef) -> FnDef {
        let resolved_params = Self::resolve_self_in_params(&method.params, type_name);
        let resolved_ret = method.ret_type.as_ref().map(|r| Self::resolve_self_type(r, type_name));
        let has_self = resolved_params.first().is_some_and(|p| p.name == "self");
        let params = if has_self {
            resolved_params
        } else {
            let mut p = vec![Param {
                name: "self".to_string(),
                ty: TypeExpr::Named(type_name.to_string()),
                default: None,
            }];
            p.extend(resolved_params);
            p
        };
        FnDef {
            name: format!("{}_{}", type_name, method.name),
            type_params: method.type_params.clone(),
            params,
            ret_type: resolved_ret,
            body: method.body.clone(),
        }
    }

    /// Register a record type — emit C struct definition.
    fn register_record(&mut self, td: &TypeDef) -> Result<(), CCodeGenError> {
        let mut field_kinds = Vec::new();
        let mut field_names = Vec::new();
        let mut c_fields = Vec::new();

        for f in &td.fields {
            let kind = self.type_expr_to_kind(&f.ty);
            let c_type = self.kind_to_c_type_str(&kind);
            c_fields.push(format!("    {} {};", c_type, f.name));
            field_kinds.push(kind);
            field_names.push(f.name.clone());
        }

        let struct_name = format!("ore_rec_{}", Self::mangle_name(&td.name));
        self.top_level.push(format!("struct {} {{", struct_name));
        for cf in &c_fields {
            self.top_level.push(cf.clone());
        }
        self.top_level.push("};".to_string());
        self.top_level.push(String::new());

        self.records.insert(td.name.clone(), RecordInfo { field_names, field_kinds });
        Ok(())
    }

    /// Register an enum type — emit C struct definition.
    fn register_enum(&mut self, ed: &EnumDef) -> Result<(), CCodeGenError> {
        let mut variants = Vec::new();
        let mut max_payload_bytes: u64 = 0;

        for (i, v) in ed.variants.iter().enumerate() {
            let mut field_names = Vec::new();
            let mut field_kinds = Vec::new();
            let mut payload_bytes: u64 = 0;

            for f in &v.fields {
                let kind = self.type_expr_to_kind(&f.ty);
                payload_bytes += self.kind_size_bytes(&kind);
                field_kinds.push(kind);
                field_names.push(f.name.clone());
            }

            if payload_bytes > max_payload_bytes {
                max_payload_bytes = payload_bytes;
            }

            variants.push(VariantInfo {
                name: v.name.clone(),
                tag: i as u8,
                field_names,
                field_kinds,
            });

            self.variant_to_enum.insert(v.name.clone(), ed.name.clone());
        }

        let num_data_i64s = max_payload_bytes.div_ceil(8) as u32;

        // Emit C struct: { int8_t tag; int64_t data[N]; }
        let struct_name = format!("ore_enum_{}", Self::mangle_name(&ed.name));
        self.top_level.push(format!("struct {} {{", struct_name));
        self.top_level.push("    int8_t tag;".to_string());
        if num_data_i64s > 0 {
            self.top_level.push(format!("    int64_t data[{}];", num_data_i64s));
        }
        self.top_level.push("};".to_string());

        // Emit payload structs for each variant with fields
        for v in &variants {
            if !v.field_names.is_empty() {
                let payload_name = format!("ore_payload_{}_{}", Self::mangle_name(&ed.name), v.name);
                self.top_level.push(format!("struct {} {{", payload_name));
                for (fname, fkind) in v.field_names.iter().zip(v.field_kinds.iter()) {
                    let c_type = self.kind_to_c_type_str(fkind);
                    self.top_level.push(format!("    {} {};", c_type, fname));
                }
                self.top_level.push("};".to_string());
            }
        }
        self.top_level.push(String::new());

        self.enums.insert(ed.name.clone(), EnumInfo { variants, num_data_i64s });
        Ok(())
    }

    /// Declare a function — emit C prototype.
    /// Skips duplicate declarations (same function name from multiple source files).
    fn declare_function(&mut self, fndef: &FnDef) -> Result<(), CCodeGenError> {
        // Skip if already declared — handles cross-module and same-file duplicates
        if self.functions.contains_key(&fndef.name) {
            return Ok(());
        }
        let ret_kind = fndef.ret_type.as_ref().map(|t| self.type_expr_to_kind(t)).unwrap_or(ValKind::Void);
        let c_fn_name = Self::mangle_fn_name(&fndef.name);

        let mut param_kinds = Vec::new();
        let mut param_strs = Vec::new();
        for p in &fndef.params {
            let kind = self.type_expr_to_kind(&p.ty);
            let c_type = self.kind_to_c_type_str(&kind);
            let c_param_name = Self::mangle_var_name(&p.name);
            param_strs.push(format!("{} {}", c_type, c_param_name));
            param_kinds.push(kind);
        }

        let ret_c_type = if fndef.name == "main" {
            "int32_t".to_string()
        } else if ret_kind == ValKind::Void {
            "void".to_string()
        } else {
            self.kind_to_c_type_str(&ret_kind)
        };

        let params_str = if param_strs.is_empty() { "void".to_string() } else { param_strs.join(", ") };
        let proto = format!("{} {}({})", ret_c_type, c_fn_name, params_str);
        self.forward_decls.push(format!("{};", proto));

        self.functions.insert(fndef.name.clone(), FnInfo {
            ret_kind: ret_kind.clone(),
            param_kinds,
        });

        // Track return type annotations for List[T] and Map[K,V]
        if let Some(TypeExpr::Generic(base, args)) = &fndef.ret_type {
            if base == "List" && !args.is_empty() {
                let elem_kind = self.type_expr_to_kind(&args[0]);
                self.fn_return_list_elem_kind.insert(fndef.name.clone(), elem_kind);
            }
            if base == "Map" && args.len() >= 2 {
                let val_kind = self.type_expr_to_kind(&args[1]);
                self.fn_return_map_val_kind.insert(fndef.name.clone(), val_kind);
            }
        }

        // Store default parameter expressions
        let defaults: Vec<std::option::Option<Expr>> = fndef.params.iter().map(|p| p.default.clone()).collect();
        if defaults.iter().any(|d| d.is_some()) {
            self.fn_defaults.insert(fndef.name.clone(), defaults);
        }

        Ok(())
    }

    /// Compile a function body — emit C function definition.
    /// Skips duplicate compilations (same function name from multiple source files).
    fn compile_function(&mut self, fndef: &FnDef) -> Result<(), CCodeGenError> {
        // Skip if already compiled — handles cross-module and same-file duplicates
        if !self.compiled_functions.insert(fndef.name.clone()) {
            return Ok(());
        }
        let fn_info = self.functions.get(&fndef.name).cloned()
            .ok_or_else(|| self.err(format!("undefined function '{}'", fndef.name)))?;
        let c_fn_name = Self::mangle_fn_name(&fndef.name);

        self.variables.clear();
        self.dynamic_kind_tags.clear();
        self.dynamic_kind_exprs.clear();
        let saved_list_ek = std::mem::take(&mut self.list_element_kinds);
        let saved_map_vk = std::mem::take(&mut self.map_value_kinds);

        let ret_c_type = if fndef.name == "main" {
            "int32_t".to_string()
        } else if fn_info.ret_kind == ValKind::Void {
            "void".to_string()
        } else {
            self.kind_to_c_type_str(&fn_info.ret_kind)
        };

        let mut param_strs = Vec::new();
        for (i, p) in fndef.params.iter().enumerate() {
            let kind = &fn_info.param_kinds[i];
            let c_type = self.kind_to_c_type_str(kind);
            let c_param_name = Self::mangle_var_name(&p.name);
            param_strs.push(format!("{} {}", c_type, c_param_name));
            self.variables.insert(p.name.clone(), VarInfo {
                c_name: c_param_name.clone(),
                kind: kind.clone(),
                is_mutable: false,
            });
            // Track element/value kinds from type annotations
            if kind.is_list() {
                if let TypeExpr::Generic(_, args) = &p.ty {
                    if let Some(elem_ty) = args.first() {
                        let elem_kind = self.type_expr_to_kind(elem_ty);
                        self.list_element_kinds.insert(p.name.clone(), elem_kind.clone());
                        if let Some(var) = self.variables.get_mut(&p.name) {
                            var.kind = ValKind::list_of(elem_kind);
                        }
                    }
                }
            }
            if kind.is_map() {
                if let TypeExpr::Generic(_, args) = &p.ty {
                    if args.len() >= 2 {
                        let val_kind = self.type_expr_to_kind(&args[1]);
                        self.map_value_kinds.insert(p.name.clone(), val_kind);
                    }
                }
            }
        }

        let params_str = if param_strs.is_empty() { "void".to_string() } else { param_strs.join(", ") };
        self.emit_raw(&format!("{} {}({}) {{", ret_c_type, c_fn_name, params_str));
        self.indent += 1;

        // Compile body
        let (last_expr, _last_kind) = self.compile_block_stmts(&fndef.body)?;

        // Return handling
        if fndef.name == "main" {
            self.emit("ore_thread_join_all();");
            self.emit("return 0;");
        } else if fndef.ret_type.is_some() {
            if let Some(ref expr_str) = last_expr {
                // Always coerce the return expression to match the declared return type.
                // The compiled expression kind may differ from the declared return kind
                // when list elements have unknown types or cross-function inference is imprecise.
                if _last_kind != fn_info.ret_kind {
                    let ret_expr = self.coerce_expr(expr_str, &_last_kind, &fn_info.ret_kind);
                    if ret_expr != *expr_str {
                        self.emit(&format!("return {};", ret_expr));
                    } else {
                        // coerce_expr couldn't do a direct conversion; try via i64
                        let ret_expr = self.coerce_from_i64_expr(expr_str, &fn_info.ret_kind);
                        self.emit(&format!("return {};", ret_expr));
                    }
                } else {
                    self.emit(&format!("return {};", expr_str));
                }
            }
        }

        // After compiling the body, if the function returns List(None) or Map(None),
        // try to infer element/value kinds from the last expression. Only infer when
        // ALL return paths agree on the element/value kind — functions with mixed-type
        // returns (e.g. error path returns [string], success path returns [list, ...])
        // must use dynamic dispatch at the call site.
        if matches!(fn_info.ret_kind, ValKind::List(None))
            && !self.fn_return_list_elem_kind.contains_key(&fndef.name) {
                let all_idents = Self::collect_return_idents(&fndef.body);
                let kinds: Vec<_> = all_idents.iter()
                    .filter_map(|n| self.list_element_kinds.get(n))
                    .collect();
                // Only infer if all return paths have tracked element kinds and they agree
                if !kinds.is_empty() && kinds.iter().all(|k| *k == kinds[0]) {
                    self.fn_return_list_elem_kind.insert(fndef.name.clone(), kinds[0].clone());
                }
            }
        if matches!(fn_info.ret_kind, ValKind::Map(None))
            && !self.fn_return_map_val_kind.contains_key(&fndef.name) {
                let all_idents = Self::collect_return_idents(&fndef.body);
                let kinds: Vec<_> = all_idents.iter()
                    .filter_map(|n| self.map_value_kinds.get(n))
                    .collect();
                if !kinds.is_empty() && kinds.iter().all(|k| *k == kinds[0]) {
                    self.fn_return_map_val_kind.insert(fndef.name.clone(), kinds[0].clone());
                }
            }

        self.indent -= 1;
        self.emit_raw("}");
        self.emit_raw("");

        // Restore per-function element/value kind tracking
        self.list_element_kinds = saved_list_ek;
        self.map_value_kinds = saved_map_vk;
        Ok(())
    }

    /// Compile a block of statements.
    /// Returns (Option<last_expr_string>, last_kind).
    pub(crate) fn compile_block_stmts(&mut self, block: &Block) -> Result<(std::option::Option<String>, ValKind), CCodeGenError> {
        let mut last_expr = None;
        let mut last_kind = ValKind::Void;
        for spanned in &block.stmts {
            self.current_line = spanned.line;
            let (expr, kind) = self.compile_stmt(&spanned.stmt).map_err(|mut e| {
                if e.line.is_none() { e.line = Some(spanned.line); }
                e
            })?;
            if expr.is_some() {
                last_expr = expr;
                last_kind = kind;
            }
        }
        Ok((last_expr, last_kind))
    }

    /// Compile the full program, returning the generated C code as a string.
    pub fn compile_program(&mut self, program: &Program) -> Result<String, CCodeGenError> {
        self.register_types(&program.items)?;
        self.declare_all_functions(&program.items)?;
        self.compile_all_functions(&program.items)?;
        self.compile_tests(&program.items)?;
        Ok(self.assemble())
    }

    /// Register type and enum definitions.
    fn register_types(&mut self, items: &[Item]) -> Result<(), CCodeGenError> {
        for item in items {
            match item {
                Item::TypeDef(td) => self.register_record(td)?,
                Item::EnumDef(ed) => self.register_enum(ed)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// Declare all regular functions (registering generics) and impl methods.
    fn declare_all_functions(&mut self, items: &[Item]) -> Result<(), CCodeGenError> {
        for item in items {
            if let Item::FnDef(f) = item {
                if !f.type_params.is_empty() {
                    self.generic_fns.insert(f.name.clone(), f.clone());
                } else {
                    self.declare_function(f)?;
                }
            }
        }
        for (type_name, methods) in Self::impl_items(items) {
            for method in methods {
                let mangled_fn = Self::mangle_impl_method(type_name, method);
                self.declare_function(&mangled_fn)?;
            }
        }
        Ok(())
    }

    /// Compile all regular functions and impl methods.
    fn compile_all_functions(&mut self, items: &[Item]) -> Result<(), CCodeGenError> {
        for item in items {
            if let Item::FnDef(f) = item {
                if f.type_params.is_empty() {
                    self.compile_function(f)?;
                }
            }
        }
        let impl_fns: Vec<_> = Self::impl_items(items)
            .flat_map(|(type_name, methods)| {
                methods.iter().map(move |m| Self::mangle_impl_method(type_name, m))
            })
            .collect();
        for mangled_fn in impl_fns {
            self.compile_function(&mangled_fn)?;
        }
        Ok(())
    }

    /// Compile test definitions and generate a test runner if needed.
    fn compile_tests(&mut self, items: &[Item]) -> Result<(), CCodeGenError> {
        let mut test_idx = 0;
        for item in items {
            if let Item::TestDef { name, body } = item {
                let fn_name = format!("ore_test_{}", test_idx);
                self.test_names.push(name.clone());
                let test_fn = FnDef {
                    name: fn_name,
                    type_params: vec![],
                    params: vec![],
                    ret_type: None,
                    body: body.clone(),
                };
                self.declare_function(&test_fn)?;
                self.compile_function(&test_fn)?;
                test_idx += 1;
            }
        }
        if !self.test_names.is_empty() && !self.functions.contains_key("main") {
            self.emit_test_runner_main();
        }
        Ok(())
    }

    /// Generate C code for a string literal, returning the C expression.
    pub(crate) fn compile_string_literal(&mut self, s: &str) -> String {
        let escaped = Self::escape_c_string(s);
        let len = s.len();
        format!("ore_str_new(\"{}\", {})", escaped, len)
    }

    /// Escape a string for use in a C string literal.
    fn escape_c_string(s: &str) -> String {
        let mut out = String::new();
        for c in s.chars() {
            match c {
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\0' => out.push_str("\\0"),
                c if c.is_ascii_control() => out.push_str(&format!("\\x{:02x}", c as u32)),
                c => out.push(c),
            }
        }
        out
    }

    /// Track variable kinds for list/map element types.
    /// Find the name of the variable in the return position of a block.
    /// Looks at the last statement, recursing into if/else branches.
    fn find_return_ident(block: &Block) -> std::option::Option<String> {
        if let Some(last) = block.stmts.last() {
            match &last.stmt {
                Stmt::Expr(Expr::Ident(name)) => Some(name.clone()),
                // Method call on a variable (e.g. result.skip(1)) — return the object var
                Stmt::Expr(Expr::MethodCall { object, .. }) => {
                    if let Expr::Ident(name) = object.as_ref() {
                        Some(name.clone())
                    } else {
                        None
                    }
                }
                Stmt::Expr(Expr::IfElse { then_block, else_block, .. }) => {
                    Self::find_return_ident(then_block)
                        .or_else(|| else_block.as_ref().and_then(Self::find_return_ident))
                }
                Stmt::Return(Some(Expr::Ident(name))) => Some(name.clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Collect ALL return variable names from a block, including early `return`
    /// statements in nested control flow. Used to verify that all return paths
    /// agree on list element / map value kinds before inferring.
    fn collect_return_idents(block: &Block) -> Vec<String> {
        let mut idents = Vec::new();
        for spanned in &block.stmts {
            match &spanned.stmt {
                Stmt::Return(Some(Expr::Ident(name))) => {
                    idents.push(name.clone());
                }
                Stmt::Expr(Expr::IfElse { then_block, else_block, .. }) => {
                    idents.extend(Self::collect_return_idents(then_block));
                    if let Some(eb) = else_block {
                        idents.extend(Self::collect_return_idents(eb));
                    }
                }
                Stmt::ForIn { body, .. } | Stmt::While { body, .. }
                | Stmt::ForEach { body, .. } | Stmt::ForEachKV { body, .. }
                | Stmt::Loop { body, .. } => {
                    idents.extend(Self::collect_return_idents(body));
                }
                _ => {}
            }
        }
        // Include the implicit return (last expression)
        if let Some(last) = block.stmts.last() {
            match &last.stmt {
                Stmt::Expr(Expr::Ident(name)) => idents.push(name.clone()),
                Stmt::Expr(Expr::MethodCall { object, .. }) => {
                    if let Expr::Ident(name) = object.as_ref() {
                        idents.push(name.clone());
                    }
                }
                _ => {}
            }
        }
        idents
    }

    fn track_variable_kinds(&mut self, name: &str, kind: &ValKind) {
        if let ValKind::List(Some(ref ek)) = kind {
            self.list_element_kinds.insert(name.to_string(), ek.as_ref().clone());
        }
        if let ValKind::Map(Some(ref vk)) = kind {
            self.map_value_kinds.insert(name.to_string(), vk.as_ref().clone());
        }
        if kind.is_list() || kind.is_map() {
            if let Some(var) = self.variables.get_mut(name) {
                var.kind = kind.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: parse an ore program and compile to C.
    fn compile_ore_to_c(src: &str) -> String {
        let tokens = ore_lexer::lex(src).expect("lex failed");
        let program = ore_parser::parse(tokens).expect("parse failed");
        let mut cg = CCodeGen::new();
        cg.compile_program(&program).expect("compile failed")
    }

    // ---------------------------------------------------------------
    // ValKind helper methods
    // ---------------------------------------------------------------

    #[test]
    fn valkind_is_list() {
        assert!(ValKind::List(None).is_list());
        assert!(ValKind::list_of(ValKind::Int).is_list());
        assert!(!ValKind::Int.is_list());
        assert!(!ValKind::Map(None).is_list());
    }

    #[test]
    fn valkind_is_map() {
        assert!(ValKind::Map(None).is_map());
        assert!(ValKind::map_of(ValKind::Str).is_map());
        assert!(!ValKind::Int.is_map());
        assert!(!ValKind::List(None).is_map());
    }

    #[test]
    fn valkind_list_elem_kind() {
        assert_eq!(ValKind::list_of(ValKind::Int).list_elem_kind(), Some(&ValKind::Int));
        assert_eq!(ValKind::List(None).list_elem_kind(), None);
        assert_eq!(ValKind::Int.list_elem_kind(), None);
    }

    #[test]
    fn valkind_map_val_kind() {
        assert_eq!(ValKind::map_of(ValKind::Str).map_val_kind(), Some(&ValKind::Str));
        assert_eq!(ValKind::Map(None).map_val_kind(), None);
        assert_eq!(ValKind::Int.map_val_kind(), None);
    }

    #[test]
    fn valkind_list_of_nested() {
        let nested = ValKind::list_of(ValKind::list_of(ValKind::Int));
        assert!(nested.is_list());
        let inner = nested.list_elem_kind().unwrap();
        assert!(inner.is_list());
        assert_eq!(inner.list_elem_kind(), Some(&ValKind::Int));
    }

    // ---------------------------------------------------------------
    // Name mangling
    // ---------------------------------------------------------------

    #[test]
    fn mangle_name_replaces_colons_and_dollar() {
        assert_eq!(CCodeGen::mangle_name("Foo::bar"), "Foo__bar");
        assert_eq!(CCodeGen::mangle_name("a$b"), "a_D_b");
        assert_eq!(CCodeGen::mangle_name("simple"), "simple");
    }

    #[test]
    fn mangle_fn_name_preserves_main() {
        assert_eq!(CCodeGen::mangle_fn_name("main"), "main");
    }

    #[test]
    fn mangle_fn_name_prefixes_reserved_words() {
        assert_eq!(CCodeGen::mangle_fn_name("int"), "ore_fn_int");
        assert_eq!(CCodeGen::mangle_fn_name("return"), "ore_fn_return");
        assert_eq!(CCodeGen::mangle_fn_name("abs"), "ore_fn_abs");
        assert_eq!(CCodeGen::mangle_fn_name("malloc"), "ore_fn_malloc");
    }

    #[test]
    fn mangle_fn_name_passes_through_normal() {
        assert_eq!(CCodeGen::mangle_fn_name("compute"), "compute");
        assert_eq!(CCodeGen::mangle_fn_name("my_func"), "my_func");
    }

    #[test]
    fn mangle_var_name_prefixes_reserved_words() {
        assert_eq!(CCodeGen::mangle_var_name("int"), "ore_v_int");
        assert_eq!(CCodeGen::mangle_var_name("for"), "ore_v_for");
        assert_eq!(CCodeGen::mangle_var_name("void"), "ore_v_void");
    }

    #[test]
    fn mangle_var_name_passes_through_normal() {
        assert_eq!(CCodeGen::mangle_var_name("x"), "x");
        assert_eq!(CCodeGen::mangle_var_name("my_var"), "my_var");
    }

    // ---------------------------------------------------------------
    // CCodeGen internal helpers
    // ---------------------------------------------------------------

    #[test]
    fn tmp_generates_unique_names() {
        let mut cg = CCodeGen::new();
        assert_eq!(cg.tmp(), "__tmp_0");
        assert_eq!(cg.tmp(), "__tmp_1");
        assert_eq!(cg.tmp(), "__tmp_2");
    }

    #[test]
    fn label_generates_unique_names() {
        let mut cg = CCodeGen::new();
        assert_eq!(cg.label("loop"), "loop_0");
        assert_eq!(cg.label("loop"), "loop_1");
        assert_eq!(cg.label("end"), "end_2");
    }

    #[test]
    fn emit_respects_indent() {
        let mut cg = CCodeGen::new();
        cg.emit("line0");
        cg.indent = 1;
        cg.emit("line1");
        cg.indent = 2;
        cg.emit("line2");
        assert_eq!(cg.lines[0], "line0");
        assert_eq!(cg.lines[1], "    line1");
        assert_eq!(cg.lines[2], "        line2");
    }

    #[test]
    fn emit_raw_ignores_indent() {
        let mut cg = CCodeGen::new();
        cg.indent = 3;
        cg.emit_raw("no indent");
        assert_eq!(cg.lines[0], "no indent");
    }

    // ---------------------------------------------------------------
    // type_resolution: kind_to_c_type_str
    // ---------------------------------------------------------------

    #[test]
    fn kind_to_c_type_str_primitives() {
        let cg = CCodeGen::new();
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Int), "int64_t");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Float), "double");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Bool), "int8_t");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Str), "void*");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Void), "int64_t");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Channel), "void*");
    }

    #[test]
    fn kind_to_c_type_str_collections() {
        let cg = CCodeGen::new();
        assert_eq!(cg.kind_to_c_type_str(&ValKind::List(None)), "void*");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::list_of(ValKind::Int)), "void*");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Map(None)), "void*");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::map_of(ValKind::Str)), "void*");
    }

    #[test]
    fn kind_to_c_type_str_record_and_enum() {
        let cg = CCodeGen::new();
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Record("Point".into())), "struct ore_rec_Point");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Enum("Token".into())), "struct ore_enum_Token");
    }

    #[test]
    fn kind_to_c_type_str_option_result() {
        let cg = CCodeGen::new();
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Option), "OreTaggedUnion");
        assert_eq!(cg.kind_to_c_type_str(&ValKind::Result), "OreTaggedUnion");
    }

    // ---------------------------------------------------------------
    // type_resolution: valkind_to_tag
    // ---------------------------------------------------------------

    #[test]
    fn valkind_to_tag_all_variants() {
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Int), 0);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Float), 1);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Bool), 2);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Str), 3);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Void), 4);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Record("X".into())), 5);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Enum("Y".into())), 6);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Option), 7);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Result), 8);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::List(None)), 9);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Map(None)), 10);
        assert_eq!(CCodeGen::valkind_to_tag(&ValKind::Channel), 11);
    }

    // ---------------------------------------------------------------
    // type_resolution: kind_to_suffix
    // ---------------------------------------------------------------

    #[test]
    fn kind_to_suffix_basic_types() {
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::Int), "Int");
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::Float), "Float");
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::Bool), "Bool");
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::Str), "Str");
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::Void), "Void");
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::Channel), "Channel");
    }

    #[test]
    fn kind_to_suffix_nested_collections() {
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::list_of(ValKind::Int)), "List_Int");
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::List(None)), "List");
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::map_of(ValKind::Str)), "Map_Str");
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::Map(None)), "Map");
    }

    #[test]
    fn kind_to_suffix_record_enum() {
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::Record("Foo".into())), "Rec_Foo");
        assert_eq!(CCodeGen::kind_to_suffix(&ValKind::Enum("Bar".into())), "Enum_Bar");
    }

    // ---------------------------------------------------------------
    // type_resolution: kind_to_type_name
    // ---------------------------------------------------------------

    #[test]
    fn kind_to_type_name_returns_record_name() {
        let cg = CCodeGen::new();
        assert_eq!(cg.kind_to_type_name(&ValKind::Record("Foo".to_string())), "Foo");
    }

    #[test]
    fn kind_to_type_name_returns_enum_name() {
        let cg = CCodeGen::new();
        assert_eq!(cg.kind_to_type_name(&ValKind::Enum("Bar".to_string())), "Bar");
    }

    #[test]
    fn kind_to_type_name_primitives() {
        let cg = CCodeGen::new();
        assert_eq!(cg.kind_to_type_name(&ValKind::Int), "Int");
        assert_eq!(cg.kind_to_type_name(&ValKind::Float), "Float");
        assert_eq!(cg.kind_to_type_name(&ValKind::Bool), "Bool");
        assert_eq!(cg.kind_to_type_name(&ValKind::Str), "Str");
        assert_eq!(cg.kind_to_type_name(&ValKind::Void), "Int"); // Void maps to Int
        assert_eq!(cg.kind_to_type_name(&ValKind::List(None)), "List");
        assert_eq!(cg.kind_to_type_name(&ValKind::Map(None)), "Map");
        assert_eq!(cg.kind_to_type_name(&ValKind::Option), "Option");
        assert_eq!(cg.kind_to_type_name(&ValKind::Result), "Result");
        assert_eq!(cg.kind_to_type_name(&ValKind::Channel), "Channel");
    }

    // ---------------------------------------------------------------
    // type_resolution: kind_size_bytes
    // ---------------------------------------------------------------

    #[test]
    fn kind_size_bytes_primitives() {
        let cg = CCodeGen::new();
        assert_eq!(cg.kind_size_bytes(&ValKind::Bool), 1);
        assert_eq!(cg.kind_size_bytes(&ValKind::Int), 8);
        assert_eq!(cg.kind_size_bytes(&ValKind::Float), 8);
        assert_eq!(cg.kind_size_bytes(&ValKind::Str), 8);
        assert_eq!(cg.kind_size_bytes(&ValKind::Void), 8);
        assert_eq!(cg.kind_size_bytes(&ValKind::Channel), 8);
        assert_eq!(cg.kind_size_bytes(&ValKind::Option), 10);
        assert_eq!(cg.kind_size_bytes(&ValKind::Result), 10);
    }

    // ---------------------------------------------------------------
    // util: valkind_to_name
    // ---------------------------------------------------------------

    #[test]
    fn valkind_to_name_all_variants() {
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Int), "Int");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Float), "Float");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Bool), "Bool");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Str), "Str");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Void), "Void");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Record("Pt".into())), "Pt");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Enum("Tok".into())), "Tok");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Option), "Option");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Result), "Result");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::List(None)), "List");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Map(None)), "Map");
        assert_eq!(CCodeGen::valkind_to_name(&ValKind::Channel), "Channel");
    }

    // ---------------------------------------------------------------
    // type_resolution: coercion helpers
    // ---------------------------------------------------------------

    #[test]
    fn value_to_i64_expr_int_passthrough() {
        let cg = CCodeGen::new();
        assert_eq!(cg.value_to_i64_expr("42", &ValKind::Int), "42");
    }

    #[test]
    fn value_to_i64_expr_float_reinterpret() {
        let cg = CCodeGen::new();
        let result = cg.value_to_i64_expr("3.14", &ValKind::Float);
        assert!(result.contains("int64_t"), "Expected int64_t cast: {}", result);
        assert!(result.contains("double"), "Expected double reference: {}", result);
    }

    #[test]
    fn value_to_i64_expr_str_intptr() {
        let cg = CCodeGen::new();
        let result = cg.value_to_i64_expr("s", &ValKind::Str);
        assert!(result.contains("intptr_t"), "Expected intptr_t cast: {}", result);
    }

    #[test]
    fn coerce_from_i64_expr_str() {
        let cg = CCodeGen::new();
        let result = cg.coerce_from_i64_expr("val", &ValKind::Str);
        assert!(result.contains("void*"), "Expected void* cast: {}", result);
    }

    #[test]
    fn coerce_from_i64_expr_float() {
        let cg = CCodeGen::new();
        let result = cg.coerce_from_i64_expr("val", &ValKind::Float);
        assert!(result.contains("double"), "Expected double cast: {}", result);
    }

    #[test]
    fn coerce_from_i64_expr_bool() {
        let cg = CCodeGen::new();
        let result = cg.coerce_from_i64_expr("val", &ValKind::Bool);
        assert!(result.contains("int8_t"), "Expected int8_t cast: {}", result);
    }

    #[test]
    fn coerce_from_i64_expr_record() {
        let cg = CCodeGen::new();
        let result = cg.coerce_from_i64_expr("val", &ValKind::Record("Pt".into()));
        assert!(result.contains("ore_rec_Pt"), "Expected ore_rec_Pt: {}", result);
    }

    #[test]
    fn coerce_from_i64_expr_enum() {
        let cg = CCodeGen::new();
        let result = cg.coerce_from_i64_expr("val", &ValKind::Enum("Tok".into()));
        assert!(result.contains("ore_enum_Tok"), "Expected ore_enum_Tok: {}", result);
    }

    #[test]
    fn coerce_expr_same_kind_is_noop() {
        let cg = CCodeGen::new();
        assert_eq!(cg.coerce_expr("x", &ValKind::Int, &ValKind::Int), "x");
        assert_eq!(cg.coerce_expr("s", &ValKind::Str, &ValKind::Str), "s");
    }

    #[test]
    fn coerce_expr_i64_to_struct() {
        let cg = CCodeGen::new();
        let result = cg.coerce_expr("val", &ValKind::Int, &ValKind::Record("Pt".into()));
        assert!(result.contains("ore_rec_Pt"), "Expected record coercion: {}", result);
    }

    #[test]
    fn coerce_expr_struct_to_i64() {
        let cg = CCodeGen::new();
        let result = cg.coerce_expr("val", &ValKind::Record("Pt".into()), &ValKind::Int);
        assert!(result.contains("int64_t") || result.contains("intptr_t"),
            "Expected i64 coercion: {}", result);
    }

    // ---------------------------------------------------------------
    // escape_c_string
    // ---------------------------------------------------------------

    #[test]
    fn escape_c_string_special_chars() {
        assert_eq!(CCodeGen::escape_c_string("hello"), "hello");
        assert_eq!(CCodeGen::escape_c_string("a\"b"), "a\\\"b");
        assert_eq!(CCodeGen::escape_c_string("a\\b"), "a\\\\b");
        assert_eq!(CCodeGen::escape_c_string("a\nb"), "a\\nb");
        assert_eq!(CCodeGen::escape_c_string("a\tb"), "a\\tb");
        assert_eq!(CCodeGen::escape_c_string("a\rb"), "a\\rb");
        assert_eq!(CCodeGen::escape_c_string("a\0b"), "a\\0b");
    }

    // ---------------------------------------------------------------
    // Assembly: output structure
    // ---------------------------------------------------------------

    #[test]
    fn assembly_contains_header() {
        let c_code = compile_ore_to_c("fn main\n  print 1");
        assert!(c_code.contains("/* Generated by ore_c_codegen */"), "Missing header comment");
        assert!(c_code.contains("#include <stdint.h>"), "Missing stdint.h");
        assert!(c_code.contains("#include <stdlib.h>"), "Missing stdlib.h");
        assert!(c_code.contains("OreTaggedUnion"), "Missing OreTaggedUnion typedef");
    }

    #[test]
    fn assembly_contains_runtime_decls() {
        let c_code = compile_ore_to_c("fn main\n  print 1");
        assert!(c_code.contains("/* Runtime function declarations */"),
            "Missing runtime declarations section");
    }

    #[test]
    fn assembly_contains_forward_decls() {
        let c_code = compile_ore_to_c("fn main\n  print 1");
        assert!(c_code.contains("/* Forward declarations */"),
            "Missing forward declarations section");
        assert!(c_code.contains("int32_t main(void);"), "Missing main prototype");
    }

    // ---------------------------------------------------------------
    // Literal compilation
    // ---------------------------------------------------------------

    #[test]
    fn compile_int_literal() {
        let c_code = compile_ore_to_c("fn main\n  x := 42\n  print x");
        assert!(c_code.contains("42"), "Expected 42 in output:\n{}", c_code);
        assert!(c_code.contains("int64_t"), "Expected int64_t type:\n{}", c_code);
    }

    #[test]
    fn compile_float_literal() {
        let c_code = compile_ore_to_c("fn main\n  x := 3.14\n  print x");
        assert!(c_code.contains("3.14"), "Expected 3.14 in output:\n{}", c_code);
        assert!(c_code.contains("double"), "Expected double type:\n{}", c_code);
    }

    #[test]
    fn compile_bool_literal() {
        let c_code = compile_ore_to_c("fn main\n  x := true\n  y := false\n  print x");
        assert!(c_code.contains("int8_t"), "Expected int8_t for bool:\n{}", c_code);
    }

    #[test]
    fn compile_string_literal() {
        let c_code = compile_ore_to_c("fn main\n  x := \"hello\"\n  print x");
        assert!(c_code.contains("ore_str_new"), "Expected ore_str_new call:\n{}", c_code);
        assert!(c_code.contains("hello"), "Expected string content:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Function compilation
    // ---------------------------------------------------------------

    #[test]
    fn compile_simple_function() {
        let src = "fn add a:Int b:Int -> Int\n  a + b\nfn main\n  print add(1, 2)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("int64_t add(int64_t a, int64_t b)"),
            "Expected add function signature:\n{}", c_code);
    }

    #[test]
    fn compile_void_function() {
        let src = "fn greet\n  print \"hi\"\nfn main\n  greet()";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("void greet(void)"),
            "Expected void function:\n{}", c_code);
    }

    #[test]
    fn compile_function_with_return_type() {
        let src = "fn square n:Int -> Int\n  n * n\nfn main\n  print square(5)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("int64_t square(int64_t n)"),
            "Expected int64_t return type:\n{}", c_code);
        assert!(c_code.contains("return"), "Expected return statement:\n{}", c_code);
    }

    #[test]
    fn compile_main_returns_int32() {
        let c_code = compile_ore_to_c("fn main\n  print 1");
        assert!(c_code.contains("int32_t main(void)"),
            "Expected int32_t main(void):\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Record types
    // ---------------------------------------------------------------

    #[test]
    fn compile_record_type() {
        let src = "type Point { x:Int, y:Int }\nfn main\n  p := Point(x: 1, y: 2)\n  print p.x";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("struct ore_rec_Point"), "Expected struct ore_rec_Point:\n{}", c_code);
        assert!(c_code.contains("int64_t x;"), "Expected field x:\n{}", c_code);
        assert!(c_code.contains("int64_t y;"), "Expected field y:\n{}", c_code);
    }

    #[test]
    fn compile_record_with_mixed_fields() {
        let src = "type Item { name:Str, count:Int, active:Bool }\nfn main\n  i := Item(name: \"a\", count: 1, active: true)\n  print i.name";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("struct ore_rec_Item"), "Expected struct:\n{}", c_code);
        assert!(c_code.contains("void* name;"), "Expected void* name:\n{}", c_code);
        assert!(c_code.contains("int64_t count;"), "Expected int64_t count:\n{}", c_code);
        assert!(c_code.contains("int8_t active;"), "Expected int8_t active:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Enum types
    // ---------------------------------------------------------------

    #[test]
    fn compile_enum_type() {
        let src = "type Color\n  Red\n  Green\n  Blue\nfn main\n  c := Red\n  print c";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("struct ore_enum_Color"),
            "Expected struct ore_enum_Color:\n{}", c_code);
        assert!(c_code.contains("int8_t tag;"), "Expected tag field:\n{}", c_code);
    }

    #[test]
    fn compile_enum_with_payload() {
        let src = "type Shape\n  Circle(r: Float)\n  Rect(w: Int, h: Int)\nfn main\n  s := Circle(r: 3.14)\n  print s";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("struct ore_enum_Shape"),
            "Expected enum struct:\n{}", c_code);
        assert!(c_code.contains("ore_payload_Shape_Circle"),
            "Expected payload struct for Circle:\n{}", c_code);
        assert!(c_code.contains("ore_payload_Shape_Rect"),
            "Expected payload struct for Rect:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Binary operations
    // ---------------------------------------------------------------

    #[test]
    fn compile_arithmetic_ops() {
        let src = "fn main\n  a := 10 + 20\n  b := a - 5\n  c := b * 2\n  d := c / 3\n  print d";
        let c_code = compile_ore_to_c(src);
        // All variables should be int64_t
        let int_count = c_code.matches("int64_t").count();
        assert!(int_count >= 4, "Expected multiple int64_t declarations:\n{}", c_code);
    }

    #[test]
    fn compile_comparison_ops() {
        let src = "fn check a:Int b:Int -> Bool\n  a < b\nfn main\n  print check(1, 2)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("int8_t check("), "Expected bool return:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Control flow
    // ---------------------------------------------------------------

    #[test]
    fn compile_if_else() {
        let src = "fn main\n  x := 5\n  if x > 3\n    print \"big\"\n  else\n    print \"small\"";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("if ("), "Expected if statement:\n{}", c_code);
        assert!(c_code.contains("else"), "Expected else branch:\n{}", c_code);
    }

    #[test]
    fn compile_while_loop() {
        let src = "fn main\n  mut i := 0\n  while i < 10\n    i = i + 1\n  print i";
        let c_code = compile_ore_to_c(src);
        // While loops use goto-based labels
        assert!(c_code.contains("goto") || c_code.contains("while"),
            "Expected loop construct:\n{}", c_code);
    }

    #[test]
    fn compile_for_range() {
        let src = "fn main\n  for i in 0..5\n    print i";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("int64_t i"), "Expected loop variable:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Match expressions
    // ---------------------------------------------------------------

    #[test]
    fn compile_match_on_int() {
        let src = "fn describe n:Int -> Str\n  match n\n    1 -> \"one\"\n    2 -> \"two\"\n    _ -> \"other\"\nfn main\n  print describe(1)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("if (") || c_code.contains("switch"),
            "Expected match compilation:\n{}", c_code);
    }

    #[test]
    fn compile_match_on_enum() {
        let src = r#"
type Color
  Red
  Green
  Blue

fn name c:Color -> Str
  match c
    Red -> "red"
    Green -> "green"
    Blue -> "blue"

fn main
  print name(Red)
"#;
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("switch") || c_code.contains(".tag"),
            "Expected enum match on tag:\n{}", c_code);
    }

    #[test]
    fn compile_match_with_payload_extraction() {
        let src = r#"
type Token
  Number(val: Int)
  Plus

fn eval t:Token -> Int
  match t
    Number val -> val
    Plus -> 0

fn main
  print eval(Number(val: 42))
"#;
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("ore_payload_Token_Number"),
            "Expected payload extraction:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // String operations
    // ---------------------------------------------------------------

    #[test]
    fn compile_string_interpolation() {
        let src = "fn main\n  x := 42\n  print \"{x}\"";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("ore_str_concat") || c_code.contains("ore_int_to_str"),
            "Expected string interpolation helpers:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Impl blocks
    // ---------------------------------------------------------------

    #[test]
    fn compile_impl_block() {
        let src = r#"
type Point { x:Int, y:Int }

impl Point
  fn sum self:Point -> Int
    self.x + self.y

fn main
  p := Point(x: 3, y: 4)
  print p.sum()
"#;
        let c_code = compile_ore_to_c(src);
        // Impl method should be mangled as Point_sum
        assert!(c_code.contains("Point_sum"),
            "Expected mangled impl method name:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Lambda / closures
    // ---------------------------------------------------------------

    #[test]
    fn compile_lambda() {
        let src = "fn main\n  xs := [1, 2, 3]\n  ys := xs.map(x => x * 2)\n  print ys";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("/* Lambda functions */") || c_code.contains("lambda"),
            "Expected lambda section:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Test definitions
    // ---------------------------------------------------------------

    #[test]
    fn compile_test_def() {
        let src = "test \"basic addition\"\n  assert 1 + 1 == 2";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("ore_test_0"), "Expected test function:\n{}", c_code);
        assert!(c_code.contains("ore_assert_set_test_mode"),
            "Expected test mode setup:\n{}", c_code);
        assert!(c_code.contains("PASS") && c_code.contains("FAIL"),
            "Expected PASS/FAIL reporting:\n{}", c_code);
    }

    #[test]
    fn compile_multiple_tests() {
        let src = "test \"test one\"\n  assert 1 == 1\n\ntest \"test two\"\n  assert 2 == 2";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("ore_test_0"), "Expected first test:\n{}", c_code);
        assert!(c_code.contains("ore_test_1"), "Expected second test:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Unary operators
    // ---------------------------------------------------------------

    #[test]
    fn compile_unary_minus() {
        let src = "fn neg n:Int -> Int\n  -n\nfn main\n  print neg(5)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("-("), "Expected unary minus:\n{}", c_code);
    }

    #[test]
    fn compile_unary_not() {
        let src = "fn flip b:Bool -> Bool\n  not b\nfn main\n  print flip(true)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("!("), "Expected unary not:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Variable shadowing
    // ---------------------------------------------------------------

    #[test]
    fn compile_variable_shadowing() {
        let src = "fn main\n  x := 1\n  x := \"hello\"\n  print x";
        let c_code = compile_ore_to_c(src);
        // Shadowing with different type should create a new C variable
        assert!(c_code.contains("int64_t x"), "Expected first declaration:\n{}", c_code);
        // Second declaration should have a unique suffix
        assert!(c_code.contains("void* x_"), "Expected shadowed variable with suffix:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Default parameters
    // ---------------------------------------------------------------

    #[test]
    fn compile_default_params() {
        let src = "fn greet name:Str greeting:Str=\"Hello\" -> Str\n  greeting\nfn main\n  print greet(\"world\")";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("ore_str_new"), "Expected default string:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Reserved word handling in generated output
    // ---------------------------------------------------------------

    #[test]
    fn reserved_word_function_name_mangled() {
        let src = "fn float x:Int -> Int\n  x\nfn main\n  print float(1)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("ore_fn_float"),
            "Expected mangled function name ore_fn_float:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Generic functions
    // ---------------------------------------------------------------

    #[test]
    fn generic_list_accessor_record_cast() {
        let src = r#"
type Point { x:Int, y:Int }

fn get_item[T] items:List[T] idx:Int -> T
  items[idx]

fn main
  pts := [Point(x: 1, y: 2)]
  p := get_item(pts, 0)
  print p.x
"#;
        let c_code = compile_ore_to_c(src);
        assert!(
            c_code.contains("ore_rec_Point"),
            "Expected ore_rec_Point in generated C code, got:\n{}",
            c_code
        );
    }

    #[test]
    fn generic_list_accessor_enum_cast() {
        let src = r#"
type Token
  Number(val: Int)
  Plus

fn get_item[T] items:List[T] idx:Int -> T
  items[idx]

fn main
  tokens := [Plus, Number(val: 1)]
  t := get_item(tokens, 0)
  print t
"#;
        let c_code = compile_ore_to_c(src);
        assert!(
            c_code.contains("ore_enum_Token"),
            "Expected ore_enum_Token in generated C code, got:\n{}",
            c_code
        );
    }

    #[test]
    fn generic_function_monomorphized_name() {
        let src = r#"
fn identity[T] x:T -> T
  x

fn main
  a := identity(42)
  b := identity("hello")
  print a
"#;
        let c_code = compile_ore_to_c(src);
        // Should contain monomorphized versions with type suffixes
        assert!(c_code.contains("identity_") || c_code.contains("identity"),
            "Expected monomorphized function:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Match arm variable scope
    // ---------------------------------------------------------------

    #[test]
    fn match_arm_variable_scope_no_leakage() {
        let src = r#"
type Token
  Keyword(kw: String)
  Number(val: Int)
  Plus

fn describe tag:Int -> String
  match tag
    0 ->
      kw := "hello"
      kw
    1 ->
      kw := "world"
      kw
    _ -> "other"

fn main
  print describe(0)
"#;
        let c_code = compile_ore_to_c(src);
        let kw_decls = c_code.matches("void* kw =").count();
        assert!(
            kw_decls >= 2,
            "Expected kw to be declared in each match arm, found {} declarations.\nGenerated C:\n{}",
            kw_decls, c_code
        );
    }

    // ---------------------------------------------------------------
    // Let destructuring
    // ---------------------------------------------------------------

    #[test]
    fn compile_let_destructure() {
        let src = "fn main\n  xs := [1, 2, 3]\n  [a, b, c] := xs\n  print a";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("ore_list_get"), "Expected ore_list_get calls:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Return statements
    // ---------------------------------------------------------------

    #[test]
    fn compile_explicit_return() {
        let src = "fn early n:Int -> Int\n  if n < 0\n    return 0\n  n\nfn main\n  print early(5)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("return"), "Expected return statement:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Multiple functions produce correct forward declarations
    // ---------------------------------------------------------------

    #[test]
    fn forward_declarations_for_all_functions() {
        let src = "fn foo -> Int\n  1\nfn bar -> Int\n  foo()\nfn main\n  print bar()";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("int64_t foo(void);"), "Expected foo prototype:\n{}", c_code);
        assert!(c_code.contains("int64_t bar(void);"), "Expected bar prototype:\n{}", c_code);
        assert!(c_code.contains("int32_t main(void);"), "Expected main prototype:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Float parameter functions
    // ---------------------------------------------------------------

    #[test]
    fn compile_float_param_function() {
        let src = "fn area r:Float -> Float\n  r * r\nfn main\n  print area(3.0)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("double area(double r)"),
            "Expected double param signature:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Option / Result types
    // ---------------------------------------------------------------

    #[test]
    fn compile_option_some_none() {
        let src = "fn maybe flag:Bool -> Option\n  if flag\n    Some(42)\n  else\n    None\nfn main\n  print maybe(true)";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("OreTaggedUnion"),
            "Expected OreTaggedUnion for Option:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // CCodeGenError display
    // ---------------------------------------------------------------

    #[test]
    fn error_display_with_line() {
        let err = CCodeGenError { msg: "bad thing".to_string(), line: Some(42) };
        assert_eq!(format!("{}", err), "line 42: bad thing");
    }

    #[test]
    fn error_display_without_line() {
        let err = CCodeGenError { msg: "bad thing".to_string(), line: None };
        assert_eq!(format!("{}", err), "bad thing");
    }

    // ---------------------------------------------------------------
    // Default trait implementation
    // ---------------------------------------------------------------

    #[test]
    fn default_creates_valid_codegen() {
        let cg = CCodeGen::default();
        assert!(cg.lines.is_empty());
        assert!(cg.variables.is_empty());
        assert!(cg.functions.is_empty());
    }

    // ---------------------------------------------------------------
    // Empty program
    // ---------------------------------------------------------------

    #[test]
    fn compile_empty_program() {
        let c_code = compile_ore_to_c("");
        assert!(c_code.contains("/* Generated by ore_c_codegen */"),
            "Empty program should still produce header:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // check_arity error
    // ---------------------------------------------------------------

    #[test]
    fn check_arity_ok_when_matching() {
        let cg = CCodeGen::new();
        assert!(cg.check_arity("foo", &[], 0).is_ok());
    }

    #[test]
    fn check_arity_err_when_mismatched() {
        let cg = CCodeGen::new();
        let result = cg.check_arity("foo", &[], 2);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.msg.contains("foo"), "Error should mention function name");
        assert!(err.msg.contains("2"), "Error should mention expected arity");
    }

    // ---------------------------------------------------------------
    // resolve_self_type
    // ---------------------------------------------------------------

    #[test]
    fn resolve_self_type_replaces_self() {
        let ty = TypeExpr::Named("Self".to_string());
        let resolved = CCodeGen::resolve_self_type(&ty, "MyType");
        assert_eq!(resolved, TypeExpr::Named("MyType".to_string()));
    }

    #[test]
    fn resolve_self_type_preserves_other() {
        let ty = TypeExpr::Named("Int".to_string());
        let resolved = CCodeGen::resolve_self_type(&ty, "MyType");
        assert_eq!(resolved, TypeExpr::Named("Int".to_string()));
    }

    #[test]
    fn resolve_self_type_in_generic() {
        let ty = TypeExpr::Generic(
            "List".to_string(),
            vec![TypeExpr::Named("Self".to_string())],
        );
        let resolved = CCodeGen::resolve_self_type(&ty, "Point");
        match resolved {
            TypeExpr::Generic(name, args) => {
                assert_eq!(name, "List");
                assert_eq!(args[0], TypeExpr::Named("Point".to_string()));
            }
            other => panic!("Expected Generic, got {:?}", other),
        }
    }

    // ---------------------------------------------------------------
    // Mutable variable reassignment
    // ---------------------------------------------------------------

    #[test]
    fn compile_mutable_reassignment() {
        let src = "fn main\n  mut x := 1\n  x = 2\n  print x";
        let c_code = compile_ore_to_c(src);
        // Should reassign, not redeclare
        let x_decls = c_code.matches("int64_t x =").count();
        assert_eq!(x_decls, 1, "Expected exactly one declaration of x, found {}:\n{}", x_decls, c_code);
        assert!(c_code.contains("x = "), "Expected reassignment:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Enum with zero-arg variant as value
    // ---------------------------------------------------------------

    #[test]
    fn compile_zero_arg_variant() {
        let src = "type Dir\n  Up\n  Down\nfn main\n  d := Up\n  print d";
        let c_code = compile_ore_to_c(src);
        // Zero-arg variant should set tag
        assert!(c_code.contains(".tag"), "Expected tag assignment:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Multiple record types
    // ---------------------------------------------------------------

    #[test]
    fn compile_multiple_record_types() {
        let src = r#"
type Point { x:Int, y:Int }
type Size { w:Int, h:Int }
fn main
  p := Point(x: 0, y: 0)
  s := Size(w: 10, h: 20)
  print p.x
"#;
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("struct ore_rec_Point"), "Expected Point struct:\n{}", c_code);
        assert!(c_code.contains("struct ore_rec_Size"), "Expected Size struct:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // Loop with break
    // ---------------------------------------------------------------

    #[test]
    fn compile_loop_with_break() {
        let src = "fn main\n  mut i := 0\n  loop\n    if i >= 5\n      break\n    i = i + 1\n  print i";
        let c_code = compile_ore_to_c(src);
        assert!(c_code.contains("goto"), "Expected goto for break:\n{}", c_code);
    }

    // ---------------------------------------------------------------
    // type_expr_to_kind
    // ---------------------------------------------------------------

    #[test]
    fn type_expr_to_kind_named_primitives() {
        let cg = CCodeGen::new();
        assert_eq!(cg.type_expr_to_kind(&TypeExpr::Named("Int".into())), ValKind::Int);
        assert_eq!(cg.type_expr_to_kind(&TypeExpr::Named("Float".into())), ValKind::Float);
        assert_eq!(cg.type_expr_to_kind(&TypeExpr::Named("Bool".into())), ValKind::Bool);
        assert_eq!(cg.type_expr_to_kind(&TypeExpr::Named("Str".into())), ValKind::Str);
        assert_eq!(cg.type_expr_to_kind(&TypeExpr::Named("Option".into())), ValKind::Option);
        assert_eq!(cg.type_expr_to_kind(&TypeExpr::Named("Result".into())), ValKind::Result);
        assert_eq!(cg.type_expr_to_kind(&TypeExpr::Named("Channel".into())), ValKind::Channel);
    }

    #[test]
    fn type_expr_to_kind_generic_list() {
        let cg = CCodeGen::new();
        let ty = TypeExpr::Generic("List".into(), vec![TypeExpr::Named("Int".into())]);
        assert_eq!(cg.type_expr_to_kind(&ty), ValKind::list_of(ValKind::Int));
    }

    #[test]
    fn type_expr_to_kind_generic_map() {
        let cg = CCodeGen::new();
        let ty = TypeExpr::Generic("Map".into(), vec![
            TypeExpr::Named("Str".into()),
            TypeExpr::Named("Int".into()),
        ]);
        assert_eq!(cg.type_expr_to_kind(&ty), ValKind::map_of(ValKind::Int));
    }

    #[test]
    fn type_expr_to_kind_unknown_defaults_to_int() {
        let cg = CCodeGen::new();
        assert_eq!(cg.type_expr_to_kind(&TypeExpr::Named("UnknownType".into())), ValKind::Int);
    }

    #[test]
    fn type_expr_to_kind_fn_type_defaults_to_int() {
        let cg = CCodeGen::new();
        let ty = TypeExpr::Fn {
            params: vec![TypeExpr::Named("Int".into())],
            ret: Box::new(TypeExpr::Named("Int".into())),
        };
        assert_eq!(cg.type_expr_to_kind(&ty), ValKind::Int);
    }

    // ---------------------------------------------------------------
    // If/else branch variable hoisting
    // ---------------------------------------------------------------

    #[test]
    fn if_else_branch_var_hoisted_to_outer_scope() {
        let src = "\
fn main
  y := -1.0
  if y < 0.0
    y_show := 0.0
  else
    y_show := y
  print y_show";
        let c_code = compile_ore_to_c(src);
        // y_show should be declared before the if and used after it
        let if_pos = c_code.find("if (").expect("if not found");
        let decl_pos = c_code.find("double y_show;").expect("hoisted declaration not found");
        assert!(
            decl_pos < if_pos,
            "y_show declaration should appear before the if statement.\nGenerated C:\n{}",
            c_code,
        );
        // The in-block usages should be assignments, not declarations
        let in_block_decls = c_code.matches("double y_show = ").count();
        assert_eq!(
            in_block_decls, 0,
            "Expected no in-block declarations of y_show (should be assignments).\nGenerated C:\n{}",
            c_code,
        );
    }

    #[test]
    fn if_else_branch_var_used_in_loop() {
        // Mirrors the showcase451 pattern: variable defined in both if/else
        // branches inside a loop, then used later in the same loop iteration.
        let src = "\
fn main
  mut max_h := 0.0
  y := -1.0
  loop
    if y < 0.0
      y_show := 0.0
    else
      y_show := y
    if y_show > max_h
      max_h = y_show
    break
  print max_h";
        let c_code = compile_ore_to_c(src);
        // Must not contain undeclared y_show — the declaration should be hoisted
        assert!(
            c_code.contains("double y_show;"),
            "Expected hoisted y_show declaration.\nGenerated C:\n{}",
            c_code,
        );
    }
}
