use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::types::BasicType;
use inkwell::IntPredicate;
use std::collections::{HashMap, HashSet};

use ore_parser::ast::*;

/// Walk an expression tree and collect identifiers that are not in `bound`.
fn find_free_vars(expr: &Expr, bound: &HashSet<String>) -> Vec<String> {
    let mut free = Vec::new();
    let mut seen = HashSet::new();
    collect_free_vars(expr, bound, &mut free, &mut seen);
    free
}

fn collect_free_vars(expr: &Expr, bound: &HashSet<String>, free: &mut Vec<String>, seen: &mut HashSet<String>) {
    match expr {
        Expr::Ident(name) => {
            if !bound.contains(name) && !seen.contains(name) {
                seen.insert(name.clone());
                free.push(name.clone());
            }
        }
        Expr::BinOp { left, right, .. } => {
            collect_free_vars(left, bound, free, seen);
            collect_free_vars(right, bound, free, seen);
        }
        Expr::UnaryMinus(inner) | Expr::UnaryNot(inner) | Expr::Print(inner)
        | Expr::OptionSome(inner) | Expr::ResultOk(inner) | Expr::ResultErr(inner)
        | Expr::Try(inner) | Expr::Sleep(inner) => {
            collect_free_vars(inner, bound, free, seen);
        }
        Expr::Call { func, args } => {
            collect_free_vars(func, bound, free, seen);
            for arg in args {
                collect_free_vars(arg, bound, free, seen);
            }
        }
        Expr::Lambda { params, body } => {
            // Lambda params introduce new bindings; they shadow outer names
            let mut inner_bound = bound.clone();
            for p in params {
                inner_bound.insert(p.clone());
            }
            collect_free_vars(body, &inner_bound, free, seen);
        }
        Expr::IfElse { cond, then_block, else_block } => {
            collect_free_vars(cond, bound, free, seen);
            for stmt in then_block.iter_stmts() {
                collect_free_vars_stmt(stmt, bound, free, seen);
            }
            if let Some(eb) = else_block {
                for stmt in eb.iter_stmts() {
                    collect_free_vars_stmt(stmt, bound, free, seen);
                }
            }
        }
        Expr::ColonMatch { cond, then_expr, else_expr } => {
            collect_free_vars(cond, bound, free, seen);
            collect_free_vars(then_expr, bound, free, seen);
            if let Some(e) = else_expr {
                collect_free_vars(e, bound, free, seen);
            }
        }
        Expr::Match { subject, arms } => {
            collect_free_vars(subject, bound, free, seen);
            for arm in arms {
                if let Some(g) = &arm.guard {
                    collect_free_vars(g, bound, free, seen);
                }
                collect_free_vars(&arm.body, bound, free, seen);
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let StringPart::Expr(e) = part {
                    collect_free_vars(e, bound, free, seen);
                }
            }
        }
        Expr::RecordConstruct { fields, .. } => {
            for (_, e) in fields {
                collect_free_vars(e, bound, free, seen);
            }
        }
        Expr::FieldAccess { object, .. } => {
            collect_free_vars(object, bound, free, seen);
        }
        Expr::MethodCall { object, args, .. } => {
            collect_free_vars(object, bound, free, seen);
            for arg in args {
                collect_free_vars(arg, bound, free, seen);
            }
        }
        Expr::ListLit(elements) => {
            for e in elements {
                collect_free_vars(e, bound, free, seen);
            }
        }
        Expr::MapLit(entries) => {
            for (k, v) in entries {
                collect_free_vars(k, bound, free, seen);
                collect_free_vars(v, bound, free, seen);
            }
        }
        Expr::Index { object, index } => {
            collect_free_vars(object, bound, free, seen);
            collect_free_vars(index, bound, free, seen);
        }
        Expr::OptionalChain { object, .. } => {
            collect_free_vars(object, bound, free, seen);
        }
        Expr::OptionalMethodCall { object, args, .. } => {
            collect_free_vars(object, bound, free, seen);
            for arg in args {
                collect_free_vars(arg, bound, free, seen);
            }
        }
        // Literals and constants have no free variables
        Expr::IntLit(_) | Expr::FloatLit(_) | Expr::BoolLit(_) | Expr::StringLit(_)
        | Expr::Break | Expr::OptionNone => {}
    }
}

fn collect_free_vars_stmt(stmt: &Stmt, bound: &HashSet<String>, free: &mut Vec<String>, seen: &mut HashSet<String>) {
    match stmt {
        Stmt::Let { value, .. } => collect_free_vars(value, bound, free, seen),
        Stmt::Assign { value, .. } => collect_free_vars(value, bound, free, seen),
        Stmt::Expr(e) => collect_free_vars(e, bound, free, seen),
        Stmt::Return(Some(e)) => collect_free_vars(e, bound, free, seen),
        Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        Stmt::Spawn(e) => collect_free_vars(e, bound, free, seen),
        Stmt::ForIn { start, end, body, .. } => {
            collect_free_vars(start, bound, free, seen);
            collect_free_vars(end, bound, free, seen);
            for s in body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::While { cond, body } => {
            collect_free_vars(cond, bound, free, seen);
            for s in body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::ForEach { iterable, body, .. } | Stmt::ForEachKV { iterable, body, .. } => {
            collect_free_vars(iterable, bound, free, seen);
            for s in body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::Loop { body } => {
            for s in body.iter_stmts() {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::IndexAssign { object, index, value } => {
            collect_free_vars(object, bound, free, seen);
            collect_free_vars(index, bound, free, seen);
            collect_free_vars(value, bound, free, seen);
        }
        Stmt::FieldAssign { object, field: _, value } => {
            collect_free_vars(object, bound, free, seen);
            collect_free_vars(value, bound, free, seen);
        }
    }
}

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
    List,
    Map,
    Channel,
}

struct RecordInfo<'ctx> {
    struct_type: inkwell::types::StructType<'ctx>,
    field_names: Vec<String>,
    field_kinds: Vec<ValKind>,
}

struct VariantInfo<'ctx> {
    name: String,
    tag: u8,
    field_names: Vec<String>,
    field_kinds: Vec<ValKind>,
    payload_type: inkwell::types::StructType<'ctx>,
}

struct EnumInfo<'ctx> {
    enum_type: inkwell::types::StructType<'ctx>,
    variants: Vec<VariantInfo<'ctx>>,
}

/// Tracks capture information for a compiled lambda/closure
#[allow(dead_code)]
struct CaptureInfo<'ctx> {
    /// The LLVM struct type holding all captured values
    struct_type: inkwell::types::StructType<'ctx>,
    /// Names of captured variables (in struct field order)
    names: Vec<String>,
    /// LLVM types of captured variables (in struct field order)
    types: Vec<inkwell::types::BasicTypeEnum<'ctx>>,
    /// ValKind of each captured variable
    kinds: Vec<ValKind>,
}

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    /// Maps variable names to (pointer, pointee type, kind, mutable)
    variables: HashMap<String, (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>, ValKind, bool)>,
    functions: HashMap<String, (FunctionValue<'ctx>, ValKind)>,
    records: HashMap<String, RecordInfo<'ctx>>,
    enums: HashMap<String, EnumInfo<'ctx>>,
    /// Maps variant name -> enum name for quick lookup
    variant_to_enum: HashMap<String, String>,
    /// Target block for `break` statements
    break_target: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    /// Target block for `continue` statements
    continue_target: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    str_counter: u32,
    lambda_counter: u32,
    /// Maps lambda function name -> capture info (only for closures with captures)
    lambda_captures: HashMap<String, CaptureInfo<'ctx>>,
    /// Maps type name -> list of method names (for method call resolution)
    method_map: HashMap<String, Vec<String>>,
    /// Maps variable name -> alloca for runtime kind tag (used for dynamic dispatch in Option/Result payloads)
    dynamic_kind_tags: HashMap<String, PointerValue<'ctx>>,
    /// Maps variable name -> element ValKind for typed lists
    list_element_kinds: HashMap<String, ValKind>,
    /// Temporary: element kind from the last compiled list literal
    last_list_elem_kind: Option<ValKind>,
    /// Maps variable name -> value ValKind for typed maps
    map_value_kinds: HashMap<String, ValKind>,
    /// Temporary: value kind from the last compiled map literal
    last_map_val_kind: Option<ValKind>,
    /// Current source line (for error reporting)
    current_line: usize,
    /// Generic function definitions (not yet monomorphized)
    generic_fns: HashMap<String, FnDef>,
}

