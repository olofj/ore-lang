use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::types::BasicType;
use std::collections::{HashMap, HashSet};

use ore_parser::ast::*;

/// Tracks whether a compiled value is a string pointer (needs RC) or a plain value
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
    List(Option<Box<ValKind>>),
    Map(Option<Box<ValKind>>),
    Channel,
}

impl ValKind {
    /// Check if this is any List variant regardless of element kind
    pub fn is_list(&self) -> bool {
        matches!(self, ValKind::List(_))
    }

    /// Extract the element kind from a List variant, if known
    pub fn list_elem_kind(&self) -> Option<&ValKind> {
        match self {
            ValKind::List(Some(k)) => Some(k),
            _ => None,
        }
    }

    /// Create a List with a known element kind
    pub fn list_of(kind: ValKind) -> ValKind {
        ValKind::List(Some(Box::new(kind)))
    }

    /// Check if this is any Map variant regardless of value kind
    pub fn is_map(&self) -> bool {
        matches!(self, ValKind::Map(_))
    }

    /// Extract the value kind from a Map variant, if known
    pub fn map_val_kind(&self) -> Option<&ValKind> {
        match self {
            ValKind::Map(Some(k)) => Some(k),
            _ => None,
        }
    }

    /// Create a Map with a known value kind
    pub fn map_of(kind: ValKind) -> ValKind {
        ValKind::Map(Some(Box::new(kind)))
    }
}

pub(crate) struct RecordInfo<'ctx> {
    pub(crate) struct_type: inkwell::types::StructType<'ctx>,
    pub(crate) field_names: Vec<String>,
    pub(crate) field_kinds: Vec<ValKind>,
}

pub(crate) struct VariantInfo<'ctx> {
    pub(crate) name: String,
    pub(crate) tag: u8,
    pub(crate) field_names: Vec<String>,
    pub(crate) field_kinds: Vec<ValKind>,
    pub(crate) payload_type: inkwell::types::StructType<'ctx>,
}

pub(crate) struct EnumInfo<'ctx> {
    pub(crate) enum_type: inkwell::types::StructType<'ctx>,
    pub(crate) variants: Vec<VariantInfo<'ctx>>,
}

/// Tracks capture information for a compiled lambda/closure
pub(crate) struct CaptureInfo<'ctx> {
    /// The LLVM struct type holding all captured values
    pub(crate) struct_type: inkwell::types::StructType<'ctx>,
    /// Names of captured variables (in struct field order)
    pub(crate) names: Vec<String>,
}

/// Info about a local variable: its alloca, LLVM type, semantic kind, and mutability.
#[derive(Clone)]
pub(crate) struct VarInfo<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub ty: inkwell::types::BasicTypeEnum<'ctx>,
    pub kind: ValKind,
    pub is_mutable: bool,
}

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub(crate) variables: HashMap<String, VarInfo<'ctx>>,
    pub(crate) functions: HashMap<String, (FunctionValue<'ctx>, ValKind)>,
    pub(crate) records: HashMap<String, RecordInfo<'ctx>>,
    pub(crate) enums: HashMap<String, EnumInfo<'ctx>>,
    /// Maps variant name -> enum name for quick lookup
    pub(crate) variant_to_enum: HashMap<String, String>,
    /// Target block for `break` statements
    pub(crate) break_target: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    /// Target block for `continue` statements
    pub(crate) continue_target: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    pub(crate) str_counter: u32,
    pub(crate) lambda_counter: u32,
    /// Maps lambda function name -> capture info (only for closures with captures)
    pub(crate) lambda_captures: HashMap<String, CaptureInfo<'ctx>>,
    /// Maps variable name -> alloca for runtime kind tag (used for dynamic dispatch in Option/Result payloads)
    pub(crate) dynamic_kind_tags: HashMap<String, PointerValue<'ctx>>,
    /// Maps variable name -> element ValKind for typed lists
    pub(crate) list_element_kinds: HashMap<String, ValKind>,
    /// Maps variable name -> value ValKind for typed maps
    pub(crate) map_value_kinds: HashMap<String, ValKind>,
    /// Current source line (for error reporting)
    pub(crate) current_line: usize,
    /// Generic function definitions (not yet monomorphized)
    pub(crate) generic_fns: HashMap<String, FnDef>,
    /// Default parameter expressions per function (name -> vec of Option<Expr>)
    pub(crate) fn_defaults: HashMap<String, Vec<Option<Expr>>>,
    /// Tracks element kind for functions returning List[T]
    pub(crate) fn_return_list_elem_kind: HashMap<String, ValKind>,
    /// Tracks value kind for functions returning Map[K, V]
    pub(crate) fn_return_map_val_kind: HashMap<String, ValKind>,
    /// Test function names in order, for `ore test`
    pub test_names: Vec<String>,
}

#[derive(Debug, Default)]
pub struct CodeGenError {
    pub msg: String,
    pub line: Option<usize>,
}

impl std::fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line {
            write!(f, "line {}: {}", line, self.msg)
        } else {
            write!(f, "{}", self.msg)
        }
    }
}

macro_rules! bld {
    ($expr:expr) => {
        $expr.map_err(|e| CodeGenError { line: None, msg: e.to_string() })
    };
}


