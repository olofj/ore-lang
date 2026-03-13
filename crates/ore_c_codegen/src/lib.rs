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
struct RecordInfo {
    field_names: Vec<String>,
    field_kinds: Vec<ValKind>,
}

/// Info about a single enum variant
struct VariantInfo {
    name: String,
    tag: u8,
    field_names: Vec<String>,
    field_kinds: Vec<ValKind>,
}

/// Info about an enum type
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
        if Self::C_RESERVED.contains(&name) || name == "main" {
            // "main" is special - keep as-is since it needs to be the entry point
            if name == "main" {
                return name.to_string();
            }
            format!("ore_fn_{}", name)
        } else {
            name.to_string()
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

    fn kind_size_bytes(&self, kind: &ValKind) -> u64 {
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

    /// Declare a function — emit C prototype.
    fn declare_function(&mut self, fndef: &FnDef) -> Result<(), CCodeGenError> {
        let ret_kind = fndef.ret_type.as_ref().map(|t| self.type_expr_to_kind(t)).unwrap_or(ValKind::Void);
        let c_fn_name = Self::mangle_fn_name(&fndef.name);

        let mut param_kinds = Vec::new();
        let mut param_strs = Vec::new();
        for p in &fndef.params {
            let kind = self.type_expr_to_kind(&p.ty);
            let c_type = self.kind_to_c_type_str(&kind);
            param_strs.push(format!("{} {}", c_type, p.name));
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
    fn compile_function(&mut self, fndef: &FnDef) -> Result<(), CCodeGenError> {
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
            param_strs.push(format!("{} {}", c_type, p.name));
            self.variables.insert(p.name.clone(), VarInfo {
                c_name: p.name.clone(),
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
                self.emit(&format!("return {};", expr_str));
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
        // Register type definitions
        for item in &program.items {
            match item {
                Item::TypeDef(td) => self.register_record(td)?,
                Item::EnumDef(ed) => self.register_enum(ed)?,
                _ => {}
            }
        }

        // Declare regular functions (skip generic)
        for item in &program.items {
            if let Item::FnDef(f) = item {
                if !f.type_params.is_empty() {
                    self.generic_fns.insert(f.name.clone(), f.clone());
                } else {
                    self.declare_function(f)?;
                }
            }
        }

        // Declare impl methods
        for (type_name, methods) in Self::impl_items(&program.items) {
            for method in methods {
                let mangled_fn = Self::mangle_impl_method(type_name, method);
                self.declare_function(&mangled_fn)?;
            }
        }

        // Compile regular functions
        for item in &program.items {
            if let Item::FnDef(f) = item {
                if f.type_params.is_empty() {
                    self.compile_function(f)?;
                }
            }
        }

        // Compile impl methods
        let impl_fns: Vec<_> = Self::impl_items(&program.items)
            .flat_map(|(type_name, methods)| {
                methods.iter().map(move |m| Self::mangle_impl_method(type_name, m))
            })
            .collect();
        for mangled_fn in impl_fns {
            self.compile_function(&mangled_fn)?;
        }

        // Compile test definitions
        let mut test_idx = 0;
        for item in &program.items {
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

        // Assemble final C code
        Ok(self.assemble())
    }

    /// Assemble all code sections into the final C output.
    fn assemble(&self) -> String {
        let mut output = Vec::new();

        // Header
        output.push("/* Generated by ore_c_codegen */".to_string());
        output.push("#include <stdint.h>".to_string());
        output.push("#include <stddef.h>".to_string());
        output.push("#include <string.h>".to_string());
        output.push("#include <stdlib.h>".to_string());
        output.push(String::new());

        // Tagged union type for Option/Result
        output.push("typedef struct { int8_t tag; int8_t kind; int64_t val; } OreTaggedUnion;".to_string());
        output.push(String::new());

        // Runtime extern declarations
        output.push("/* Runtime function declarations */".to_string());
        output.extend(runtime_decls::runtime_declarations());
        output.push(String::new());

        // Type definitions (structs)
        if !self.top_level.is_empty() {
            output.push("/* Type definitions */".to_string());
            output.extend(self.top_level.iter().cloned());
            output.push(String::new());
        }

        // Forward declarations
        if !self.forward_decls.is_empty() {
            output.push("/* Forward declarations */".to_string());
            output.extend(self.forward_decls.iter().cloned());
            output.push(String::new());
        }

        // Lambda function bodies
        if !self.lambda_bodies.is_empty() {
            output.push("/* Lambda functions */".to_string());
            output.extend(self.lambda_bodies.iter().cloned());
            output.push(String::new());
        }

        // Main function bodies
        output.push("/* Function definitions */".to_string());
        output.extend(self.lines.iter().cloned());

        output.join("\n")
    }

    /// Infer expression kind without compilation (lightweight).
    pub(crate) fn infer_expr_kind(&self, expr: &Expr) -> ValKind {
        match expr {
            Expr::StringLit(_) | Expr::StringInterp(_) => ValKind::Str,
            Expr::IntLit(_) => ValKind::Int,
            Expr::FloatLit(_) => ValKind::Float,
            Expr::BoolLit(_) => ValKind::Bool,
            Expr::ListLit(_) | Expr::ListComp { .. } => ValKind::List(None),
            Expr::MapLit(_) => ValKind::Map(None),
            Expr::Ident(name) => {
                self.variables.get(name).map(|v| v.kind.clone()).unwrap_or(ValKind::Int)
            }
            Expr::MethodCall { method, .. } => {
                match method.as_str() {
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
            Expr::BinOp { op, left, right } => {
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        let lk = self.infer_expr_kind(left);
                        let rk = self.infer_expr_kind(right);
                        if lk == ValKind::Float || rk == ValKind::Float {
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
            Expr::IfElse { then_block, .. } => {
                if let Some(last) = then_block.stmts.last() {
                    if let Stmt::Expr(e) = &last.stmt {
                        return self.infer_expr_kind(e);
                    }
                }
                ValKind::Int
            }
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
            _ => expr.to_string(),
        }
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