#[derive(Debug, Default)]
pub struct CodeGenError {
    pub msg: String,
    #[allow(dead_code)]
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
            map_value_kinds: HashMap::new(),
            last_map_val_kind: None,
            current_line: 0,
            generic_fns: HashMap::new(),
        }
    }

    fn ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(inkwell::AddressSpace::default())
    }

    /// Built-in Option type: { i8, i64 } where tag=0 is None, tag=1 is Some
    /// Built-in Option type: { i8 tag, i8 kind, i64 payload }
    /// tag: 0=None, 1=Some; kind: ValKind discriminant of the payload
    fn option_type(&self) -> inkwell::types::StructType<'ctx> {
        self.context.struct_type(
            &[self.context.i8_type().into(), self.context.i8_type().into(), self.context.i64_type().into()],
            false,
        )
    }

    /// Built-in Result type: { i8 tag, i8 kind, i64 payload }
    /// tag: 0=Ok, 1=Err; kind: ValKind discriminant of the payload
    fn result_type(&self) -> inkwell::types::StructType<'ctx> {
        self.context.struct_type(
            &[self.context.i8_type().into(), self.context.i8_type().into(), self.context.i64_type().into()],
            false,
        )
    }

    fn valkind_to_tag(&self, kind: &ValKind) -> u8 {
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
            ValKind::List => 9,
            ValKind::Map => 10,
            ValKind::Channel => 11,
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
                let mangled_name = format!("{}_{}", type_name, method.name);
                method_names.push(method.name.clone());
                // Prepend implicit `self` parameter if not already declared
                let has_self = method.params.first().map_or(false, |p| p.name == "self");
                let params = if has_self {
                    method.params.clone()
                } else {
                    let mut p = vec![Param {
                        name: "self".to_string(),
                        ty: TypeExpr::Named(type_name.clone()),
                    }];
                    p.extend(method.params.clone());
                    p
                };
                let mangled_fn = FnDef {
                    name: mangled_name,
                    type_params: method.type_params.clone(),
                    params,
                    ret_type: method.ret_type.clone(),
                    body: method.body.clone(),
                };
                self.declare_function(&mangled_fn)?;
            }
            // Merge with existing methods if any
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
                let mangled_name = format!("{}_{}", type_name, method.name);
                let has_self = method.params.first().map_or(false, |p| p.name == "self");
                let params = if has_self {
                    method.params.clone()
                } else {
                    let mut p = vec![Param {
                        name: "self".to_string(),
                        ty: TypeExpr::Named(type_name.clone()),
                    }];
                    p.extend(method.params.clone());
                    p
                };
                let mangled_fn = FnDef {
                    name: mangled_name,
                    type_params: method.type_params.clone(),
                    params,
                    ret_type: method.ret_type.clone(),
                    body: method.body.clone(),
                };
                self.compile_function(&mangled_fn)?;
            }
        }

        Ok(())
    }

    fn register_record(&mut self, td: &TypeDef) -> Result<(), CodeGenError> {
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

    fn register_enum(&mut self, ed: &EnumDef) -> Result<(), CodeGenError> {
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

        // Enum layout: { i8 (tag), [max_payload_size x i8] (data) }
        let i8_type = self.context.i8_type();
        let data_array = i8_type.array_type(max_payload_size as u32);
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

    fn type_size_bytes(&self, ty: &inkwell::types::BasicTypeEnum<'ctx>) -> u64 {
        match ty {
            inkwell::types::BasicTypeEnum::IntType(t) => {
                (t.get_bit_width() as u64 + 7) / 8
            }
            inkwell::types::BasicTypeEnum::FloatType(_) => 8, // f64
            inkwell::types::BasicTypeEnum::PointerType(_) => 8, // 64-bit pointer
            inkwell::types::BasicTypeEnum::StructType(t) => {
                t.get_field_types().iter().map(|f| self.type_size_bytes(&f)).sum()
            }
            inkwell::types::BasicTypeEnum::ArrayType(t) => {
                let elem_size = self.type_size_bytes(&t.get_element_type());
                elem_size * t.len() as u64
            }
            _ => 8, // fallback
        }
    }

    fn declare_runtime_functions(&mut self) {
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
        // ore_thread_join_all()
        self.module.add_function("ore_thread_join_all", void_type.fn_type(&[], false), ext);
        // ore_sleep(i64)
        self.module.add_function("ore_sleep", void_type.fn_type(&[i64_type.into()], false), ext);
        // List operations
        // ore_list_new() -> ptr
        self.module.add_function("ore_list_new", ptr_type.fn_type(&[], false), ext);
        // ore_list_push(ptr, i64)
        self.module.add_function("ore_list_push", void_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_get(ptr, i64) -> i64
        self.module.add_function("ore_list_get", i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_len(ptr) -> i64
        self.module.add_function("ore_list_len", i64_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_set(ptr, i64, i64)
        self.module.add_function("ore_list_set", void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
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
        // ore_list_sort(ptr)
        self.module.add_function("ore_list_sort", void_type.fn_type(&[ptr_type.into()], false), ext);
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
        // ore_list_enumerate(ptr) -> ptr
        self.module.add_function("ore_list_enumerate", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_join_str(ptr, sep) -> ptr
        self.module.add_function("ore_list_join_str", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_flat_map(ptr, fn_ptr, env_ptr) -> ptr
        self.module.add_function("ore_list_flat_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_range(i64, i64) -> ptr
        self.module.add_function("ore_range", ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false), ext);
        // ore_list_take(ptr, i64) -> ptr
        self.module.add_function("ore_list_take", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_skip(ptr, i64) -> ptr
        self.module.add_function("ore_list_skip", ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_sum(ptr) -> i64
        self.module.add_function("ore_list_sum", i64_type.fn_type(&[ptr_type.into()], false), ext);
        // String utilities
        self.module.add_function("ore_float_to_str", ptr_type.fn_type(&[f64_type.into()], false), ext);
        self.module.add_function("ore_str_len", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_eq", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_cmp", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // String methods
        self.module.add_function("ore_str_contains", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_trim", ptr_type.fn_type(&[ptr_type.into()], false), ext);
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
        self.module.add_function("ore_str_index_of", i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_str_slice", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_assert_fail", void_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_split_whitespace", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_min", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_max", i64_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_count", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_sort_by", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        self.module.add_function("ore_list_index_of", i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false), ext);
        self.module.add_function("ore_list_unique", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_flatten", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_str_reverse", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_reverse_new", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_list_slice", ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false), ext);
        // ore_list_reduce(ptr, i64, fn_ptr, env_ptr) -> i64
        self.module.add_function("ore_list_reduce", i64_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_find(ptr, fn_ptr, env_ptr, default) -> i64
        self.module.add_function("ore_list_find", i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
        // ore_list_join(ptr, sep) -> ptr
        self.module.add_function("ore_list_join", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // Map operations
        self.module.add_function("ore_map_new", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_map_set", void_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false), ext);
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
        // I/O
        self.module.add_function("ore_readln", ptr_type.fn_type(&[], false), ext);
        self.module.add_function("ore_file_read", ptr_type.fn_type(&[ptr_type.into()], false), ext);
        self.module.add_function("ore_file_write", i8_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
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

    fn type_expr_to_kind(&self, ty: &TypeExpr) -> ValKind {
        match ty {
            TypeExpr::Named(n) => match n.as_str() {
                "Int" => ValKind::Int,
                "Float" => ValKind::Float,
                "Bool" => ValKind::Bool,
                "Str" => ValKind::Str,
                "Option" => ValKind::Option,
                "Result" => ValKind::Result,
                "List" => ValKind::List,
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
            TypeExpr::Generic(name, _args) => {
                // For now, treat generic types by their base name
                match name.as_str() {
                    "List" => ValKind::List,
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

    fn kind_to_llvm_type(&self, kind: &ValKind) -> inkwell::types::BasicTypeEnum<'ctx> {
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
            ValKind::List | ValKind::Map | ValKind::Channel => self.ptr_type().into(),
        }
    }

    fn kind_to_param_type(&self, kind: &ValKind) -> inkwell::types::BasicMetadataTypeEnum<'ctx> {
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
            ValKind::List | ValKind::Map | ValKind::Channel => self.ptr_type().into(),
        }
    }

    fn declare_function(&mut self, fndef: &FnDef) -> Result<(), CodeGenError> {
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
        Ok(())
    }

    fn compile_function(&mut self, fndef: &FnDef) -> Result<(), CodeGenError> {
        let (func, _ret_kind) = self.functions.get(&fndef.name).unwrap();
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
            self.variables.insert(param.name.clone(), (alloca, ty, kind, false));
        }

        let last_val = self.compile_block_stmts(&fndef.body, func)?;

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            if fndef.name == "main" {
                // Join all spawned threads before returning from main
                let join_all = self.module.get_function("ore_thread_join_all").unwrap();
                bld!(self.builder.build_call(join_all, &[], ""))?;
                let zero = self.context.i32_type().const_int(0, false);
                bld!(self.builder.build_return(Some(&zero)))?;
            } else if fndef.ret_type.is_some() {
                if let Some(val) = last_val {
                    bld!(self.builder.build_return(Some(&val)))?;
                } else {
                    return Err(CodeGenError {
                        line: None, msg: format!("function '{}' must return a value", fndef.name),
                    });
                }
            } else {
                bld!(self.builder.build_return(None))?;
            }
        }

        Ok(())
    }

    fn compile_block_stmts(
        &mut self,
        block: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenError> {
        let (val, _kind) = self.compile_block_stmts_with_kind(block, func)?;
        Ok(val)
    }

    fn compile_block_stmts_with_kind(
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
                                    Expr::ListLit(_) => ValKind::List,
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

    fn compile_stmt(
        &mut self,
        stmt: &Stmt,
        func: FunctionValue<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenError> {
        match stmt {
            Stmt::Let { name, mutable, value } => {
                self.last_list_elem_kind = None;
                self.last_map_val_kind = None;
                let (val, kind) = self.compile_expr_with_kind(value, func)?;
                let ty = val.get_type();
                let alloca = bld!(self.builder.build_alloca(ty, name))?;
                bld!(self.builder.build_store(alloca, val))?;
                self.variables.insert(name.clone(), (alloca, ty, kind.clone(), *mutable));
                // Track element kind for typed lists
                if kind == ValKind::List {
                    if let Some(ek) = self.last_list_elem_kind.take() {
                        self.list_element_kinds.insert(name.clone(), ek);
                    }
                }
                // Track value kind for typed maps
                if kind == ValKind::Map {
                    if let Some(vk) = self.last_map_val_kind.take() {
                        self.map_value_kinds.insert(name.clone(), vk);
                    }
                }
                Ok(None)
            }
            Stmt::Assign { name, value } => {
                let (val, _kind) = self.compile_expr_with_kind(value, func)?;
                let (ptr, _, _, is_mut) = self.variables.get(name).ok_or_else(|| CodeGenError {
                    line: None, msg: format!("undefined variable '{}'", name),
                })?;
                if !is_mut {
                    return Err(CodeGenError {
                        line: None, msg: format!("cannot assign to immutable variable '{}'", name),
                    });
                }
                bld!(self.builder.build_store(*ptr, val))?;
                Ok(None)
            }
            Stmt::IndexAssign { object, index, value } => {
                let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
                let idx_val = self.compile_expr(index, func)?;
                let (val, _val_kind) = self.compile_expr_with_kind(value, func)?;
                match obj_kind {
                    ValKind::List => {
                        let rt = self.module.get_function("ore_list_set").unwrap();
                        bld!(self.builder.build_call(rt, &[obj_val.into(), idx_val.into(), val.into()], ""))?;
                    }
                    ValKind::Map => {
                        let rt = self.module.get_function("ore_map_set").unwrap();
                        bld!(self.builder.build_call(rt, &[obj_val.into(), idx_val.into(), val.into()], ""))?;
                    }
                    _ => return Err(CodeGenError { line: None, msg: "index assignment only supported on lists and maps".into() }),
                }
                Ok(None)
            }
            Stmt::FieldAssign { object, field, value } => {
                let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
                let (val, _val_kind) = self.compile_expr_with_kind(value, func)?;
                match obj_kind {
                    ValKind::Record(ref name) => {
                        let rec_info = self.records.get(name).ok_or_else(|| CodeGenError {
                            line: None, msg: format!("undefined record type '{}'", name),
                        })?;
                        let field_idx = rec_info.field_names.iter().position(|f| f == field).ok_or_else(|| CodeGenError {
                            line: None, msg: format!("unknown field '{}' on record '{}'", field, name),
                        })?;
                        let struct_type = rec_info.struct_type;
                        let field_ptr = bld!(self.builder.build_struct_gep(
                            struct_type, obj_val.into_pointer_value(), field_idx as u32, &format!("fld_{}", field)
                        ))?;
                        bld!(self.builder.build_store(field_ptr, val))?;
                    }
                    _ => return Err(CodeGenError { line: None, msg: format!("field assignment not supported on {:?}", obj_kind) }),
                }
                Ok(None)
            }
            Stmt::Expr(expr) => {
                let (val, _kind) = self.compile_expr_with_kind(expr, func)?;
                Ok(Some(val))
            }
            Stmt::Return(Some(expr)) => {
                let (val, _kind) = self.compile_expr_with_kind(expr, func)?;
                bld!(self.builder.build_return(Some(&val)))?;
                Ok(None)
            }
            Stmt::Return(None) => {
                bld!(self.builder.build_return(None))?;
                Ok(None)
            }
            Stmt::ForIn { var, start, end, body } => {
                self.compile_for_in(var, start, end, body, func)?;
                Ok(None)
            }
            Stmt::ForEach { var, iterable, body } => {
                self.compile_for_each(var, iterable, body, func)?;
                Ok(None)
            }
            Stmt::ForEachKV { key_var, val_var, iterable, body } => {
                self.compile_for_each_kv(key_var, val_var, iterable, body, func)?;
                Ok(None)
            }
            Stmt::While { cond, body } => {
                self.compile_while(cond, body, func)?;
                Ok(None)
            }
            Stmt::Loop { body } => {
                self.compile_loop(body, func)?;
                Ok(None)
            }
            Stmt::Break => {
                if let Some(target) = self.break_target {
                    bld!(self.builder.build_unconditional_branch(target))?;
                } else {
                    return Err(CodeGenError { line: None, msg: "break outside of loop".into() });
                }
                Ok(None)
            }
            Stmt::Continue => {
                if let Some(target) = self.continue_target {
                    bld!(self.builder.build_unconditional_branch(target))?;
                } else {
                    return Err(CodeGenError { line: None, msg: "continue outside of loop".into() });
                }
                Ok(None)
            }
            Stmt::Spawn(expr) => {
                match expr {
                    Expr::Call { func: callee, args } => {
                        let name = match callee.as_ref() {
                            Expr::Ident(n) => n.clone(),
                            _ => return Err(CodeGenError { line: None, msg: "spawn requires a named function call".into() }),
                        };
                        let (target_fn, _) = self.resolve_function(&name)?;
                        let fn_ptr = target_fn.as_global_value().as_pointer_value();

                        if args.is_empty() {
                            let ore_spawn = self.module.get_function("ore_spawn").unwrap();
                            bld!(self.builder.build_call(ore_spawn, &[fn_ptr.into()], ""))?;
                        } else if args.len() == 1 {
                            let arg_val = self.compile_expr(&args[0], func)?;
                            let i64_val = self.value_to_i64(arg_val)?;
                            let ore_spawn = self.module.get_function("ore_spawn_with_arg").unwrap();
                            bld!(self.builder.build_call(ore_spawn, &[fn_ptr.into(), i64_val.into()], ""))?;
                        } else {
                            return Err(CodeGenError { line: None, msg: "spawn supports at most 1 argument".into() });
                        }
                        Ok(None)
                    }
                    _ => Err(CodeGenError { line: None, msg: "spawn requires a function call".into() }),
                }
            }
        }
    }

    fn compile_expr(
        &mut self,
        expr: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        self.compile_expr_with_kind(expr, func).map(|(v, _)| v)
    }

    fn compile_expr_with_kind(
        &mut self,
        expr: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match expr {
            Expr::IntLit(n) => {
                Ok((self.context.i64_type().const_int(*n as u64, true).into(), ValKind::Int))
            }
            Expr::FloatLit(f) => {
                Ok((self.context.f64_type().const_float(*f).into(), ValKind::Float))
            }
            Expr::BoolLit(b) => {
                Ok((self.context.bool_type().const_int(*b as u64, false).into(), ValKind::Bool))
            }
            Expr::StringLit(s) => {
                let ptr = self.compile_string_literal(s)?;
                Ok((ptr.into(), ValKind::Str))
            }
            Expr::StringInterp(parts) => {
                let ptr = self.compile_string_interp(parts, func)?;
                Ok((ptr.into(), ValKind::Str))
            }
            Expr::Lambda { params, body } => {
                let lambda_fn = self.compile_lambda(params, body, func)?;
                // Return the function pointer
                let ptr = lambda_fn.as_global_value().as_pointer_value();
                Ok((ptr.into(), ValKind::Int)) // Kind is approximate; lambdas are function pointers
            }
            Expr::Ident(name) => {
                let (ptr, ty, kind, _) = self.variables.get(name).ok_or_else(|| CodeGenError {
                    line: None, msg: format!("undefined variable '{}'", name),
                })?;
                let val = bld!(self.builder.build_load(*ty, *ptr, name))?;
                let kind = kind.clone();
                // Restore list element kind tracking for method dispatch
                if kind == ValKind::List {
                    if let Some(elem_kind) = self.list_element_kinds.get(name) {
                        self.last_list_elem_kind = Some(elem_kind.clone());
                    }
                }
                // Restore map value kind tracking for method dispatch
                if kind == ValKind::Map {
                    if let Some(val_kind) = self.map_value_kinds.get(name) {
                        self.last_map_val_kind = Some(val_kind.clone());
                    }
                }
                Ok((val, kind))
            }
            Expr::BinOp { op, left, right } => {
                if *op == BinOp::Pipe {
                    return self.compile_pipeline_with_kind(left, right, func);
                }
                let (lhs, lk) = self.compile_expr_with_kind(left, func)?;
                let (rhs, _rk) = self.compile_expr_with_kind(right, func)?;

                // List concatenation: list + list
                if lk == ValKind::List && *op == BinOp::Add {
                    let rt = self.module.get_function("ore_list_concat").unwrap();
                    let result = bld!(self.builder.build_call(rt, &[lhs.into(), rhs.into()], "lcat"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::List));
                }

                // String repetition: str * int
                if lk == ValKind::Str && *op == BinOp::Mul {
                    let rt = self.module.get_function("ore_str_repeat").unwrap();
                    let result = bld!(self.builder.build_call(rt, &[lhs.into(), rhs.into()], "srepeat"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Str));
                }

                let result = self.compile_binop(*op, lhs, rhs)?;
                // Determine result kind
                let result_kind = match op {
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq
                    | BinOp::And | BinOp::Or => ValKind::Bool,
                    _ => lk,
                };
                Ok((result, result_kind))
            }
            Expr::UnaryMinus(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                match val {
                    BasicValueEnum::IntValue(v) => {
                        Ok((bld!(self.builder.build_int_neg(v, "neg"))?.into(), kind))
                    }
                    BasicValueEnum::FloatValue(v) => {
                        Ok((bld!(self.builder.build_float_neg(v, "fneg"))?.into(), kind))
                    }
                    _ => Err(CodeGenError { line: None, msg: "cannot negate this type".into() }),
                }
            }
            Expr::UnaryNot(inner) => {
                let val = self.compile_expr(inner, func)?;
                match val {
                    BasicValueEnum::IntValue(v) => {
                        Ok((bld!(self.builder.build_not(v, "not"))?.into(), ValKind::Bool))
                    }
                    _ => Err(CodeGenError { line: None, msg: "cannot apply 'not' to this type".into() }),
                }
            }
            Expr::Print(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                // Check if printing a dynamic-kind variable (from Result/Option match)
                if let Expr::Ident(name) = inner.as_ref() {
                    if let Some(kind_alloca) = self.dynamic_kind_tags.get(name).copied() {
                        let kind_i8 = bld!(self.builder.build_load(self.context.i8_type(), kind_alloca, "dyn_kind"))?.into_int_value();
                        let dyn_fn = self.module.get_function("ore_dynamic_to_str").unwrap();
                        let result = bld!(self.builder.build_call(dyn_fn, &[val.into(), kind_i8.into()], "dyntos"))?;
                        let str_ptr = self.call_result_to_value(result)?.into_pointer_value();
                        let pf = self.module.get_function("ore_str_print").unwrap();
                        bld!(self.builder.build_call(pf, &[str_ptr.into()], ""))?;
                        let release = self.module.get_function("ore_str_release").unwrap();
                        bld!(self.builder.build_call(release, &[str_ptr.into()], ""))?;
                        return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void));
                    }
                    // Check for typed list printing
                    if kind == ValKind::List {
                        if let Some(elem_kind) = self.list_element_kinds.get(name).cloned() {
                            match elem_kind {
                                ValKind::Int => {} // Fall through to default int list print
                                _ => {
                                    // Generate inline typed list print loop
                                    self.compile_typed_list_print(val.into_pointer_value(), &elem_kind)?;
                                    return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void));
                                }
                            }
                        }
                    }
                }
                // Check for typed list printing via last_list_elem_kind (for method calls etc.)
                if kind == ValKind::List {
                    if let Some(elem_kind) = self.last_list_elem_kind.take() {
                        if elem_kind != ValKind::Int {
                            self.compile_typed_list_print(val.into_pointer_value(), &elem_kind)?;
                            return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void));
                        }
                    }
                }
                // Check for string-valued map printing
                if kind == ValKind::Map {
                    if let Some(ValKind::Str) = self.last_map_val_kind.take() {
                        let pf = self.module.get_function("ore_map_print_str").unwrap();
                        bld!(self.builder.build_call(pf, &[val.into()], ""))?;
                        return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void));
                    }
                }
                self.compile_print(val, kind)?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            Expr::Sleep(inner) => {
                let val = self.compile_expr(inner, func)?;
                let ore_sleep = self.module.get_function("ore_sleep").unwrap();
                bld!(self.builder.build_call(ore_sleep, &[val.into()], ""))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            Expr::Call { func: callee, args } => {
                let name = match callee.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(CodeGenError { line: None, msg: "only named function calls supported".into() }),
                };

                // Built-in stdlib functions
                match name.as_str() {
                    "abs" => {
                        if args.len() != 1 {
                            return Err(CodeGenError { line: None, msg: "abs takes 1 argument".into() });
                        }
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        match kind {
                            ValKind::Int => {
                                // abs for int: (x ^ (x >> 63)) - (x >> 63)
                                let x = val.into_int_value();
                                let shift = self.context.i64_type().const_int(63, false);
                                let sign = bld!(self.builder.build_right_shift(x, shift, true, "sign"))?;
                                let xored = bld!(self.builder.build_xor(x, sign, "xor"))?;
                                let result = bld!(self.builder.build_int_sub(xored, sign, "abs"))?;
                                return Ok((result.into(), ValKind::Int));
                            }
                            ValKind::Float => {
                                let x = val.into_float_value();
                                let neg = bld!(self.builder.build_float_neg(x, "neg"))?;
                                let zero = self.context.f64_type().const_float(0.0);
                                let is_neg = bld!(self.builder.build_float_compare(
                                    inkwell::FloatPredicate::OLT, x, zero, "is_neg"
                                ))?;
                                let result = bld!(self.builder.build_select(is_neg, neg, x, "abs"))?;
                                return Ok((result, ValKind::Float));
                            }
                            _ => return Err(CodeGenError { line: None, msg: "abs requires Int or Float".into() }),
                        }
                    }
                    "min" => {
                        if args.len() != 2 {
                            return Err(CodeGenError { line: None, msg: "min takes 2 arguments".into() });
                        }
                        let a = self.compile_expr(&args[0], func)?;
                        let b = self.compile_expr(&args[1], func)?;
                        let cmp = bld!(self.builder.build_int_compare(
                            inkwell::IntPredicate::SLT, a.into_int_value(), b.into_int_value(), "cmp"
                        ))?;
                        let result = bld!(self.builder.build_select(cmp, a, b, "min"))?;
                        return Ok((result, ValKind::Int));
                    }
                    "max" => {
                        if args.len() != 2 {
                            return Err(CodeGenError { line: None, msg: "max takes 2 arguments".into() });
                        }
                        let a = self.compile_expr(&args[0], func)?;
                        let b = self.compile_expr(&args[1], func)?;
                        let cmp = bld!(self.builder.build_int_compare(
                            inkwell::IntPredicate::SGT, a.into_int_value(), b.into_int_value(), "cmp"
                        ))?;
                        let result = bld!(self.builder.build_select(cmp, a, b, "max"))?;
                        return Ok((result, ValKind::Int));
                    }
                    "channel" => {
                        let rt = self.module.get_function("ore_channel_new").unwrap();
                        let result = bld!(self.builder.build_call(rt, &[], "ch"))?;
                        let val = self.call_result_to_value(result)?;
                        return Ok((val, ValKind::Channel));
                    }
                    "readln" => {
                        let rt = self.module.get_function("ore_readln").unwrap();
                        let result = bld!(self.builder.build_call(rt, &[], "readln"))?;
                        let val = self.call_result_to_value(result)?;
                        return Ok((val, ValKind::Str));
                    }
                    "file_read" => {
                        if args.len() != 1 {
                            return Err(CodeGenError { line: None, msg: "file_read takes 1 argument".into() });
                        }
                        let path_val = self.compile_expr(&args[0], func)?;
                        let rt = self.module.get_function("ore_file_read").unwrap();
                        let result = bld!(self.builder.build_call(rt, &[path_val.into()], "file_read"))?;
                        let val = self.call_result_to_value(result)?;
                        return Ok((val, ValKind::Str));
                    }
                    "file_write" => {
                        if args.len() != 2 {
                            return Err(CodeGenError { line: None, msg: "file_write takes 2 arguments".into() });
                        }
                        let path_val = self.compile_expr(&args[0], func)?;
                        let content_val = self.compile_expr(&args[1], func)?;
                        let rt = self.module.get_function("ore_file_write").unwrap();
                        let result = bld!(self.builder.build_call(rt, &[path_val.into(), content_val.into()], "file_write"))?;
                        let val = self.call_result_to_value(result)?;
                        return Ok((val, ValKind::Bool));
                    }
                    "range" => {
                        if args.len() != 2 {
                            return Err(CodeGenError { line: None, msg: "range takes 2 arguments (start, end)".into() });
                        }
                        let start = self.compile_expr(&args[0], func)?;
                        let end = self.compile_expr(&args[1], func)?;
                        let rt = self.module.get_function("ore_range").unwrap();
                        let result = bld!(self.builder.build_call(rt, &[start.into(), end.into()], "range"))?;
                        let val = self.call_result_to_value(result)?;
                        self.last_list_elem_kind = Some(ValKind::Int);
                        return Ok((val, ValKind::List));
                    }
                    "int" => {
                        if args.len() != 1 {
                            return Err(CodeGenError { line: None, msg: "int() takes 1 argument".into() });
                        }
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        let result = match kind {
                            ValKind::Int => val,
                            ValKind::Float => {
                                bld!(self.builder.build_float_to_signed_int(
                                    val.into_float_value(), self.context.i64_type(), "ftoi"
                                ))?.into()
                            }
                            ValKind::Bool => {
                                bld!(self.builder.build_int_z_extend(
                                    val.into_int_value(), self.context.i64_type(), "btoi"
                                ))?.into()
                            }
                            ValKind::Str => {
                                let rt = self.module.get_function("ore_str_to_int").unwrap();
                                let r = bld!(self.builder.build_call(rt, &[val.into()], "stoi"))?;
                                self.call_result_to_value(r)?
                            }
                            _ => val,
                        };
                        return Ok((result, ValKind::Int));
                    }
                    "float" => {
                        if args.len() != 1 {
                            return Err(CodeGenError { line: None, msg: "float() takes 1 argument".into() });
                        }
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        let result = match kind {
                            ValKind::Float => val,
                            ValKind::Int => {
                                bld!(self.builder.build_signed_int_to_float(
                                    val.into_int_value(), self.context.f64_type(), "itof"
                                ))?.into()
                            }
                            ValKind::Str => {
                                let rt = self.module.get_function("ore_str_to_float").unwrap();
                                let r = bld!(self.builder.build_call(rt, &[val.into()], "stof"))?;
                                self.call_result_to_value(r)?
                            }
                            _ => val,
                        };
                        return Ok((result, ValKind::Float));
                    }
                    "str" => {
                        if args.len() != 1 {
                            return Err(CodeGenError { line: None, msg: "str() takes 1 argument".into() });
                        }
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        let str_ptr = self.value_to_str(val, kind)?;
                        return Ok((str_ptr.into(), ValKind::Str));
                    }
                    "len" => {
                        if args.len() != 1 {
                            return Err(CodeGenError { line: None, msg: "len() takes 1 argument".into() });
                        }
                        let (val, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        match kind {
                            ValKind::Str => {
                                let rt = self.module.get_function("ore_str_len").unwrap();
                                let result = bld!(self.builder.build_call(rt, &[val.into()], "slen"))?;
                                return Ok((self.call_result_to_value(result)?, ValKind::Int));
                            }
                            ValKind::List => {
                                let rt = self.module.get_function("ore_list_len").unwrap();
                                let result = bld!(self.builder.build_call(rt, &[val.into()], "llen"))?;
                                return Ok((self.call_result_to_value(result)?, ValKind::Int));
                            }
                            ValKind::Map => {
                                let rt = self.module.get_function("ore_map_len").unwrap();
                                let result = bld!(self.builder.build_call(rt, &[val.into()], "mlen"))?;
                                return Ok((self.call_result_to_value(result)?, ValKind::Int));
                            }
                            _ => return Err(CodeGenError { line: None, msg: "len() not supported on this type".into() }),
                        }
                    }
                    "assert" => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(CodeGenError { line: None, msg: "assert takes 1-2 arguments (condition, optional message)".into() });
                        }
                        let (cond, _) = self.compile_expr_with_kind(&args[0], func)?;
                        let cond_bool = cond.into_int_value();

                        let pass_bb = self.context.append_basic_block(func, "assert_pass");
                        let fail_bb = self.context.append_basic_block(func, "assert_fail");
                        bld!(self.builder.build_conditional_branch(cond_bool, pass_bb, fail_bb))?;

                        self.builder.position_at_end(fail_bb);
                        let msg = if args.len() == 2 {
                            self.compile_expr(&args[1], func)?.into_pointer_value()
                        } else {
                            let line = self.current_line;
                            self.compile_string_literal(&format!("assertion failed at line {}", line))?
                        };
                        let rt = self.module.get_function("ore_assert_fail").unwrap();
                        bld!(self.builder.build_call(rt, &[msg.into()], ""))?;
                        bld!(self.builder.build_unreachable())?;

                        self.builder.position_at_end(pass_bb);
                        return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void));
                    }
                    "typeof" => {
                        if args.len() != 1 {
                            return Err(CodeGenError { line: None, msg: "typeof takes 1 argument".into() });
                        }
                        let (_, kind) = self.compile_expr_with_kind(&args[0], func)?;
                        let type_name = match kind {
                            ValKind::Int => "Int",
                            ValKind::Float => "Float",
                            ValKind::Bool => "Bool",
                            ValKind::Str => "Str",
                            ValKind::List => "List",
                            ValKind::Map => "Map",
                            ValKind::Option => "Option",
                            ValKind::Result => "Result",
                            ValKind::Void => "Void",
                            ValKind::Record(ref n) => n.as_str(),
                            ValKind::Enum(ref n) => n.as_str(),
                            ValKind::Channel => "Channel",
                        };
                        let str_val = self.compile_string_literal(type_name)?;
                        return Ok((str_val.into(), ValKind::Str));
                    }
                    _ => {}
                }

                // Check if this is a variant construction (e.g. Red() or Circle(radius: 5.0))
                if self.variant_to_enum.contains_key(&name) {
                    // Treat as RecordConstruct with variant name
                    let construct = Expr::RecordConstruct {
                        type_name: name.clone(),
                        fields: vec![], // Zero-field variant
                    };
                    return self.compile_expr_with_kind(&construct, func);
                }

                // Try resolving as a named function first, or monomorphize generic
                let resolved = match self.resolve_function(&name) {
                    Ok(fk) => Some(fk),
                    Err(_) if self.generic_fns.contains_key(&name) => {
                        // Compile args to determine their kinds for monomorphization
                        let mut compiled_args = Vec::new();
                        let mut arg_kinds = Vec::new();
                        for arg in args {
                            let (val, kind) = self.compile_expr_with_kind(arg, func)?;
                            compiled_args.push(val.into());
                            arg_kinds.push(kind);
                        }
                        let (called_fn, ret_kind) = self.monomorphize(&name, &arg_kinds, func)?;
                        let result = bld!(self.builder.build_call(called_fn, &compiled_args, "call"))?;
                        let val = self.call_result_to_value(result)?;
                        return Ok((val, ret_kind));
                    }
                    Err(_) => None,
                };

                if let Some((called_fn, ret_kind)) = resolved {
                    let mut compiled_args = Vec::new();
                    for arg in args {
                        compiled_args.push(self.compile_expr(arg, func)?.into());
                    }
                    let result = bld!(self.builder.build_call(called_fn, &compiled_args, "call"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ret_kind))
                } else {
                    // Check if it's a variable holding a function pointer (closure)
                    if let Some((ptr, _ty, _kind, _mutable)) = self.variables.get(&name).cloned() {
                        let fn_ptr_val = bld!(self.builder.build_load(self.ptr_type(), ptr, "fn_ptr"))?;
                        let fn_ptr = fn_ptr_val.into_pointer_value();

                        // Check for closure (env_ptr stored alongside)
                        let env_var_name = format!("{}_env", name);
                        let has_env = self.variables.contains_key(&env_var_name);

                        let mut compiled_args = Vec::new();
                        for arg in args {
                            compiled_args.push(self.compile_expr(arg, func)?.into());
                        }

                        if has_env {
                            let (env_ptr, _, _, _) = self.variables[&env_var_name].clone();
                            let env_val = bld!(self.builder.build_load(self.ptr_type(), env_ptr, "env"))?;
                            let mut all_args = vec![env_val.into()];
                            all_args.extend(compiled_args);

                            let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![self.ptr_type().into()];
                            for _ in &all_args[1..] {
                                param_types.push(self.context.i64_type().into());
                            }
                            let fn_type = self.context.i64_type().fn_type(&param_types, false);
                            let result = bld!(self.builder.build_indirect_call(fn_type, fn_ptr, &all_args, "closurecall"))?;
                            let val = self.call_result_to_value(result)?;
                            Ok((val, ValKind::Int))
                        } else {
                            let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = compiled_args.iter().map(|_| self.context.i64_type().into()).collect();
                            let fn_type = self.context.i64_type().fn_type(&param_types, false);
                            let result = bld!(self.builder.build_indirect_call(fn_type, fn_ptr, &compiled_args, "fncall"))?;
                            let val = self.call_result_to_value(result)?;
                            Ok((val, ValKind::Int))
                        }
                    } else {
                        Err(CodeGenError { line: None, msg: format!("undefined function '{}'", name) })
                    }
                }
            }
            Expr::IfElse { cond, then_block, else_block } => {
                self.compile_if_else_with_kind(cond, then_block, else_block.as_ref(), func)
            }
            Expr::ColonMatch { cond, then_expr, else_expr } => {
                self.compile_colon_match_with_kind(cond, then_expr, else_expr.as_deref(), func)
            }
            Expr::RecordConstruct { type_name, fields } => {
                // Check if this is actually an enum variant construction
                if self.variant_to_enum.contains_key(type_name) {
                    return self.compile_variant_construct(type_name, fields, func);
                }
                self.compile_record_construct(type_name, fields, func)
            }
            Expr::Match { subject, arms } => {
                self.compile_match(subject, arms, func)
            }
            Expr::FieldAccess { object, field } => {
                self.compile_field_access(object, field, func)
            }
            Expr::MethodCall { object, method, args } => {
                self.compile_method_call(object, method, args, func)
            }
            Expr::ListLit(elements) => {
                self.compile_list_lit(elements, func)
            }
            Expr::MapLit(entries) => {
                self.compile_map_lit(entries, func)
            }
            Expr::Index { object, index } => {
                self.compile_index(object, index, func)
            }
            Expr::OptionNone => {
                let opt_ty = self.option_type();
                let alloca = bld!(self.builder.build_alloca(opt_ty, "opt_none"))?;
                let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
                bld!(self.builder.build_store(tag_ptr, self.context.i8_type().const_int(0, false)))?;
                let result = bld!(self.builder.build_load(opt_ty, alloca, "none_val"))?;
                Ok((result, ValKind::Option))
            }
            Expr::OptionSome(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                let opt_ty = self.option_type();
                let alloca = bld!(self.builder.build_alloca(opt_ty, "opt_some"))?;
                let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
                bld!(self.builder.build_store(tag_ptr, self.context.i8_type().const_int(1, false)))?;
                let kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 1, "kind_ptr"))?;
                let kind_tag = self.valkind_to_tag(&kind);
                bld!(self.builder.build_store(kind_ptr, self.context.i8_type().const_int(kind_tag as u64, false)))?;
                let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 2, "val_ptr"))?;
                let i64_val = self.value_to_i64(val)?;
                bld!(self.builder.build_store(val_ptr, i64_val))?;
                let result = bld!(self.builder.build_load(opt_ty, alloca, "some_val"))?;
                Ok((result, ValKind::Option))
            }
            Expr::ResultOk(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                let res_ty = self.result_type();
                let alloca = bld!(self.builder.build_alloca(res_ty, "res_ok"))?;
                let tag_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 0, "tag_ptr"))?;
                bld!(self.builder.build_store(tag_ptr, self.context.i8_type().const_int(0, false)))?;
                let kind_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 1, "kind_ptr"))?;
                let kind_tag = self.valkind_to_tag(&kind);
                bld!(self.builder.build_store(kind_ptr, self.context.i8_type().const_int(kind_tag as u64, false)))?;
                let val_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 2, "val_ptr"))?;
                let i64_val = self.value_to_i64(val)?;
                bld!(self.builder.build_store(val_ptr, i64_val))?;
                let result = bld!(self.builder.build_load(res_ty, alloca, "ok_val"))?;
                Ok((result, ValKind::Result))
            }
            Expr::ResultErr(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                let res_ty = self.result_type();
                let alloca = bld!(self.builder.build_alloca(res_ty, "res_err"))?;
                let tag_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 0, "tag_ptr"))?;
                bld!(self.builder.build_store(tag_ptr, self.context.i8_type().const_int(1, false)))?;
                let kind_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 1, "kind_ptr"))?;
                let kind_tag = self.valkind_to_tag(&kind);
                bld!(self.builder.build_store(kind_ptr, self.context.i8_type().const_int(kind_tag as u64, false)))?;
                let val_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 2, "val_ptr"))?;
                let i64_val = self.value_to_i64(val)?;
                bld!(self.builder.build_store(val_ptr, i64_val))?;
                let result = bld!(self.builder.build_load(res_ty, alloca, "err_val"))?;
                Ok((result, ValKind::Result))
            }
            Expr::Try(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
                if kind == ValKind::Result {
                    return self.compile_try_result(val, func);
                }
                let opt_ty = self.option_type();
                // Store the option value so we can extract from it
                let alloca = bld!(self.builder.build_alloca(opt_ty, "try_opt"))?;
                bld!(self.builder.build_store(alloca, val))?;
                // Load tag
                let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
                let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
                let is_none = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(0, false), "is_none"
                ))?;
                let some_bb = self.context.append_basic_block(func, "try_some");
                let none_bb = self.context.append_basic_block(func, "try_none");
                bld!(self.builder.build_conditional_branch(is_none, none_bb, some_bb))?;
                // None branch: return None from current function
                self.builder.position_at_end(none_bb);
                let none_alloca = bld!(self.builder.build_alloca(opt_ty, "ret_none"))?;
                let none_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, none_alloca, 0, "ret_tag"))?;
                bld!(self.builder.build_store(none_tag_ptr, self.context.i8_type().const_int(0, false)))?;
                let none_ret = bld!(self.builder.build_load(opt_ty, none_alloca, "none_ret"))?;
                bld!(self.builder.build_return(Some(&none_ret)))?;
                // Some branch: extract value
                self.builder.position_at_end(some_bb);
                let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 2, "val_ptr"))?;
                let extracted = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "unwrapped"))?;
                Ok((extracted, ValKind::Int))
            }
            Expr::OptionalChain { object, field } => {
                self.compile_optional_chain(object, field, func)
            }
            Expr::OptionalMethodCall { object, method, args } => {
                self.compile_optional_method_call(object, method, args, func)
            }
            Expr::Break => {
                if let Some(target) = self.break_target {
                    bld!(self.builder.build_unconditional_branch(target))?;
                } else {
                    return Err(CodeGenError { line: None, msg: "break outside of loop".into() });
                }
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
        }
    }

    fn compile_record_construct(
        &mut self,
        type_name: &str,
        fields: &[(String, Expr)],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let info = self.records.get(type_name).ok_or_else(|| CodeGenError {
            line: None, msg: format!("undefined type '{}'", type_name),
        })?;
        let struct_type = info.struct_type;
        let field_names = info.field_names.clone();

        let alloca = bld!(self.builder.build_alloca(struct_type, type_name))?;

        for (name, expr) in fields {
            let idx = field_names.iter().position(|n| n == name).ok_or_else(|| CodeGenError {
                line: None, msg: format!("unknown field '{}' on type '{}'", name, type_name),
            })?;
            let val = self.compile_expr(expr, func)?;
            let field_ptr = bld!(self.builder.build_struct_gep(struct_type, alloca, idx as u32, &format!("{}.{}", type_name, name)))?;
            bld!(self.builder.build_store(field_ptr, val))?;
        }

        let result = bld!(self.builder.build_load(struct_type, alloca, "record"))?;
        Ok((result, ValKind::Record(type_name.to_string())))
    }

    fn compile_field_access(
        &mut self,
        object: &Expr,
        field: &str,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        let type_name = match &obj_kind {
            ValKind::Record(name) => name.clone(),
            _ => return Err(CodeGenError { line: None, msg: "field access on non-record type".into() }),
        };

        let info = self.records.get(&type_name).ok_or_else(|| CodeGenError {
            line: None, msg: format!("undefined type '{}'", type_name),
        })?;
        let struct_type = info.struct_type;
        let idx = info.field_names.iter().position(|n| n == field).ok_or_else(|| CodeGenError {
            line: None, msg: format!("unknown field '{}' on type '{}'", field, type_name),
        })?;
        let field_kind = info.field_kinds[idx].clone();

        // Store the struct to an alloca so we can GEP into it
        let alloca = bld!(self.builder.build_alloca(struct_type, "tmp"))?;
        bld!(self.builder.build_store(alloca, obj_val))?;
        let field_ty = self.kind_to_llvm_type(&field_kind);
        let field_ptr = bld!(self.builder.build_struct_gep(struct_type, alloca, idx as u32, field))?;
        let result = bld!(self.builder.build_load(field_ty, field_ptr, field))?;
        Ok((result, field_kind))
    }

    fn compile_optional_chain(
        &mut self,
        object: &Expr,
        field: &str,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        if obj_kind != ValKind::Option {
            return Err(CodeGenError { line: None, msg: "?. operator requires an Option value".into() });
        }

        let opt_ty = self.option_type();
        let alloca = bld!(self.builder.build_alloca(opt_ty, "optchain"))?;
        bld!(self.builder.build_store(alloca, obj_val))?;

        let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
        let is_some = bld!(self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
        ))?;

        let some_bb = self.context.append_basic_block(func, "optchain_some");
        let none_bb = self.context.append_basic_block(func, "optchain_none");
        let merge_bb = self.context.append_basic_block(func, "optchain_merge");

        bld!(self.builder.build_conditional_branch(is_some, some_bb, none_bb))?;

        // Some branch: unwrap, field access, wrap in Some
        self.builder.position_at_end(some_bb);
        let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 2, "val_ptr"))?;
        let inner_i64 = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?.into_int_value();

        // Perform field access on the inner value
        let inner_expr = Expr::FieldAccess {
            object: Box::new(object.clone()),
            field: field.to_string(),
        };
        // Instead, use the inner value directly - reinterpret as the record type
        // For simplicity, wrap the result in Some
        let kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 1, "kind_ptr"))?;
        let inner_kind_tag = bld!(self.builder.build_load(self.context.i8_type(), kind_ptr, "ikind"))?.into_int_value();
        let _ = inner_kind_tag;
        let _ = inner_expr;

        // Build a new Some option with the field value
        // For now, we just pass through the i64 payload as the field result
        // This works for record fields stored as i64
        let result_alloca = bld!(self.builder.build_alloca(opt_ty, "optres"))?;
        let res_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_alloca, 0, "res_tag"))?;
        bld!(self.builder.build_store(res_tag_ptr, self.context.i8_type().const_int(1, false)))?;
        let res_kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_alloca, 1, "res_kind"))?;
        // Store Int kind for now (we don't know the actual kind of the field)
        bld!(self.builder.build_store(res_kind_ptr, self.context.i8_type().const_int(0, false)))?;
        let res_val_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_alloca, 2, "res_val"))?;
        bld!(self.builder.build_store(res_val_ptr, inner_i64))?;
        let some_result = bld!(self.builder.build_load(opt_ty, result_alloca, "some_res"))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;

        // None branch: return None
        self.builder.position_at_end(none_bb);
        let none_alloca = bld!(self.builder.build_alloca(opt_ty, "none_res"))?;
        let none_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, none_alloca, 0, "none_tag"))?;
        bld!(self.builder.build_store(none_tag_ptr, self.context.i8_type().const_int(0, false)))?;
        let none_result = bld!(self.builder.build_load(opt_ty, none_alloca, "none_res"))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;

        self.builder.position_at_end(merge_bb);
        let phi = bld!(self.builder.build_phi(opt_ty, "optchain_result"))?;
        phi.add_incoming(&[(&some_result, some_bb), (&none_result, none_bb)]);

        Ok((phi.as_basic_value(), ValKind::Option))
    }

    fn compile_optional_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        if obj_kind != ValKind::Option {
            return Err(CodeGenError { line: None, msg: "?. operator requires an Option value".into() });
        }

        let opt_ty = self.option_type();
        let alloca = bld!(self.builder.build_alloca(opt_ty, "optmethod"))?;
        bld!(self.builder.build_store(alloca, obj_val))?;

        let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
        let is_some = bld!(self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
        ))?;

        let some_bb = self.context.append_basic_block(func, "optmethod_some");
        let none_bb = self.context.append_basic_block(func, "optmethod_none");
        let merge_bb = self.context.append_basic_block(func, "optmethod_merge");

        bld!(self.builder.build_conditional_branch(is_some, some_bb, none_bb))?;

        // Some branch: unwrap, call method, wrap result in Some
        self.builder.position_at_end(some_bb);
        let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 2, "val_ptr"))?;
        let inner_val = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?;
        let kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 1, "kind_ptr"))?;
        let inner_kind_tag = bld!(self.builder.build_load(self.context.i8_type(), kind_ptr, "ikind"))?.into_int_value();

        // Determine inner ValKind from tag and call method on the inner value
        // For now, try calling method on inner as Int (most common case)
        let _ = inner_kind_tag;
        let inner_kind = ValKind::Int;
        let (result_val, result_kind) = self.call_method_on_value(inner_val, &inner_kind, method, args, func)?;

        // Wrap result in Some
        let result_opt_alloca = bld!(self.builder.build_alloca(opt_ty, "optres"))?;
        let res_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_opt_alloca, 0, "res_tag"))?;
        bld!(self.builder.build_store(res_tag_ptr, self.context.i8_type().const_int(1, false)))?;
        let res_kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_opt_alloca, 1, "res_kind"))?;
        let rk_tag = self.valkind_to_tag(&result_kind);
        bld!(self.builder.build_store(res_kind_ptr, self.context.i8_type().const_int(rk_tag as u64, false)))?;
        let res_val_ptr = bld!(self.builder.build_struct_gep(opt_ty, result_opt_alloca, 2, "res_val"))?;
        let result_i64 = self.value_to_i64(result_val)?;
        bld!(self.builder.build_store(res_val_ptr, result_i64))?;
        let some_result = bld!(self.builder.build_load(opt_ty, result_opt_alloca, "some_res"))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;
        let some_end = self.builder.get_insert_block().unwrap();

        // None branch
        self.builder.position_at_end(none_bb);
        let none_alloca = bld!(self.builder.build_alloca(opt_ty, "none_res"))?;
        let none_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, none_alloca, 0, "none_tag"))?;
        bld!(self.builder.build_store(none_tag_ptr, self.context.i8_type().const_int(0, false)))?;
        let none_result = bld!(self.builder.build_load(opt_ty, none_alloca, "none_res"))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;

        self.builder.position_at_end(merge_bb);
        let phi = bld!(self.builder.build_phi(opt_ty, "optmethod_result"))?;
        phi.add_incoming(&[(&some_result, some_end), (&none_result, none_bb)]);

        Ok((phi.as_basic_value(), ValKind::Option))
    }

    fn call_method_on_value(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: &ValKind,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        // Dispatch to appropriate method handler based on kind
        match kind {
            ValKind::Str => self.compile_str_method(val.into_pointer_value().into(), method, args, func),
            ValKind::List => self.compile_list_method(val.into_pointer_value().into(), method, args, func),
            ValKind::Int => {
                match method {
                    "abs" => {
                        let i = val.into_int_value();
                        let neg = bld!(self.builder.build_int_neg(i, "neg"))?;
                        let is_neg = bld!(self.builder.build_int_compare(
                            IntPredicate::SLT, i, self.context.i64_type().const_int(0, false), "is_neg"
                        ))?;
                        let result = bld!(self.builder.build_select(is_neg, neg, i, "abs"))?;
                        Ok((result, ValKind::Int))
                    }
                    "to_float" => {
                        let f = bld!(self.builder.build_signed_int_to_float(
                            val.into_int_value(), self.context.f64_type(), "itof"
                        ))?;
                        Ok((f.into(), ValKind::Float))
                    }
                    _ => Err(CodeGenError { line: None, msg: format!("unknown method '{}' on Int", method) }),
                }
            }
            _ => Err(CodeGenError { line: None, msg: format!("cannot call method '{}' on {:?} in optional chain", method, kind) }),
        }
    }

    fn compile_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;

        // Handle list built-in methods
        if obj_kind == ValKind::List {
            return self.compile_list_method(obj_val, method, args, func);
        }

        // Handle string built-in methods
        if obj_kind == ValKind::Str {
            return self.compile_str_method(obj_val, method, args, func);
        }

        // Handle map built-in methods
        if obj_kind == ValKind::Map {
            return self.compile_map_method(obj_val, method, args, func);
        }

        // Handle Option methods
        if obj_kind == ValKind::Option {
            return self.compile_option_method(obj_val, method, args, func);
        }

        // Handle Channel methods
        if obj_kind == ValKind::Channel {
            return self.compile_channel_method(obj_val, method, args, func);
        }

        // Handle to_str() on primitive types
        if method == "to_str" {
            let str_val = self.value_to_str(obj_val, obj_kind)?;
            return Ok((str_val.into(), ValKind::Str));
        }

        // Int methods
        if obj_kind == ValKind::Int {
            match method {
                "to_float" => {
                    let f_val = bld!(self.builder.build_signed_int_to_float(
                        obj_val.into_int_value(),
                        self.context.f64_type(),
                        "i2f"
                    ))?;
                    return Ok((f_val.into(), ValKind::Float));
                }
                "abs" => {
                    let int_val = obj_val.into_int_value();
                    let zero = self.context.i64_type().const_zero();
                    let is_neg = bld!(self.builder.build_int_compare(
                        inkwell::IntPredicate::SLT, int_val, zero, "is_neg"
                    ))?;
                    let neg_val = bld!(self.builder.build_int_neg(int_val, "neg"))?;
                    let result = bld!(self.builder.build_select(is_neg, neg_val, int_val, "abs"))?;
                    return Ok((result, ValKind::Int));
                }
                "max" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: None, msg: "Int.max() takes 1 argument".into() });
                    }
                    let (other, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let a = obj_val.into_int_value();
                    let b = other.into_int_value();
                    let cmp = bld!(self.builder.build_int_compare(IntPredicate::SGT, a, b, "gt"))?;
                    let result = bld!(self.builder.build_select(cmp, a, b, "max"))?;
                    return Ok((result, ValKind::Int));
                }
                "min" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: None, msg: "Int.min() takes 1 argument".into() });
                    }
                    let (other, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let a = obj_val.into_int_value();
                    let b = other.into_int_value();
                    let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, a, b, "lt"))?;
                    let result = bld!(self.builder.build_select(cmp, a, b, "min"))?;
                    return Ok((result, ValKind::Int));
                }
                "clamp" => {
                    if args.len() != 2 {
                        return Err(CodeGenError { line: None, msg: "Int.clamp() takes 2 arguments (min, max)".into() });
                    }
                    let (lo_val, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let (hi_val, _) = self.compile_expr_with_kind(&args[1], func)?;
                    let x = obj_val.into_int_value();
                    let lo = lo_val.into_int_value();
                    let hi = hi_val.into_int_value();
                    let cmp_lo = bld!(self.builder.build_int_compare(IntPredicate::SLT, x, lo, "lt_lo"))?;
                    let v1 = bld!(self.builder.build_select(cmp_lo, lo, x, "clamp_lo"))?;
                    let cmp_hi = bld!(self.builder.build_int_compare(IntPredicate::SGT, v1.into_int_value(), hi, "gt_hi"))?;
                    let result = bld!(self.builder.build_select(cmp_hi, hi, v1.into_int_value(), "clamp"))?;
                    return Ok((result, ValKind::Int));
                }
                "pow" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: None, msg: "Int.pow() takes 1 argument".into() });
                    }
                    let (exp_val, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let rt = self.module.get_function("ore_int_pow").unwrap();
                    let result = bld!(self.builder.build_call(rt, &[obj_val.into(), exp_val.into()], "pow"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Int));
                }
                _ => return Err(CodeGenError { line: None, msg: format!("unknown Int method '{}'", method) }),
            }
        }

        // Float methods
        if obj_kind == ValKind::Float {
            match method {
                "to_int" => {
                    let i_val = bld!(self.builder.build_float_to_signed_int(
                        obj_val.into_float_value(),
                        self.context.i64_type(),
                        "f2i"
                    ))?;
                    return Ok((i_val.into(), ValKind::Int));
                }
                "round" => {
                    let round_fn = self.module.get_function("llvm.round.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.round.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(round_fn, &[obj_val.into()], "round"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "floor" => {
                    let floor_fn = self.module.get_function("llvm.floor.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.floor.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(floor_fn, &[obj_val.into()], "floor"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "ceil" => {
                    let ceil_fn = self.module.get_function("llvm.ceil.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.ceil.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(ceil_fn, &[obj_val.into()], "ceil"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "abs" => {
                    let abs_fn = self.module.get_function("llvm.fabs.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.fabs.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(abs_fn, &[obj_val.into()], "fabs"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "sqrt" => {
                    let sqrt_fn = self.module.get_function("llvm.sqrt.f64").unwrap_or_else(|| {
                        let f64_type = self.context.f64_type();
                        self.module.add_function(
                            "llvm.sqrt.f64",
                            f64_type.fn_type(&[f64_type.into()], false),
                            None,
                        )
                    });
                    let result = bld!(self.builder.build_call(sqrt_fn, &[obj_val.into()], "sqrt"))?;
                    let val = self.call_result_to_value(result)?;
                    return Ok((val, ValKind::Float));
                }
                "max" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: None, msg: "Float.max() takes 1 argument".into() });
                    }
                    let (other, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let a = obj_val.into_float_value();
                    let b = other.into_float_value();
                    let cmp = bld!(self.builder.build_float_compare(inkwell::FloatPredicate::OGT, a, b, "fgt"))?;
                    let result = bld!(self.builder.build_select(cmp, a, b, "fmax"))?;
                    return Ok((result, ValKind::Float));
                }
                "min" => {
                    if args.len() != 1 {
                        return Err(CodeGenError { line: None, msg: "Float.min() takes 1 argument".into() });
                    }
                    let (other, _) = self.compile_expr_with_kind(&args[0], func)?;
                    let a = obj_val.into_float_value();
                    let b = other.into_float_value();
                    let cmp = bld!(self.builder.build_float_compare(inkwell::FloatPredicate::OLT, a, b, "flt"))?;
                    let result = bld!(self.builder.build_select(cmp, a, b, "fmin"))?;
                    return Ok((result, ValKind::Float));
                }
                _ => return Err(CodeGenError { line: None, msg: format!("unknown Float method '{}'", method) }),
            }
        }

        let type_name = match &obj_kind {
            ValKind::Record(name) => name.clone(),
            _ => return Err(CodeGenError { line: None, msg: format!("method call on unsupported type: {:?}", obj_kind) }),
        };

        // Look up the mangled function name
        let mangled_name = format!("{}_{}", type_name, method);
        let (called_fn, ret_kind) = self.resolve_function(&mangled_name)?;

        // Build args: object as first arg, then the rest
        let mut compiled_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = Vec::new();
        compiled_args.push(obj_val.into());
        for arg in args {
            compiled_args.push(self.compile_expr(arg, func)?.into());
        }

        let result = bld!(self.builder.build_call(called_fn, &compiled_args, "mcall"))?;
        let val = self.call_result_to_value(result)?;
        Ok((val, ret_kind))
    }

    fn compile_list_method(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "len" => {
                let list_len = self.module.get_function("ore_list_len").unwrap();
                let result = bld!(self.builder.build_call(list_len, &[list_val.into()], "len"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "push" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "push takes exactly 1 argument".into() });
                }
                let arg = self.compile_expr(&args[0], func)?;
                let list_push = self.module.get_function("ore_list_push").unwrap();
                bld!(self.builder.build_call(list_push, &[list_val.into(), arg.into()], ""))?;
                Ok((list_val, ValKind::List))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "get takes exactly 1 argument".into() });
                }
                let idx = self.compile_expr(&args[0], func)?;
                let list_get = self.module.get_function("ore_list_get").unwrap();
                let result = bld!(self.builder.build_call(list_get, &[list_val.into(), idx.into()], "get"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "map" | "filter" | "flat_map" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: format!("{} takes exactly 1 argument", method) });
                }
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: None, msg: format!("{} argument must be a function", method) }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let runtime_fn_name = format!("ore_list_{}", method);
                let runtime_fn = self.module.get_function(&runtime_fn_name).unwrap();
                let result = bld!(self.builder.build_call(
                    runtime_fn,
                    &[list_val.into(), fn_ptr.into(), env_ptr.into()],
                    method
                ))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List))
            }
            "each" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "each takes exactly 1 argument".into() });
                }
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: None, msg: "each argument must be a function".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let runtime_fn = self.module.get_function("ore_list_each").unwrap();
                bld!(self.builder.build_call(
                    runtime_fn,
                    &[list_val.into(), fn_ptr.into(), env_ptr.into()],
                    ""
                ))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            "par_map" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "par_map takes exactly 1 argument".into() });
                }
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => self.compile_lambda(params, body, func)?,
                    Expr::Ident(name) => self.resolve_function(name)?.0,
                    _ => return Err(CodeGenError { line: None, msg: "par_map argument must be a function".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let rt = self.module.get_function("ore_list_par_map").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], "par_map"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List))
            }
            "par_each" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "par_each takes exactly 1 argument".into() });
                }
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => self.compile_lambda(params, body, func)?,
                    Expr::Ident(name) => self.resolve_function(name)?.0,
                    _ => return Err(CodeGenError { line: None, msg: "par_each argument must be a function".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let rt = self.module.get_function("ore_list_par_each").unwrap();
                bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], ""))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            "sort" => {
                if args.is_empty() {
                    let rt = self.module.get_function("ore_list_sort").unwrap();
                    bld!(self.builder.build_call(rt, &[list_val.into()], ""))?;
                    return Ok((list_val, ValKind::List));
                }
                // sort(comparator) - sort_by
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => self.compile_lambda(params, body, func)?,
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: None, msg: "sort requires a comparator function".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let rt = self.module.get_function("ore_list_sort_by").unwrap();
                bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], ""))?;
                Ok((list_val, ValKind::List))
            }
            "reverse" => {
                let rt = self.module.get_function("ore_list_reverse").unwrap();
                bld!(self.builder.build_call(rt, &[list_val.into()], ""))?;
                Ok((list_val, ValKind::List))
            }
            "contains" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "contains takes exactly 1 argument".into() });
                }
                let val = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_list_contains").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), val.into()], "lcontains"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    i8_val,
                    self.context.i8_type().const_int(0, false),
                    "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "reduce" => {
                // reduce(init, fn(acc, elem) -> acc)
                if args.len() != 2 {
                    return Err(CodeGenError { line: None, msg: "reduce takes 2 arguments (init, fn)".into() });
                }
                let init_val = self.compile_expr(&args[0], func)?;
                let lambda_fn = match &args[1] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: None, msg: "reduce second argument must be a function".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let rt = self.module.get_function("ore_list_reduce").unwrap();
                let result = bld!(self.builder.build_call(
                    rt, &[list_val.into(), init_val.into(), fn_ptr.into(), env_ptr.into()], "reduce"
                ))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "find" => {
                // find(fn(elem) -> bool) — returns element or 0
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "find takes 1 argument".into() });
                }
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: None, msg: "find argument must be a function".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let default_val = self.context.i64_type().const_int(0, false);
                let rt = self.module.get_function("ore_list_find").unwrap();
                let result = bld!(self.builder.build_call(
                    rt, &[list_val.into(), fn_ptr.into(), env_ptr.into(), default_val.into()], "find"
                ))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "join" => {
                // join(separator_str)
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "join takes 1 argument (separator)".into() });
                }
                let sep = self.compile_expr(&args[0], func)?;
                // Use join_str for string lists, join for int lists
                let elem_kind = self.last_list_elem_kind.clone();
                let fn_name = if matches!(elem_kind, Some(ValKind::Str)) {
                    "ore_list_join_str"
                } else {
                    "ore_list_join"
                };
                let rt = self.module.get_function(fn_name).unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), sep.into()], "join"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "take" | "skip" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: format!("{} takes 1 argument (count)", method) });
                }
                let n = self.compile_expr(&args[0], func)?;
                let runtime_fn_name = format!("ore_list_{}", method);
                let rt = self.module.get_function(&runtime_fn_name).unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), n.into()], method))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List))
            }
            "sum" => {
                let rt = self.module.get_function("ore_list_sum").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "sum"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "any" | "all" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: format!("{} takes 1 argument (predicate)", method) });
                }
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: None, msg: format!("{} argument must be a function", method) }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let runtime_fn_name = format!("ore_list_{}", method);
                let rt = self.module.get_function(&runtime_fn_name).unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), fn_ptr.into(), env_ptr.into()], method))?;
                let val = self.call_result_to_value(result)?;
                let bool_val = bld!(self.builder.build_int_truncate(val.into_int_value(), self.context.bool_type(), &format!("{}_bool", method)))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "zip" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "zip takes 1 argument (other list)".into() });
                }
                let other = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_list_zip").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), other.into()], "zip"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List))
            }
            "enumerate" => {
                let rt = self.module.get_function("ore_list_enumerate").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "enum"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List))
            }
            "slice" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: None, msg: "slice takes 2 arguments (start, end)".into() });
                }
                let start = self.compile_expr(&args[0], func)?;
                let end = self.compile_expr(&args[1], func)?;
                let rt = self.module.get_function("ore_list_slice").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), start.into(), end.into()], "lslice"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List))
            }
            "index_of" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "index_of takes 1 argument".into() });
                }
                let val = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_list_index_of").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), val.into()], "lidx"))?;
                let v = self.call_result_to_value(result)?;
                Ok((v, ValKind::Int))
            }
            "unique" => {
                let rt = self.module.get_function("ore_list_unique").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "luniq"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List))
            }
            "flatten" => {
                let rt = self.module.get_function("ore_list_flatten").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lflat"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List))
            }
            "first" => {
                let rt = self.module.get_function("ore_list_get").unwrap();
                let zero = self.context.i64_type().const_int(0, false);
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), zero.into()], "first"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "last" => {
                let rt = self.module.get_function("ore_list_get").unwrap();
                let neg_one = self.context.i64_type().const_int((-1i64) as u64, true);
                let result = bld!(self.builder.build_call(rt, &[list_val.into(), neg_one.into()], "last"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "min" => {
                let rt = self.module.get_function("ore_list_min").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lmin"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "max" => {
                let rt = self.module.get_function("ore_list_max").unwrap();
                let result = bld!(self.builder.build_call(rt, &[list_val.into()], "lmax"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "count" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "count takes 1 argument (predicate)".into() });
                }
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => self.compile_lambda(params, body, func)?,
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { line: None, msg: "count requires a function or lambda".into() }),
                };
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let env_ptr = if self.lambda_captures.contains_key(&lambda_name) {
                    self.build_captures_struct(&lambda_name)?
                } else {
                    self.context.ptr_type(inkwell::AddressSpace::default()).const_null()
                };
                let rt = self.module.get_function("ore_list_count").unwrap();
                let result = bld!(self.builder.build_call(
                    rt,
                    &[list_val.into(), fn_ptr.into(), env_ptr.into()],
                    "count"
                ))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            _ => Err(CodeGenError { line: None, msg: format!("unknown list method '{}'", method) }),
        }
    }

    fn compile_str_method(
        &mut self,
        str_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "len" => {
                let rt = self.module.get_function("ore_str_len").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "slen"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "contains" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "contains takes 1 argument".into() });
                }
                let needle = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_str_contains").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), needle.into()], "scontains"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    i8_val,
                    self.context.i8_type().const_int(0, false),
                    "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "trim" => {
                let rt = self.module.get_function("ore_str_trim").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "strim"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "split" => {
                if args.is_empty() {
                    // split() with no args = split on whitespace
                    let rt = self.module.get_function("ore_str_split_whitespace").unwrap();
                    let result = bld!(self.builder.build_call(rt, &[str_val.into()], "ssplit"))?;
                    let val = self.call_result_to_value(result)?;
                    self.last_list_elem_kind = Some(ValKind::Str);
                    return Ok((val, ValKind::List));
                }
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "split takes 0 or 1 arguments".into() });
                }
                let delim = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_str_split").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), delim.into()], "ssplit"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok((val, ValKind::List))
            }
            "to_int" => {
                let rt = self.module.get_function("ore_str_to_int").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "stoi"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "to_float" => {
                let rt = self.module.get_function("ore_str_to_float").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "stof"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Float))
            }
            "replace" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: None, msg: "replace takes 2 arguments (from, to)".into() });
                }
                let from = self.compile_expr(&args[0], func)?;
                let to = self.compile_expr(&args[1], func)?;
                let rt = self.module.get_function("ore_str_replace").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), from.into(), to.into()], "sreplace"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "starts_with" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "starts_with takes 1 argument".into() });
                }
                let prefix = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_str_starts_with").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), prefix.into()], "ssw"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE, i8_val,
                    self.context.i8_type().const_int(0, false), "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "ends_with" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "ends_with takes 1 argument".into() });
                }
                let suffix = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_str_ends_with").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), suffix.into()], "sew"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE, i8_val,
                    self.context.i8_type().const_int(0, false), "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "to_upper" => {
                let rt = self.module.get_function("ore_str_to_upper").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "supper"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "to_lower" => {
                let rt = self.module.get_function("ore_str_to_lower").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "slower"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "substr" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: None, msg: "substr takes 2 arguments (start, len)".into() });
                }
                let start = self.compile_expr(&args[0], func)?;
                let len = self.compile_expr(&args[1], func)?;
                let rt = self.module.get_function("ore_str_substr").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), start.into(), len.into()], "ssub"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "chars" => {
                let rt = self.module.get_function("ore_str_chars").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "schars"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok((val, ValKind::List))
            }
            "index_of" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "index_of takes 1 argument".into() });
                }
                let needle = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_str_index_of").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), needle.into()], "sidx"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "slice" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: None, msg: "slice takes 2 arguments (start, end)".into() });
                }
                let start = self.compile_expr(&args[0], func)?;
                let end = self.compile_expr(&args[1], func)?;
                let rt = self.module.get_function("ore_str_slice").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), start.into(), end.into()], "sslice"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "reverse" => {
                let rt = self.module.get_function("ore_str_reverse").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "srev"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            "parse_int" => {
                let rt = self.module.get_function("ore_str_parse_int").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "parse_int"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "parse_float" => {
                let rt = self.module.get_function("ore_str_parse_float").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into()], "parse_float"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Float))
            }
            "repeat" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "repeat takes 1 argument".into() });
                }
                let count = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_str_repeat").unwrap();
                let result = bld!(self.builder.build_call(rt, &[str_val.into(), count.into()], "srep"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Str))
            }
            _ => Err(CodeGenError { line: None, msg: format!("unknown string method '{}'", method) }),
        }
    }

    fn compile_option_method(
        &mut self,
        opt_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let opt_ty = self.option_type();
        let alloca = bld!(self.builder.build_alloca(opt_ty, "opt_m"))?;
        bld!(self.builder.build_store(alloca, opt_val))?;

        let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
        let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 2, "val_ptr"))?;

        match method {
            "unwrap_or" => {
                // Returns inner value if Some, else the provided default
                if args.is_empty() {
                    return Err(CodeGenError { line: None, msg: "unwrap_or requires a default argument".into() });
                }
                let (default_val, default_kind) = self.compile_expr_with_kind(&args[0], func)?;
                let is_some = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
                ))?;
                let inner = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?;

                let some_bb = self.context.append_basic_block(func, "unwrap_some");
                let none_bb = self.context.append_basic_block(func, "unwrap_none");
                let merge_bb = self.context.append_basic_block(func, "unwrap_merge");

                bld!(self.builder.build_conditional_branch(is_some, some_bb, none_bb))?;

                self.builder.position_at_end(some_bb);
                let some_result = self.coerce_from_i64(inner, &default_kind)?;
                bld!(self.builder.build_unconditional_branch(merge_bb))?;

                self.builder.position_at_end(none_bb);
                bld!(self.builder.build_unconditional_branch(merge_bb))?;

                self.builder.position_at_end(merge_bb);
                let phi = bld!(self.builder.build_phi(some_result.get_type(), "unwrap_val"))?;
                phi.add_incoming(&[(&some_result, some_bb), (&default_val, none_bb)]);

                Ok((phi.as_basic_value(), default_kind))
            }
            "unwrap" => {
                // Just return inner value (unsafe - crashes on None in real use, but useful)
                let inner = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "unwrapped"))?;
                Ok((inner, ValKind::Int))
            }
            "is_some" => {
                let is_some = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
                ))?;
                Ok((is_some.into(), ValKind::Bool))
            }
            "is_none" => {
                let is_none = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(0, false), "is_none"
                ))?;
                Ok((is_none.into(), ValKind::Bool))
            }
            "map" => {
                // opt.map(fn) -> applies fn to inner value if Some, returns Option
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "map takes 1 argument (function)".into() });
                }
                let is_some = bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_some"
                ))?;
                let some_bb = self.context.append_basic_block(func, "optmap_some");
                let none_bb = self.context.append_basic_block(func, "optmap_none");
                let merge_bb = self.context.append_basic_block(func, "optmap_merge");
                bld!(self.builder.build_conditional_branch(is_some, some_bb, none_bb))?;

                // Some branch: unwrap, apply function, wrap result
                self.builder.position_at_end(some_bb);
                let inner = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "inner"))?;

                // Compile the lambda/function and call it with inner value
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        self.module.get_function(name).ok_or_else(|| CodeGenError {
                            line: None, msg: format!("unknown function '{}'", name),
                        })?
                    }
                    _ => return Err(CodeGenError { line: None, msg: "map requires a function or lambda".into() }),
                };

                let map_result = bld!(self.builder.build_call(lambda_fn, &[inner.into()], "mapped"))?;
                let mapped_val = self.call_result_to_value(map_result)?;

                // Wrap result in Some
                let opt_ty = self.option_type();
                let res_alloca = bld!(self.builder.build_alloca(opt_ty, "optres"))?;
                let res_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, res_alloca, 0, "res_tag"))?;
                bld!(self.builder.build_store(res_tag_ptr, self.context.i8_type().const_int(1, false)))?;
                let res_kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, res_alloca, 1, "res_kind"))?;
                bld!(self.builder.build_store(res_kind_ptr, self.context.i8_type().const_int(0, false)))?;
                let res_val_ptr = bld!(self.builder.build_struct_gep(opt_ty, res_alloca, 2, "res_val"))?;
                let mapped_i64 = self.value_to_i64(mapped_val)?;
                bld!(self.builder.build_store(res_val_ptr, mapped_i64))?;
                let some_result = bld!(self.builder.build_load(opt_ty, res_alloca, "some_res"))?;
                bld!(self.builder.build_unconditional_branch(merge_bb))?;
                let some_end = self.builder.get_insert_block().unwrap();

                // None branch
                self.builder.position_at_end(none_bb);
                let none_alloca = bld!(self.builder.build_alloca(opt_ty, "none_res"))?;
                let none_tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, none_alloca, 0, "none_tag"))?;
                bld!(self.builder.build_store(none_tag_ptr, self.context.i8_type().const_int(0, false)))?;
                let none_result = bld!(self.builder.build_load(opt_ty, none_alloca, "none_res"))?;
                bld!(self.builder.build_unconditional_branch(merge_bb))?;

                self.builder.position_at_end(merge_bb);
                let phi = bld!(self.builder.build_phi(opt_ty, "optmap_result"))?;
                phi.add_incoming(&[(&some_result, some_end), (&none_result, none_bb)]);
                Ok((phi.as_basic_value(), ValKind::Option))
            }
            _ => Err(CodeGenError { line: None, msg: format!("unknown method '{}' on Option", method) }),
        }
    }

    fn compile_map_method(
        &mut self,
        map_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "set" => {
                if args.len() != 2 {
                    return Err(CodeGenError { line: None, msg: "set takes 2 arguments (key, value)".into() });
                }
                let key = self.compile_expr(&args[0], func)?;
                let (val, val_kind) = self.compile_expr_with_kind(&args[1], func)?;
                let i64_val = match val_kind {
                    ValKind::Int => val.into_int_value(),
                    ValKind::Bool => {
                        bld!(self.builder.build_int_z_extend(
                            val.into_int_value(), self.context.i64_type(), "bool_to_i64"
                        ))?
                    }
                    ValKind::Str | ValKind::List | ValKind::Map => {
                        bld!(self.builder.build_ptr_to_int(
                            val.into_pointer_value(), self.context.i64_type(), "ptr_to_i64"
                        ))?
                    }
                    _ => val.into_int_value(),
                };
                let rt = self.module.get_function("ore_map_set").unwrap();
                bld!(self.builder.build_call(rt, &[map_val.into(), key.into(), i64_val.into()], ""))?;
                Ok((map_val, ValKind::Map))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "get takes 1 argument (key)".into() });
                }
                let key = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_map_get").unwrap();
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), key.into()], "mget"))?;
                let i64_val = self.call_result_to_value(result)?;

                // Determine value kind from map tracking
                // Check if the map object is a variable with a tracked value kind
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                match &val_kind {
                    ValKind::Str => {
                        // Convert i64 back to pointer
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            i64_val.into_int_value(), self.ptr_type(), "i64_to_ptr"
                        ))?;
                        Ok((ptr.into(), ValKind::Str))
                    }
                    ValKind::List => {
                        let ptr = bld!(self.builder.build_int_to_ptr(
                            i64_val.into_int_value(), self.ptr_type(), "i64_to_ptr"
                        ))?;
                        Ok((ptr.into(), ValKind::List))
                    }
                    _ => Ok((i64_val, val_kind))
                }
            }
            "contains" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "contains takes 1 argument (key)".into() });
                }
                let key = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_map_contains").unwrap();
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), key.into()], "mcontains"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                let bool_val = bld!(self.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    i8_val,
                    self.context.i8_type().const_int(0, false),
                    "tobool"
                ))?;
                Ok((bool_val.into(), ValKind::Bool))
            }
            "len" => {
                let rt = self.module.get_function("ore_map_len").unwrap();
                let result = bld!(self.builder.build_call(rt, &[map_val.into()], "mlen"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "remove" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "remove takes 1 argument (key)".into() });
                }
                let key = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_map_remove").unwrap();
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), key.into()], "mremove"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "keys" => {
                let rt = self.module.get_function("ore_map_keys").unwrap();
                let result = bld!(self.builder.build_call(rt, &[map_val.into()], "mkeys"))?;
                let val = self.call_result_to_value(result)?;
                self.last_list_elem_kind = Some(ValKind::Str);
                Ok((val, ValKind::List))
            }
            "values" => {
                let rt = self.module.get_function("ore_map_values").unwrap();
                let result = bld!(self.builder.build_call(rt, &[map_val.into()], "mvalues"))?;
                let val = self.call_result_to_value(result)?;
                // Track the value kind from the map
                let val_kind = self.last_map_val_kind.clone().unwrap_or(ValKind::Int);
                self.last_list_elem_kind = Some(val_kind);
                Ok((val, ValKind::List))
            }
            "merge" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "merge takes 1 argument (other map)".into() });
                }
                let other = self.compile_expr(&args[0], func)?;
                let rt = self.module.get_function("ore_map_merge").unwrap();
                let result = bld!(self.builder.build_call(rt, &[map_val.into(), other.into()], "mmerge"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Map))
            }
            "clear" => {
                let rt = self.module.get_function("ore_map_clear").unwrap();
                bld!(self.builder.build_call(rt, &[map_val.into()], ""))?;
                Ok((map_val, ValKind::Map))
            }
            _ => Err(CodeGenError { line: None, msg: format!("unknown map method '{}'", method) }),
        }
    }

    fn compile_channel_method(
        &mut self,
        ch_val: BasicValueEnum<'ctx>,
        method: &str,
        args: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        match method {
            "send" => {
                if args.len() != 1 {
                    return Err(CodeGenError { line: None, msg: "channel.send() takes 1 argument".into() });
                }
                let val = self.compile_expr(&args[0], func)?;
                let i64_val = self.value_to_i64(val)?;
                let rt = self.module.get_function("ore_channel_send").unwrap();
                bld!(self.builder.build_call(rt, &[ch_val.into(), i64_val.into()], ""))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            "recv" => {
                let rt = self.module.get_function("ore_channel_recv").unwrap();
                let result = bld!(self.builder.build_call(rt, &[ch_val.into()], "recv"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            _ => Err(CodeGenError { line: None, msg: format!("unknown channel method '{}'", method) }),
        }
    }

    fn compile_variant_construct(
        &mut self,
        variant_name: &str,
        fields: &[(String, Expr)],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let enum_name = self.variant_to_enum.get(variant_name).ok_or_else(|| CodeGenError {
            line: None, msg: format!("unknown variant '{}'", variant_name),
        })?.clone();

        let enum_info = self.enums.get(&enum_name).ok_or_else(|| CodeGenError {
            line: None, msg: format!("undefined enum '{}'", enum_name),
        })?;
        let enum_type = enum_info.enum_type;

        // Find the variant
        let variant = enum_info.variants.iter().find(|v| v.name == variant_name).ok_or_else(|| CodeGenError {
            line: None, msg: format!("unknown variant '{}'", variant_name),
        })?;
        let tag = variant.tag;
        let payload_type = variant.payload_type;
        let variant_field_names = variant.field_names.clone();

        // Alloca the enum
        let alloca = bld!(self.builder.build_alloca(enum_type, "enum_val"))?;

        // Store tag
        let tag_ptr = bld!(self.builder.build_struct_gep(enum_type, alloca, 0, "tag_ptr"))?;
        let tag_val = self.context.i8_type().const_int(tag as u64, false);
        bld!(self.builder.build_store(tag_ptr, tag_val))?;

        // Store payload fields
        let data_ptr = bld!(self.builder.build_struct_gep(enum_type, alloca, 1, "data_ptr"))?;
        let payload_ptr = bld!(self.builder.build_pointer_cast(
            data_ptr,
            self.ptr_type(),
            "payload_ptr"
        ))?;

        for (name, expr) in fields {
            let idx = variant_field_names.iter().position(|n| n == name).ok_or_else(|| CodeGenError {
                line: None, msg: format!("unknown field '{}' on variant '{}'", name, variant_name),
            })?;
            let val = self.compile_expr(expr, func)?;
            let field_ptr = bld!(self.builder.build_struct_gep(payload_type, payload_ptr, idx as u32, &format!("{}.{}", variant_name, name)))?;
            bld!(self.builder.build_store(field_ptr, val))?;
        }

        let result = bld!(self.builder.build_load(enum_type, alloca, "enum_loaded"))?;
        Ok((result, ValKind::Enum(enum_name)))
    }

    fn compile_match(
        &mut self,
        subject: &Expr,
        arms: &[MatchArm],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (subject_val, subject_kind) = self.compile_expr_with_kind(subject, func)?;

        if subject_kind == ValKind::Option {
            return self.compile_option_match(subject_val, arms, func);
        }
        if subject_kind == ValKind::Result {
            return self.compile_result_match(subject_val, arms, func);
        }

        // Check if patterns are literal patterns (Int, String, etc.)
        let has_literal_patterns = arms.iter().any(|arm| matches!(
            &arm.pattern,
            Pattern::IntLit(_) | Pattern::FloatLit(_) | Pattern::BoolLit(_) | Pattern::StringLit(_) | Pattern::Range(_, _) | Pattern::Or(_)
        ));
        if has_literal_patterns || matches!(subject_kind, ValKind::Int | ValKind::Float | ValKind::Bool | ValKind::Str) {
            return self.compile_literal_match(subject_val, &subject_kind, arms, func);
        }

        let enum_name = match &subject_kind {
            ValKind::Enum(name) => name.clone(),
            _ => return Err(CodeGenError { line: None, msg: "match subject must be an enum type".into() }),
        };

        let enum_info = self.enums.get(&enum_name).ok_or_else(|| CodeGenError {
            line: None, msg: format!("undefined enum '{}'", enum_name),
        })?;
        let enum_type = enum_info.enum_type;

        // Store subject to an alloca so we can extract tag and data
        let subject_alloca = bld!(self.builder.build_alloca(enum_type, "match_subject"))?;
        bld!(self.builder.build_store(subject_alloca, subject_val))?;

        // Load the tag
        let tag_ptr = bld!(self.builder.build_struct_gep(enum_type, subject_alloca, 0, "tag_ptr"))?;
        let tag_val = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?;

        let merge_bb = self.context.append_basic_block(func, "match_merge");

        // Build switch cases
        let default_bb = self.context.append_basic_block(func, "match_default");
        let mut case_blocks: Vec<(inkwell::values::IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut branch_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut result_kind = ValKind::Void;
        let mut wildcard_arm: Option<&MatchArm> = None;

        // Pre-collect variant info needed for each arm
        let variant_infos: Vec<_> = enum_info.variants.iter().map(|v| {
            (v.name.clone(), v.tag, v.payload_type, v.field_names.clone(), v.field_kinds.clone())
        }).collect();

        for arm in arms {
            match &arm.pattern {
                Pattern::Variant { name, bindings } => {
                    let variant = variant_infos.iter().find(|v| v.0 == *name).ok_or_else(|| CodeGenError {
                        line: None, msg: format!("unknown variant '{}' in match", name),
                    })?;
                    let (_, vtag, payload_type, _field_names, field_kinds) = variant;

                    let case_bb = self.context.append_basic_block(func, &format!("match_{}", name));
                    let tag_const = self.context.i8_type().const_int(*vtag as u64, false);
                    case_blocks.push((tag_const, case_bb));

                    self.builder.position_at_end(case_bb);

                    // Save variables and bind variant fields
                    let saved_vars = self.variables.clone();

                    // Extract payload
                    let data_ptr = bld!(self.builder.build_struct_gep(enum_type, subject_alloca, 1, "data_ptr"))?;
                    let payload_ptr = bld!(self.builder.build_pointer_cast(
                        data_ptr,
                        self.ptr_type(),
                        "payload_ptr"
                    ))?;

                    for (i, binding) in bindings.iter().enumerate() {
                        let field_kind = &field_kinds[i];
                        let field_ty = self.kind_to_llvm_type(field_kind);
                        let field_ptr = bld!(self.builder.build_struct_gep(*payload_type, payload_ptr, i as u32, binding))?;
                        let val = bld!(self.builder.build_load(field_ty, field_ptr, binding))?;
                        let alloca = bld!(self.builder.build_alloca(field_ty, binding))?;
                        bld!(self.builder.build_store(alloca, val))?;
                        self.variables.insert(binding.clone(), (alloca, field_ty, field_kind.clone(), false));
                    }

                    // Guard condition
                    if let Some(guard) = &arm.guard {
                        let (guard_val, _) = self.compile_expr_with_kind(guard, func)?;
                        let guard_bool = guard_val.into_int_value();
                        let body_bb = self.context.append_basic_block(func, &format!("guard_pass_{}", name));
                        bld!(self.builder.build_conditional_branch(guard_bool, body_bb, default_bb))?;
                        self.builder.position_at_end(body_bb);
                    }

                    let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
                    result_kind = arm_kind;

                    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                        bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    }
                    let end_bb = self.builder.get_insert_block().unwrap();
                    branch_results.push((arm_val, end_bb));

                    self.variables = saved_vars;
                }
                Pattern::Wildcard => {
                    wildcard_arm = Some(arm);
                }
                _ => return Err(CodeGenError { line: None, msg: "literal patterns not supported in enum match".into() }),
            }
        }

        // Handle wildcard/default
        self.builder.position_at_end(default_bb);
        if let Some(arm) = wildcard_arm {
            let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
            result_kind = arm_kind;
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                bld!(self.builder.build_unconditional_branch(merge_bb))?;
            }
            let end_bb = self.builder.get_insert_block().unwrap();
            branch_results.push((arm_val, end_bb));
        } else {
            // Unreachable default
            bld!(self.builder.build_unreachable())?;
        }

        // Build the switch
        // Position back at the block before the switch
        let switch_bb = tag_val.as_instruction_value().unwrap().get_parent().unwrap();
        self.builder.position_at_end(switch_bb);
        let switch = bld!(self.builder.build_switch(
            tag_val.into_int_value(),
            default_bb,
            &case_blocks.iter().map(|(v, bb)| (*v, *bb)).collect::<Vec<_>>()
        ))?;
        let _ = switch;

        // Build merge phi
        self.builder.position_at_end(merge_bb);
        if branch_results.is_empty() {
            return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void));
        }

        let phi = bld!(self.builder.build_phi(branch_results[0].0.get_type(), "match_val"))?;
        for (val, bb) in &branch_results {
            phi.add_incoming(&[(val, *bb)]);
        }

        Ok((phi.as_basic_value(), result_kind))
    }

    fn compile_option_match(
        &mut self,
        subject_val: BasicValueEnum<'ctx>,
        arms: &[MatchArm],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let opt_ty = self.option_type();

        // Store subject so we can GEP into it
        let subject_alloca = bld!(self.builder.build_alloca(opt_ty, "opt_match"))?;
        bld!(self.builder.build_store(subject_alloca, subject_val))?;

        // Load tag
        let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, subject_alloca, 0, "tag_ptr"))?;
        let tag_val = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();

        let merge_bb = self.context.append_basic_block(func, "opt_merge");
        let default_bb = self.context.append_basic_block(func, "opt_default");
        let mut case_blocks: Vec<(inkwell::values::IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut branch_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut result_kind = ValKind::Void;
        let mut wildcard_arm: Option<&MatchArm> = None;

        for arm in arms {
            match &arm.pattern {
                Pattern::Variant { name, bindings } => {
                    let vtag: u8 = match name.as_str() {
                        "None" => 0,
                        "Some" => 1,
                        _ => return Err(CodeGenError { line: None, msg: format!("unknown Option variant '{}'", name) }),
                    };

                    let case_bb = self.context.append_basic_block(func, &format!("opt_{}", name));
                    let tag_const = self.context.i8_type().const_int(vtag as u64, false);
                    case_blocks.push((tag_const, case_bb));

                    self.builder.position_at_end(case_bb);
                    let saved_vars = self.variables.clone();

                    // If Some, bind the payload
                    if vtag == 1 && !bindings.is_empty() {
                        // Read the kind tag to know the payload type
                        let kind_ptr = bld!(self.builder.build_struct_gep(opt_ty, subject_alloca, 1, "kind_ptr"))?;
                        let kind_i8 = bld!(self.builder.build_load(self.context.i8_type(), kind_ptr, "kind_tag"))?.into_int_value();
                        let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, subject_alloca, 2, "val_ptr"))?;
                        let payload = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, &bindings[0]))?;
                        let alloca = bld!(self.builder.build_alloca(self.context.i64_type(), &bindings[0]))?;
                        bld!(self.builder.build_store(alloca, payload))?;
                        // Store kind tag for dynamic dispatch in string interpolation
                        let kind_alloca = bld!(self.builder.build_alloca(self.context.i8_type(), &format!("{}_kind", bindings[0])))?;
                        bld!(self.builder.build_store(kind_alloca, kind_i8))?;
                        self.variables.insert(bindings[0].clone(), (alloca, self.context.i64_type().into(), ValKind::Int, false));
                        self.dynamic_kind_tags.insert(bindings[0].clone(), kind_alloca);
                    }

                    let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
                    result_kind = arm_kind;

                    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                        bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    }
                    let end_bb = self.builder.get_insert_block().unwrap();
                    branch_results.push((arm_val, end_bb));

                    self.variables = saved_vars;
                }
                Pattern::Wildcard => {
                    wildcard_arm = Some(arm);
                }
                _ => return Err(CodeGenError { line: None, msg: "literal patterns not supported in Option match".into() }),
            }
        }

        // Handle wildcard/default
        self.builder.position_at_end(default_bb);
        if let Some(arm) = wildcard_arm {
            let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
            result_kind = arm_kind;
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                bld!(self.builder.build_unconditional_branch(merge_bb))?;
            }
            let end_bb = self.builder.get_insert_block().unwrap();
            branch_results.push((arm_val, end_bb));
        } else {
            bld!(self.builder.build_unreachable())?;
        }

        // Build the switch from the original block
        let switch_bb = tag_ptr.as_instruction_value().unwrap().get_parent().unwrap();
        self.builder.position_at_end(switch_bb);
        bld!(self.builder.build_switch(
            tag_val,
            default_bb,
            &case_blocks.iter().map(|(v, bb)| (*v, *bb)).collect::<Vec<_>>()
        ))?;

        // Build merge phi
        self.builder.position_at_end(merge_bb);
        if branch_results.is_empty() {
            return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void));
        }

        let phi = bld!(self.builder.build_phi(branch_results[0].0.get_type(), "opt_val"))?;
        for (val, bb) in &branch_results {
            phi.add_incoming(&[(val, *bb)]);
        }

        Ok((phi.as_basic_value(), result_kind))
    }

    fn compile_literal_match(
        &mut self,
        subject_val: BasicValueEnum<'ctx>,
        subject_kind: &ValKind,
        arms: &[MatchArm],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        // Chain of if-else comparisons for literal patterns
        let merge_bb = self.context.append_basic_block(func, "lmatch_merge");
        let i64_type = self.context.i64_type();

        let result_alloca = bld!(self.builder.build_alloca(i64_type, "lmatch_result"))?;
        let mut result_kind = ValKind::Int;
        let mut has_wildcard = false;

        for (i, arm) in arms.iter().enumerate() {
            let is_last = i == arms.len() - 1;
            let else_bb = if is_last {
                merge_bb
            } else {
                self.context.append_basic_block(func, &format!("lmatch_next_{}", i))
            };

            match &arm.pattern {
                Pattern::Wildcard => {
                    has_wildcard = true;
                    // Wildcard with guard: check guard, fall through if false
                    if let Some(guard) = &arm.guard {
                        let (guard_val, _) = self.compile_expr_with_kind(guard, func)?;
                        let guard_bool = guard_val.into_int_value();
                        let body_bb = self.context.append_basic_block(func, &format!("lmatch_wguard_{}", i));
                        bld!(self.builder.build_conditional_branch(guard_bool, body_bb, else_bb))?;
                        self.builder.position_at_end(body_bb);
                    }
                    let (body_val, bk) = self.compile_expr_with_kind(&arm.body, func)?;
                    result_kind = bk;
                    let store_val = self.coerce_to_i64(body_val, &result_kind)?;
                    bld!(self.builder.build_store(result_alloca, store_val))?;
                    bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    if else_bb != merge_bb {
                        self.builder.position_at_end(else_bb);
                    }
                }
                _ => {
                    // Build comparison
                    let cmp = self.compile_pattern_cmp(subject_val, subject_kind, &arm.pattern, func)?;

                    let then_bb = self.context.append_basic_block(func, &format!("lmatch_arm_{}", i));

                    bld!(self.builder.build_conditional_branch(cmp, then_bb, else_bb))?;

                    self.builder.position_at_end(then_bb);

                    // Guard condition: if guard fails, jump to else_bb
                    if let Some(guard) = &arm.guard {
                        let (guard_val, _) = self.compile_expr_with_kind(guard, func)?;
                        let guard_bool = guard_val.into_int_value();
                        let body_bb = self.context.append_basic_block(func, &format!("lmatch_guarded_{}", i));
                        bld!(self.builder.build_conditional_branch(guard_bool, body_bb, else_bb))?;
                        self.builder.position_at_end(body_bb);
                    }

                    let (body_val, bk) = self.compile_expr_with_kind(&arm.body, func)?;
                    result_kind = bk;
                    let store_val = self.coerce_to_i64(body_val, &result_kind)?;
                    bld!(self.builder.build_store(result_alloca, store_val))?;
                    bld!(self.builder.build_unconditional_branch(merge_bb))?;

                    if else_bb != merge_bb {
                        self.builder.position_at_end(else_bb);
                    }
                }
            }
        }

        // If no wildcard, ensure we branch to merge from the last else block
        if !has_wildcard {
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                bld!(self.builder.build_store(result_alloca, i64_type.const_int(0, false)))?;
                bld!(self.builder.build_unconditional_branch(merge_bb))?;
            }
        }

        self.builder.position_at_end(merge_bb);
        let result = bld!(self.builder.build_load(i64_type, result_alloca, "lmatch_val"))?;

        // Convert back from i64 if needed
        let final_val = self.coerce_from_i64(result, &result_kind)?;
        Ok((final_val, result_kind))
    }

    fn compile_pattern_cmp(
        &mut self,
        subject: BasicValueEnum<'ctx>,
        _subject_kind: &ValKind,
        pattern: &Pattern,
        _func: FunctionValue<'ctx>,
    ) -> Result<IntValue<'ctx>, CodeGenError> {
        match pattern {
            Pattern::IntLit(n) => {
                let const_val = self.context.i64_type().const_int(*n as u64, true);
                bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, subject.into_int_value(), const_val, "pcmp"
                ))
            }
            Pattern::BoolLit(b) => {
                let const_val = self.context.bool_type().const_int(if *b { 1 } else { 0 }, false);
                bld!(self.builder.build_int_compare(
                    IntPredicate::EQ, subject.into_int_value(), const_val, "pcmp"
                ))
            }
            Pattern::StringLit(s) => {
                // Create string constant and compare
                let str_val = self.compile_string_literal(s)?;
                let rt = self.module.get_function("ore_str_eq").unwrap();
                let result = bld!(self.builder.build_call(rt, &[subject.into(), str_val.into()], "seq"))?;
                let i8_val = self.call_result_to_value(result)?.into_int_value();
                bld!(self.builder.build_int_compare(
                    IntPredicate::NE, i8_val,
                    self.context.i8_type().const_int(0, false), "tobool"
                ))
            }
            Pattern::Range(start, end) => {
                let i64_type = self.context.i64_type();
                let start_val = i64_type.const_int(*start as u64, true);
                let end_val = i64_type.const_int(*end as u64, true);
                let subj = subject.into_int_value();
                let ge = bld!(self.builder.build_int_compare(IntPredicate::SGE, subj, start_val, "rge"))?;
                let le = bld!(self.builder.build_int_compare(IntPredicate::SLE, subj, end_val, "rle"))?;
                bld!(self.builder.build_and(ge, le, "range_cmp"))
            }
            Pattern::Or(alternatives) => {
                // Or pattern: check any alternative matches
                let first = self.compile_pattern_cmp(subject, _subject_kind, &alternatives[0], _func)?;
                let mut result = first;
                for alt in &alternatives[1..] {
                    let alt_cmp = self.compile_pattern_cmp(subject, _subject_kind, alt, _func)?;
                    result = bld!(self.builder.build_or(result, alt_cmp, "or_pat"))?;
                }
                Ok(result)
            }
            _ => Err(CodeGenError { line: None, msg: "unsupported pattern in literal match".into() }),
        }
    }

    fn coerce_to_i64(&mut self, val: BasicValueEnum<'ctx>, kind: &ValKind) -> Result<IntValue<'ctx>, CodeGenError> {
        match kind {
            ValKind::Int => Ok(val.into_int_value()),
            ValKind::Bool => {
                bld!(self.builder.build_int_z_extend(
                    val.into_int_value(), self.context.i64_type(), "btoi64"
                ))
            }
            ValKind::Str | ValKind::List | ValKind::Map => {
                bld!(self.builder.build_ptr_to_int(
                    val.into_pointer_value(), self.context.i64_type(), "ptoi64"
                ))
            }
            _ => Ok(val.into_int_value()),
        }
    }

    fn coerce_from_i64(&mut self, val: BasicValueEnum<'ctx>, kind: &ValKind) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match kind {
            ValKind::Str | ValKind::List | ValKind::Map => {
                let ptr = bld!(self.builder.build_int_to_ptr(
                    val.into_int_value(), self.context.ptr_type(inkwell::AddressSpace::default()), "i64toptr"
                ))?;
                Ok(ptr.into())
            }
            ValKind::Bool => {
                let cmp = bld!(self.builder.build_int_compare(
                    IntPredicate::NE, val.into_int_value(),
                    self.context.i64_type().const_int(0, false), "i64tobool"
                ))?;
                Ok(cmp.into())
            }
            _ => Ok(val),
        }
    }

    fn compile_result_match(
        &mut self,
        subject_val: BasicValueEnum<'ctx>,
        arms: &[MatchArm],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let res_ty = self.result_type();

        let subject_alloca = bld!(self.builder.build_alloca(res_ty, "res_match"))?;
        bld!(self.builder.build_store(subject_alloca, subject_val))?;

        let tag_ptr = bld!(self.builder.build_struct_gep(res_ty, subject_alloca, 0, "tag_ptr"))?;
        let tag_val = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();

        let merge_bb = self.context.append_basic_block(func, "res_merge");
        let default_bb = self.context.append_basic_block(func, "res_default");
        let mut case_blocks: Vec<(inkwell::values::IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut branch_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
        let mut result_kind = ValKind::Void;
        let mut wildcard_arm: Option<&MatchArm> = None;

        for arm in arms {
            match &arm.pattern {
                Pattern::Variant { name, bindings } => {
                    let vtag: u8 = match name.as_str() {
                        "Ok" => 0,
                        "Err" => 1,
                        _ => return Err(CodeGenError { line: None, msg: format!("unknown Result variant '{}'", name) }),
                    };

                    let case_bb = self.context.append_basic_block(func, &format!("res_{}", name));
                    let tag_const = self.context.i8_type().const_int(vtag as u64, false);
                    case_blocks.push((tag_const, case_bb));

                    self.builder.position_at_end(case_bb);
                    let saved_vars = self.variables.clone();

                    if !bindings.is_empty() {
                        let kind_ptr = bld!(self.builder.build_struct_gep(res_ty, subject_alloca, 1, "kind_ptr"))?;
                        let kind_i8 = bld!(self.builder.build_load(self.context.i8_type(), kind_ptr, "kind_tag"))?.into_int_value();
                        let val_ptr = bld!(self.builder.build_struct_gep(res_ty, subject_alloca, 2, "val_ptr"))?;
                        let payload = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, &bindings[0]))?;
                        let alloca = bld!(self.builder.build_alloca(self.context.i64_type(), &bindings[0]))?;
                        bld!(self.builder.build_store(alloca, payload))?;
                        let kind_alloca = bld!(self.builder.build_alloca(self.context.i8_type(), &format!("{}_kind", bindings[0])))?;
                        bld!(self.builder.build_store(kind_alloca, kind_i8))?;
                        self.variables.insert(bindings[0].clone(), (alloca, self.context.i64_type().into(), ValKind::Int, false));
                        self.dynamic_kind_tags.insert(bindings[0].clone(), kind_alloca);
                    }

                    let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
                    result_kind = arm_kind;

                    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                        bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    }
                    let end_bb = self.builder.get_insert_block().unwrap();
                    branch_results.push((arm_val, end_bb));

                    self.variables = saved_vars;
                }
                Pattern::Wildcard => {
                    wildcard_arm = Some(arm);
                }
                _ => return Err(CodeGenError { line: None, msg: "literal patterns not supported in Result match".into() }),
            }
        }

        self.builder.position_at_end(default_bb);
        if let Some(arm) = wildcard_arm {
            let (arm_val, arm_kind) = self.compile_expr_with_kind(&arm.body, func)?;
            result_kind = arm_kind;
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                bld!(self.builder.build_unconditional_branch(merge_bb))?;
            }
            let end_bb = self.builder.get_insert_block().unwrap();
            branch_results.push((arm_val, end_bb));
        } else {
            bld!(self.builder.build_unreachable())?;
        }

        let switch_bb = tag_ptr.as_instruction_value().unwrap().get_parent().unwrap();
        self.builder.position_at_end(switch_bb);
        bld!(self.builder.build_switch(
            tag_val,
            default_bb,
            &case_blocks.iter().map(|(v, bb)| (*v, *bb)).collect::<Vec<_>>()
        ))?;

        self.builder.position_at_end(merge_bb);
        if branch_results.is_empty() {
            return Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void));
        }

        let phi = bld!(self.builder.build_phi(branch_results[0].0.get_type(), "res_val"))?;
        for (val, bb) in &branch_results {
            phi.add_incoming(&[(val, *bb)]);
        }

        Ok((phi.as_basic_value(), result_kind))
    }

    fn compile_try_result(
        &mut self,
        val: BasicValueEnum<'ctx>,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let res_ty = self.result_type();
        let alloca = bld!(self.builder.build_alloca(res_ty, "try_res"))?;
        bld!(self.builder.build_store(alloca, val))?;
        let tag_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();
        let is_err = bld!(self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.context.i8_type().const_int(1, false), "is_err"
        ))?;
        let ok_bb = self.context.append_basic_block(func, "try_ok");
        let err_bb = self.context.append_basic_block(func, "try_err");
        bld!(self.builder.build_conditional_branch(is_err, err_bb, ok_bb))?;
        // Err branch: return the Err result from the current function
        self.builder.position_at_end(err_bb);
        let err_ret = bld!(self.builder.build_load(res_ty, alloca, "err_ret"))?;
        bld!(self.builder.build_return(Some(&err_ret)))?;
        // Ok branch: extract the value
        self.builder.position_at_end(ok_bb);
        let val_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 2, "val_ptr"))?;
        let extracted = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "unwrapped"))?;
        Ok((extracted, ValKind::Int))
    }

    fn compile_list_lit(
        &mut self,
        elements: &[Expr],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let list_new = self.module.get_function("ore_list_new").unwrap();
        let list_push = self.module.get_function("ore_list_push").unwrap();

        let list_result = bld!(self.builder.build_call(list_new, &[], "list"))?;
        let list_ptr = self.call_result_to_value(list_result)?.into_pointer_value();

        let mut elem_kind = ValKind::Int;
        for elem in elements {
            let (val, kind) = self.compile_expr_with_kind(elem, func)?;
            elem_kind = kind.clone();
            // For records/enums, heap-allocate and push the pointer
            let push_val = match &kind {
                ValKind::Record(name) => {
                    let info = &self.records[name];
                    let st = info.struct_type;
                    let heap_ptr = bld!(self.builder.build_malloc(st, "heap_rec"))?;
                    bld!(self.builder.build_store(heap_ptr, val))?;
                    let i64_val = bld!(self.builder.build_ptr_to_int(heap_ptr, self.context.i64_type(), "p2i"))?;
                    i64_val.into()
                }
                ValKind::Str => {
                    // Strings are already pointers, convert to i64
                    let i64_val = bld!(self.builder.build_ptr_to_int(val.into_pointer_value(), self.context.i64_type(), "p2i"))?;
                    i64_val.into()
                }
                ValKind::Float => {
                    // Floats need bitcast to i64 for storage
                    let i64_val = bld!(self.builder.build_bit_cast(val, self.context.i64_type(), "f2i"))?;
                    i64_val
                }
                ValKind::Bool => {
                    // Bools need zero-extension to i64
                    let i64_val = bld!(self.builder.build_int_z_extend(val.into_int_value(), self.context.i64_type(), "b2i"))?;
                    i64_val.into()
                }
                _ => val,
            };
            bld!(self.builder.build_call(list_push, &[list_ptr.into(), push_val.into()], ""))?;
        }

        // Store element kind for later extraction
        self.last_list_elem_kind = Some(elem_kind);

        Ok((list_ptr.into(), ValKind::List))
    }

    fn compile_map_lit(
        &mut self,
        entries: &[(Expr, Expr)],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let map_new = self.module.get_function("ore_map_new").unwrap();
        let map_set = self.module.get_function("ore_map_set").unwrap();

        let map_result = bld!(self.builder.build_call(map_new, &[], "map"))?;
        let map_ptr = self.call_result_to_value(map_result)?.into_pointer_value();

        let mut first_val_kind = None;
        for (key, value) in entries {
            let key_val = self.compile_expr(key, func)?;
            let (val, val_kind) = self.compile_expr_with_kind(value, func)?;
            if first_val_kind.is_none() {
                first_val_kind = Some(val_kind.clone());
            }
            // Convert value to i64 for storage
            let i64_val = match val_kind {
                ValKind::Int => val.into_int_value(),
                ValKind::Bool => {
                    bld!(self.builder.build_int_z_extend(
                        val.into_int_value(),
                        self.context.i64_type(),
                        "bool_to_i64"
                    ))?
                }
                ValKind::Str | ValKind::List | ValKind::Map => {
                    bld!(self.builder.build_ptr_to_int(
                        val.into_pointer_value(),
                        self.context.i64_type(),
                        "ptr_to_i64"
                    ))?
                }
                _ => val.into_int_value(),
            };
            bld!(self.builder.build_call(
                map_set,
                &[map_ptr.into(), key_val.into(), i64_val.into()],
                ""
            ))?;
        }

        self.last_map_val_kind = first_val_kind;
        Ok((map_ptr.into(), ValKind::Map))
    }

    fn compile_index(
        &mut self,
        object: &Expr,
        index: &Expr,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let (obj_val, obj_kind) = self.compile_expr_with_kind(object, func)?;
        let idx_val = self.compile_expr(index, func)?;

        match obj_kind {
            ValKind::List => {
                let list_get = self.module.get_function("ore_list_get").unwrap();
                let result = bld!(self.builder.build_call(
                    list_get,
                    &[obj_val.into(), idx_val.into()],
                    "list_get"
                ))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            ValKind::Map => {
                let map_get = self.module.get_function("ore_map_get").unwrap();
                let result = bld!(self.builder.build_call(
                    map_get,
                    &[obj_val.into(), idx_val.into()],
                    "map_get"
                ))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            _ => Err(CodeGenError { line: None, msg: "indexing only supported on lists and maps".into() }),
        }
    }

    fn compile_string_literal(&mut self, s: &str) -> Result<PointerValue<'ctx>, CodeGenError> {
        let bytes = s.as_bytes();
        let global_name = format!(".str.{}", self.str_counter);
        self.str_counter += 1;

        let i8_type = self.context.i8_type();
        let arr_type = i8_type.array_type(bytes.len() as u32);
        let global = self.module.add_global(arr_type, None, &global_name);
        global.set_initializer(&i8_type.const_array(
            &bytes.iter().map(|&b| i8_type.const_int(b as u64, false)).collect::<Vec<_>>(),
        ));
        global.set_constant(true);

        let str_new = self.module.get_function("ore_str_new").unwrap();
        let ptr = bld!(self.builder.build_pointer_cast(
            global.as_pointer_value(),
            self.ptr_type(),
            "strptr"
        ))?;
        let len = self.context.i32_type().const_int(bytes.len() as u64, false);
        let result = bld!(self.builder.build_call(str_new, &[ptr.into(), len.into()], "str"))?;
        match result.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(BasicValueEnum::PointerValue(p)) => Ok(p),
            inkwell::values::ValueKind::Basic(v) => {
                // Should be pointer but handle gracefully
                Ok(v.into_pointer_value())
            }
            _ => Err(CodeGenError { line: None, msg: "ore_str_new did not return a pointer".into() }),
        }
    }

    /// Create a global constant string and return a pointer to its data.
    fn builder_string_const(&mut self, s: &str) -> PointerValue<'ctx> {
        let bytes = s.as_bytes();
        let global_name = format!(".str.{}", self.str_counter);
        self.str_counter += 1;
        let i8_type = self.context.i8_type();
        let arr_type = i8_type.array_type(bytes.len() as u32);
        let global = self.module.add_global(arr_type, None, &global_name);
        global.set_initializer(&i8_type.const_array(
            &bytes.iter().map(|&b| i8_type.const_int(b as u64, false)).collect::<Vec<_>>(),
        ));
        global.set_constant(true);
        // build_pointer_cast can't fail for globals in practice
        self.builder.build_pointer_cast(
            global.as_pointer_value(),
            self.ptr_type(),
            "strptr",
        ).unwrap()
    }

    fn compile_string_interp(
        &mut self,
        parts: &[StringPart],
        func: FunctionValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let concat_fn = self.module.get_function("ore_str_concat").unwrap();
        let release_fn = self.module.get_function("ore_str_release").unwrap();

        let mut result: Option<PointerValue<'ctx>> = None;
        let mut temps: Vec<PointerValue<'ctx>> = Vec::new();

        for part in parts {
            let part_ptr = match part {
                StringPart::Lit(s) => {
                    let p = self.compile_string_literal(s)?;
                    temps.push(p);
                    p
                }
                StringPart::Expr(expr) => {
                    let (val, kind) = self.compile_expr_with_kind(expr, func)?;
                    // Check if this is a variable with a dynamic kind tag (from Result/Option match)
                    let p = if let Expr::Ident(name) = expr {
                        if let Some(kind_alloca) = self.dynamic_kind_tags.get(name).copied() {
                            let kind_i8 = bld!(self.builder.build_load(self.context.i8_type(), kind_alloca, "dyn_kind"))?.into_int_value();
                            let dyn_fn = self.module.get_function("ore_dynamic_to_str").unwrap();
                            let result = bld!(self.builder.build_call(dyn_fn, &[val.into(), kind_i8.into()], "dyntos"))?;
                            self.call_result_to_value(result)?.into_pointer_value()
                        } else {
                            self.value_to_str(val, kind)?
                        }
                    } else {
                        self.value_to_str(val, kind)?
                    };
                    temps.push(p);
                    p
                }
            };

            result = Some(match result {
                None => part_ptr,
                Some(acc) => {
                    let concat_result = bld!(self.builder.build_call(
                        concat_fn,
                        &[acc.into(), part_ptr.into()],
                        "concat"
                    ))?;
                    let new_ptr = self.call_result_to_value(concat_result)?.into_pointer_value();
                    // Release the old accumulator (it was a concat result, not a literal or conversion)
                    // We'll release all temps at the end
                    temps.push(new_ptr);
                    new_ptr
                }
            });
        }

        let final_ptr = result.unwrap_or_else(|| self.ptr_type().const_null());

        // Retain the final result before releasing temps
        if !final_ptr.is_null() {
            let retain_fn = self.module.get_function("ore_str_retain").unwrap();
            bld!(self.builder.build_call(retain_fn, &[final_ptr.into()], ""))?;
        }

        // Release all temporaries
        for temp in &temps {
            bld!(self.builder.build_call(release_fn, &[(*temp).into()], ""))?;
        }

        Ok(final_ptr)
    }

    /// Convert any BasicValueEnum to i64 for storage in Option/Result payloads.
    fn value_to_i64(&mut self, val: BasicValueEnum<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, CodeGenError> {
        match val {
            BasicValueEnum::IntValue(v) => {
                if v.get_type().get_bit_width() < 64 {
                    Ok(bld!(self.builder.build_int_z_extend(v, self.context.i64_type(), "zext"))?)
                } else {
                    Ok(v)
                }
            }
            BasicValueEnum::FloatValue(v) => {
                Ok(bld!(self.builder.build_bit_cast(v, self.context.i64_type(), "f2i"))?.into_int_value())
            }
            BasicValueEnum::PointerValue(v) => {
                Ok(bld!(self.builder.build_ptr_to_int(v, self.context.i64_type(), "p2i"))?)
            }
            _ => Ok(val.into_int_value()),
        }
    }

    fn value_to_str(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: ValKind,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        match kind {
            ValKind::Str => {
                // Already a string pointer, retain it
                let retain_fn = self.module.get_function("ore_str_retain").unwrap();
                let ptr = val.into_pointer_value();
                bld!(self.builder.build_call(retain_fn, &[ptr.into()], ""))?;
                Ok(ptr)
            }
            ValKind::Int => {
                let int_to_str = self.module.get_function("ore_int_to_str").unwrap();
                let result = bld!(self.builder.build_call(int_to_str, &[val.into()], "itos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
            ValKind::Float => {
                let float_to_str = self.module.get_function("ore_float_to_str").unwrap();
                let result = bld!(self.builder.build_call(float_to_str, &[val.into()], "ftos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
            ValKind::Bool => {
                let bool_to_str = self.module.get_function("ore_bool_to_str").unwrap();
                let int_val = val.into_int_value();
                let ext = bld!(self.builder.build_int_z_extend(int_val, self.context.i8_type(), "zext"))?;
                let result = bld!(self.builder.build_call(bool_to_str, &[ext.into()], "btos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
            ValKind::Record(ref name) => {
                self.record_to_str(val, name)
            }
            ValKind::Enum(ref name) => {
                self.enum_to_str(val, name)
            }
            _ => {
                // Fallback: convert as int
                let int_to_str = self.module.get_function("ore_int_to_str").unwrap();
                let result = bld!(self.builder.build_call(int_to_str, &[val.into()], "itos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
        }
    }

    fn record_to_str(
        &mut self,
        val: BasicValueEnum<'ctx>,
        type_name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let info = self.records.get(type_name).ok_or_else(|| CodeGenError {
            line: None, msg: format!("undefined type '{}' for display", type_name),
        })?;
        let struct_type = info.struct_type;
        let field_names = info.field_names.clone();
        let field_kinds = info.field_kinds.clone();

        let str_new = self.module.get_function("ore_str_new").unwrap();
        let concat_fn = self.module.get_function("ore_str_concat").unwrap();
        let release_fn = self.module.get_function("ore_str_release").unwrap();

        // Store the struct to an alloca so we can GEP into it
        let alloca = bld!(self.builder.build_alloca(struct_type, "rec_tmp"))?;
        bld!(self.builder.build_store(alloca, val))?;

        // Helper: call ore_str_new and get pointer
        let make_str = |cg: &mut Self, s: &str| -> Result<PointerValue<'ctx>, CodeGenError> {
            let ptr = cg.builder_string_const(s);
            let len = cg.context.i32_type().const_int(s.len() as u64, false);
            let result = bld!(cg.builder.build_call(str_new, &[ptr.into(), len.into()], "s"))?;
            Ok(cg.call_result_to_value(result)?.into_pointer_value())
        };

        // Helper: concat two strings, releasing both inputs
        let concat_and_release = |cg: &mut Self, a: PointerValue<'ctx>, b: PointerValue<'ctx>| -> Result<PointerValue<'ctx>, CodeGenError> {
            let result = bld!(cg.builder.build_call(concat_fn, &[a.into(), b.into()], "cat"))?;
            let p = cg.call_result_to_value(result)?.into_pointer_value();
            bld!(cg.builder.build_call(release_fn, &[a.into()], ""))?;
            bld!(cg.builder.build_call(release_fn, &[b.into()], ""))?;
            Ok(p)
        };

        // Start with "TypeName("
        let prefix = format!("{}(", type_name);
        let mut current = make_str(self, &prefix)?;

        for (i, (fname, fkind)) in field_names.iter().zip(field_kinds.iter()).enumerate() {
            let label = if i == 0 { format!("{}: ", fname) } else { format!(", {}: ", fname) };
            let label_str = make_str(self, &label)?;
            current = concat_and_release(self, current, label_str)?;

            // Extract field value and convert to string
            let field_ptr = bld!(self.builder.build_struct_gep(struct_type, alloca, i as u32, &format!("f_{}", fname)))?;
            let field_ty = struct_type.get_field_type_at_index(i as u32).unwrap();
            let field_val = bld!(self.builder.build_load(field_ty, field_ptr, fname))?;
            let field_str = self.value_to_str(field_val, fkind.clone())?;
            current = concat_and_release(self, current, field_str)?;
        }

        // Append ")"
        let suffix_str = make_str(self, ")")?;
        current = concat_and_release(self, current, suffix_str)?;

        Ok(current)
    }

    fn enum_to_str(
        &mut self,
        val: BasicValueEnum<'ctx>,
        enum_name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let enum_info = self.enums.get(enum_name).ok_or_else(|| CodeGenError {
            line: None, msg: format!("undefined enum '{}' for display", enum_name),
        })?;
        let enum_type = enum_info.enum_type;
        let variants: Vec<_> = enum_info.variants.iter().map(|v| {
            (v.name.clone(), v.tag, v.field_names.clone(), v.field_kinds.clone(), v.payload_type)
        }).collect();

        let str_new = self.module.get_function("ore_str_new").unwrap();
        let concat_fn = self.module.get_function("ore_str_concat").unwrap();
        let release_fn = self.module.get_function("ore_str_release").unwrap();

        // Store enum to alloca
        let alloca = bld!(self.builder.build_alloca(enum_type, "enum_tmp"))?;
        bld!(self.builder.build_store(alloca, val))?;

        // Result alloca (must be before the switch)
        let result_alloca = bld!(self.builder.build_alloca(self.ptr_type(), "enum_str_result"))?;

        // Read tag
        let tag_ptr = bld!(self.builder.build_struct_gep(enum_type, alloca, 0, "tag_ptr"))?;
        let tag = bld!(self.builder.build_load(self.context.i8_type(), tag_ptr, "tag"))?.into_int_value();

        let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();

        // Create merge block and default block
        let merge_bb = self.context.append_basic_block(current_fn, "enum_str_merge");
        let default_bb = self.context.append_basic_block(current_fn, "enum_str_default");

        // Build variant blocks first (collect cases), then build switch in entry block
        let mut cases = Vec::new();
        for (vname, vtag, field_names, field_kinds, payload_type) in &variants {
            let bb = self.context.append_basic_block(current_fn, &format!("enum_str_{}", vname));
            self.builder.position_at_end(bb);

            if field_names.is_empty() {
                let name_ptr = self.builder_string_const(vname);
                let name_str = bld!(self.builder.build_call(str_new, &[name_ptr.into(), self.context.i32_type().const_int(vname.len() as u64, false).into()], "s"))?;
                let name_val = self.call_result_to_value(name_str)?.into_pointer_value();
                bld!(self.builder.build_store(result_alloca, name_val))?;
            } else {
                let prefix = format!("{}(", vname);
                let prefix_ptr = self.builder_string_const(&prefix);
                let prefix_len = self.context.i32_type().const_int(prefix.len() as u64, false);
                let prefix_str = bld!(self.builder.build_call(str_new, &[prefix_ptr.into(), prefix_len.into()], "s"))?;
                let mut current = self.call_result_to_value(prefix_str)?.into_pointer_value();

                let data_ptr = bld!(self.builder.build_struct_gep(enum_type, alloca, 1, "data_ptr"))?;
                let payload_ptr = bld!(self.builder.build_pointer_cast(data_ptr, self.ptr_type(), "payload"))?;

                for (i, (fname, fkind)) in field_names.iter().zip(field_kinds.iter()).enumerate() {
                    let label = if i == 0 { format!("{}: ", fname) } else { format!(", {}: ", fname) };
                    let label_ptr = self.builder_string_const(&label);
                    let label_len = self.context.i32_type().const_int(label.len() as u64, false);
                    let label_str = bld!(self.builder.build_call(str_new, &[label_ptr.into(), label_len.into()], "s"))?;
                    let label_val = self.call_result_to_value(label_str)?.into_pointer_value();
                    let next = bld!(self.builder.build_call(concat_fn, &[current.into(), label_val.into()], "cat"))?;
                    let next_ptr = self.call_result_to_value(next)?.into_pointer_value();
                    bld!(self.builder.build_call(release_fn, &[current.into()], ""))?;
                    bld!(self.builder.build_call(release_fn, &[label_val.into()], ""))?;
                    current = next_ptr;

                    let field_ptr = bld!(self.builder.build_struct_gep(*payload_type, payload_ptr, i as u32, &format!("f_{}", fname)))?;
                    let field_ty = payload_type.get_field_type_at_index(i as u32).unwrap();
                    let field_val = bld!(self.builder.build_load(field_ty, field_ptr, fname))?;
                    let field_str = self.value_to_str(field_val, fkind.clone())?;

                    let next2 = bld!(self.builder.build_call(concat_fn, &[current.into(), field_str.into()], "cat"))?;
                    let next2_ptr = self.call_result_to_value(next2)?.into_pointer_value();
                    bld!(self.builder.build_call(release_fn, &[current.into()], ""))?;
                    bld!(self.builder.build_call(release_fn, &[field_str.into()], ""))?;
                    current = next2_ptr;
                }

                let suffix_ptr = self.builder_string_const(")");
                let suffix_str = bld!(self.builder.build_call(str_new, &[suffix_ptr.into(), self.context.i32_type().const_int(1, false).into()], "s"))?;
                let suffix_val = self.call_result_to_value(suffix_str)?.into_pointer_value();
                let final_str = bld!(self.builder.build_call(concat_fn, &[current.into(), suffix_val.into()], "cat"))?;
                let final_ptr = self.call_result_to_value(final_str)?.into_pointer_value();
                bld!(self.builder.build_call(release_fn, &[current.into()], ""))?;
                bld!(self.builder.build_call(release_fn, &[suffix_val.into()], ""))?;
                bld!(self.builder.build_store(result_alloca, final_ptr))?;
            }

            bld!(self.builder.build_unconditional_branch(merge_bb))?;
            cases.push((self.context.i8_type().const_int(*vtag as u64, false), bb));
        }

        // Default block
        self.builder.position_at_end(default_bb);
        let unknown_s = self.builder_string_const("<unknown>");
        let unknown_str = bld!(self.builder.build_call(str_new, &[unknown_s.into(), self.context.i32_type().const_int(9, false).into()], "s"))?;
        let unknown_ptr = self.call_result_to_value(unknown_str)?.into_pointer_value();
        bld!(self.builder.build_store(result_alloca, unknown_ptr))?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;

        // Now we need to insert the switch. The entry block where we read the tag
        // needs a terminator (the switch). But we already moved the builder away.
        // We need to go back to the entry block and add the switch there.
        // The entry block is the one containing the tag load. Let's use a dedicated block.
        // Actually, the tag load was in the current insert block before we started building variant blocks.
        // We need to split: create a switch block right after the tag load.

        // The trick: the alloca + tag load were in the original block. We need to terminate
        // that block with a branch to a switch block. But the original block might already
        // have other code. Let's just use an unconditional branch from wherever we were
        // to a new switch block.

        // Actually, the simplest approach: the entry block (where tag was loaded) doesn't have
        // a terminator yet. We need to go back there and add one.
        // But we've moved the builder. The tag was loaded in the block that was current
        // when enum_to_str was called. That block now has no terminator.

        // Let's find that block: it's the one containing the alloca instruction
        let entry_block = alloca.as_instruction_value().unwrap().get_parent().unwrap();
        self.builder.position_at_end(entry_block);
        bld!(self.builder.build_switch(tag, default_bb, &cases))?;

        // Position at merge for subsequent code
        self.builder.position_at_end(merge_bb);
        let result = bld!(self.builder.build_load(self.ptr_type(), result_alloca, "enum_str_val"))?.into_pointer_value();
        Ok(result)
    }

    fn compile_typed_list_print(
        &mut self,
        list_ptr: PointerValue<'ctx>,
        elem_kind: &ValKind,
    ) -> Result<(), CodeGenError> {
        // Print "[" using ore_str_print
        let open_bracket = self.compile_string_literal("[")?;
        let str_print = self.module.get_function("ore_str_print_no_newline").unwrap();
        bld!(self.builder.build_call(str_print, &[open_bracket.into()], ""))?;
        let release = self.module.get_function("ore_str_release").unwrap();
        bld!(self.builder.build_call(release, &[open_bracket.into()], ""))?;

        let list_len = self.module.get_function("ore_list_len").unwrap();
        let list_get = self.module.get_function("ore_list_get").unwrap();

        let len_result = bld!(self.builder.build_call(list_len, &[list_ptr.into()], "len"))?;
        let len = self.call_result_to_value(len_result)?.into_int_value();

        let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();

        // Loop: for i in 0..len
        let idx_alloca = bld!(self.builder.build_alloca(self.context.i64_type(), "idx"))?;
        bld!(self.builder.build_store(idx_alloca, self.context.i64_type().const_int(0, false)))?;

        let loop_check = self.context.append_basic_block(current_fn, "list_print_check");
        let loop_body = self.context.append_basic_block(current_fn, "list_print_body");
        let loop_end = self.context.append_basic_block(current_fn, "list_print_end");

        bld!(self.builder.build_unconditional_branch(loop_check))?;

        self.builder.position_at_end(loop_check);
        let i = bld!(self.builder.build_load(self.context.i64_type(), idx_alloca, "i"))?.into_int_value();
        let cond = bld!(self.builder.build_int_compare(IntPredicate::SLT, i, len, "cmp"))?;
        bld!(self.builder.build_conditional_branch(cond, loop_body, loop_end))?;

        self.builder.position_at_end(loop_body);
        let i = bld!(self.builder.build_load(self.context.i64_type(), idx_alloca, "i"))?.into_int_value();

        // Print ", " if not first element
        let is_first = bld!(self.builder.build_int_compare(IntPredicate::EQ, i, self.context.i64_type().const_int(0, false), "first"))?;
        let sep_bb = self.context.append_basic_block(current_fn, "print_sep");
        let elem_bb = self.context.append_basic_block(current_fn, "print_elem");
        bld!(self.builder.build_conditional_branch(is_first, elem_bb, sep_bb))?;

        self.builder.position_at_end(sep_bb);
        let sep = self.compile_string_literal(", ")?;
        bld!(self.builder.build_call(str_print, &[sep.into()], ""))?;
        bld!(self.builder.build_call(release, &[sep.into()], ""))?;
        bld!(self.builder.build_unconditional_branch(elem_bb))?;

        self.builder.position_at_end(elem_bb);
        let i = bld!(self.builder.build_load(self.context.i64_type(), idx_alloca, "i"))?.into_int_value();

        // Get element
        let elem_result = bld!(self.builder.build_call(list_get, &[list_ptr.into(), i.into()], "elem"))?;
        let elem_i64 = self.call_result_to_value(elem_result)?.into_int_value();

        // Convert and print based on element kind
        match elem_kind {
            ValKind::Str => {
                let elem_ptr = bld!(self.builder.build_int_to_ptr(elem_i64, self.ptr_type(), "str_ptr"))?;
                bld!(self.builder.build_call(str_print, &[elem_ptr.into()], ""))?;
            }
            ValKind::Float => {
                let f = bld!(self.builder.build_bit_cast(elem_i64, self.context.f64_type(), "f"))?.into_float_value();
                let print_float = self.module.get_function("ore_print_float_no_newline").unwrap();
                bld!(self.builder.build_call(print_float, &[f.into()], ""))?;
            }
            ValKind::Bool => {
                let b = bld!(self.builder.build_int_truncate(elem_i64, self.context.i8_type(), "b"))?;
                let print_bool = self.module.get_function("ore_print_bool_no_newline").unwrap();
                bld!(self.builder.build_call(print_bool, &[b.into()], ""))?;
            }
            _ => {
                let print_int = self.module.get_function("ore_print_int_no_newline").unwrap();
                bld!(self.builder.build_call(print_int, &[elem_i64.into()], ""))?;
            }
        }

        // Increment
        let next_i = bld!(self.builder.build_int_add(i, self.context.i64_type().const_int(1, false), "next_i"))?;
        bld!(self.builder.build_store(idx_alloca, next_i))?;
        bld!(self.builder.build_unconditional_branch(loop_check))?;

        self.builder.position_at_end(loop_end);
        // Print "]\n"
        let close_str = self.compile_string_literal("]")?;
        let print_str_fn = self.module.get_function("ore_str_print").unwrap();
        bld!(self.builder.build_call(print_str_fn, &[close_str.into()], ""))?;
        bld!(self.builder.build_call(release, &[close_str.into()], ""))?;

        Ok(())
    }

    fn compile_print(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: ValKind,
    ) -> Result<(), CodeGenError> {
        match kind {
            ValKind::Str => {
                let pf = self.module.get_function("ore_str_print").unwrap();
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
            ValKind::Bool => {
                let pf = self.module.get_function("ore_print_bool").unwrap();
                let int_val = val.into_int_value();
                let ext = bld!(self.builder.build_int_z_extend(int_val, self.context.i8_type(), "zext"))?;
                bld!(self.builder.build_call(pf, &[ext.into()], ""))?;
            }
            ValKind::Float => {
                let pf = self.module.get_function("ore_print_float").unwrap();
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
            ValKind::List => {
                let pf = self.module.get_function("ore_list_print").unwrap();
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
            ValKind::Map => {
                let pf = self.module.get_function("ore_map_print").unwrap();
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
            ValKind::Record(ref name) => {
                let s = self.record_to_str(val, name)?;
                let pf = self.module.get_function("ore_str_print").unwrap();
                bld!(self.builder.build_call(pf, &[s.into()], ""))?;
                let release = self.module.get_function("ore_str_release").unwrap();
                bld!(self.builder.build_call(release, &[s.into()], ""))?;
            }
            ValKind::Enum(ref name) => {
                let s = self.enum_to_str(val, name)?;
                let pf = self.module.get_function("ore_str_print").unwrap();
                bld!(self.builder.build_call(pf, &[s.into()], ""))?;
                let release = self.module.get_function("ore_str_release").unwrap();
                bld!(self.builder.build_call(release, &[s.into()], ""))?;
            }
            _ => {
                let pf = self.module.get_function("ore_print_int").unwrap();
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
        }
        Ok(())
    }

    fn resolve_function(&self, name: &str) -> Result<(FunctionValue<'ctx>, ValKind), CodeGenError> {
        if let Some((f, k)) = self.functions.get(name) {
            return Ok((*f, k.clone()));
        }
        if let Some(f) = self.module.get_function(name) {
            return Ok((f, ValKind::Void));
        }
        Err(CodeGenError {
            line: None, msg: format!("undefined function '{}'", name),
        })
    }

    /// Map a ValKind back to a TypeExpr for monomorphization substitution.
    fn valkind_to_type_expr(kind: &ValKind) -> TypeExpr {
        match kind {
            ValKind::Int => TypeExpr::Named("Int".to_string()),
            ValKind::Float => TypeExpr::Named("Float".to_string()),
            ValKind::Bool => TypeExpr::Named("Bool".to_string()),
            ValKind::Str => TypeExpr::Named("Str".to_string()),
            ValKind::Void => TypeExpr::Named("Void".to_string()),
            ValKind::Record(name) => TypeExpr::Named(name.clone()),
            ValKind::Enum(name) => TypeExpr::Named(name.clone()),
            ValKind::Option => TypeExpr::Named("Option".to_string()),
            ValKind::Result => TypeExpr::Named("Result".to_string()),
            ValKind::List => TypeExpr::Named("List".to_string()),
            ValKind::Map => TypeExpr::Named("Map".to_string()),
            ValKind::Channel => TypeExpr::Named("Channel".to_string()),
        }
    }

    /// Create a mangled name for a monomorphized function.
    fn mangle_generic_name(base: &str, concrete_kinds: &[ValKind]) -> String {
        let mut name = base.to_string();
        for k in concrete_kinds {
            name.push('$');
            match k {
                ValKind::Int => name.push_str("Int"),
                ValKind::Float => name.push_str("Float"),
                ValKind::Bool => name.push_str("Bool"),
                ValKind::Str => name.push_str("Str"),
                ValKind::Void => name.push_str("Void"),
                ValKind::Record(n) => name.push_str(n),
                ValKind::Enum(n) => name.push_str(n),
                ValKind::Option => name.push_str("Option"),
                ValKind::Result => name.push_str("Result"),
                ValKind::List => name.push_str("List"),
                ValKind::Map => name.push_str("Map"),
                ValKind::Channel => name.push_str("Channel"),
            }
        }
        name
    }

    /// Substitute type parameters in a TypeExpr.
    fn substitute_type_expr(ty: &TypeExpr, subst: &HashMap<String, TypeExpr>) -> TypeExpr {
        match ty {
            TypeExpr::Named(name) => {
                if let Some(replacement) = subst.get(name) {
                    replacement.clone()
                } else {
                    ty.clone()
                }
            }
            TypeExpr::Generic(name, args) => {
                let new_args: Vec<TypeExpr> = args.iter()
                    .map(|a| Self::substitute_type_expr(a, subst))
                    .collect();
                TypeExpr::Generic(name.clone(), new_args)
            }
            TypeExpr::Fn { params, ret } => {
                let new_params: Vec<TypeExpr> = params.iter()
                    .map(|p| Self::substitute_type_expr(p, subst))
                    .collect();
                let new_ret = Box::new(Self::substitute_type_expr(ret, subst));
                TypeExpr::Fn { params: new_params, ret: new_ret }
            }
        }
    }

    /// Monomorphize a generic function for the given concrete argument kinds.
    /// Returns the (FunctionValue, return ValKind) for the specialized version.
    fn monomorphize(
        &mut self,
        generic_name: &str,
        arg_kinds: &[ValKind],
        _current_fn: FunctionValue<'ctx>,
    ) -> Result<(FunctionValue<'ctx>, ValKind), CodeGenError> {
        let mangled = Self::mangle_generic_name(generic_name, arg_kinds);

        // Already monomorphized?
        if let Some((f, k)) = self.functions.get(&mangled) {
            return Ok((*f, k.clone()));
        }

        let generic_fn = self.generic_fns.get(generic_name).cloned().ok_or_else(|| {
            CodeGenError { line: None, msg: format!("no generic function '{}'", generic_name) }
        })?;

        // Build substitution map: type_param_name -> concrete TypeExpr
        let mut subst = HashMap::new();
        for (i, tp) in generic_fn.type_params.iter().enumerate() {
            // Match type params to arg kinds based on param positions
            // Find which argument position uses this type param
            let concrete = if let Some(kind) = self.find_concrete_for_type_param(&tp.name, &generic_fn.params, arg_kinds) {
                kind
            } else if i < arg_kinds.len() {
                // Fallback: positional mapping
                Self::valkind_to_type_expr(&arg_kinds[i])
            } else {
                TypeExpr::Named("Int".to_string()) // default fallback
            };
            subst.insert(tp.name.clone(), concrete);
        }

        // Create specialized FnDef
        let specialized_params: Vec<Param> = generic_fn.params.iter().map(|p| {
            Param {
                name: p.name.clone(),
                ty: Self::substitute_type_expr(&p.ty, &subst),
            }
        }).collect();

        let specialized_ret = generic_fn.ret_type.as_ref().map(|t| Self::substitute_type_expr(t, &subst));

        let specialized_fn = FnDef {
            name: mangled.clone(),
            type_params: vec![], // No longer generic
            params: specialized_params,
            ret_type: specialized_ret,
            body: generic_fn.body.clone(),
        };

        // Declare and compile the specialized function
        self.declare_function(&specialized_fn)?;

        // Save/restore state around compilation
        let saved_bb = self.builder.get_insert_block();
        let saved_vars = std::mem::take(&mut self.variables);
        let saved_break = self.break_target.take();
        let saved_continue = self.continue_target.take();

        self.compile_function(&specialized_fn)?;

        self.variables = saved_vars;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        // Restore builder position to where we were
        if let Some(bb) = saved_bb {
            self.builder.position_at_end(bb);
        }

        let (f, k) = self.functions.get(&mangled).unwrap().clone();
        Ok((f, k))
    }

    /// Find the concrete TypeExpr for a type parameter by scanning param declarations.
    fn find_concrete_for_type_param(
        &self,
        type_param: &str,
        params: &[Param],
        arg_kinds: &[ValKind],
    ) -> Option<TypeExpr> {
        for (i, param) in params.iter().enumerate() {
            if i >= arg_kinds.len() { break; }
            if let TypeExpr::Named(name) = &param.ty {
                if name == type_param {
                    return Some(Self::valkind_to_type_expr(&arg_kinds[i]));
                }
            }
        }
        None
    }

    fn call_result_to_value(
        &self,
        result: inkwell::values::CallSiteValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match result.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(val) => Ok(val),
            inkwell::values::ValueKind::Instruction(_) => {
                Ok(self.context.i64_type().const_int(0, false).into())
            }
        }
    }

    fn compile_pipeline_with_kind(
        &mut self,
        arg: &Expr,
        func_expr: &Expr,
        current_fn: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        // Desugar pipeline: if the target is a function name/call that isn't a known
        // function, convert to a method call on the piped argument instead.
        // e.g. `list | each(lambda)` becomes `list.each(lambda)`
        // e.g. `list | map(lambda)` becomes `list.map(lambda)`
        match func_expr {
            Expr::Ident(name) => {
                if self.functions.contains_key(name) || self.module.get_function(name).is_some() {
                    let arg_val = self.compile_expr(arg, current_fn)?;
                    let (called_fn, ret_kind) = self.resolve_function(name)?;
                    let result = bld!(self.builder.build_call(called_fn, &[arg_val.into()], "pipe"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ret_kind))
                } else if self.generic_fns.contains_key(name) {
                    let (arg_val, arg_kind) = self.compile_expr_with_kind(arg, current_fn)?;
                    let (called_fn, ret_kind) = self.monomorphize(name, &[arg_kind], current_fn)?;
                    let result = bld!(self.builder.build_call(called_fn, &[arg_val.into()], "pipe"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ret_kind))
                } else {
                    // Treat as method call: arg.name()
                    let method_call = Expr::MethodCall {
                        object: Box::new(arg.clone()),
                        method: name.clone(),
                        args: vec![],
                    };
                    self.compile_expr_with_kind(&method_call, current_fn)
                }
            }
            Expr::Call { func, args } => {
                let name = match func.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(CodeGenError { line: None, msg: "pipeline target must be a function".into() }),
                };
                if self.functions.contains_key(&name) || self.module.get_function(&name).is_some() {
                    let arg_val = self.compile_expr(arg, current_fn)?;
                    let (called_fn, ret_kind) = self.resolve_function(&name)?;

                    let mut compiled_args = vec![arg_val.into()];
                    for a in args {
                        compiled_args.push(self.compile_expr(a, current_fn)?.into());
                    }

                    let result = bld!(self.builder.build_call(called_fn, &compiled_args, "pipe"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ret_kind))
                } else if self.generic_fns.contains_key(&name) {
                    let (arg_val, arg_kind) = self.compile_expr_with_kind(arg, current_fn)?;
                    let mut compiled_args = vec![arg_val.into()];
                    let mut arg_kinds = vec![arg_kind];
                    for a in args {
                        let (v, k) = self.compile_expr_with_kind(a, current_fn)?;
                        compiled_args.push(v.into());
                        arg_kinds.push(k);
                    }
                    let (called_fn, ret_kind) = self.monomorphize(&name, &arg_kinds, current_fn)?;
                    let result = bld!(self.builder.build_call(called_fn, &compiled_args, "pipe"))?;
                    let val = self.call_result_to_value(result)?;
                    Ok((val, ret_kind))
                } else {
                    // Treat as method call: arg.name(args...)
                    let method_call = Expr::MethodCall {
                        object: Box::new(arg.clone()),
                        method: name.clone(),
                        args: args.clone(),
                    };
                    self.compile_expr_with_kind(&method_call, current_fn)
                }
            }
            Expr::Lambda { params, body } => {
                let arg_val = self.compile_expr(arg, current_fn)?;
                let lambda_fn = self.compile_lambda(params, body, current_fn)?;
                let lambda_name = lambda_fn.get_name().to_str().unwrap().to_string();

                let mut call_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = Vec::new();
                if self.lambda_captures.contains_key(&lambda_name) {
                    let env_ptr = self.build_captures_struct(&lambda_name)?;
                    call_args.push(env_ptr.into());
                }
                call_args.push(arg_val.into());

                let result = bld!(self.builder.build_call(lambda_fn, &call_args, "pipe"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            _ => Err(CodeGenError { line: None, msg: "unsupported pipeline target".into() }),
        }
    }

    fn compile_for_in(
        &mut self,
        var: &str,
        start_expr: &Expr,
        end_expr: &Expr,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();
        let start_val = self.compile_expr(start_expr, func)?.into_int_value();
        let end_val = self.compile_expr(end_expr, func)?.into_int_value();

        // Alloca for loop variable
        let var_alloca = bld!(self.builder.build_alloca(i64_type, var))?;
        bld!(self.builder.build_store(var_alloca, start_val))?;
        self.variables.insert(var.to_string(), (var_alloca, i64_type.into(), ValKind::Int, false));

        let cond_bb = self.context.append_basic_block(func, "for_cond");
        let body_bb = self.context.append_basic_block(func, "for_body");
        let inc_bb = self.context.append_basic_block(func, "for_inc");
        let end_bb = self.context.append_basic_block(func, "for_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        // Condition: i < end
        self.builder.position_at_end(cond_bb);
        let current = bld!(self.builder.build_load(i64_type, var_alloca, var))?.into_int_value();
        let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, current, end_val, "for_cmp"))?;
        bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

        // Body
        self.builder.position_at_end(body_bb);
        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(inc_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(inc_bb))?;
        }

        // Increment
        self.builder.position_at_end(inc_bb);
        let current = bld!(self.builder.build_load(i64_type, var_alloca, var))?.into_int_value();
        let next = bld!(self.builder.build_int_add(current, i64_type.const_int(1, false), "inc"))?;
        bld!(self.builder.build_store(var_alloca, next))?;
        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    fn compile_for_each(
        &mut self,
        var: &str,
        iterable: &Expr,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        // Check if the iterable is a map — if so, iterate over its keys
        let is_map = if let Expr::Ident(name) = iterable {
            self.map_value_kinds.contains_key(name)
        } else {
            false
        };

        if is_map {
            // For maps: get keys list and iterate over it
            let map_ptr = self.compile_expr(iterable, func)?.into_pointer_value();
            let keys_fn = self.module.get_function("ore_map_keys").unwrap();
            let keys_result = bld!(self.builder.build_call(keys_fn, &[map_ptr.into()], "keys"))?;
            let list_ptr = self.call_result_to_value(keys_result)?.into_pointer_value();
            return self.compile_for_each_over_list(var, list_ptr, ValKind::Str, body, func);
        }

        // Determine element kind from the iterable
        let elem_kind = if let Expr::Ident(name) = iterable {
            self.list_element_kinds.get(name).cloned().unwrap_or(ValKind::Int)
        } else {
            ValKind::Int
        };

        let list_ptr = self.compile_expr(iterable, func)?.into_pointer_value();

        self.compile_for_each_over_list(var, list_ptr, elem_kind, body, func)
    }

    fn compile_for_each_over_list(
        &mut self,
        var: &str,
        list_ptr: PointerValue<'ctx>,
        elem_kind: ValKind,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();

        // Get list length
        let list_len_fn = self.module.get_function("ore_list_len").unwrap();
        let len_result = bld!(self.builder.build_call(list_len_fn, &[list_ptr.into()], "len"))?;
        let len_val = self.call_result_to_value(len_result)?.into_int_value();

        // Index variable
        let idx_alloca = bld!(self.builder.build_alloca(i64_type, "idx"))?;
        bld!(self.builder.build_store(idx_alloca, i64_type.const_int(0, false)))?;

        // Element variable — use appropriate type based on element kind
        let (elem_alloca, elem_ty): (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>) = match &elem_kind {
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let alloca = bld!(self.builder.build_alloca(st, var))?;
                (alloca, st.into())
            }
            ValKind::Str => {
                let pt = self.context.ptr_type(inkwell::AddressSpace::default());
                let alloca = bld!(self.builder.build_alloca(pt, var))?;
                (alloca, pt.into())
            }
            _ => {
                let alloca = bld!(self.builder.build_alloca(i64_type, var))?;
                (alloca, i64_type.into())
            }
        };
        self.variables.insert(var.to_string(), (elem_alloca, elem_ty, elem_kind.clone(), false));

        let cond_bb = self.context.append_basic_block(func, "foreach_cond");
        let body_bb = self.context.append_basic_block(func, "foreach_body");
        let inc_bb = self.context.append_basic_block(func, "foreach_inc");
        let end_bb = self.context.append_basic_block(func, "foreach_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        // Condition: idx < len
        self.builder.position_at_end(cond_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, idx, len_val, "foreach_cmp"))?;
        bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

        // Body: load element, execute body
        self.builder.position_at_end(body_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let list_get_fn = self.module.get_function("ore_list_get").unwrap();
        let elem_result = bld!(self.builder.build_call(list_get_fn, &[list_ptr.into(), idx.into()], "elem"))?;
        let raw_val = self.call_result_to_value(elem_result)?;
        // For records, the i64 is a heap pointer — dereference to get the struct
        match &elem_kind {
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let ptr = bld!(self.builder.build_int_to_ptr(
                    raw_val.into_int_value(),
                    self.context.ptr_type(inkwell::AddressSpace::default()),
                    "i2p"
                ))?;
                let struct_val = bld!(self.builder.build_load(st, ptr, "rec_elem"))?;
                bld!(self.builder.build_store(elem_alloca, struct_val))?;
            }
            ValKind::Str => {
                let ptr = bld!(self.builder.build_int_to_ptr(
                    raw_val.into_int_value(),
                    self.context.ptr_type(inkwell::AddressSpace::default()),
                    "i2p"
                ))?;
                bld!(self.builder.build_store(elem_alloca, ptr))?;
            }
            _ => {
                bld!(self.builder.build_store(elem_alloca, raw_val))?;
            }
        }

        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(inc_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(inc_bb))?;
        }

        // Increment index
        self.builder.position_at_end(inc_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let next = bld!(self.builder.build_int_add(idx, i64_type.const_int(1, false), "inc"))?;
        bld!(self.builder.build_store(idx_alloca, next))?;
        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    fn compile_for_each_kv(
        &mut self,
        key_var: &str,
        val_var: &str,
        iterable: &Expr,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        // Detect if iterable is a map or a list
        let is_map = if let Expr::Ident(name) = iterable {
            self.map_value_kinds.contains_key(name)
        } else {
            false
        };

        if is_map {
            return self.compile_for_each_kv_map(key_var, val_var, iterable, body, func);
        }

        // List enumeration: for i, x in list
        let elem_kind = if let Expr::Ident(name) = iterable {
            self.list_element_kinds.get(name).cloned().unwrap_or(ValKind::Int)
        } else {
            ValKind::Int
        };

        let list_ptr = self.compile_expr(iterable, func)?.into_pointer_value();

        let list_len_fn = self.module.get_function("ore_list_len").unwrap();
        let len_result = bld!(self.builder.build_call(list_len_fn, &[list_ptr.into()], "len"))?;
        let len_val = self.call_result_to_value(len_result)?.into_int_value();

        // Index variable (exposed as key_var)
        let idx_alloca = bld!(self.builder.build_alloca(i64_type, key_var))?;
        bld!(self.builder.build_store(idx_alloca, i64_type.const_int(0, false)))?;
        self.variables.insert(key_var.to_string(), (idx_alloca, i64_type.into(), ValKind::Int, false));

        // Element variable
        let (elem_alloca, elem_ty): (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>) = match &elem_kind {
            ValKind::Str => {
                let alloca = bld!(self.builder.build_alloca(ptr_type, val_var))?;
                (alloca, ptr_type.into())
            }
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let alloca = bld!(self.builder.build_alloca(st, val_var))?;
                (alloca, st.into())
            }
            _ => {
                let alloca = bld!(self.builder.build_alloca(i64_type, val_var))?;
                (alloca, i64_type.into())
            }
        };
        self.variables.insert(val_var.to_string(), (elem_alloca, elem_ty, elem_kind.clone(), false));

        let cond_bb = self.context.append_basic_block(func, "forenum_cond");
        let body_bb = self.context.append_basic_block(func, "forenum_body");
        let inc_bb = self.context.append_basic_block(func, "forenum_inc");
        let end_bb = self.context.append_basic_block(func, "forenum_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(cond_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, idx, len_val, "forenum_cmp"))?;
        bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

        self.builder.position_at_end(body_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let list_get_fn = self.module.get_function("ore_list_get").unwrap();
        let elem_result = bld!(self.builder.build_call(list_get_fn, &[list_ptr.into(), idx.into()], "elem"))?;
        let raw_val = self.call_result_to_value(elem_result)?;
        match &elem_kind {
            ValKind::Record(name) => {
                let st = self.records[name].struct_type;
                let p = bld!(self.builder.build_int_to_ptr(raw_val.into_int_value(), ptr_type, "i2p"))?;
                let sv = bld!(self.builder.build_load(st, p, "rec_elem"))?;
                bld!(self.builder.build_store(elem_alloca, sv))?;
            }
            ValKind::Str => {
                let p = bld!(self.builder.build_int_to_ptr(raw_val.into_int_value(), ptr_type, "i2p"))?;
                bld!(self.builder.build_store(elem_alloca, p))?;
            }
            _ => {
                bld!(self.builder.build_store(elem_alloca, raw_val))?;
            }
        }

        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(inc_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(inc_bb))?;
        }

        self.builder.position_at_end(inc_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let next = bld!(self.builder.build_int_add(idx, i64_type.const_int(1, false), "inc"))?;
        bld!(self.builder.build_store(idx_alloca, next))?;
        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    fn compile_for_each_kv_map(
        &mut self,
        key_var: &str,
        val_var: &str,
        iterable: &Expr,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        let val_kind = if let Expr::Ident(name) = iterable {
            self.map_value_kinds.get(name).cloned().unwrap_or(ValKind::Int)
        } else {
            ValKind::Int
        };

        let map_ptr = self.compile_expr(iterable, func)?.into_pointer_value();

        let keys_fn = self.module.get_function("ore_map_keys").unwrap();
        let keys_result = bld!(self.builder.build_call(keys_fn, &[map_ptr.into()], "keys"))?;
        let keys_list = self.call_result_to_value(keys_result)?.into_pointer_value();

        let list_len_fn = self.module.get_function("ore_list_len").unwrap();
        let len_result = bld!(self.builder.build_call(list_len_fn, &[keys_list.into()], "len"))?;
        let len_val = self.call_result_to_value(len_result)?.into_int_value();

        let idx_alloca = bld!(self.builder.build_alloca(i64_type, "idx"))?;
        bld!(self.builder.build_store(idx_alloca, i64_type.const_int(0, false)))?;

        let key_alloca = bld!(self.builder.build_alloca(ptr_type, key_var))?;
        self.variables.insert(key_var.to_string(), (key_alloca, ptr_type.into(), ValKind::Str, false));

        let (val_alloca, val_ty): (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>) = match &val_kind {
            ValKind::Str => {
                let alloca = bld!(self.builder.build_alloca(ptr_type, val_var))?;
                (alloca, ptr_type.into())
            }
            _ => {
                let alloca = bld!(self.builder.build_alloca(i64_type, val_var))?;
                (alloca, i64_type.into())
            }
        };
        self.variables.insert(val_var.to_string(), (val_alloca, val_ty, val_kind.clone(), false));

        let cond_bb = self.context.append_basic_block(func, "forkv_cond");
        let body_bb = self.context.append_basic_block(func, "forkv_body");
        let inc_bb = self.context.append_basic_block(func, "forkv_inc");
        let end_bb = self.context.append_basic_block(func, "forkv_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(cond_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let cmp = bld!(self.builder.build_int_compare(IntPredicate::SLT, idx, len_val, "forkv_cmp"))?;
        bld!(self.builder.build_conditional_branch(cmp, body_bb, end_bb))?;

        self.builder.position_at_end(body_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();

        let list_get_fn = self.module.get_function("ore_list_get").unwrap();
        let key_result = bld!(self.builder.build_call(list_get_fn, &[keys_list.into(), idx.into()], "key_raw"))?;
        let key_raw = self.call_result_to_value(key_result)?.into_int_value();
        let key_ptr = bld!(self.builder.build_int_to_ptr(key_raw, ptr_type, "key_ptr"))?;
        bld!(self.builder.build_store(key_alloca, key_ptr))?;

        let map_get_fn = self.module.get_function("ore_map_get").unwrap();
        let val_result = bld!(self.builder.build_call(map_get_fn, &[map_ptr.into(), key_ptr.into()], "val_raw"))?;
        let val_raw = self.call_result_to_value(val_result)?;
        match &val_kind {
            ValKind::Str => {
                let val_ptr = bld!(self.builder.build_int_to_ptr(val_raw.into_int_value(), ptr_type, "val_ptr"))?;
                bld!(self.builder.build_store(val_alloca, val_ptr))?;
            }
            _ => {
                bld!(self.builder.build_store(val_alloca, val_raw))?;
            }
        }

        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(inc_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(inc_bb))?;
        }

        self.builder.position_at_end(inc_bb);
        let idx = bld!(self.builder.build_load(i64_type, idx_alloca, "idx"))?.into_int_value();
        let next = bld!(self.builder.build_int_add(idx, i64_type.const_int(1, false), "inc"))?;
        bld!(self.builder.build_store(idx_alloca, next))?;
        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    fn compile_while(
        &mut self,
        cond_expr: &Expr,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let cond_bb = self.context.append_basic_block(func, "while_cond");
        let body_bb = self.context.append_basic_block(func, "while_body");
        let end_bb = self.context.append_basic_block(func, "while_end");

        bld!(self.builder.build_unconditional_branch(cond_bb))?;

        self.builder.position_at_end(cond_bb);
        let cond_val = self.compile_expr(cond_expr, func)?.into_int_value();
        bld!(self.builder.build_conditional_branch(cond_val, body_bb, end_bb))?;

        self.builder.position_at_end(body_bb);
        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(cond_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(cond_bb))?;
        }

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    fn compile_loop(
        &mut self,
        body: &Block,
        func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        let body_bb = self.context.append_basic_block(func, "loop_body");
        let end_bb = self.context.append_basic_block(func, "loop_end");

        bld!(self.builder.build_unconditional_branch(body_bb))?;

        self.builder.position_at_end(body_bb);
        let saved_break = self.break_target;
        let saved_continue = self.continue_target;
        self.break_target = Some(end_bb);
        self.continue_target = Some(body_bb);
        self.compile_block_stmts(body, func)?;
        self.break_target = saved_break;
        self.continue_target = saved_continue;
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(body_bb))?;
        }

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    fn compile_if_else_with_kind(
        &mut self,
        cond: &Expr,
        then_block: &Block,
        else_block: Option<&Block>,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let cond_val = self.compile_expr(cond, func)?;
        let cond_int = match cond_val {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(CodeGenError { line: None, msg: "condition must be a boolean".into() }),
        };

        let i64_type = self.context.i64_type();
        let then_bb = self.context.append_basic_block(func, "then");
        let else_bb = self.context.append_basic_block(func, "else");
        let merge_bb = self.context.append_basic_block(func, "merge");

        bld!(self.builder.build_conditional_branch(cond_int, then_bb, else_bb))?;

        // Compile then branch — get last expression's value and kind
        self.builder.position_at_end(then_bb);
        let (then_val, then_kind) = self.compile_block_stmts_with_kind(then_block, func)?;
        let then_val = then_val.unwrap_or_else(|| i64_type.const_int(0, false).into());
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(merge_bb))?;
        }
        let then_end_bb = self.builder.get_insert_block().unwrap();

        // Compile else branch
        self.builder.position_at_end(else_bb);
        let (else_val, _else_kind) = if let Some(eb) = else_block {
            let (v, k) = self.compile_block_stmts_with_kind(eb, func)?;
            (v.unwrap_or_else(|| i64_type.const_int(0, false).into()), k)
        } else {
            (i64_type.const_int(0, false).into(), ValKind::Int)
        };

        // Ensure both branches produce the same type for the phi node
        // If types differ, coerce both to i64
        let (then_coerced, else_coerced) = if then_val.get_type() != else_val.get_type() {
            let t = self.coerce_to_i64(then_val, &then_kind)?;
            // Need to position back in else block for the coercion
            // Actually, coercions were already done in the right blocks
            // We need to handle this differently
            // For now, just use i64 for both
            self.builder.position_at_end(then_end_bb);
            // Re-do: we need to insert coercions before the branch
            // This is getting complex. Let's just use an alloca approach instead.
            (t.into(), else_val) // May fail if types differ
        } else {
            (then_val, else_val)
        };

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(merge_bb))?;
        }
        let else_end_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(merge_bb);
        let phi = bld!(self.builder.build_phi(then_coerced.get_type(), "ifval"))?;
        phi.add_incoming(&[(&then_coerced, then_end_bb), (&else_coerced, else_end_bb)]);
        Ok((phi.as_basic_value(), then_kind))
    }

    fn compile_colon_match_with_kind(
        &mut self,
        cond: &Expr,
        then_expr: &Expr,
        else_expr: Option<&Expr>,
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let cond_val = self.compile_expr(cond, func)?;
        let cond_int = match cond_val {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(CodeGenError { line: None, msg: "condition must be a boolean".into() }),
        };

        let then_bb = self.context.append_basic_block(func, "cthen");
        let else_bb = self.context.append_basic_block(func, "celse");
        let merge_bb = self.context.append_basic_block(func, "cmerge");

        bld!(self.builder.build_conditional_branch(cond_int, then_bb, else_bb))?;

        self.builder.position_at_end(then_bb);
        let (then_val, then_kind) = self.compile_expr_with_kind(then_expr, func)?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;
        let then_end_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(else_bb);
        let (else_val, _) = if let Some(e) = else_expr {
            self.compile_expr_with_kind(e, func)?
        } else {
            (self.context.i64_type().const_int(0, false).into(), ValKind::Int)
        };
        bld!(self.builder.build_unconditional_branch(merge_bb))?;
        let else_end_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(merge_bb);
        let phi = bld!(self.builder.build_phi(then_val.get_type(), "cval"))?;
        phi.add_incoming(&[(&then_val, then_end_bb), (&else_val, else_end_bb)]);
        Ok((phi.as_basic_value(), then_kind))
    }

    fn compile_lambda(
        &mut self,
        params: &[String],
        body: &Expr,
        _parent_fn: FunctionValue<'ctx>,
    ) -> Result<FunctionValue<'ctx>, CodeGenError> {
        let name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        let i64_type = self.context.i64_type();
        let ptr_type = self.ptr_type();

        // Detect free variables (captures) — identifiers in body not in params
        let bound: HashSet<String> = params.iter().cloned().collect();
        let free_vars = find_free_vars(body, &bound);

        // Filter to only variables that exist in the current scope
        let mut capture_names = Vec::new();
        let mut capture_types = Vec::new();
        let mut capture_kinds = Vec::new();
        for fv in &free_vars {
            if let Some((_ptr, ty, kind, _)) = self.variables.get(fv) {
                capture_names.push(fv.clone());
                capture_types.push(*ty);
                capture_kinds.push(kind.clone());
            }
        }
        let has_captures = !capture_names.is_empty();

        // Build function signature: if captures, first param is env_ptr
        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = Vec::new();
        if has_captures {
            param_types.push(ptr_type.into()); // env_ptr
        }
        for _ in params {
            param_types.push(i64_type.into());
        }
        let fn_type = i64_type.fn_type(&param_types, false);
        let lambda_fn = self.module.add_function(&name, fn_type, None);

        // Build the captures struct type and store CaptureInfo if needed
        let captures_struct_type = if has_captures {
            let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> = capture_types.clone();
            let st = self.context.struct_type(&field_types, false);
            self.lambda_captures.insert(name.clone(), CaptureInfo {
                struct_type: st,
                names: capture_names.clone(),
                types: capture_types.clone(),
                kinds: capture_kinds.clone(),
            });
            Some(st)
        } else {
            None
        };

        // Save current state
        let saved_vars = self.variables.clone();
        let saved_block = self.builder.get_insert_block();

        // Build lambda body
        let entry = self.context.append_basic_block(lambda_fn, "entry");
        self.builder.position_at_end(entry);
        self.variables.clear();

        // If we have captures, extract them from the env_ptr (first param)
        if has_captures {
            let env_ptr = lambda_fn.get_nth_param(0).unwrap().into_pointer_value();
            let st = captures_struct_type.unwrap();
            for (i, cap_name) in capture_names.iter().enumerate() {
                let field_ptr = bld!(self.builder.build_struct_gep(
                    st, env_ptr, i as u32, &format!("cap_{}", cap_name)
                ))?;
                let field_ty = capture_types[i];
                let val = bld!(self.builder.build_load(field_ty, field_ptr, cap_name))?;
                let alloca = bld!(self.builder.build_alloca(field_ty, cap_name))?;
                bld!(self.builder.build_store(alloca, val))?;
                self.variables.insert(cap_name.clone(), (alloca, field_ty, capture_kinds[i].clone(), false));
            }
        }

        // Bind lambda parameters (offset by 1 if captures exist)
        let param_offset: u32 = if has_captures { 1 } else { 0 };
        for (i, param_name) in params.iter().enumerate() {
            let val = lambda_fn.get_nth_param(i as u32 + param_offset).unwrap();
            let ty = val.get_type();
            let alloca = bld!(self.builder.build_alloca(ty, param_name))?;
            bld!(self.builder.build_store(alloca, val))?;
            self.variables.insert(param_name.clone(), (alloca, ty, ValKind::Int, false));
        }

        let (result, kind) = self.compile_expr_with_kind(body, lambda_fn)?;

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            // Coerce result to i64 if needed (e.g. bool i1 from comparisons)
            let ret_val = match kind {
                ValKind::Bool => {
                    bld!(self.builder.build_int_z_extend(
                        result.into_int_value(),
                        self.context.i64_type(),
                        "bool_to_i64"
                    ))?.into()
                }
                _ => result,
            };
            bld!(self.builder.build_return(Some(&ret_val)))?;
        }

        // Restore state
        self.variables = saved_vars;
        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }

        self.functions.insert(name, (lambda_fn, ValKind::Int));
        Ok(lambda_fn)
    }

    /// Build the captures struct on the stack and fill it with current variable values.
    /// Returns a pointer to the alloca'd struct.
    fn build_captures_struct(
        &mut self,
        lambda_name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenError> {
        let cap_info = self.lambda_captures.get(lambda_name).ok_or_else(|| CodeGenError {
            line: None, msg: format!("no capture info for lambda '{}'", lambda_name),
        })?;
        let struct_type = cap_info.struct_type;
        let names = cap_info.names.clone();
        let types = cap_info.types.clone();

        let alloca = bld!(self.builder.build_alloca(struct_type, "captures"))?;

        for (i, cap_name) in names.iter().enumerate() {
            let (var_ptr, var_ty, _kind, _) = self.variables.get(cap_name).ok_or_else(|| CodeGenError {
                line: None, msg: format!("captured variable '{}' not found in scope", cap_name),
            })?;
            let val = bld!(self.builder.build_load(*var_ty, *var_ptr, cap_name))?;
            let field_ptr = bld!(self.builder.build_struct_gep(
                struct_type, alloca, i as u32, &format!("cap_store_{}", cap_name)
            ))?;
            // If types don't exactly match (e.g. i64 vs i64), just store directly
            let _ = types[i]; // ensure we have the type
            bld!(self.builder.build_store(field_ptr, val))?;
        }

        Ok(alloca)
    }

    fn compile_binop(
        &mut self,
        op: BinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match (lhs, rhs) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                if l.get_type().get_bit_width() == 1 && r.get_type().get_bit_width() == 1 {
                    return self.compile_bool_binop(op, l, r);
                }
                let result: IntValue<'ctx> = match op {
                    BinOp::Add => bld!(self.builder.build_int_add(l, r, "add")),
                    BinOp::Sub => bld!(self.builder.build_int_sub(l, r, "sub")),
                    BinOp::Mul => bld!(self.builder.build_int_mul(l, r, "mul")),
                    BinOp::Div => bld!(self.builder.build_int_signed_div(l, r, "div")),
                    BinOp::Mod => bld!(self.builder.build_int_signed_rem(l, r, "rem")),
                    BinOp::Eq => bld!(self.builder.build_int_compare(IntPredicate::EQ, l, r, "eq")),
                    BinOp::NotEq => bld!(self.builder.build_int_compare(IntPredicate::NE, l, r, "ne")),
                    BinOp::Lt => bld!(self.builder.build_int_compare(IntPredicate::SLT, l, r, "lt")),
                    BinOp::Gt => bld!(self.builder.build_int_compare(IntPredicate::SGT, l, r, "gt")),
                    BinOp::LtEq => bld!(self.builder.build_int_compare(IntPredicate::SLE, l, r, "le")),
                    BinOp::GtEq => bld!(self.builder.build_int_compare(IntPredicate::SGE, l, r, "ge")),
                    BinOp::And => bld!(self.builder.build_and(l, r, "and")),
                    BinOp::Or => bld!(self.builder.build_or(l, r, "or")),
                    BinOp::Pipe => unreachable!("pipe handled separately"),
                }?;
                Ok(result.into())
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                use inkwell::FloatPredicate;
                let result: BasicValueEnum<'ctx> = match op {
                    BinOp::Add => bld!(self.builder.build_float_add(l, r, "fadd"))?.into(),
                    BinOp::Sub => bld!(self.builder.build_float_sub(l, r, "fsub"))?.into(),
                    BinOp::Mul => bld!(self.builder.build_float_mul(l, r, "fmul"))?.into(),
                    BinOp::Div => bld!(self.builder.build_float_div(l, r, "fdiv"))?.into(),
                    BinOp::Lt => bld!(self.builder.build_float_compare(FloatPredicate::OLT, l, r, "flt"))?.into(),
                    BinOp::Gt => bld!(self.builder.build_float_compare(FloatPredicate::OGT, l, r, "fgt"))?.into(),
                    BinOp::Eq => bld!(self.builder.build_float_compare(FloatPredicate::OEQ, l, r, "feq"))?.into(),
                    _ => return Err(CodeGenError { line: None, msg: format!("unsupported float op {:?}", op) }),
                };
                Ok(result)
            }
            (BasicValueEnum::PointerValue(l), BasicValueEnum::PointerValue(r)) => {
                // String comparison via ore_str_eq
                match op {
                    BinOp::Eq => {
                        let rt = self.module.get_function("ore_str_eq").unwrap();
                        let result = bld!(self.builder.build_call(rt, &[l.into(), r.into()], "seq"))?;
                        let i8_val = self.call_result_to_value(result)?.into_int_value();
                        let bool_val = bld!(self.builder.build_int_compare(
                            IntPredicate::NE, i8_val,
                            self.context.i8_type().const_int(0, false), "tobool"
                        ))?;
                        Ok(bool_val.into())
                    }
                    BinOp::NotEq => {
                        let rt = self.module.get_function("ore_str_eq").unwrap();
                        let result = bld!(self.builder.build_call(rt, &[l.into(), r.into()], "seq"))?;
                        let i8_val = self.call_result_to_value(result)?.into_int_value();
                        let bool_val = bld!(self.builder.build_int_compare(
                            IntPredicate::EQ, i8_val,
                            self.context.i8_type().const_int(0, false), "tobool"
                        ))?;
                        Ok(bool_val.into())
                    }
                    BinOp::Add => {
                        // String concatenation
                        let rt = self.module.get_function("ore_str_concat").unwrap();
                        let result = bld!(self.builder.build_call(rt, &[l.into(), r.into()], "sconcat"))?;
                        let val = self.call_result_to_value(result)?;
                        Ok(val)
                    }
                    BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                        // String ordering via ore_str_cmp
                        let rt = self.module.get_function("ore_str_cmp").unwrap();
                        let result = bld!(self.builder.build_call(rt, &[l.into(), r.into()], "scmp"))?;
                        let cmp_val = self.call_result_to_value(result)?.into_int_value();
                        let zero = self.context.i64_type().const_int(0, false);
                        let pred = match op {
                            BinOp::Lt => IntPredicate::SLT,
                            BinOp::Gt => IntPredicate::SGT,
                            BinOp::LtEq => IntPredicate::SLE,
                            BinOp::GtEq => IntPredicate::SGE,
                            _ => unreachable!(),
                        };
                        let bool_val = bld!(self.builder.build_int_compare(pred, cmp_val, zero, "scmpres"))?;
                        Ok(bool_val.into())
                    }
                    _ => Err(CodeGenError { line: None, msg: format!("unsupported pointer op {:?}", op) }),
                }
            }
            _ => Err(CodeGenError { line: None, msg: "type mismatch in binary operation".into() }),
        }
    }

    fn compile_bool_binop(
        &mut self,
        op: BinOp,
        l: IntValue<'ctx>,
        r: IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let result = match op {
            BinOp::And => bld!(self.builder.build_and(l, r, "band")),
            BinOp::Or => bld!(self.builder.build_or(l, r, "bor")),
            BinOp::Eq => bld!(self.builder.build_int_compare(IntPredicate::EQ, l, r, "beq")),
            BinOp::NotEq => bld!(self.builder.build_int_compare(IntPredicate::NE, l, r, "bne")),
            _ => return Err(CodeGenError { line: None, msg: format!("unsupported bool op {:?}", op) }),
        }?;
        Ok(result.into())
    }
}
