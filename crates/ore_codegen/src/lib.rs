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
    Map,
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
    /// Temporary: element kind from the last compiled list literal
    pub(crate) last_list_elem_kind: Option<ValKind>,
    pub(crate) last_lambda_return_kind: Option<ValKind>,
    /// Maps variable name -> value ValKind for typed maps
    pub(crate) map_value_kinds: HashMap<String, ValKind>,
    /// Temporary: value kind from the last compiled map literal
    pub(crate) last_map_val_kind: Option<ValKind>,
    /// Current source line (for error reporting)
    pub(crate) current_line: usize,
    /// Generic function definitions (not yet monomorphized)
    pub(crate) generic_fns: HashMap<String, FnDef>,
    /// Default parameter expressions per function (name -> vec of Option<Expr>)
    pub(crate) fn_defaults: HashMap<String, Vec<Option<Expr>>>,
    /// Tracks element kind for functions returning List[T]
    pub(crate) fn_return_list_elem_kind: HashMap<String, ValKind>,
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
            last_list_elem_kind: None,
            last_lambda_return_kind: None,
            map_value_kinds: HashMap::new(),
            last_map_val_kind: None,
            current_line: 0,
            generic_fns: HashMap::new(),
            fn_defaults: HashMap::new(),
            fn_return_list_elem_kind: HashMap::new(),
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
            ValKind::Map => 10,
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
            Item::ImplBlock { type_name, methods } => Some((type_name, methods)),
            Item::ImplTrait { type_name, methods, .. } => Some((type_name, methods)),
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
            inkwell::types::BasicTypeEnum::FloatType(_) => 8, // f64
            inkwell::types::BasicTypeEnum::PointerType(_) => 8, // 64-bit pointer
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
            Expr::ListLit(_) | Expr::ListComp { .. } => ValKind::List(None),
            Expr::MapLit(_) => ValKind::Map,
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
                "Map" => ValKind::Map,
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
                    "Map" => ValKind::Map,
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

    pub(crate) fn kind_to_llvm_type(&self, kind: &ValKind) -> inkwell::types::BasicTypeEnum<'ctx> {
        match kind {
            ValKind::Int => self.context.i64_type().into(),
            ValKind::Float => self.context.f64_type().into(),
            ValKind::Bool => self.context.bool_type().into(),
            ValKind::Str => self.ptr_type().into(),
            ValKind::Void => self.context.i64_type().into(),
            ValKind::Record(name) => self.records[name].struct_type.into(),
            ValKind::Enum(name) => self.enums[name].enum_type.into(),
            ValKind::Option | ValKind::Result => self.tagged_union_type().into(),
            ValKind::List(_) | ValKind::Map | ValKind::Channel => self.ptr_type().into(),
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
            if kind == ValKind::Map {
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
        if !self.fn_return_list_elem_kind.contains_key(&fndef.name) {
            if let Some(ref ret_ty) = fndef.ret_type {
                let is_plain_list = matches!(ret_ty, TypeExpr::Named(n) if n == "List");
                if is_plain_list {
                    if let Some(ref ek) = self.last_list_elem_kind {
                        self.fn_return_list_elem_kind.insert(fndef.name.clone(), ek.clone());
                    }
                }
            }
        }

        if self.current_block()?.get_terminator().is_none() {
            if fndef.name == "main" {
                // Join all spawned threads before returning from main
                let join_all = self.rt("ore_thread_join_all")?;
                bld!(self.builder.build_call(join_all, &[], ""))?;
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
            let val = self.compile_stmt(&spanned.stmt, func).map_err(|mut e| {
                if e.line.is_none() { e.line = Some(spanned.line); }
                e
            })?;
            if val.is_some() {
                last_val = val;
                // Determine kind from the last expression statement
                if let Stmt::Expr(expr) = &spanned.stmt {
                    // We already compiled it, but we need the kind.
                    // Use a heuristic based on the value type.
                    if let Some(v) = &last_val {
                        last_kind = match v {
                            BasicValueEnum::PointerValue(_) => {
                                // Could be Str, List, Map — check the expression
                                match expr {
                                    Expr::StringLit(_) | Expr::StringInterp(_) => ValKind::Str,
                                    Expr::ListLit(_) | Expr::ListComp { .. } => ValKind::List(None),
                                    Expr::MapLit(_) => ValKind::Map,
                                    _ => ValKind::Str, // Best guess for pointer values
                                }
                            }
                            BasicValueEnum::IntValue(iv) => {
                                if iv.get_type().get_bit_width() == 1 {
                                    ValKind::Bool
                                } else {
                                    ValKind::Int
                                }
                            }
                            BasicValueEnum::FloatValue(_) => ValKind::Float,
                            _ => ValKind::Int,
                        };
                    }
                }
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
}
