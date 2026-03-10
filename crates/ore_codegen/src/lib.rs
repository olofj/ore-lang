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
            for stmt in &then_block.stmts {
                collect_free_vars_stmt(stmt, bound, free, seen);
            }
            if let Some(eb) = else_block {
                for stmt in &eb.stmts {
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
        Expr::Index { object, index } => {
            collect_free_vars(object, bound, free, seen);
            collect_free_vars(index, bound, free, seen);
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
        Stmt::Return(None) | Stmt::Break => {}
        Stmt::Spawn(e) => collect_free_vars(e, bound, free, seen),
        Stmt::ForIn { start, end, body, .. } => {
            collect_free_vars(start, bound, free, seen);
            collect_free_vars(end, bound, free, seen);
            for s in &body.stmts {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::While { cond, body } => {
            collect_free_vars(cond, bound, free, seen);
            for s in &body.stmts {
                collect_free_vars_stmt(s, bound, free, seen);
            }
        }
        Stmt::Loop { body } => {
            for s in &body.stmts {
                collect_free_vars_stmt(s, bound, free, seen);
            }
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
    str_counter: u32,
    lambda_counter: u32,
    /// Maps lambda function name -> capture info (only for closures with captures)
    lambda_captures: HashMap<String, CaptureInfo<'ctx>>,
    /// Maps type name -> list of method names (for method call resolution)
    method_map: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct CodeGenError {
    pub msg: String,
}

impl std::fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "codegen error: {}", self.msg)
    }
}

macro_rules! bld {
    ($expr:expr) => {
        $expr.map_err(|e| CodeGenError { msg: e.to_string() })
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
            str_counter: 0,
            lambda_counter: 0,
            lambda_captures: HashMap::new(),
            method_map: HashMap::new(),
        }
    }

    fn ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(inkwell::AddressSpace::default())
    }

    /// Built-in Option type: { i8, i64 } where tag=0 is None, tag=1 is Some
    fn option_type(&self) -> inkwell::types::StructType<'ctx> {
        self.context.struct_type(
            &[self.context.i8_type().into(), self.context.i64_type().into()],
            false,
        )
    }

    /// Built-in Result type: { i8, i64 } where tag=0 is Ok, tag=1 is Err
    fn result_type(&self) -> inkwell::types::StructType<'ctx> {
        self.context.struct_type(
            &[self.context.i8_type().into(), self.context.i64_type().into()],
            false,
        )
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

        // Declare regular functions
        for item in &program.items {
            if let Item::FnDef(f) = item {
                self.declare_function(f)?;
            }
        }

        // Declare impl block methods (mangled names: TypeName_methodName)
        for item in &program.items {
            if let Item::ImplBlock { type_name, methods } = item {
                let mut method_names = Vec::new();
                for method in methods {
                    let mangled_name = format!("{}_{}", type_name, method.name);
                    method_names.push(method.name.clone());
                    // Create a copy of the FnDef with the mangled name for declaration
                    let mangled_fn = FnDef {
                        name: mangled_name,
                        type_params: method.type_params.clone(),
                        params: method.params.clone(),
                        ret_type: method.ret_type.clone(),
                        body: method.body.clone(),
                    };
                    self.declare_function(&mangled_fn)?;
                }
                self.method_map.insert(type_name.clone(), method_names);
            }
        }

        // Compile regular functions
        for item in &program.items {
            if let Item::FnDef(f) = item {
                self.compile_function(f)?;
            }
        }

        // Compile impl block methods
        for item in &program.items {
            if let Item::ImplBlock { type_name, methods } = item {
                for method in methods {
                    let mangled_name = format!("{}_{}", type_name, method.name);
                    let mangled_fn = FnDef {
                        name: mangled_name,
                        type_params: method.type_params.clone(),
                        params: method.params.clone(),
                        ret_type: method.ret_type.clone(),
                        body: method.body.clone(),
                    };
                    self.compile_function(&mangled_fn)?;
                }
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
        let void_type = self.context.void_type();
        let ptr_type = self.ptr_type();

        let ext = Some(inkwell::module::Linkage::External);

        // ore_print_int(i64)
        self.module.add_function("ore_print_int", void_type.fn_type(&[i64_type.into()], false), ext);
        // ore_print_bool(i8)
        self.module.add_function("ore_print_bool", void_type.fn_type(&[i8_type.into()], false), ext);
        // ore_print_float(f64)
        let f64_type = self.context.f64_type();
        self.module.add_function("ore_print_float", void_type.fn_type(&[f64_type.into()], false), ext);
        // ore_str_new(ptr, u32) -> ptr
        self.module.add_function("ore_str_new", ptr_type.fn_type(&[ptr_type.into(), i32_type.into()], false), ext);
        // ore_str_concat(ptr, ptr) -> ptr
        self.module.add_function("ore_str_concat", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // ore_str_print(ptr)
        self.module.add_function("ore_str_print", void_type.fn_type(&[ptr_type.into()], false), ext);
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
        // ore_list_print(ptr)
        self.module.add_function("ore_list_print", void_type.fn_type(&[ptr_type.into()], false), ext);
        // ore_list_map(ptr, fn_ptr) -> ptr
        self.module.add_function("ore_list_map", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_filter(ptr, fn_ptr) -> ptr
        self.module.add_function("ore_list_filter", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
        // ore_list_each(ptr, fn_ptr)
        self.module.add_function("ore_list_each", void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), ext);
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
            TypeExpr::Generic(name, _args) => {
                // For now, treat generic types by their base name
                match name.as_str() {
                    "List" => ValKind::List,
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
            ValKind::List => self.ptr_type().into(),
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
            ValKind::List => self.ptr_type().into(),
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

        let mut last_val: Option<BasicValueEnum<'ctx>> = None;
        for stmt in &fndef.body.stmts {
            last_val = self.compile_stmt(stmt, func)?;
        }

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
                        msg: format!("function '{}' must return a value", fndef.name),
                    });
                }
            } else {
                bld!(self.builder.build_return(None))?;
            }
        }

        Ok(())
    }

    fn compile_stmt(
        &mut self,
        stmt: &Stmt,
        func: FunctionValue<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenError> {
        match stmt {
            Stmt::Let { name, mutable, value } => {
                let (val, kind) = self.compile_expr_with_kind(value, func)?;
                let ty = val.get_type();
                let alloca = bld!(self.builder.build_alloca(ty, name))?;
                bld!(self.builder.build_store(alloca, val))?;
                self.variables.insert(name.clone(), (alloca, ty, kind, *mutable));
                Ok(None)
            }
            Stmt::Assign { name, value } => {
                let (val, _kind) = self.compile_expr_with_kind(value, func)?;
                let (ptr, _, _, is_mut) = self.variables.get(name).ok_or_else(|| CodeGenError {
                    msg: format!("undefined variable '{}'", name),
                })?;
                if !is_mut {
                    return Err(CodeGenError {
                        msg: format!("cannot assign to immutable variable '{}'", name),
                    });
                }
                bld!(self.builder.build_store(*ptr, val))?;
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
                    return Err(CodeGenError { msg: "break outside of loop".into() });
                }
                Ok(None)
            }
            Stmt::Spawn(expr) => {
                // spawn only works with zero-argument function calls
                match expr {
                    Expr::Call { func: callee, args } if args.is_empty() => {
                        let name = match callee.as_ref() {
                            Expr::Ident(n) => n.clone(),
                            _ => return Err(CodeGenError { msg: "spawn requires a named function call".into() }),
                        };
                        let (target_fn, _) = self.resolve_function(&name)?;
                        let fn_ptr = target_fn.as_global_value().as_pointer_value();
                        let ore_spawn = self.module.get_function("ore_spawn").unwrap();
                        bld!(self.builder.build_call(ore_spawn, &[fn_ptr.into()], ""))?;
                        Ok(None)
                    }
                    _ => Err(CodeGenError { msg: "spawn requires a zero-argument function call".into() }),
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
                    msg: format!("undefined variable '{}'", name),
                })?;
                let val = bld!(self.builder.build_load(*ty, *ptr, name))?;
                Ok((val, kind.clone()))
            }
            Expr::BinOp { op, left, right } => {
                if *op == BinOp::Pipe {
                    return self.compile_pipeline_with_kind(left, right, func);
                }
                let (lhs, lk) = self.compile_expr_with_kind(left, func)?;
                let (rhs, _rk) = self.compile_expr_with_kind(right, func)?;
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
                    _ => Err(CodeGenError { msg: "cannot negate this type".into() }),
                }
            }
            Expr::UnaryNot(inner) => {
                let val = self.compile_expr(inner, func)?;
                match val {
                    BasicValueEnum::IntValue(v) => {
                        Ok((bld!(self.builder.build_not(v, "not"))?.into(), ValKind::Bool))
                    }
                    _ => Err(CodeGenError { msg: "cannot apply 'not' to this type".into() }),
                }
            }
            Expr::Print(inner) => {
                let (val, kind) = self.compile_expr_with_kind(inner, func)?;
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
                    _ => return Err(CodeGenError { msg: "only named function calls supported".into() }),
                };
                let (called_fn, ret_kind) = self.resolve_function(&name)?;

                let mut compiled_args = Vec::new();
                for arg in args {
                    compiled_args.push(self.compile_expr(arg, func)?.into());
                }

                let result = bld!(self.builder.build_call(called_fn, &compiled_args, "call"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ret_kind))
            }
            Expr::IfElse { cond, then_block, else_block } => {
                let val = self.compile_if_else(cond, then_block, else_block.as_ref(), func)?;
                // TODO: track kind properly through branches
                Ok((val, ValKind::Int))
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
            Expr::Index { object, index } => {
                self.compile_index(object, index, func)
            }
            Expr::OptionNone => {
                let opt_ty = self.option_type();
                let alloca = bld!(self.builder.build_alloca(opt_ty, "opt_none"))?;
                let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
                let tag_val = self.context.i8_type().const_int(0, false);
                bld!(self.builder.build_store(tag_ptr, tag_val))?;
                let result = bld!(self.builder.build_load(opt_ty, alloca, "none_val"))?;
                Ok((result, ValKind::Option))
            }
            Expr::OptionSome(inner) => {
                let (val, _kind) = self.compile_expr_with_kind(inner, func)?;
                let opt_ty = self.option_type();
                let alloca = bld!(self.builder.build_alloca(opt_ty, "opt_some"))?;
                let tag_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 0, "tag_ptr"))?;
                let tag_val = self.context.i8_type().const_int(1, false);
                bld!(self.builder.build_store(tag_ptr, tag_val))?;
                let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 1, "val_ptr"))?;
                // Convert the inner value to i64 for storage
                let i64_val = match val {
                    BasicValueEnum::IntValue(v) => {
                        if v.get_type().get_bit_width() < 64 {
                            bld!(self.builder.build_int_z_extend(v, self.context.i64_type(), "zext"))?
                        } else {
                            v
                        }
                    }
                    _ => val.into_int_value(),
                };
                bld!(self.builder.build_store(val_ptr, i64_val))?;
                let result = bld!(self.builder.build_load(opt_ty, alloca, "some_val"))?;
                Ok((result, ValKind::Option))
            }
            Expr::ResultOk(inner) => {
                let (val, _kind) = self.compile_expr_with_kind(inner, func)?;
                let res_ty = self.result_type();
                let alloca = bld!(self.builder.build_alloca(res_ty, "res_ok"))?;
                let tag_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 0, "tag_ptr"))?;
                let tag_val = self.context.i8_type().const_int(0, false); // Ok = 0
                bld!(self.builder.build_store(tag_ptr, tag_val))?;
                let val_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 1, "val_ptr"))?;
                let i64_val = match val {
                    BasicValueEnum::IntValue(v) => {
                        if v.get_type().get_bit_width() < 64 {
                            bld!(self.builder.build_int_z_extend(v, self.context.i64_type(), "zext"))?
                        } else {
                            v
                        }
                    }
                    _ => val.into_int_value(),
                };
                bld!(self.builder.build_store(val_ptr, i64_val))?;
                let result = bld!(self.builder.build_load(res_ty, alloca, "ok_val"))?;
                Ok((result, ValKind::Result))
            }
            Expr::ResultErr(inner) => {
                let (val, _kind) = self.compile_expr_with_kind(inner, func)?;
                let res_ty = self.result_type();
                let alloca = bld!(self.builder.build_alloca(res_ty, "res_err"))?;
                let tag_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 0, "tag_ptr"))?;
                let tag_val = self.context.i8_type().const_int(1, false); // Err = 1
                bld!(self.builder.build_store(tag_ptr, tag_val))?;
                let val_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 1, "val_ptr"))?;
                let i64_val = match val {
                    BasicValueEnum::IntValue(v) => {
                        if v.get_type().get_bit_width() < 64 {
                            bld!(self.builder.build_int_z_extend(v, self.context.i64_type(), "zext"))?
                        } else {
                            v
                        }
                    }
                    _ => val.into_int_value(),
                };
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
                let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, alloca, 1, "val_ptr"))?;
                let extracted = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, "unwrapped"))?;
                Ok((extracted, ValKind::Int))
            }
            Expr::Break => {
                if let Some(target) = self.break_target {
                    bld!(self.builder.build_unconditional_branch(target))?;
                } else {
                    return Err(CodeGenError { msg: "break outside of loop".into() });
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
            msg: format!("undefined type '{}'", type_name),
        })?;
        let struct_type = info.struct_type;
        let field_names = info.field_names.clone();

        let alloca = bld!(self.builder.build_alloca(struct_type, type_name))?;

        for (name, expr) in fields {
            let idx = field_names.iter().position(|n| n == name).ok_or_else(|| CodeGenError {
                msg: format!("unknown field '{}' on type '{}'", name, type_name),
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
            _ => return Err(CodeGenError { msg: "field access on non-record type".into() }),
        };

        let info = self.records.get(&type_name).ok_or_else(|| CodeGenError {
            msg: format!("undefined type '{}'", type_name),
        })?;
        let struct_type = info.struct_type;
        let idx = info.field_names.iter().position(|n| n == field).ok_or_else(|| CodeGenError {
            msg: format!("unknown field '{}' on type '{}'", field, type_name),
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

        let type_name = match &obj_kind {
            ValKind::Record(name) => name.clone(),
            _ => return Err(CodeGenError { msg: format!("method call on unsupported type: {:?}", obj_kind) }),
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
                    return Err(CodeGenError { msg: "push takes exactly 1 argument".into() });
                }
                let arg = self.compile_expr(&args[0], func)?;
                let list_push = self.module.get_function("ore_list_push").unwrap();
                bld!(self.builder.build_call(list_push, &[list_val.into(), arg.into()], ""))?;
                Ok((list_val, ValKind::List))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(CodeGenError { msg: "get takes exactly 1 argument".into() });
                }
                let idx = self.compile_expr(&args[0], func)?;
                let list_get = self.module.get_function("ore_list_get").unwrap();
                let result = bld!(self.builder.build_call(list_get, &[list_val.into(), idx.into()], "get"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            "map" | "filter" => {
                if args.len() != 1 {
                    return Err(CodeGenError { msg: format!("{} takes exactly 1 argument", method) });
                }
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { msg: format!("{} argument must be a function", method) }),
                };
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let runtime_fn_name = format!("ore_list_{}", method);
                let runtime_fn = self.module.get_function(&runtime_fn_name).unwrap();
                let result = bld!(self.builder.build_call(
                    runtime_fn,
                    &[list_val.into(), fn_ptr.into()],
                    method
                ))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::List))
            }
            "each" => {
                if args.len() != 1 {
                    return Err(CodeGenError { msg: "each takes exactly 1 argument".into() });
                }
                let lambda_fn = match &args[0] {
                    Expr::Lambda { params, body } => {
                        self.compile_lambda(params, body, func)?
                    }
                    Expr::Ident(name) => {
                        let (f, _) = self.resolve_function(name)?;
                        f
                    }
                    _ => return Err(CodeGenError { msg: "each argument must be a function".into() }),
                };
                let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
                let runtime_fn = self.module.get_function("ore_list_each").unwrap();
                bld!(self.builder.build_call(
                    runtime_fn,
                    &[list_val.into(), fn_ptr.into()],
                    ""
                ))?;
                Ok((self.context.i64_type().const_int(0, false).into(), ValKind::Void))
            }
            _ => Err(CodeGenError { msg: format!("unknown list method '{}'", method) }),
        }
    }

    fn compile_variant_construct(
        &mut self,
        variant_name: &str,
        fields: &[(String, Expr)],
        func: FunctionValue<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, ValKind), CodeGenError> {
        let enum_name = self.variant_to_enum.get(variant_name).ok_or_else(|| CodeGenError {
            msg: format!("unknown variant '{}'", variant_name),
        })?.clone();

        let enum_info = self.enums.get(&enum_name).ok_or_else(|| CodeGenError {
            msg: format!("undefined enum '{}'", enum_name),
        })?;
        let enum_type = enum_info.enum_type;

        // Find the variant
        let variant = enum_info.variants.iter().find(|v| v.name == variant_name).ok_or_else(|| CodeGenError {
            msg: format!("unknown variant '{}'", variant_name),
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
                msg: format!("unknown field '{}' on variant '{}'", name, variant_name),
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

        let enum_name = match &subject_kind {
            ValKind::Enum(name) => name.clone(),
            _ => return Err(CodeGenError { msg: "match subject must be an enum type".into() }),
        };

        let enum_info = self.enums.get(&enum_name).ok_or_else(|| CodeGenError {
            msg: format!("undefined enum '{}'", enum_name),
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
                        msg: format!("unknown variant '{}' in match", name),
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
                        _ => return Err(CodeGenError { msg: format!("unknown Option variant '{}'", name) }),
                    };

                    let case_bb = self.context.append_basic_block(func, &format!("opt_{}", name));
                    let tag_const = self.context.i8_type().const_int(vtag as u64, false);
                    case_blocks.push((tag_const, case_bb));

                    self.builder.position_at_end(case_bb);
                    let saved_vars = self.variables.clone();

                    // If Some, bind the payload
                    if vtag == 1 && !bindings.is_empty() {
                        let val_ptr = bld!(self.builder.build_struct_gep(opt_ty, subject_alloca, 1, "val_ptr"))?;
                        let payload = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, &bindings[0]))?;
                        let alloca = bld!(self.builder.build_alloca(self.context.i64_type(), &bindings[0]))?;
                        bld!(self.builder.build_store(alloca, payload))?;
                        self.variables.insert(bindings[0].clone(), (alloca, self.context.i64_type().into(), ValKind::Int, false));
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
                        _ => return Err(CodeGenError { msg: format!("unknown Result variant '{}'", name) }),
                    };

                    let case_bb = self.context.append_basic_block(func, &format!("res_{}", name));
                    let tag_const = self.context.i8_type().const_int(vtag as u64, false);
                    case_blocks.push((tag_const, case_bb));

                    self.builder.position_at_end(case_bb);
                    let saved_vars = self.variables.clone();

                    if !bindings.is_empty() {
                        let val_ptr = bld!(self.builder.build_struct_gep(res_ty, subject_alloca, 1, "val_ptr"))?;
                        let payload = bld!(self.builder.build_load(self.context.i64_type(), val_ptr, &bindings[0]))?;
                        let alloca = bld!(self.builder.build_alloca(self.context.i64_type(), &bindings[0]))?;
                        bld!(self.builder.build_store(alloca, payload))?;
                        self.variables.insert(bindings[0].clone(), (alloca, self.context.i64_type().into(), ValKind::Int, false));
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
        let val_ptr = bld!(self.builder.build_struct_gep(res_ty, alloca, 1, "val_ptr"))?;
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

        for elem in elements {
            let val = self.compile_expr(elem, func)?;
            bld!(self.builder.build_call(list_push, &[list_ptr.into(), val.into()], ""))?;
        }

        Ok((list_ptr.into(), ValKind::List))
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
            _ => Err(CodeGenError { msg: "indexing only supported on lists".into() }),
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
            _ => Err(CodeGenError { msg: "ore_str_new did not return a pointer".into() }),
        }
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
                    let p = self.value_to_str(val, kind)?;
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
            ValKind::Bool => {
                let bool_to_str = self.module.get_function("ore_bool_to_str").unwrap();
                let int_val = val.into_int_value();
                let ext = bld!(self.builder.build_int_z_extend(int_val, self.context.i8_type(), "zext"))?;
                let result = bld!(self.builder.build_call(bool_to_str, &[ext.into()], "btos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
            _ => {
                // Fallback: convert as int
                let int_to_str = self.module.get_function("ore_int_to_str").unwrap();
                let result = bld!(self.builder.build_call(int_to_str, &[val.into()], "itos"))?;
                Ok(self.call_result_to_value(result)?.into_pointer_value())
            }
        }
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
            msg: format!("undefined function '{}'", name),
        })
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
        let arg_val = self.compile_expr(arg, current_fn)?;

        match func_expr {
            Expr::Ident(name) => {
                let (called_fn, ret_kind) = self.resolve_function(name)?;
                let result = bld!(self.builder.build_call(called_fn, &[arg_val.into()], "pipe"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ret_kind))
            }
            Expr::Call { func, args } => {
                let name = match func.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(CodeGenError { msg: "pipeline target must be a function".into() }),
                };
                let (called_fn, ret_kind) = self.resolve_function(&name)?;

                let mut compiled_args = vec![arg_val.into()];
                for a in args {
                    compiled_args.push(self.compile_expr(a, current_fn)?.into());
                }

                let result = bld!(self.builder.build_call(called_fn, &compiled_args, "pipe"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ret_kind))
            }
            Expr::Lambda { params, body } => {
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
            _ => Err(CodeGenError { msg: "unsupported pipeline target".into() }),
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
        self.break_target = Some(end_bb);
        for stmt in &body.stmts {
            self.compile_stmt(stmt, func)?;
        }
        self.break_target = saved_break;

        // Increment
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            let current = bld!(self.builder.build_load(i64_type, var_alloca, var))?.into_int_value();
            let next = bld!(self.builder.build_int_add(current, i64_type.const_int(1, false), "inc"))?;
            bld!(self.builder.build_store(var_alloca, next))?;
            bld!(self.builder.build_unconditional_branch(cond_bb))?;
        }

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
        self.break_target = Some(end_bb);
        for stmt in &body.stmts {
            self.compile_stmt(stmt, func)?;
        }
        self.break_target = saved_break;
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
        self.break_target = Some(end_bb);
        for stmt in &body.stmts {
            self.compile_stmt(stmt, func)?;
        }
        self.break_target = saved_break;
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(body_bb))?;
        }

        self.builder.position_at_end(end_bb);
        Ok(())
    }

    fn compile_if_else(
        &mut self,
        cond: &Expr,
        then_block: &Block,
        else_block: Option<&Block>,
        func: FunctionValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let cond_val = self.compile_expr(cond, func)?;
        let cond_int = match cond_val {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(CodeGenError { msg: "condition must be a boolean".into() }),
        };

        let then_bb = self.context.append_basic_block(func, "then");
        let else_bb = self.context.append_basic_block(func, "else");
        let merge_bb = self.context.append_basic_block(func, "merge");

        bld!(self.builder.build_conditional_branch(cond_int, then_bb, else_bb))?;

        self.builder.position_at_end(then_bb);
        let mut then_val: BasicValueEnum<'ctx> = self.context.i64_type().const_int(0, false).into();
        for stmt in &then_block.stmts {
            if let Some(v) = self.compile_stmt(stmt, func)? {
                then_val = v;
            }
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(merge_bb))?;
        }
        let then_end_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(else_bb);
        let mut else_val: BasicValueEnum<'ctx> = self.context.i64_type().const_int(0, false).into();
        if let Some(eb) = else_block {
            for stmt in &eb.stmts {
                if let Some(v) = self.compile_stmt(stmt, func)? {
                    else_val = v;
                }
            }
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_unconditional_branch(merge_bb))?;
        }
        let else_end_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(merge_bb);
        let phi = bld!(self.builder.build_phi(then_val.get_type(), "ifval"))?;
        phi.add_incoming(&[(&then_val, then_end_bb), (&else_val, else_end_bb)]);
        Ok(phi.as_basic_value())
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
            _ => return Err(CodeGenError { msg: "condition must be a boolean".into() }),
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
            msg: format!("no capture info for lambda '{}'", lambda_name),
        })?;
        let struct_type = cap_info.struct_type;
        let names = cap_info.names.clone();
        let types = cap_info.types.clone();

        let alloca = bld!(self.builder.build_alloca(struct_type, "captures"))?;

        for (i, cap_name) in names.iter().enumerate() {
            let (var_ptr, var_ty, _kind, _) = self.variables.get(cap_name).ok_or_else(|| CodeGenError {
                msg: format!("captured variable '{}' not found in scope", cap_name),
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
                    _ => return Err(CodeGenError { msg: format!("unsupported float op {:?}", op) }),
                };
                Ok(result)
            }
            _ => Err(CodeGenError { msg: "type mismatch in binary operation".into() }),
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
            _ => return Err(CodeGenError { msg: format!("unsupported bool op {:?}", op) }),
        }?;
        Ok(result.into())
    }
}
