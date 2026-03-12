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
    /// LLVM types of captured variables (in struct field order)
    pub(crate) types: Vec<inkwell::types::BasicTypeEnum<'ctx>>,
}

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    /// Maps variable names to (pointer, pointee type, kind, mutable)
    pub(crate) variables: HashMap<String, (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>, ValKind, bool)>,
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
    /// Maps type name -> list of method names (for method call resolution)
    pub(crate) method_map: HashMap<String, Vec<String>>,
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
            method_map: HashMap::new(),
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

    /// Built-in Option type: { i8, i64 } where tag=0 is None, tag=1 is Some
    /// Built-in Option type: { i8 tag, i8 kind, i64 payload }
    /// tag: 0=None, 1=Some; kind: ValKind discriminant of the payload
    pub(crate) fn option_type(&self) -> inkwell::types::StructType<'ctx> {
        self.context.struct_type(
            &[self.context.i8_type().into(), self.context.i8_type().into(), self.context.i64_type().into()],
            false,
        )
    }

    /// Built-in Result type: { i8 tag, i8 kind, i64 payload }
    /// tag: 0=Ok, 1=Err; kind: ValKind discriminant of the payload
    pub(crate) fn result_type(&self) -> inkwell::types::StructType<'ctx> {
        self.context.struct_type(
            &[self.context.i8_type().into(), self.context.i8_type().into(), self.context.i64_type().into()],
            false,
        )
    }

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
        for item in &program.items {
            let (type_name, methods) = match item {
                Item::ImplBlock { type_name, methods } => (type_name, methods),
                Item::ImplTrait { type_name, methods, .. } => (type_name, methods),
                _ => continue,
            };
            let mut method_names = Vec::new();
            for method in methods {
                method_names.push(method.name.clone());
                let mangled_fn = Self::mangle_impl_method(type_name, method);
                self.declare_function(&mangled_fn)?;
            }
            self.method_map.entry(type_name.clone())
                .or_default()
                .extend(method_names);
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
        for item in &program.items {
            let (type_name, methods) = match item {
                Item::ImplBlock { type_name, methods } => (type_name, methods),
                Item::ImplTrait { type_name, methods, .. } => (type_name, methods),
                _ => continue,
            };
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

    pub(crate) fn register_record(&mut self, td: &TypeDef) -> Result<(), CodeGenError> {
        let mut field_types = Vec::new();
        let mut field_kinds = Vec::new();
        let mut field_names = Vec::new();

        for f in &td.fields {
            let kind = self.type_expr_to_kind(&f.ty);
            let llvm_ty = self.kind_to_llvm_type(&kind);
            field_types.push(llvm_ty);
            field_kinds.push(kind);
            field_names.push(f.name.clone());
        }

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
            let mut field_types = Vec::new();
            let mut field_kinds = Vec::new();
            let mut field_names = Vec::new();

            for f in &v.fields {
                let kind = self.type_expr_to_kind(&f.ty);
                let llvm_ty = self.kind_to_llvm_type(&kind);
                field_types.push(llvm_ty);
                field_kinds.push(kind);
                field_names.push(f.name.clone());
            }

            let payload_type = self.context.struct_type(&field_types, false);
            // Compute payload size in bytes (manual estimation)
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

    pub(crate) fn declare_runtime_functions(&mut self) {
        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();
        let f64_type = self.context.f64_type();
        let void_type = self.context.void_type();
        let ptr_type = self.ptr_type();

        let ext = Some(inkwell::module::Linkage::External);

        // ore_print_int(i64)
        self.module.add_function("ore_print_int", void_type.fn_type(&[i64_type.into()], false), ext);
        // ore_print_bool(i8)
        self.module.add_function("ore_print_bool", void_type.fn_type(&[i8_type.into()], false), ext);
        // ore_print_float(f64)
        self.module.add_function("ore_print_float", void_type.fn_type(&[f64_type.into()], false), ext);
        // ore_str_new(ptr, u32) -> ptr
        self.module.add_function("ore_str_new", ptr_type.fn_type(&[ptr_type.into(), i32_type.into()], false), ext);
        // ore_str_concat(ptr, ptr) -> ptr
        self.module.add_function("ore_str_concat", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // ore_str_print(ptr)
        self.module.add_function("ore_str_print", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_str_print_no_newline(ptr)
        self.module.add_function("ore_str_print_no_newline", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_print_int_no_newline(i64)
        self.module.add_function("ore_print_int_no_newline", void_type.fn_type(&[i64_type.into()], false), ext);
        // ore_print_float_no_newline(f64)
        self.module.add_function("ore_print_float_no_newline", void_type.fn_type(&[f64_type.into()], false), ext);
        // ore_print_bool_no_newline(i8)
        self.module.add_function("ore_print_bool_no_newline", void_type.fn_type(&[i8_type.into()], false), ext);
        // stderr print functions
        self.module.add_function("ore_eprint_str", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_eprint_int", void_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_eprint_float", void_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_eprint_bool", void_type.fn_type(&[i8_type.into()], false), ext);
        // ore_str_retain(ptr)
        self.module.add_function("ore_str_retain", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_str_release(ptr)
        self.module.add_function("ore_str_release", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_int_to_str(i64) -> ptr
        self.module.add_function("ore_int_to_str", ptr_type.fn_type(&[i64_type.into()], false), ext);
        // ore_bool_to_str(i8) -> ptr
        self.module.add_function("ore_bool_to_str", ptr_type.fn_type(&[i8_type.into()], false), ext);
        // ore_spawn(ptr) — takes a function pointer
        self.module.add_function("ore_spawn", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_spawn_with_arg(ptr, i64) — takes a function pointer and one i64 arg
        self.module.add_function("ore_spawn_with_arg", void_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_spawn_with_2args", void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_spawn_with_3args", void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into(), i64_type.into()], false), ext);
        // ore_thread_join_all()
        self.module.add_function("ore_thread_join_all", void_type.fn_type(&[], false), ext);
        // ore_sleep(i64)
        self.module.add_function("ore_sleep", void_type.fn_type(&[i64_type.into()], false), ext);
        // ore_assert(i1, *const u8, i64) — assert with message and line
        self.module.add_function("ore_assert", void_type.fn_type(&[i8_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        // ore_assert_eq_int(i64, i64, *const u8, i64)
        self.module.add_function("ore_assert_eq_int", void_type.fn_type(&[i64_type.into(), i64_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        // ore_assert_eq_float(f64, f64, *const u8, i64)
        self.module.add_function("ore_assert_eq_float", void_type.fn_type(&[f64_type.into(), f64_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        // ore_assert_eq_str(ptr, ptr, *const u8, i64)
        self.module.add_function("ore_assert_eq_str", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        // ore_assert_ne_int(i64, i64, *const u8, i64)
        self.module.add_function("ore_assert_ne_int", void_type.fn_type(&[i64_type.into(), i64_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        // ore_assert_ne_str(ptr, ptr, *const u8, i64)
        self.module.add_function("ore_assert_ne_str", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        // List operations
        // ore_list_new() -> ptr
        self.module.add_function("ore_list_new", ptr_type.fn_type(&[], false), ext);
        // ore_list_push(ptr, i64)
        self.module.add_function("ore_list_push", void_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_pop", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_clear", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_insert", void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_remove_at", i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_get(ptr, i64) -> i64
        self.module.add_function("ore_list_get", i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_set", void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_get_or", i64_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        // ore_list_len(ptr) -> i64
        self.module.add_function("ore_list_len", i64_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_print(ptr)
        self.module.add_function("ore_list_print", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_print_typed(ptr, i64)
        self.module.add_function("ore_list_print_typed", void_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_print_str(ptr)
        self.module.add_function("ore_list_print_str", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_print_float(ptr)
        self.module.add_function("ore_list_print_float", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_print_bool(ptr)
        self.module.add_function("ore_list_print_bool", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_map(ptr, fn_ptr, env_ptr) -> ptr
        self.module.add_function("ore_list_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_filter(ptr, fn_ptr, env_ptr) -> ptr
        self.module.add_function("ore_list_filter", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_each(ptr, fn_ptr, env_ptr)
        self.module.add_function("ore_list_each", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_find_index(ptr, fn_ptr, env_ptr) -> i64
        self.module.add_function("ore_list_find_index", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_find", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_fold(ptr, init_i64, fn_ptr, env_ptr) -> i64
        self.module.add_function("ore_list_fold", i64_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_sort(ptr) -> ptr (returns new sorted list)
        self.module.add_function("ore_list_sort", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_reverse(ptr)
        self.module.add_function("ore_list_reverse", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_contains(ptr, i64) -> i8
        self.module.add_function("ore_list_contains", i8_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_concat(ptr, ptr) -> ptr
        self.module.add_function("ore_list_concat", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_par_map(ptr, fn_ptr, env_ptr) -> ptr
        self.module.add_function("ore_list_par_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_par_each(ptr, fn_ptr, env_ptr)
        self.module.add_function("ore_list_par_each", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_any(ptr, fn_ptr, env_ptr) -> i8
        self.module.add_function("ore_list_any", i8_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_all(ptr, fn_ptr, env_ptr) -> i8
        self.module.add_function("ore_list_all", i8_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_zip(ptr, ptr) -> ptr
        self.module.add_function("ore_list_zip", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_zip_with", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_enumerate(ptr) -> ptr
        self.module.add_function("ore_list_enumerate", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_join_str(ptr, sep) -> ptr
        self.module.add_function("ore_list_join_str", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_join_float(ptr, sep) -> ptr
        self.module.add_function("ore_list_join_float", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_flat_map(ptr, fn_ptr, env_ptr) -> ptr
        self.module.add_function("ore_list_flat_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_count_by(ptr, fn_ptr, env_ptr) -> ptr (map)
        self.module.add_function("ore_list_count_by", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_count_by_int", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_group_by(ptr, fn_ptr, env_ptr) -> ptr (map)
        self.module.add_function("ore_list_group_by", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_range(i64, i64) -> ptr
        self.module.add_function("ore_range", ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_range_step", ptr_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_repeat", ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false), ext);
        // ore_list_take(ptr, i64) -> ptr
        self.module.add_function("ore_list_take", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_skip(ptr, i64) -> ptr
        self.module.add_function("ore_list_skip", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_step", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_sum(ptr) -> i64
        self.module.add_function("ore_list_sum", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_product", i64_type.fn_type(&[ptr_type.into()], false), ext);
        // String utilities
        self.module.add_function("ore_float_to_str", ptr_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_str_len", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_eq", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_cmp", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // String methods
        self.module.add_function("ore_str_contains", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_trim", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_trim_start", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_trim_end", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_lines", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_char_at", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_ord", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_chr", ptr_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_str_capitalize", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_split", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_to_int", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_to_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_replace", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_starts_with", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_ends_with", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_to_upper", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_to_lower", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_substr", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_str_chars", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_repeat", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_str_pad_left", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_pad_right", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_count", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_strip_prefix", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_strip_suffix", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_index_of", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_slice", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_assert_fail", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_split_whitespace", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_min", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_max", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_count", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_by", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_by_key", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_min_by", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_max_by", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_by_key_str", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_index_of", i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_unique", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_flatten", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_partition", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_scan", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_take_while", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_drop_while", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_window", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_chunks", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_str_reverse", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_reverse_new", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_slice", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        // ore_list_reduce(ptr, i64, fn_ptr, env_ptr) -> i64
        self.module.add_function("ore_list_reduce", i64_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_reduce1", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_find(ptr, fn_ptr, env_ptr, default) -> i64
        self.module.add_function("ore_list_find", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_join(ptr, sep) -> ptr
        self.module.add_function("ore_list_join", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // Map operations
        self.module.add_function("ore_map_new", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_map_set", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_map_set_typed", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into(), i8_type.into()], false), ext);
        self.module.add_function("ore_map_get", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_contains", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_len", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_remove", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_keys", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_values", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_print", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_print_str", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_merge", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_clear", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_each", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_map_values", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_filter", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_map_entries", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_map_get_or", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_to_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_frequencies", ptr_type.fn_type(&[ptr_type.into(), i8_type.into()], false), ext);
        self.module.add_function("ore_list_intersperse", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_sort_str", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_float", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_dedup", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_tap", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_map_with_index", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_each_with_index", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_min_str", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_max_str", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_contains_str", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_index_of_str", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_unique_by", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_unique_str", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_average", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_average_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sum_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_product_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_min_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_max_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
        // Math functions
        self.module.add_function("ore_math_sqrt", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_sin", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_cos", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_tan", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_log", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_log10", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_exp", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_pow", f64_type.fn_type(&[f64_type.into(), f64_type.into()], false), ext);
        self.module.add_function("ore_math_abs", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_floor", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_ceil", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_round", f64_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_math_pi", f64_type.fn_type(&[], false), ext);
        self.module.add_function("ore_math_e", f64_type.fn_type(&[], false), ext);
        self.module.add_function("ore_math_atan2", f64_type.fn_type(&[f64_type.into(), f64_type.into()], false), ext);
        self.module.add_function("ore_float_round_to", f64_type.fn_type(&[f64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_float_format", ptr_type.fn_type(&[f64_type.into(), i64_type.into()], false), ext);
        // I/O
        self.module.add_function("ore_readln", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_file_read", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_file_read_lines", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_file_write", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_file_exists", i8_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_file_append", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // Process
        self.module.add_function("ore_args", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_exit", void_type.fn_type(&[i64_type.into()], false), ext);
        self.module.add_function("ore_env_get", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        // Runtime errors
        self.module.add_function("ore_div_by_zero", void_type.fn_type(&[], false), ext);
        // JSON
        self.module.add_function("ore_json_parse", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_json_stringify", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        // Time
        self.module.add_function("ore_time_now", i64_type.fn_type(&[], false), ext);
        self.module.add_function("ore_time_ms", i64_type.fn_type(&[], false), ext);
        // Random
        self.module.add_function("ore_rand_int", i64_type.fn_type(&[i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_exec", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_type_of", ptr_type.fn_type(&[i8_type.into()], false), ext);
        self.module.add_function("ore_env_set", void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // ore_dynamic_to_str(i64, i8) -> ptr — dynamic dispatch for Result/Option payload to string
        self.module.add_function("ore_dynamic_to_str", ptr_type.fn_type(&[i64_type.into(), i8_type.into()], false), ext);
        // Channels
        self.module.add_function("ore_channel_new", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_channel_send", void_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_channel_recv", i64_type.fn_type(&[ptr_type.into()], false), ext);

        // Int math
        self.module.add_function("ore_int_pow", i64_type.fn_type(&[i64_type.into(), i64_type.into()], false), ext);

        // String parsing
        self.module.add_function("ore_str_parse_int", i64_type.fn_type(&[ptr_type.into()], false), ext);
        let f64_type = self.context.f64_type();
        self.module.add_function("ore_str_parse_float", f64_type.fn_type(&[ptr_type.into()], false), ext);
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
                if let Some((_, _, kind, _)) = self.variables.get(name) {
                    kind.clone()
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
            ValKind::Option => self.option_type().into(),
            ValKind::Result => self.result_type().into(),
            ValKind::List(_) | ValKind::Map | ValKind::Channel => self.ptr_type().into(),
        }
    }

    pub(crate) fn kind_to_param_type(&self, kind: &ValKind) -> inkwell::types::BasicMetadataTypeEnum<'ctx> {
        match kind {
            ValKind::Int => self.context.i64_type().into(),
            ValKind::Float => self.context.f64_type().into(),
            ValKind::Bool => self.context.bool_type().into(),
            ValKind::Str => self.ptr_type().into(),
            ValKind::Void => self.context.i64_type().into(),
            ValKind::Record(name) => self.records[name].struct_type.into(),
            ValKind::Enum(name) => self.enums[name].enum_type.into(),
            ValKind::Option => self.option_type().into(),
            ValKind::Result => self.result_type().into(),
            ValKind::List(_) | ValKind::Map | ValKind::Channel => self.ptr_type().into(),
        }
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
        let (func, _ret_kind) = self.functions.get(&fndef.name).ok_or_else(|| CodeGenError {
            line: Some(self.current_line),
            msg: format!("undefined function '{}' (not declared before compilation)", fndef.name),
        })?;
        let func = *func;
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);
        self.variables.clear();

        for (i, param) in fndef.params.iter().enumerate() {
            let val = func.get_nth_param(i as u32).unwrap();
            let ty = val.get_type();
            let kind = self.type_expr_to_kind(&param.ty);
            let alloca = bld!(self.builder.build_alloca(ty, &param.name))?;
            bld!(self.builder.build_store(alloca, val))?;
            self.variables.insert(param.name.clone(), (alloca, ty, kind.clone(), false));
            // Track element kinds for List[T] and Map value kinds from type annotations
            if kind.is_list() {
                if let TypeExpr::Generic(_, args) = &param.ty {
                    if let Some(elem_ty) = args.first() {
                        let elem_kind = self.type_expr_to_kind(elem_ty);
                        self.list_element_kinds.insert(param.name.clone(), elem_kind.clone());
                        // Also update the variable's ValKind to carry the element kind
                        if let Some(entry) = self.variables.get_mut(&param.name) {
                            entry.2 = ValKind::list_of(elem_kind);
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
                    return Err(CodeGenError {
                        line: Some(self.current_line), msg: format!("function '{}' must return a value", fndef.name),
                    });
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
