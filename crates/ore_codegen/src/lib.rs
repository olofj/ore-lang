use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::types::BasicType;
use inkwell::IntPredicate;
use std::collections::HashMap;

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

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    /// Maps variable names to (pointer, pointee type, kind)
    variables: HashMap<String, (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>, ValKind)>,
    functions: HashMap<String, (FunctionValue<'ctx>, ValKind)>,
    records: HashMap<String, RecordInfo<'ctx>>,
    enums: HashMap<String, EnumInfo<'ctx>>,
    /// Maps variant name -> enum name for quick lookup
    variant_to_enum: HashMap<String, String>,
    str_counter: u32,
    lambda_counter: u32,
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
            str_counter: 0,
            lambda_counter: 0,
        }
    }

    fn ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(inkwell::AddressSpace::default())
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

        for item in &program.items {
            if let Item::FnDef(f) = item {
                self.declare_function(f)?;
            }
        }

        for item in &program.items {
            if let Item::FnDef(f) = item {
                self.compile_function(f)?;
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
    }

    fn type_expr_to_kind(&self, ty: &TypeExpr) -> ValKind {
        match ty {
            TypeExpr::Named(n) => match n.as_str() {
                "Int" => ValKind::Int,
                "Float" => ValKind::Float,
                "Bool" => ValKind::Bool,
                "Str" => ValKind::Str,
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
            self.variables.insert(param.name.clone(), (alloca, ty, kind));
        }

        let mut last_val: Option<BasicValueEnum<'ctx>> = None;
        for stmt in &fndef.body.stmts {
            last_val = self.compile_stmt(stmt, func)?;
        }

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            if fndef.name == "main" {
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
            Stmt::Let { name, value, .. } => {
                let (val, kind) = self.compile_expr_with_kind(value, func)?;
                let ty = val.get_type();
                let alloca = bld!(self.builder.build_alloca(ty, name))?;
                bld!(self.builder.build_store(alloca, val))?;
                self.variables.insert(name.clone(), (alloca, ty, kind));
                Ok(None)
            }
            Stmt::Assign { name, value } => {
                let (val, _kind) = self.compile_expr_with_kind(value, func)?;
                let (ptr, _, _) = self.variables.get(name).ok_or_else(|| CodeGenError {
                    msg: format!("undefined variable '{}'", name),
                })?;
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
                let (ptr, ty, kind) = self.variables.get(name).ok_or_else(|| CodeGenError {
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
                        self.variables.insert(binding.clone(), (alloca, field_ty, field_kind.clone()));
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
                let result = bld!(self.builder.build_call(lambda_fn, &[arg_val.into()], "pipe"))?;
                let val = self.call_result_to_value(result)?;
                Ok((val, ValKind::Int))
            }
            _ => Err(CodeGenError { msg: "unsupported pipeline target".into() }),
        }
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

        // For now, lambdas take and return i64 (no captures)
        let i64_type = self.context.i64_type();
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> =
            params.iter().map(|_| i64_type.into()).collect();
        let fn_type = i64_type.fn_type(&param_types, false);
        let lambda_fn = self.module.add_function(&name, fn_type, None);

        // Save current state
        let saved_vars = self.variables.clone();
        let saved_block = self.builder.get_insert_block();

        // Build lambda body
        let entry = self.context.append_basic_block(lambda_fn, "entry");
        self.builder.position_at_end(entry);
        self.variables.clear();

        for (i, param_name) in params.iter().enumerate() {
            let val = lambda_fn.get_nth_param(i as u32).unwrap();
            let ty = val.get_type();
            let alloca = bld!(self.builder.build_alloca(ty, param_name))?;
            bld!(self.builder.build_store(alloca, val))?;
            self.variables.insert(param_name.clone(), (alloca, ty, ValKind::Int));
        }

        let (result, _kind) = self.compile_expr_with_kind(body, lambda_fn)?;

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            bld!(self.builder.build_return(Some(&result)))?;
        }

        // Restore state
        self.variables = saved_vars;
        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }

        self.functions.insert(name, (lambda_fn, ValKind::Int));
        Ok(lambda_fn)
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
