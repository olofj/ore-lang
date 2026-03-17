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
                // Cast to the function's return type if needed to avoid C type mismatch.
                // Only coerce when the if/else produced int64_t but the function expects
                // a pointer type (Str, List, Map) or a tagged union (Option, Result).
                let needs_cast = match &fn_info.ret_kind {
                    ValKind::Str | ValKind::List(_) | ValKind::Map(_) | ValKind::Channel
                    | ValKind::Enum(_) | ValKind::Record(_) | ValKind::Option | ValKind::Result => {
                        // The expression might be an int64_t from compile_if_else or list_get
                        _last_kind == ValKind::Int
                    }
                    _ => false,
                };
                if needs_cast {
                    let ret_expr = self.coerce_from_i64_expr(expr_str, &fn_info.ret_kind);
                    self.emit(&format!("return {};", ret_expr));
                } else {
                    self.emit(&format!("return {};", expr_str));
                }
            }
        }

        // After compiling the body, if the function returns List(None) or Map(None),
        // try to infer element/value kinds from the last expression. Only infer from
        // the actual return variable, not from any list in the function.
        if matches!(fn_info.ret_kind, ValKind::List(None))
            && !self.fn_return_list_elem_kind.contains_key(&fndef.name) {
                if let Some(name) = Self::find_return_ident(&fndef.body) {
                    if let Some(ek) = self.list_element_kinds.get(&name) {
                        self.fn_return_list_elem_kind.insert(fndef.name.clone(), ek.clone());
                    }
                }
            }
        if matches!(fn_info.ret_kind, ValKind::Map(None))
            && !self.fn_return_map_val_kind.contains_key(&fndef.name) {
                if let Some(name) = Self::find_return_ident(&fndef.body) {
                    if let Some(vk) = self.map_value_kinds.get(&name) {
                        self.fn_return_map_val_kind.insert(fndef.name.clone(), vk.clone());
                    }
                }
            }

        self.indent -= 1;
        self.emit_raw("}");
        self.emit_raw("");
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
