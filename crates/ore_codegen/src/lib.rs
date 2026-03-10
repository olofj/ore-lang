use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;
use std::collections::HashMap;

use ore_parser::ast::*;

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    /// Maps variable names to (pointer, pointee type)
    variables: HashMap<String, (PointerValue<'ctx>, inkwell::types::BasicTypeEnum<'ctx>)>,
    functions: HashMap<String, FunctionValue<'ctx>>,
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
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), CodeGenError> {
        self.declare_runtime_functions();

        // First pass: declare all functions
        for item in &program.items {
            match item {
                Item::FnDef(f) => self.declare_function(f)?,
            }
        }

        // Second pass: compile all functions
        for item in &program.items {
            match item {
                Item::FnDef(f) => self.compile_function(f)?,
            }
        }

        Ok(())
    }

    fn declare_runtime_functions(&mut self) {
        let i64_type = self.context.i64_type();
        let i8_type = self.context.i8_type();
        let void_type = self.context.void_type();

        let print_int_ty = void_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("ore_print_int", print_int_ty, Some(inkwell::module::Linkage::External));

        let print_bool_ty = void_type.fn_type(&[i8_type.into()], false);
        self.module.add_function("ore_print_bool", print_bool_ty, Some(inkwell::module::Linkage::External));
    }

    fn declare_function(&mut self, fndef: &FnDef) -> Result<(), CodeGenError> {
        let i64_type = self.context.i64_type();
        let i1_type = self.context.bool_type();

        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = fndef
            .params
            .iter()
            .map(|p| match &p.ty {
                TypeExpr::Named(n) => match n.as_str() {
                    "Bool" => i1_type.into(),
                    _ => i64_type.into(),
                },
            })
            .collect();

        let fn_type = if fndef.name == "main" {
            self.context.i32_type().fn_type(&param_types, false)
        } else {
            match &fndef.ret_type {
                Some(TypeExpr::Named(n)) => match n.as_str() {
                    "Bool" => i1_type.fn_type(&param_types, false),
                    _ => i64_type.fn_type(&param_types, false),
                },
                None => self.context.void_type().fn_type(&param_types, false),
            }
        };

        let func = self.module.add_function(&fndef.name, fn_type, None);
        self.functions.insert(fndef.name.clone(), func);
        Ok(())
    }

    fn compile_function(&mut self, fndef: &FnDef) -> Result<(), CodeGenError> {
        let func = *self.functions.get(&fndef.name).unwrap();
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);
        self.variables.clear();

        // Bind parameters
        for (i, param) in fndef.params.iter().enumerate() {
            let val = func.get_nth_param(i as u32).unwrap();
            let ty = val.get_type();
            let alloca = bld!(self.builder.build_alloca(ty, &param.name))?;
            bld!(self.builder.build_store(alloca, val))?;
            self.variables.insert(param.name.clone(), (alloca, ty));
        }

        let mut last_val: Option<BasicValueEnum<'ctx>> = None;
        for stmt in &fndef.body.stmts {
            last_val = self.compile_stmt(stmt, func)?;
        }

        // Add terminator if needed
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
                let val = self.compile_expr(value, func)?;
                let ty = val.get_type();
                let alloca = bld!(self.builder.build_alloca(ty, name))?;
                bld!(self.builder.build_store(alloca, val))?;
                self.variables.insert(name.clone(), (alloca, ty));
                Ok(None)
            }
            Stmt::Assign { name, value } => {
                let val = self.compile_expr(value, func)?;
                let (ptr, _) = self.variables.get(name).ok_or_else(|| CodeGenError {
                    msg: format!("undefined variable '{}'", name),
                })?;
                bld!(self.builder.build_store(*ptr, val))?;
                Ok(None)
            }
            Stmt::Expr(expr) => {
                let val = self.compile_expr(expr, func)?;
                Ok(Some(val))
            }
            Stmt::Return(Some(expr)) => {
                let val = self.compile_expr(expr, func)?;
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
        match expr {
            Expr::IntLit(n) => {
                Ok(self.context.i64_type().const_int(*n as u64, true).into())
            }
            Expr::FloatLit(f) => {
                Ok(self.context.f64_type().const_float(*f).into())
            }
            Expr::BoolLit(b) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }
            Expr::Ident(name) => {
                let (ptr, ty) = self.variables.get(name).ok_or_else(|| CodeGenError {
                    msg: format!("undefined variable '{}'", name),
                })?;
                let val = bld!(self.builder.build_load(*ty, *ptr, name))?;
                Ok(val)
            }
            Expr::BinOp { op, left, right } => {
                if *op == BinOp::Pipe {
                    return self.compile_pipeline(left, right, func);
                }
                let lhs = self.compile_expr(left, func)?;
                let rhs = self.compile_expr(right, func)?;
                self.compile_binop(*op, lhs, rhs)
            }
            Expr::UnaryMinus(inner) => {
                let val = self.compile_expr(inner, func)?;
                match val {
                    BasicValueEnum::IntValue(v) => {
                        Ok(bld!(self.builder.build_int_neg(v, "neg"))?.into())
                    }
                    BasicValueEnum::FloatValue(v) => {
                        Ok(bld!(self.builder.build_float_neg(v, "fneg"))?.into())
                    }
                    _ => Err(CodeGenError { msg: "cannot negate this type".into() }),
                }
            }
            Expr::UnaryNot(inner) => {
                let val = self.compile_expr(inner, func)?;
                match val {
                    BasicValueEnum::IntValue(v) => {
                        Ok(bld!(self.builder.build_not(v, "not"))?.into())
                    }
                    _ => Err(CodeGenError { msg: "cannot apply 'not' to this type".into() }),
                }
            }
            Expr::Print(inner) => {
                let val = self.compile_expr(inner, func)?;
                match val {
                    BasicValueEnum::IntValue(v) if v.get_type().get_bit_width() == 64 => {
                        let pf = self.module.get_function("ore_print_int").unwrap();
                        bld!(self.builder.build_call(pf, &[v.into()], ""))?;
                    }
                    BasicValueEnum::IntValue(v) if v.get_type().get_bit_width() == 1 => {
                        let pf = self.module.get_function("ore_print_bool").unwrap();
                        let ext = bld!(self.builder.build_int_z_extend(v, self.context.i8_type(), "zext"))?;
                        bld!(self.builder.build_call(pf, &[ext.into()], ""))?;
                    }
                    _ => {
                        let pf = self.module.get_function("ore_print_int").unwrap();
                        bld!(self.builder.build_call(pf, &[val.into()], ""))?;
                    }
                }
                Ok(self.context.i64_type().const_int(0, false).into())
            }
            Expr::Call { func: callee, args } => {
                let name = match callee.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(CodeGenError { msg: "only named function calls supported".into() }),
                };
                let called_fn = self.resolve_function(&name)?;

                let mut compiled_args = Vec::new();
                for arg in args {
                    compiled_args.push(self.compile_expr(arg, func)?.into());
                }

                let result = bld!(self.builder.build_call(called_fn, &compiled_args, "call"))?;
                self.call_result_to_value(result)
            }
            Expr::IfElse { cond, then_block, else_block } => {
                self.compile_if_else(cond, then_block, else_block.as_ref(), func)
            }
            Expr::ColonMatch { cond, then_expr, else_expr } => {
                self.compile_colon_match(cond, then_expr, else_expr.as_deref(), func)
            }
            Expr::StringLit(_) => {
                Ok(self.context.i64_type().const_int(0, false).into())
            }
        }
    }

    fn resolve_function(&self, name: &str) -> Result<FunctionValue<'ctx>, CodeGenError> {
        self.functions
            .get(name)
            .copied()
            .or_else(|| self.module.get_function(name))
            .ok_or_else(|| CodeGenError {
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

    fn compile_pipeline(
        &mut self,
        arg: &Expr,
        func_expr: &Expr,
        current_fn: FunctionValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let arg_val = self.compile_expr(arg, current_fn)?;

        match func_expr {
            Expr::Ident(name) => {
                let called_fn = self.resolve_function(name)?;
                let result = bld!(self.builder.build_call(called_fn, &[arg_val.into()], "pipe"))?;
                self.call_result_to_value(result)
            }
            Expr::Call { func, args } => {
                let name = match func.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(CodeGenError { msg: "pipeline target must be a function".into() }),
                };
                let called_fn = self.resolve_function(&name)?;

                let mut compiled_args = vec![arg_val.into()];
                for a in args {
                    compiled_args.push(self.compile_expr(a, current_fn)?.into());
                }

                let result = bld!(self.builder.build_call(called_fn, &compiled_args, "pipe"))?;
                self.call_result_to_value(result)
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

    fn compile_colon_match(
        &mut self,
        cond: &Expr,
        then_expr: &Expr,
        else_expr: Option<&Expr>,
        func: FunctionValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
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
        let then_val = self.compile_expr(then_expr, func)?;
        bld!(self.builder.build_unconditional_branch(merge_bb))?;
        let then_end_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(else_bb);
        let else_val = if let Some(e) = else_expr {
            self.compile_expr(e, func)?
        } else {
            self.context.i64_type().const_int(0, false).into()
        };
        bld!(self.builder.build_unconditional_branch(merge_bb))?;
        let else_end_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(merge_bb);
        let phi = bld!(self.builder.build_phi(then_val.get_type(), "cval"))?;
        phi.add_incoming(&[(&then_val, then_end_bb), (&else_val, else_end_bb)]);
        Ok(phi.as_basic_value())
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