mod util;
mod lambda;
mod str_methods;
mod list;
mod map;
mod match_compile;
mod methods;
mod builtins;
mod stmts;
mod expr;
mod runtime_decls;

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            functions: HashMap::new(),
            records: HashMap::new(),
            enums: HashMap::new(),
            variant_to_enum: HashMap::new(),
            break_target: None,
            continue_target: None,
            str_counter: 0,
            lambda_counter: 0,
            lambda_captures: HashMap::new(),
            dynamic_kind_tags: HashMap::new(),
            list_element_kinds: HashMap::new(),
            map_value_kinds: HashMap::new(),
            current_line: 0,
            generic_fns: HashMap::new(),
            fn_defaults: HashMap::new(),
            fn_return_list_elem_kind: HashMap::new(),
            fn_return_map_val_kind: HashMap::new(),
            test_names: Vec::new(),
        }
    }

    pub(crate) fn ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(inkwell::AddressSpace::default())
    }

    /// Shorthand for creating a CodeGenError at the current source line.
    pub(crate) fn err(&self, msg: impl Into<String>) -> CodeGenError {
        CodeGenError { line: Some(self.current_line), msg: msg.into() }
    }

    /// Check that `args` has exactly `expected` elements, or return an error.
    pub(crate) fn check_arity(&self, name: &str, args: &[Expr], expected: usize) -> Result<(), CodeGenError> {
        if args.len() != expected {
            Err(self.err(format!("{} takes {} argument(s)", name, expected)))
        } else {
            Ok(())
        }
    }

    /// Tagged union layout shared by Option and Result: { i8 tag, i8 kind, i64 payload }
    /// Option: tag 0=None, 1=Some.  Result: tag 0=Ok, 1=Err.
    pub(crate) fn tagged_union_type(&self) -> inkwell::types::StructType<'ctx> {
        self.context.struct_type(
            &[self.context.i8_type().into(), self.context.i8_type().into(), self.context.i64_type().into()],
            false,
        )
    }

    /// Alias for readability at Option call sites.
    pub(crate) fn option_type(&self) -> inkwell::types::StructType<'ctx> { self.tagged_union_type() }

    /// Alias for readability at Result call sites.
    pub(crate) fn result_type(&self) -> inkwell::types::StructType<'ctx> { self.tagged_union_type() }

    pub(crate) fn valkind_to_tag(&self, kind: &ValKind) -> u8 {
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


    /// Replace `Self` type references with the actual type name in impl blocks.
    pub(crate) fn resolve_self_type(ty: &TypeExpr, type_name: &str) -> TypeExpr {
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

    pub(crate) fn resolve_self_in_params(params: &[Param], type_name: &str) -> Vec<Param> {
        params.iter().map(|p| Param {
            name: p.name.clone(),
            ty: Self::resolve_self_type(&p.ty, type_name),
            default: p.default.clone(),
        }).collect()
    }

    /// Iterate over impl block and impl-trait items, yielding (type_name, methods).
    fn impl_items(items: &[Item]) -> impl Iterator<Item = (&String, &Vec<FnDef>)> {
        items.iter().filter_map(|item| match item {
            Item::ImplBlock { type_name, methods }
            | Item::ImplTrait { type_name, methods, .. } => Some((type_name, methods)),
            _ => None,
        })
    }

    /// Build a mangled FnDef from an impl method, resolving Self types and prepending `self` param.
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
            context: method.context.clone(),
            body: method.body.clone(),
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), CodeGenError> {
        self.declare_runtime_functions();

        // Register type definitions
        for item in &program.items {
            match item {
                Item::TypeDef(td) => self.register_record(td)?,
                Item::EnumDef(ed) => self.register_enum(ed)?,
                _ => {}
            }
        }

        // Declare regular functions (skip generic ones — they'll be monomorphized on demand)
        for item in &program.items {
            if let Item::FnDef(f) = item {
                if !f.type_params.is_empty() {
                    self.generic_fns.insert(f.name.clone(), f.clone());
                } else {
                    self.declare_function(f)?;
                }
            }
        }

        // Declare impl block and impl-trait methods (mangled names: TypeName_methodName)
        for (type_name, methods) in Self::impl_items(&program.items) {
            for method in methods {
                let mangled_fn = Self::mangle_impl_method(type_name, method);
                self.declare_function(&mangled_fn)?;
            }
        }

        // Compile regular functions (skip generic ones)
        for item in &program.items {
            if let Item::FnDef(f) = item {
                if f.type_params.is_empty() {
                    self.compile_function(f)?;
                }
            }
        }

        // Compile impl block and impl-trait methods
        for (type_name, methods) in Self::impl_items(&program.items) {
            for method in methods {
                let mangled_fn = Self::mangle_impl_method(type_name, method);
                self.compile_function(&mangled_fn)?;
            }
        }

        // Compile test definitions as void functions
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
                    context: vec![],
                    body: body.clone(),
                };
                self.declare_function(&test_fn)?;
                self.compile_function(&test_fn)?;
                test_idx += 1;
            }
        }

        Ok(())
    }

    /// Collect LLVM types, ValKinds, and names from a list of field definitions.
    fn collect_fields(&self, fields: &[FieldDef]) -> (Vec<inkwell::types::BasicTypeEnum<'ctx>>, Vec<ValKind>, Vec<String>) {
        let mut types = Vec::new();
        let mut kinds = Vec::new();
        let mut names = Vec::new();
        for f in fields {
            let kind = self.type_expr_to_kind(&f.ty);
            types.push(self.kind_to_llvm_type(&kind));
            kinds.push(kind);
            names.push(f.name.clone());
        }
        (types, kinds, names)
    }

    pub(crate) fn register_record(&mut self, td: &TypeDef) -> Result<(), CodeGenError> {
        let (field_types, field_kinds, field_names) = self.collect_fields(&td.fields);
        let struct_type = self.context.struct_type(&field_types, false);
        self.records.insert(td.name.clone(), RecordInfo {
            struct_type,
            field_names,
            field_kinds,
        });
        Ok(())
    }

    pub(crate) fn register_enum(&mut self, ed: &EnumDef) -> Result<(), CodeGenError> {
        let mut variants = Vec::new();
        let mut max_payload_size: u64 = 0;

        for (i, v) in ed.variants.iter().enumerate() {
            let (field_types, field_kinds, field_names) = self.collect_fields(&v.fields);

            let payload_type = self.context.struct_type(&field_types, false);
            let payload_size: u64 = field_types.iter().map(|ty| self.type_size_bytes(ty)).sum();
            if payload_size > max_payload_size {
                max_payload_size = payload_size;
            }

            variants.push(VariantInfo {
                name: v.name.clone(),
                tag: i as u8,
                field_names,
                field_kinds,
                payload_type,
            });

            self.variant_to_enum.insert(v.name.clone(), ed.name.clone());
        }

        // Enum layout: { i8 (tag), [ceil(max_payload_size/8) x i64] (data) }
        // Using i64 array ensures 8-byte alignment for float fields
        let i8_type = self.context.i8_type();
        let num_i64s = max_payload_size.div_ceil(8);
        let data_array = self.context.i64_type().array_type(num_i64s as u32);
        let enum_type = self.context.struct_type(
            &[i8_type.into(), data_array.into()],
            false,
        );

        self.enums.insert(ed.name.clone(), EnumInfo {
            enum_type,
            variants,
        });
        Ok(())
    }

    pub(crate) fn type_size_bytes(&self, ty: &inkwell::types::BasicTypeEnum<'ctx>) -> u64 {
        match ty {
            inkwell::types::BasicTypeEnum::IntType(t) => {
                (t.get_bit_width() as u64).div_ceil(8)
            }
            inkwell::types::BasicTypeEnum::FloatType(_)
            | inkwell::types::BasicTypeEnum::PointerType(_) => 8, // f64 or 64-bit pointer
            inkwell::types::BasicTypeEnum::StructType(t) => {
                t.get_field_types().iter().map(|f| self.type_size_bytes(f)).sum()
            }
            inkwell::types::BasicTypeEnum::ArrayType(t) => {
                let elem_size = self.type_size_bytes(&t.get_element_type());
                elem_size * t.len() as u64
            }
            _ => 8, // fallback
        }
    }

    /// Lightweight kind inference from expression syntax (no compilation needed).
    pub(crate) fn infer_expr_kind(&self, expr: &Expr) -> ValKind {
        match expr {
            Expr::StringLit(_) | Expr::StringInterp(_) => ValKind::Str,
            Expr::IntLit(_) => ValKind::Int,
            Expr::FloatLit(_) => ValKind::Float,
            Expr::BoolLit(_) => ValKind::Bool,
            Expr::TupleLit(_) => ValKind::List(None),
            Expr::ListLit(_) | Expr::ListComp { .. } => ValKind::List(None),
            Expr::MapLit(entries) => {
                if let Some((type_name, _)) = self.try_as_record_fields(entries) {
                    ValKind::Record(type_name)
                } else {
                    ValKind::Map(None)
                }
            }
            Expr::Ident(name) => {
                if let Some(var_info) = self.variables.get(name) {
                    var_info.kind.clone()
                } else {
                    ValKind::Int
                }
            }
            Expr::MethodCall { method, .. } => {
                // Infer return kind from well-known methods
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
                        // If either operand is Float, result is Float
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
                    BinOp::Pipe => {
                        // Pipeline result depends on the function being piped to
                        self.infer_expr_kind(right)
                    }
                }
            }
            Expr::IfElse { then_block, .. } => {
                // Infer from then branch's last expression
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
            TypeExpr::Fn { .. } => {
                // Function types are treated as opaque pointers in codegen
                ValKind::Int
            }
            TypeExpr::Generic(name, args) => {
                // Extract element/value kinds from generic type args
                match name.as_str() {
                    "List" => {
                        if let Some(elem_ty) = args.first() {
                            ValKind::list_of(self.type_expr_to_kind(elem_ty))
                        } else {
                            ValKind::List(None)
                        }
                    }
                    "Map" => ValKind::Map(None),
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

    /// Infer the list element kind from a function body's return expression.
    /// Used for functions with plain `List` return type annotations.
    fn infer_list_return_kind(&self, body: &Block) -> Option<ValKind> {
        let last = body.stmts.last()?;
        let expr = match &last.stmt {
            Stmt::Return(Some(e)) | Stmt::Expr(e) => e,
            _ => return None,
        };
        self.infer_list_elem_from_expr(expr)
    }

    fn infer_list_elem_from_expr(&self, expr: &Expr) -> Option<ValKind> {
        // If returning a named variable, check list_element_kinds first (most accurate)
        if let Expr::Ident(name) = expr {
            if let Some(ek) = self.list_element_kinds.get(name) {
                return Some(ek.clone());
            }
        }
        // If returning a method call on a tracked list variable, check its element kind
        if let Expr::MethodCall { object, .. } = expr {
            if let Expr::Ident(name) = object.as_ref() {
                if let Some(ek) = self.list_element_kinds.get(name) {
                    return Some(ek.clone());
                }
            }
        }
        // Recurse into if/else branches
        if let Expr::IfElse { then_block, else_block, .. } = expr {
            // Try then branch
            if let Some(last) = then_block.stmts.last() {
                if let Stmt::Expr(ref e) | Stmt::Return(Some(ref e)) = last.stmt {
                    if let Some(ek) = self.infer_list_elem_from_expr(e) {
                        return Some(ek);
                    }
                }
            }
            // Try else branch
            if let Some(else_blk) = else_block {
                if let Some(last) = else_blk.stmts.last() {
                    if let Stmt::Expr(ref e) | Stmt::Return(Some(ref e)) = last.stmt {
                        if let Some(ek) = self.infer_list_elem_from_expr(e) {
                            return Some(ek);
                        }
                    }
                }
            }
        }
        // Infer the kind and extract element type
        let kind = self.infer_expr_kind(expr);
        match kind {
            ValKind::List(Some(ek)) => Some(*ek),
            _ => None,
        }
    }

    pub(crate) fn kind_to_llvm_type(&self, kind: &ValKind) -> inkwell::types::BasicTypeEnum<'ctx> {
        match kind {
            ValKind::Int | ValKind::Void => self.context.i64_type().into(),
            ValKind::Float => self.context.f64_type().into(),
            ValKind::Bool => self.context.bool_type().into(),
            ValKind::Str | ValKind::List(_) | ValKind::Map(_) | ValKind::Channel => self.ptr_type().into(),
            ValKind::Record(name) => self.records[name].struct_type.into(),
            ValKind::Enum(name) => self.enums[name].enum_type.into(),
            ValKind::Option | ValKind::Result => self.tagged_union_type().into(),
        }
    }

    pub(crate) fn kind_to_param_type(&self, kind: &ValKind) -> inkwell::types::BasicMetadataTypeEnum<'ctx> {
        self.kind_to_llvm_type(kind).into()
    }

    pub(crate) fn declare_function(&mut self, fndef: &FnDef) -> Result<(), CodeGenError> {
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = fndef
            .params
            .iter()
            .map(|p| self.kind_to_param_type(&self.type_expr_to_kind(&p.ty)))
            .collect();

        let ret_kind = fndef.ret_type.as_ref().map(|t| self.type_expr_to_kind(t)).unwrap_or(ValKind::Void);

        let fn_type = if fndef.name == "main" {
            self.context.i32_type().fn_type(&param_types, false)
        } else {
            match &ret_kind {
                ValKind::Void => self.context.void_type().fn_type(&param_types, false),
                kind => {
                    let ret_ty = self.kind_to_llvm_type(kind);
                    ret_ty.fn_type(&param_types, false)
                }
            }
        };

        let func = self.module.add_function(&fndef.name, fn_type, None);
        self.functions.insert(fndef.name.clone(), (func, ret_kind));

        // Track element kind for functions returning List[T] or Map[K, V]
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

        // Store default parameter expressions if any exist
        let defaults: Vec<Option<Expr>> = fndef.params.iter().map(|p| p.default.clone()).collect();
        if defaults.iter().any(|d| d.is_some()) {
            self.fn_defaults.insert(fndef.name.clone(), defaults);
        }
        Ok(())
    }

    pub(crate) fn compile_function(&mut self, fndef: &FnDef) -> Result<(), CodeGenError> {
        let (func, _ret_kind) = self.functions.get(&fndef.name).ok_or_else(|| self.err(format!("undefined function '{}' (not declared before compilation)", fndef.name)))?;
        let func = *func;
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);
        self.variables.clear();
        // Note: list_element_kinds and map_value_kinds are intentionally NOT cleared here.
        // They carry over between functions to provide element kind info for untyped List/Map
        // parameters (e.g., `fn foo tokens:List` without `List[Str]` annotation).

        for (i, param) in fndef.params.iter().enumerate() {
            let val = func.get_nth_param(i as u32).ok_or_else(|| self.err(format!("missing parameter '{}' at index {}", param.name, i)))?;
            let ty = val.get_type();
            let kind = self.type_expr_to_kind(&param.ty);
            let alloca = bld!(self.builder.build_alloca(ty, &param.name))?;
            bld!(self.builder.build_store(alloca, val))?;
            self.variables.insert(param.name.clone(), VarInfo { ptr: alloca, ty, kind: kind.clone(), is_mutable: false });
            // Track element kinds for List[T] and Map value kinds from type annotations
            if kind.is_list() {
                if let TypeExpr::Generic(_, args) = &param.ty {
                    if let Some(elem_ty) = args.first() {
                        let elem_kind = self.type_expr_to_kind(elem_ty);
                        self.list_element_kinds.insert(param.name.clone(), elem_kind.clone());
                        // Also update the variable's ValKind to carry the element kind
                        if let Some(entry) = self.variables.get_mut(&param.name) {
                            entry.kind = ValKind::list_of(elem_kind);
                        }
                    }
                }
            }
            if kind.is_map() {
                if let TypeExpr::Generic(_, args) = &param.ty {
                    if args.len() >= 2 {
                        let val_kind = self.type_expr_to_kind(&args[1]);
                        self.map_value_kinds.insert(param.name.clone(), val_kind);
                    }
                }
            }
        }

        // Pre-scan function body for map.set() calls to track value kinds
        self.prescan_map_value_kinds(&fndef.body);

        let last_val = self.compile_block_stmts(&fndef.body, func)?;

        // Capture element kind for functions returning List without explicit List[T] annotation
        // by checking the last block's returned ValKind
        if !self.fn_return_list_elem_kind.contains_key(&fndef.name) {
            if let Some(ref ret_ty) = fndef.ret_type {
                let is_plain_list = matches!(ret_ty, TypeExpr::Named(n) if n == "List");
                if is_plain_list {
                    // Infer element kind from the function body's last expression
                    let inferred_ek = self.infer_list_return_kind(&fndef.body);
                    if let Some(ek) = inferred_ek {
                        self.fn_return_list_elem_kind.insert(fndef.name.clone(), ek);
                    }
                }
            }
        }

        if self.current_block()?.get_terminator().is_none() {
            if fndef.name == "main" {
                // Join all spawned threads before returning from main
                self.call_rt("ore_thread_join_all", &[], "")?;
                let zero = self.context.i32_type().const_int(0, false);
                bld!(self.builder.build_return(Some(&zero)))?;
            } else if fndef.ret_type.is_some() {
                if let Some(val) = last_val {
                    bld!(self.builder.build_return(Some(&val)))?;
                } else {
                    return Err(self.err(format!("function '{}' must return a value", fndef.name)));
                }
            } else {
                bld!(self.builder.build_return(None))?;
            }
        }

        Ok(())
    }

    pub(crate) fn compile_block_stmts(
        &mut self,
        block: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenError> {
        let (val, _kind) = self.compile_block_stmts_with_kind(block, func)?;
        Ok(val)
    }

    pub(crate) fn compile_block_stmts_with_kind(
        &mut self,
        block: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(Option<BasicValueEnum<'ctx>>, ValKind), CodeGenError> {
        let mut last_val = None;
        let mut last_kind = ValKind::Void;
        for spanned in &block.stmts {
            self.current_line = spanned.line;
            let (val, kind) = self.compile_stmt(&spanned.stmt, func).map_err(|mut e| {
                if e.line.is_none() { e.line = Some(spanned.line); }
                e
            })?;
            if val.is_some() {
                last_val = val;
                last_kind = kind;
            }
        }
        Ok((last_val, last_kind))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_to_ir(src: &str) -> String {
        let tokens = ore_lexer::lex(src).expect("lex failed");
        let program = ore_parser::parse(tokens).expect("parse failed");
        let context = inkwell::context::Context::create();
        let mut codegen = CodeGen::new(&context, "test");
        codegen.compile_program(&program).expect("codegen failed");
        codegen.module.print_to_string().to_string()
    }

    // ── Existing: arithmetic & basics ──────────────────────────────────

    #[test]
    fn test_int_arithmetic_ir() {
        let ir = compile_to_ir("fn add a:Int b:Int -> Int\n  a + b");
        assert!(ir.contains("add i64"), "expected 'add i64' in IR:\n{}", ir);
    }

    #[test]
    fn test_float_arithmetic_ir() {
        let ir = compile_to_ir("fn add a:Float b:Float -> Float\n  a + b");
        assert!(ir.contains("fadd double"), "expected 'fadd double' in IR:\n{}", ir);
    }

    #[test]
    fn test_function_declaration_ir() {
        let ir = compile_to_ir("fn add a:Int b:Int -> Int\n  a + b");
        assert!(ir.contains("define i64 @add(i64 %0, i64 %1)"), "expected 'define i64 @add(i64 %0, i64 %1)' in IR:\n{}", ir);
    }

    #[test]
    fn test_comparison_ir() {
        let ir = compile_to_ir("fn cmp a:Int b:Int -> Bool\n  a < b");
        assert!(ir.contains("icmp slt"), "expected 'icmp slt' in IR:\n{}", ir);
    }

    #[test]
    fn test_float_comparison_ir() {
        let ir = compile_to_ir("fn cmp a:Float b:Float -> Bool\n  a >= b");
        assert!(ir.contains("fcmp oge"), "expected 'fcmp oge' in IR:\n{}", ir);
    }

    #[test]
    fn test_string_literal_ir() {
        let ir = compile_to_ir("fn main\n  x := \"hello\"");
        assert!(ir.contains("@ore_str_new"), "expected '@ore_str_new' in IR:\n{}", ir);
    }

    #[test]
    fn test_int_to_float_promotion_ir() {
        let ir = compile_to_ir("fn add a:Int b:Float -> Float\n  a + b");
        assert!(ir.contains("sitofp"), "expected 'sitofp' in IR:\n{}", ir);
        assert!(ir.contains("fadd"), "expected 'fadd' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_push_ir() {
        let ir = compile_to_ir("fn main\n  x := [1, 2, 3]");
        assert!(ir.contains("@ore_list_new"), "expected '@ore_list_new' in IR:\n{}", ir);
        assert!(ir.contains("@ore_list_push"), "expected '@ore_list_push' in IR:\n{}", ir);
    }

    // ── Arithmetic: remaining operators ────────────────────────────────

    #[test]
    fn test_int_subtraction_ir() {
        let ir = compile_to_ir("fn sub a:Int b:Int -> Int\n  a - b");
        assert!(ir.contains("sub i64"), "expected 'sub i64' in IR:\n{}", ir);
    }

    #[test]
    fn test_int_multiplication_ir() {
        let ir = compile_to_ir("fn mul a:Int b:Int -> Int\n  a * b");
        assert!(ir.contains("mul i64"), "expected 'mul i64' in IR:\n{}", ir);
    }

    #[test]
    fn test_int_division_ir() {
        let ir = compile_to_ir("fn div a:Int b:Int -> Int\n  a / b");
        assert!(ir.contains("sdiv i64"), "expected 'sdiv i64' in IR:\n{}", ir);
    }

    #[test]
    fn test_int_modulo_ir() {
        let ir = compile_to_ir("fn rem a:Int b:Int -> Int\n  a % b");
        assert!(ir.contains("srem i64"), "expected 'srem i64' in IR:\n{}", ir);
    }

    #[test]
    fn test_float_subtraction_ir() {
        let ir = compile_to_ir("fn sub a:Float b:Float -> Float\n  a - b");
        assert!(ir.contains("fsub double"), "expected 'fsub double' in IR:\n{}", ir);
    }

    #[test]
    fn test_float_multiplication_ir() {
        let ir = compile_to_ir("fn mul a:Float b:Float -> Float\n  a * b");
        assert!(ir.contains("fmul double"), "expected 'fmul double' in IR:\n{}", ir);
    }

    #[test]
    fn test_float_division_ir() {
        let ir = compile_to_ir("fn div a:Float b:Float -> Float\n  a / b");
        assert!(ir.contains("fdiv double"), "expected 'fdiv double' in IR:\n{}", ir);
    }

    // ── Comparison operators ───────────────────────────────────────────

    #[test]
    fn test_int_greater_than_ir() {
        let ir = compile_to_ir("fn cmp a:Int b:Int -> Bool\n  a > b");
        assert!(ir.contains("icmp sgt"), "expected 'icmp sgt' in IR:\n{}", ir);
    }

    #[test]
    fn test_int_less_equal_ir() {
        let ir = compile_to_ir("fn cmp a:Int b:Int -> Bool\n  a <= b");
        assert!(ir.contains("icmp sle"), "expected 'icmp sle' in IR:\n{}", ir);
    }

    #[test]
    fn test_int_greater_equal_ir() {
        let ir = compile_to_ir("fn cmp a:Int b:Int -> Bool\n  a >= b");
        assert!(ir.contains("icmp sge"), "expected 'icmp sge' in IR:\n{}", ir);
    }

    #[test]
    fn test_int_equality_ir() {
        let ir = compile_to_ir("fn cmp a:Int b:Int -> Bool\n  a == b");
        assert!(ir.contains("icmp eq"), "expected 'icmp eq' in IR:\n{}", ir);
    }

    #[test]
    fn test_int_not_equal_ir() {
        let ir = compile_to_ir("fn cmp a:Int b:Int -> Bool\n  a != b");
        assert!(ir.contains("icmp ne"), "expected 'icmp ne' in IR:\n{}", ir);
    }

    #[test]
    fn test_float_less_than_ir() {
        let ir = compile_to_ir("fn cmp a:Float b:Float -> Bool\n  a < b");
        assert!(ir.contains("fcmp olt"), "expected 'fcmp olt' in IR:\n{}", ir);
    }

    #[test]
    fn test_float_greater_than_ir() {
        let ir = compile_to_ir("fn cmp a:Float b:Float -> Bool\n  a > b");
        assert!(ir.contains("fcmp ogt"), "expected 'fcmp ogt' in IR:\n{}", ir);
    }

    // ── Boolean operations ─────────────────────────────────────────────

    #[test]
    fn test_bool_literal_true_ir() {
        let ir = compile_to_ir("fn main\n  x := true");
        assert!(ir.contains("i1 true") || ir.contains("const_int(1)") || ir.contains("store i1"),
            "expected bool literal in IR:\n{}", ir);
    }

    #[test]
    fn test_bool_literal_false_ir() {
        let ir = compile_to_ir("fn main\n  x := false");
        assert!(ir.contains("i1 false") || ir.contains("store i1"),
            "expected bool literal in IR:\n{}", ir);
    }

    // ── Mutable variables & assignment ─────────────────────────────────

    #[test]
    fn test_mutable_binding_ir() {
        let ir = compile_to_ir("fn main\n  mut x := 0\n  x = 42");
        // Should have alloca for the variable and two stores
        assert!(ir.contains("alloca"), "expected 'alloca' in IR:\n{}", ir);
        assert!(ir.contains("store"), "expected 'store' in IR:\n{}", ir);
    }

    // ── If/else ────────────────────────────────────────────────────────

    #[test]
    fn test_if_else_branches_ir() {
        let ir = compile_to_ir("fn choose x:Bool -> Int\n  if x\n    1\n  else\n    2\n");
        assert!(ir.contains("br i1"), "expected conditional branch 'br i1' in IR:\n{}", ir);
    }

    #[test]
    fn test_if_else_phi_ir() {
        let ir = compile_to_ir("fn choose x:Bool -> Int\n  if x\n    1\n  else\n    2\n");
        assert!(ir.contains("phi"), "expected phi node in IR:\n{}", ir);
    }

    // ── For loop ───────────────────────────────────────────────────────

    #[test]
    fn test_for_range_loop_ir() {
        let ir = compile_to_ir("fn main\n  for i in 0..10\n    print i\n");
        // Loop structure: comparison, conditional branch, increment
        assert!(ir.contains("icmp slt"), "expected loop comparison in IR:\n{}", ir);
        assert!(ir.contains("br i1"), "expected conditional branch in IR:\n{}", ir);
        assert!(ir.contains("@ore_print_int"), "expected print call in IR:\n{}", ir);
    }

    // ── While loop ─────────────────────────────────────────────────────

    #[test]
    fn test_while_loop_ir() {
        let ir = compile_to_ir("fn main\n  mut x := 0\n  while x < 5\n    x = x + 1\n");
        assert!(ir.contains("br i1"), "expected conditional branch in IR:\n{}", ir);
        assert!(ir.contains("icmp slt"), "expected comparison in IR:\n{}", ir);
    }

    // ── Print codegen ──────────────────────────────────────────────────

    #[test]
    fn test_print_int_ir() {
        let ir = compile_to_ir("fn main\n  print 42");
        assert!(ir.contains("@ore_print_int"), "expected '@ore_print_int' in IR:\n{}", ir);
    }

    #[test]
    fn test_print_string_ir() {
        let ir = compile_to_ir("fn main\n  print \"hello\"");
        assert!(ir.contains("@ore_str_print"), "expected '@ore_str_print' in IR:\n{}", ir);
    }

    #[test]
    fn test_print_bool_ir() {
        let ir = compile_to_ir("fn main\n  print true");
        assert!(ir.contains("@ore_print_bool"), "expected '@ore_print_bool' in IR:\n{}", ir);
    }

    #[test]
    fn test_print_float_ir() {
        let ir = compile_to_ir("fn main\n  print 3.14");
        assert!(ir.contains("@ore_print_float"), "expected '@ore_print_float' in IR:\n{}", ir);
    }

    // ── String interpolation ───────────────────────────────────────────

    #[test]
    fn test_string_interpolation_ir() {
        let ir = compile_to_ir("fn greet name:Str -> Str\n  \"Hello, {name}!\"");
        assert!(ir.contains("@ore_str_concat"), "expected '@ore_str_concat' in IR:\n{}", ir);
    }

    // ── String methods ─────────────────────────────────────────────────

    #[test]
    fn test_str_len_ir() {
        let ir = compile_to_ir("fn main\n  x := \"hello\"\n  x.len()");
        assert!(ir.contains("@ore_str_len"), "expected '@ore_str_len' in IR:\n{}", ir);
    }

    #[test]
    fn test_str_contains_ir() {
        let ir = compile_to_ir("fn main\n  x := \"hello world\"\n  x.contains(\"world\")");
        assert!(ir.contains("@ore_str_contains"), "expected '@ore_str_contains' in IR:\n{}", ir);
    }

    #[test]
    fn test_str_trim_ir() {
        let ir = compile_to_ir("fn main\n  x := \" hello \"\n  x.trim()");
        assert!(ir.contains("@ore_str_trim"), "expected '@ore_str_trim' in IR:\n{}", ir);
    }

    #[test]
    fn test_str_split_ir() {
        let ir = compile_to_ir("fn main\n  x := \"a,b,c\"\n  x.split(\",\")");
        assert!(ir.contains("@ore_str_split"), "expected '@ore_str_split' in IR:\n{}", ir);
    }

    #[test]
    fn test_str_replace_ir() {
        let ir = compile_to_ir("fn main\n  x := \"hello\"\n  x.replace(\"l\", \"r\")");
        assert!(ir.contains("@ore_str_replace"), "expected '@ore_str_replace' in IR:\n{}", ir);
    }

    #[test]
    fn test_str_starts_with_ir() {
        let ir = compile_to_ir("fn main\n  x := \"hello\"\n  x.starts_with(\"he\")");
        assert!(ir.contains("@ore_str_starts_with"), "expected '@ore_str_starts_with' in IR:\n{}", ir);
    }

    #[test]
    fn test_str_ends_with_ir() {
        let ir = compile_to_ir("fn main\n  x := \"hello\"\n  x.ends_with(\"lo\")");
        assert!(ir.contains("@ore_str_ends_with"), "expected '@ore_str_ends_with' in IR:\n{}", ir);
    }

    #[test]
    fn test_str_to_int_ir() {
        let ir = compile_to_ir("fn main\n  x := \"42\"\n  x.to_int()");
        assert!(ir.contains("@ore_str_to_int"), "expected '@ore_str_to_int' in IR:\n{}", ir);
    }

    #[test]
    fn test_str_to_float_ir() {
        let ir = compile_to_ir("fn main\n  x := \"3.14\"\n  x.to_float()");
        assert!(ir.contains("@ore_str_to_float"), "expected '@ore_str_to_float' in IR:\n{}", ir);
    }

    // ── Map operations ─────────────────────────────────────────────────

    #[test]
    fn test_map_literal_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1, \"b\": 2}");
        assert!(ir.contains("@ore_map_new"), "expected '@ore_map_new' in IR:\n{}", ir);
        assert!(ir.contains("@ore_map_set"), "expected '@ore_map_set' in IR:\n{}", ir);
    }

    #[test]
    fn test_map_get_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1}\n  m.get(\"a\")");
        assert!(ir.contains("@ore_map_get"), "expected '@ore_map_get' in IR:\n{}", ir);
    }

    #[test]
    fn test_map_contains_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1}\n  m.contains(\"a\")");
        assert!(ir.contains("@ore_map_contains"), "expected '@ore_map_contains' in IR:\n{}", ir);
    }

    #[test]
    fn test_map_len_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1}\n  m.len()");
        assert!(ir.contains("@ore_map_len"), "expected '@ore_map_len' in IR:\n{}", ir);
    }

    #[test]
    fn test_map_remove_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1}\n  m.remove(\"a\")");
        assert!(ir.contains("@ore_map_remove"), "expected '@ore_map_remove' in IR:\n{}", ir);
    }

    #[test]
    fn test_map_keys_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1}\n  m.keys()");
        assert!(ir.contains("@ore_map_keys"), "expected '@ore_map_keys' in IR:\n{}", ir);
    }

    #[test]
    fn test_map_values_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1}\n  m.values()");
        assert!(ir.contains("@ore_map_values"), "expected '@ore_map_values' in IR:\n{}", ir);
    }

    #[test]
    fn test_map_merge_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1}\n  n := {\"b\": 2}\n  m.merge(n)");
        assert!(ir.contains("@ore_map_merge"), "expected '@ore_map_merge' in IR:\n{}", ir);
    }

    // ── Lambda codegen ─────────────────────────────────────────────────

    #[test]
    fn test_lambda_basic_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.map(n => n * 2)");
        // Lambda should generate a separate function definition
        assert!(ir.contains("@__lambda_"), "expected lambda function '@__lambda_' in IR:\n{}", ir);
    }

    #[test]
    fn test_lambda_with_capture_ir() {
        let ir = compile_to_ir("fn main\n  factor := 10\n  xs := [1, 2, 3]\n  xs.map(n => n * factor)");
        // Captured variables create an environment struct
        assert!(ir.contains("@__lambda_"), "expected lambda function in IR:\n{}", ir);
    }

    #[test]
    fn test_lambda_filter_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3, 4, 5]\n  xs.filter(n => n > 3)");
        assert!(ir.contains("@ore_list_filter"), "expected '@ore_list_filter' in IR:\n{}", ir);
        assert!(ir.contains("@__lambda_"), "expected lambda function in IR:\n{}", ir);
    }

    #[test]
    fn test_lambda_each_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.each(n => print n)");
        assert!(ir.contains("@ore_list_each"), "expected '@ore_list_each' in IR:\n{}", ir);
    }

    // ── List methods ───────────────────────────────────────────────────

    #[test]
    fn test_list_len_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.len()");
        assert!(ir.contains("@ore_list_len"), "expected '@ore_list_len' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_pop_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.pop()");
        assert!(ir.contains("@ore_list_pop"), "expected '@ore_list_pop' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_get_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.get(0)");
        assert!(ir.contains("@ore_list_get"), "expected '@ore_list_get' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_map_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.map(n => n * 2)");
        assert!(ir.contains("@ore_list_map"), "expected '@ore_list_map' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_filter_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.filter(n => n > 1)");
        assert!(ir.contains("@ore_list_filter"), "expected '@ore_list_filter' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_fold_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.fold(0, (acc, n => acc + n))");
        assert!(ir.contains("@ore_list_fold"), "expected '@ore_list_fold' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_any_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.any(n => n > 2)");
        assert!(ir.contains("@ore_list_any"), "expected '@ore_list_any' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_all_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.all(n => n > 0)");
        assert!(ir.contains("@ore_list_all"), "expected '@ore_list_all' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_sort_ir() {
        let ir = compile_to_ir("fn main\n  xs := [3, 1, 2]\n  xs.sort()");
        assert!(ir.contains("@ore_list_sort"), "expected '@ore_list_sort' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_reverse_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.reverse()");
        assert!(ir.contains("@ore_list_reverse"), "expected '@ore_list_reverse' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_join_ir() {
        let ir = compile_to_ir("fn main\n  xs := [\"a\", \"b\", \"c\"]\n  xs.join(\", \")");
        assert!(ir.contains("@ore_list_join"), "expected '@ore_list_join' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_contains_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.contains(2)");
        assert!(ir.contains("@ore_list_contains"), "expected '@ore_list_contains' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_sum_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.sum()");
        assert!(ir.contains("@ore_list_sum"), "expected '@ore_list_sum' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_unique_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 2, 3]\n  xs.unique()");
        assert!(ir.contains("@ore_list_unique"), "expected '@ore_list_unique' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_take_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3, 4]\n  xs.take(2)");
        assert!(ir.contains("@ore_list_take"), "expected '@ore_list_take' in IR:\n{}", ir);
    }

    #[test]
    fn test_list_skip_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3, 4]\n  xs.skip(2)");
        assert!(ir.contains("@ore_list_skip"), "expected '@ore_list_skip' in IR:\n{}", ir);
    }

    // ── Match compilation ──────────────────────────────────────────────

    #[test]
    fn test_literal_match_int_ir() {
        let ir = compile_to_ir("fn label x:Int -> Str\n  x :\n    1 -> \"one\"\n    2 -> \"two\"\n    _ -> \"other\"\n");
        // Literal match generates integer comparisons and branches
        assert!(ir.contains("icmp eq"), "expected 'icmp eq' for match comparison in IR:\n{}", ir);
        assert!(ir.contains("br i1"), "expected conditional branch in IR:\n{}", ir);
    }

    #[test]
    fn test_literal_match_string_ir() {
        let ir = compile_to_ir("fn label x:Str -> Str\n  x :\n    \"a\" -> \"first\"\n    _ -> \"other\"\n");
        // String matching uses ore_str_eq
        assert!(ir.contains("@ore_str_eq"), "expected '@ore_str_eq' in IR:\n{}", ir);
    }

    #[test]
    fn test_literal_match_bool_ir() {
        let ir = compile_to_ir("fn label x:Bool -> Str\n  x :\n    true -> \"yes\"\n    false -> \"no\"\n");
        assert!(ir.contains("icmp eq"), "expected 'icmp eq' in match IR:\n{}", ir);
    }

    #[test]
    fn test_enum_match_ir() {
        let ir = compile_to_ir(
            "type Shape\n  Circle(radius: Float)\n  Rect(width: Float, height: Float)\n\n\
             fn area s:Shape -> Float\n  s :\n    Circle r -> r * r\n    Rect w h -> w * h\n"
        );
        // Enum match: tag extraction via GEP, dispatch via switch
        assert!(ir.contains("@area"), "expected '@area' function in IR:\n{}", ir);
        assert!(ir.contains("switch i8"), "expected tag switch in IR:\n{}", ir);
    }

    #[test]
    fn test_enum_variant_construct_ir() {
        let ir = compile_to_ir(
            "type Shape\n  Circle(radius: Float)\n  Rect(width: Float, height: Float)\n\n\
             fn main\n  s := Circle(radius: 3.14)\n"
        );
        // Enum construction: alloca struct, store tag, store payload
        assert!(ir.contains("alloca"), "expected alloca for enum in IR:\n{}", ir);
        assert!(ir.contains("store i8"), "expected tag store in IR:\n{}", ir);
    }

    #[test]
    fn test_match_wildcard_ir() {
        let ir = compile_to_ir("fn classify x:Int -> Str\n  x :\n    0 -> \"zero\"\n    _ -> \"nonzero\"\n");
        // Wildcard arm should have a default branch
        assert!(ir.contains("br i1"), "expected branch in IR:\n{}", ir);
        assert!(ir.contains("@ore_str_new"), "expected string creation in IR:\n{}", ir);
    }

    // ── Record codegen ─────────────────────────────────────────────────

    #[test]
    fn test_record_construct_ir() {
        let ir = compile_to_ir("type Point { x:Float, y:Float }\n\nfn main\n  p := Point(x: 1.0, y: 2.0)\n");
        // Record construction: alloca struct, store fields
        assert!(ir.contains("alloca"), "expected alloca for record in IR:\n{}", ir);
        assert!(ir.contains("store double"), "expected float field store in IR:\n{}", ir);
    }

    #[test]
    fn test_record_field_access_ir() {
        let ir = compile_to_ir(
            "type Point { x:Float, y:Float }\n\n\
             fn getx p:Point -> Float\n  p.x\n"
        );
        assert!(ir.contains("@getx"), "expected '@getx' function in IR:\n{}", ir);
    }

    // ── Impl blocks / method calls ─────────────────────────────────────

    #[test]
    fn test_impl_method_ir() {
        let ir = compile_to_ir(
            "type Point { x:Float, y:Float }\n\n\
             impl Point\n  fn magnitude self:Point -> Float\n    self.x * self.x + self.y * self.y\n"
        );
        // Impl generates a function named after the type
        assert!(ir.contains("@Point_magnitude") || ir.contains("magnitude"),
            "expected method function in IR:\n{}", ir);
        assert!(ir.contains("fmul double"), "expected float multiplication in method body IR:\n{}", ir);
    }

    #[test]
    fn test_impl_method_call_ir() {
        let ir = compile_to_ir(
            "type Point { x:Float, y:Float }\n\n\
             impl Point\n  fn magnitude self:Point -> Float\n    self.x * self.x + self.y * self.y\n\n\
             fn main\n  p := Point(x: 3.0, y: 4.0)\n  p.magnitude()\n"
        );
        // Should call the magnitude method
        assert!(ir.contains("call"), "expected method call in IR:\n{}", ir);
    }

    // ── Builtin functions ──────────────────────────────────────────────

    #[test]
    fn test_builtin_abs_int_ir() {
        let ir = compile_to_ir("fn do_abs x:Int -> Int\n  abs(x)");
        // abs(Int) is inlined as (x ^ sign) - sign
        assert!(ir.contains("xor"), "expected xor in abs codegen IR:\n{}", ir);
    }

    #[test]
    fn test_builtin_min_ir() {
        let ir = compile_to_ir("fn do_min a:Int b:Int -> Int\n  min(a, b)");
        // min generates icmp + select
        assert!(ir.contains("icmp slt"), "expected 'icmp slt' in min IR:\n{}", ir);
        assert!(ir.contains("select"), "expected 'select' in min IR:\n{}", ir);
    }

    #[test]
    fn test_builtin_max_ir() {
        let ir = compile_to_ir("fn do_max a:Int b:Int -> Int\n  max(a, b)");
        assert!(ir.contains("icmp sgt"), "expected 'icmp sgt' in max IR:\n{}", ir);
        assert!(ir.contains("select"), "expected 'select' in max IR:\n{}", ir);
    }

    // ── Unary operations ───────────────────────────────────────────────

    #[test]
    fn test_unary_minus_int_ir() {
        let ir = compile_to_ir("fn neg x:Int -> Int\n  -x");
        assert!(ir.contains("sub i64"), "expected 'sub i64' for negation in IR:\n{}", ir);
    }

    #[test]
    fn test_unary_minus_float_ir() {
        let ir = compile_to_ir("fn neg x:Float -> Float\n  -x");
        assert!(ir.contains("fneg"), "expected 'fneg' for float negation in IR:\n{}", ir);
    }

    // ── Function calls ─────────────────────────────────────────────────

    #[test]
    fn test_function_call_ir() {
        let ir = compile_to_ir("fn double x:Int -> Int\n  x * 2\n\nfn main\n  double(5)");
        assert!(ir.contains("call i64 @double"), "expected 'call i64 @double' in IR:\n{}", ir);
    }

    #[test]
    fn test_recursive_function_ir() {
        let ir = compile_to_ir(
            "fn factorial n:Int -> Int\n  if n < 2\n    1\n  else\n    n * factorial(n - 1)\n"
        );
        assert!(ir.contains("call i64 @factorial"), "expected recursive call in IR:\n{}", ir);
        assert!(ir.contains("mul i64"), "expected multiplication in IR:\n{}", ir);
    }

    // ── Multiple functions ─────────────────────────────────────────────

    #[test]
    fn test_multiple_functions_ir() {
        let ir = compile_to_ir(
            "fn add a:Int b:Int -> Int\n  a + b\n\n\
             fn mul a:Int b:Int -> Int\n  a * b\n"
        );
        assert!(ir.contains("define i64 @add"), "expected '@add' function in IR:\n{}", ir);
        assert!(ir.contains("define i64 @mul"), "expected '@mul' function in IR:\n{}", ir);
    }

    // ── Return type handling ───────────────────────────────────────────

    #[test]
    fn test_void_function_ir() {
        let ir = compile_to_ir("fn greet\n  print \"hello\"");
        assert!(ir.contains("define void @greet") || ir.contains("define i64 @greet"),
            "expected greet function definition in IR:\n{}", ir);
    }

    #[test]
    fn test_bool_return_ir() {
        let ir = compile_to_ir("fn is_positive x:Int -> Bool\n  x > 0");
        assert!(ir.contains("define i1 @is_positive"), "expected 'define i1 @is_positive' in IR:\n{}", ir);
    }

    // ── List comprehension ─────────────────────────────────────────────

    #[test]
    fn test_list_comprehension_ir() {
        let ir = compile_to_ir("fn main\n  xs := [x * x for x in 0..5]");
        assert!(ir.contains("@ore_list_new"), "expected list creation in comprehension IR:\n{}", ir);
        assert!(ir.contains("@ore_list_push"), "expected list push in comprehension IR:\n{}", ir);
        assert!(ir.contains("mul i64"), "expected multiplication in comprehension IR:\n{}", ir);
    }

    // ── Index access ───────────────────────────────────────────────────

    #[test]
    fn test_list_index_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs[1]");
        assert!(ir.contains("@ore_list_get"), "expected '@ore_list_get' for indexing in IR:\n{}", ir);
    }

    // ── Pipeline ───────────────────────────────────────────────────────

    #[test]
    fn test_pipeline_ir() {
        let ir = compile_to_ir("fn double x:Int -> Int\n  x * 2\n\nfn main\n  result := 5 | double");
        assert!(ir.contains("call i64 @double"), "expected pipeline function call in IR:\n{}", ir);
    }

    // ── Compound assignment ────────────────────────────────────────────

    #[test]
    fn test_compound_add_assign_ir() {
        let ir = compile_to_ir("fn main\n  mut x := 10\n  x += 5");
        assert!(ir.contains("add i64"), "expected 'add i64' for += in IR:\n{}", ir);
    }

    #[test]
    fn test_compound_sub_assign_ir() {
        let ir = compile_to_ir("fn main\n  mut x := 10\n  x -= 3");
        assert!(ir.contains("sub i64"), "expected 'sub i64' for -= in IR:\n{}", ir);
    }

    #[test]
    fn test_compound_mul_assign_ir() {
        let ir = compile_to_ir("fn main\n  mut x := 10\n  x *= 4");
        assert!(ir.contains("mul i64"), "expected 'mul i64' for *= in IR:\n{}", ir);
    }

    // ── Enum zero-arg variant ──────────────────────────────────────────

    #[test]
    fn test_enum_zero_arg_variant_ir() {
        let ir = compile_to_ir(
            "type Color\n  Red\n  Green\n  Blue\n\n\
             fn main\n  c := Red\n"
        );
        // Zero-arg variant stores a tag
        assert!(ir.contains("store i8"), "expected tag store for zero-arg variant in IR:\n{}", ir);
    }

    // ── Multiple return paths (phi merging) ────────────────────────────

    #[test]
    fn test_multi_branch_phi_ir() {
        let ir = compile_to_ir(
            "fn sign x:Int -> Int\n  if x > 0\n    1\n  else if x < 0\n    -1\n  else\n    0\n"
        );
        assert!(ir.contains("phi"), "expected phi node for multi-branch in IR:\n{}", ir);
    }

    // ── Runtime declarations ───────────────────────────────────────────

    #[test]
    fn test_runtime_declarations_present() {
        let ir = compile_to_ir("fn main\n  print 42");
        // Basic runtime functions should be declared
        assert!(ir.contains("declare"), "expected runtime declarations in IR:\n{}", ir);
        assert!(ir.contains("ore_print_int"), "expected ore_print_int declaration in IR:\n{}", ir);
    }

    // ── String list operations ─────────────────────────────────────────

    #[test]
    fn test_string_list_ir() {
        let ir = compile_to_ir("fn main\n  xs := [\"a\", \"b\", \"c\"]");
        assert!(ir.contains("@ore_list_new"), "expected '@ore_list_new' in IR:\n{}", ir);
        assert!(ir.contains("@ore_str_new"), "expected '@ore_str_new' for string elements in IR:\n{}", ir);
    }

    // ── List reduce ────────────────────────────────────────────────────

    #[test]
    fn test_list_reduce_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.reduce((a, b) => a + b)");
        assert!(ir.contains("@ore_list_reduce1"), "expected '@ore_list_reduce1' in IR:\n{}", ir);
    }

    // ── For-each loop ──────────────────────────────────────────────────

    #[test]
    fn test_for_each_loop_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  for x in xs\n    print x\n");
        assert!(ir.contains("@ore_list_len"), "expected list len call in foreach IR:\n{}", ir);
        assert!(ir.contains("@ore_list_get"), "expected list get call in foreach IR:\n{}", ir);
    }

    // ── Map each/iteration ─────────────────────────────────────────────

    #[test]
    fn test_map_each_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1, \"b\": 2}\n  m.each((k, v) => print k)");
        assert!(ir.contains("@ore_map_each"), "expected '@ore_map_each' in IR:\n{}", ir);
    }

    // ── Enum with fields match ─────────────────────────────────────────

    #[test]
    fn test_enum_match_with_payload_ir() {
        let ir = compile_to_ir(
            "type Shape\n  Circle(radius: Float)\n  Rect(width: Float, height: Float)\n\n\
             fn describe s:Shape -> Float\n  match s\n    Circle r -> r\n    Rect w h -> w + h\n"
        );
        // Match should extract payload from enum variant
        assert!(ir.contains("@describe"), "expected describe function in IR:\n{}", ir);
        assert!(ir.contains("fadd double"), "expected float add in Rect arm in IR:\n{}", ir);
    }

    // ── List sort_by ───────────────────────────────────────────────────

    #[test]
    fn test_list_sort_by_ir() {
        let ir = compile_to_ir("fn main\n  xs := [3, 1, 2]\n  xs.sort_by(x => x)");
        assert!(ir.contains("@ore_list_sort_by_key"), "expected '@ore_list_sort_by_key' in IR:\n{}", ir);
    }

    // ── List find ──────────────────────────────────────────────────────

    #[test]
    fn test_list_find_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 2, 3]\n  xs.find(n => n > 1)");
        assert!(ir.contains("@ore_list_find"), "expected '@ore_list_find' in IR:\n{}", ir);
    }

    // ── Map clear ──────────────────────────────────────────────────────

    #[test]
    fn test_map_clear_ir() {
        let ir = compile_to_ir("fn main\n  m := {\"a\": 1}\n  m.clear()");
        assert!(ir.contains("@ore_map_clear"), "expected '@ore_map_clear' in IR:\n{}", ir);
    }

    // ── List flatten ───────────────────────────────────────────────────

    #[test]
    fn test_list_flatten_ir() {
        let ir = compile_to_ir("fn main\n  xs := [[1, 2], [3, 4]]\n  xs.flatten()");
        assert!(ir.contains("@ore_list_flatten"), "expected '@ore_list_flatten' in IR:\n{}", ir);
    }

    // ── List dedup ─────────────────────────────────────────────────────

    #[test]
    fn test_list_dedup_ir() {
        let ir = compile_to_ir("fn main\n  xs := [1, 1, 2, 2, 3]\n  xs.dedup()");
        assert!(ir.contains("@ore_list_dedup"), "expected '@ore_list_dedup' in IR:\n{}", ir);
    }
}
