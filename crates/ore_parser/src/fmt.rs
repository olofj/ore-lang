//! ore fmt — opinionated formatter that pretty-prints an AST back to Ore source.

use crate::ast::*;

pub fn format_program(program: &Program) -> String {
    let mut f = Formatter::new();
    for (i, item) in program.items.iter().enumerate() {
        if i > 0 {
            f.out.push('\n');
        }
        f.format_item(item, 0);
    }
    f.out
}

struct Formatter {
    out: String,
}

impl Formatter {
    fn new() -> Self {
        Self { out: String::new() }
    }

    /// Format an expression as inline (single line), using `if/then/else` for conditionals.
    fn format_expr_inline(&mut self, expr: &Expr, level: usize) {
        match expr {
            Expr::IfElse { cond, then_block, else_block } => {
                self.out.push_str("if ");
                self.format_expr_inline(cond, level);
                self.out.push_str(" then ");
                if let Some(s) = then_block.stmts.last() {
                    if let Stmt::Expr(e) = &s.stmt {
                        self.format_expr_inline(e, level);
                    }
                }
                if let Some(eb) = else_block {
                    self.out.push_str(" else ");
                    if let Some(s) = eb.stmts.last() {
                        if let Stmt::Expr(e) = &s.stmt {
                            self.format_expr_inline(e, level);
                        }
                    }
                }
            }
            _ => self.format_expr(expr, level),
        }
    }

    fn indent(&mut self, level: usize) {
        for _ in 0..level {
            self.out.push_str("  ");
        }
    }

    fn format_item(&mut self, item: &Item, level: usize) {
        match item {
            Item::Use { path } => {
                self.indent(level);
                self.out.push_str(&format!("use \"{}\"\n", path));
            }
            Item::FnDef(f) => self.format_fn_def(f, level),
            Item::TypeDef(td) => {
                self.indent(level);
                if !td.type_params.is_empty() {
                let tp_strs: Vec<String> = td.type_params.iter().map(|tp| {
                    if let Some(ref b) = tp.bound { format!("{}: {}", tp.name, b) } else { tp.name.clone() }
                }).collect();
                self.out.push_str(&format!("type {}[{}] {{ ", td.name, tp_strs.join(", ")));
            } else {
                self.out.push_str(&format!("type {} {{ ", td.name));
            }
                for (i, f) in td.fields.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.out.push_str(&format!("{}:{}", f.name, format_type_expr(&f.ty)));
                }
                self.out.push_str(" }\n");
            }
            Item::EnumDef(ed) => {
                self.indent(level);
                self.out.push_str(&format!("type {}\n", ed.name));
                for v in &ed.variants {
                    self.indent(level + 1);
                    self.out.push_str(&v.name);
                    if !v.fields.is_empty() {
                        self.out.push('(');
                        for (i, f) in v.fields.iter().enumerate() {
                            if i > 0 {
                                self.out.push_str(", ");
                            }
                            self.out.push_str(&format!("{}:{}", f.name, format_type_expr(&f.ty)));
                        }
                        self.out.push(')');
                    }
                    self.out.push('\n');
                }
            }
            Item::ImplBlock { type_name, methods } => {
                self.indent(level);
                self.out.push_str(&format!("impl {}\n", type_name));
                for m in methods {
                    self.format_fn_def(m, level + 1);
                }
            }
            Item::TraitDef(td) => {
                self.indent(level);
                self.out.push_str(&format!("trait {}\n", td.name));
                for m in &td.methods {
                    self.indent(level + 1);
                    self.out.push_str(&format!("fn {}", m.name));
                    for p in &m.params {
                        self.out.push_str(&format!(" {}:{}", p.name, format_type_expr(&p.ty)));
                    }
                    if let Some(ret) = &m.ret_type {
                        self.out.push_str(&format!(" -> {}", format_type_expr(ret)));
                    }
                    self.out.push('\n');
                }
            }
            Item::ImplTrait { trait_name, type_name, methods } => {
                self.indent(level);
                self.out.push_str(&format!("impl {} for {}\n", trait_name, type_name));
                for m in methods {
                    self.format_fn_def(m, level + 1);
                }
            }
            Item::TestDef { name, body } => {
                self.indent(level);
                self.out.push_str(&format!("test \"{}\"\n", name));
                self.format_block(body, level + 1);
            }
        }
    }

    fn format_fn_def(&mut self, f: &FnDef, level: usize) {
        self.indent(level);
        self.out.push_str(&format!("fn {}", f.name));
        if !f.type_params.is_empty() {
            let tp_strs: Vec<String> = f.type_params.iter().map(|tp| {
                if let Some(ref b) = tp.bound { format!("{}: {}", tp.name, b) } else { tp.name.clone() }
            }).collect();
            self.out.push_str(&format!("[{}]", tp_strs.join(", ")));
        }
        for p in &f.params {
            self.out.push_str(&format!(" {}:{}", p.name, format_type_expr(&p.ty)));
        }
        if let Some(ret) = &f.ret_type {
            self.out.push_str(&format!(" -> {}", format_type_expr(ret)));
        }
        self.out.push('\n');
        self.format_block(&f.body, level + 1);
    }

    fn format_block(&mut self, block: &Block, level: usize) {
        for spanned in &block.stmts {
            self.format_stmt(&spanned.stmt, level);
        }
    }

    fn format_stmt(&mut self, stmt: &Stmt, level: usize) {
        match stmt {
            Stmt::Let { name, mutable, value } => {
                self.indent(level);
                if *mutable {
                    self.out.push_str(&format!("mut {} := ", name));
                } else {
                    self.out.push_str(&format!("{} := ", name));
                }
                self.format_expr(value, level);
                self.out.push('\n');
            }
            Stmt::LetDestructure { names, value } => {
                self.indent(level);
                self.out.push('[');
                self.out.push_str(&names.join(", "));
                self.out.push_str("] := ");
                self.format_expr(value, level);
                self.out.push('\n');
            }
            Stmt::Assign { name, value } => {
                self.indent(level);
                self.out.push_str(&format!("{} = ", name));
                self.format_expr(value, level);
                self.out.push('\n');
            }
            Stmt::IndexAssign { object, index, value } => {
                self.indent(level);
                self.format_expr(object, level);
                self.out.push('[');
                self.format_expr(index, level);
                self.out.push_str("] = ");
                self.format_expr(value, level);
                self.out.push('\n');
            }
            Stmt::FieldAssign { object, field, value } => {
                self.indent(level);
                self.format_expr(object, level);
                self.out.push_str(&format!(".{} = ", field));
                self.format_expr(value, level);
                self.out.push('\n');
            }
            Stmt::Expr(expr) => {
                self.indent(level);
                self.format_expr(expr, level);
                self.out.push('\n');
            }
            Stmt::Return(None) => {
                self.indent(level);
                self.out.push_str("return\n");
            }
            Stmt::Return(Some(expr)) => {
                self.indent(level);
                self.out.push_str("return ");
                self.format_expr(expr, level);
                self.out.push('\n');
            }
            Stmt::ForIn { var, start, end, step, body } => {
                self.indent(level);
                self.out.push_str(&format!("for {} in ", var));
                self.format_expr(start, level);
                self.out.push_str("..");
                self.format_expr(end, level);
                if let Some(step_expr) = step {
                    self.out.push_str(" step ");
                    self.format_expr(step_expr, level);
                }
                self.out.push('\n');
                self.format_block(body, level + 1);
            }
            Stmt::While { cond, body } => {
                self.indent(level);
                self.out.push_str("while ");
                self.format_expr(cond, level);
                self.out.push('\n');
                self.format_block(body, level + 1);
            }
            Stmt::ForEach { var, iterable, body } => {
                self.indent(level);
                self.out.push_str(&format!("for {} in ", var));
                self.format_expr(iterable, level);
                self.out.push('\n');
                self.format_block(body, level + 1);
            }
            Stmt::ForEachKV { key_var, val_var, iterable, body } => {
                self.indent(level);
                self.out.push_str(&format!("for {}, {} in ", key_var, val_var));
                self.format_expr(iterable, level);
                self.out.push('\n');
                self.format_block(body, level + 1);
            }
            Stmt::Loop { body } => {
                self.indent(level);
                self.out.push_str("loop\n");
                self.format_block(body, level + 1);
            }
            Stmt::Break => {
                self.indent(level);
                self.out.push_str("break\n");
            }
            Stmt::Continue => {
                self.indent(level);
                self.out.push_str("continue\n");
            }
            Stmt::Spawn(expr) => {
                self.indent(level);
                self.out.push_str("spawn ");
                self.format_expr(expr, level);
                self.out.push('\n');
            }
            Stmt::LocalFn(fndef) => {
                self.format_fn_def(fndef, level);
            }
        }
    }

    fn format_expr(&mut self, expr: &Expr, level: usize) {
        match expr {
            Expr::IntLit(n) => self.out.push_str(&n.to_string()),
            Expr::FloatLit(f) => {
                if *f == f.floor() && !f.is_infinite() && !f.is_nan() {
                    self.out.push_str(&format!("{:.1}", f));
                } else {
                    self.out.push_str(&f.to_string());
                }
            }
            Expr::BoolLit(b) => self.out.push_str(if *b { "true" } else { "false" }),
            Expr::StringLit(s) => {
                self.out.push('"');
                self.out.push_str(&s.replace('\\', "\\\\").replace('"', "\\\""));
                self.out.push('"');
            }
            Expr::Ident(name) => self.out.push_str(name),
            Expr::BinOp { op, left, right } => {
                let needs_parens_left = binop_needs_parens(left, *op);
                let needs_parens_right = binop_needs_parens(right, *op);
                if needs_parens_left { self.out.push('('); }
                self.format_expr(left, level);
                if needs_parens_left { self.out.push(')'); }
                self.out.push_str(&format!(" {} ", format_binop(*op)));
                if needs_parens_right { self.out.push('('); }
                self.format_expr(right, level);
                if needs_parens_right { self.out.push(')'); }
            }
            Expr::UnaryMinus(inner) => {
                self.out.push('-');
                let needs_parens = matches!(inner.as_ref(), Expr::BinOp { .. });
                if needs_parens { self.out.push('('); }
                self.format_expr(inner, level);
                if needs_parens { self.out.push(')'); }
            }
            Expr::UnaryNot(inner) => {
                self.out.push_str("not ");
                self.format_expr(inner, level);
            }
            Expr::Call { func, args } => {
                self.format_expr(func, level);
                self.out.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.out.push_str(", "); }
                    self.format_expr(arg, level);
                }
                self.out.push(')');
            }
            Expr::Print(inner) => {
                self.out.push_str("print ");
                self.format_expr(inner, level);
            }
            Expr::Sleep(inner) => {
                self.out.push_str("sleep ");
                self.format_expr(inner, level);
            }
            Expr::IfElse { cond, then_block, else_block } => {
                // Check if this is a compact if/then/else (single expr in each branch)
                let is_compact = then_block.stmts.len() == 1
                    && matches!(&then_block.stmts[0].stmt, Stmt::Expr(e) if !matches!(e, Expr::IfElse { .. } | Expr::BlockExpr(_)))
                    && else_block.as_ref().map_or(false, |eb| {
                        eb.stmts.len() == 1 && matches!(&eb.stmts[0].stmt, Stmt::Expr(_))
                    });

                if is_compact {
                    // Inline form: if cond then expr else expr
                    self.out.push_str("if ");
                    self.format_expr(cond, level);
                    self.out.push_str(" then ");
                    if let Stmt::Expr(e) = &then_block.stmts[0].stmt {
                        self.format_expr(e, level);
                    }
                    if let Some(eb) = else_block {
                        if let Stmt::Expr(e) = &eb.stmts[0].stmt {
                            self.out.push('\n');
                            self.indent(level);
                            self.out.push_str("else ");
                            // If else branch is another if/else, recurse
                            self.format_expr(e, level);
                        }
                    }
                } else {
                    self.out.push_str("if ");
                    self.format_expr(cond, level);
                    self.out.push('\n');
                    self.format_block(then_block, level + 1);
                    if let Some(eb) = else_block {
                        // Check if else block is a single `else if` (IfElse expr statement)
                        if eb.stmts.len() == 1 {
                            if let Stmt::Expr(Expr::IfElse { .. }) = &eb.stmts[0].stmt {
                                self.indent(level);
                                self.out.push_str("else ");
                                self.format_expr(&Expr::IfElse {
                                    cond: match &eb.stmts[0].stmt {
                                        Stmt::Expr(Expr::IfElse { cond, .. }) => cond.clone(),
                                        _ => unreachable!(),
                                    },
                                    then_block: match &eb.stmts[0].stmt {
                                        Stmt::Expr(Expr::IfElse { then_block, .. }) => then_block.clone(),
                                        _ => unreachable!(),
                                    },
                                    else_block: match &eb.stmts[0].stmt {
                                        Stmt::Expr(Expr::IfElse { else_block, .. }) => else_block.clone(),
                                        _ => unreachable!(),
                                    },
                                }, level);
                                return;
                            }
                        }
                        self.indent(level);
                        self.out.push_str("else\n");
                        self.format_block(eb, level + 1);
                    }
                }
            }
            Expr::ColonMatch { cond, then_expr, else_expr } => {
                self.format_expr(cond, level);
                self.out.push_str(" : ");
                self.format_expr(then_expr, level);
                if let Some(e) = else_expr {
                    self.out.push_str(" : ");
                    self.format_expr(e, level);
                }
            }
            Expr::Match { subject, arms } => {
                self.format_expr(subject, level);
                self.out.push_str(" :\n");
                for arm in arms {
                    self.indent(level + 1);
                    self.format_pattern(&arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.out.push_str(" if ");
                        self.format_expr(guard, level + 1);
                    }
                    self.out.push_str(" -> ");
                    self.format_expr(&arm.body, level + 1);
                    self.out.push('\n');
                }
            }
            Expr::StringInterp(parts) => {
                self.out.push('"');
                for part in parts {
                    match part {
                        StringPart::Lit(s) => {
                            self.out.push_str(&s.replace('\\', "\\\\").replace('"', "\\\""));
                        }
                        StringPart::Expr(expr) => {
                            self.out.push('{');
                            self.format_expr_inline(expr, level);
                            self.out.push('}');
                        }
                    }
                }
                self.out.push('"');
            }
            Expr::BlockExpr(block) => {
                self.out.push('\n');
                self.format_block(block, level + 1);
            }
            Expr::Lambda { params, body } => {
                if params.len() == 1 {
                    self.out.push_str(&params[0]);
                    self.out.push_str(" => ");
                    self.format_expr(body, level);
                } else {
                    self.out.push('(');
                    for (i, p) in params.iter().enumerate() {
                        if i > 0 { self.out.push_str(", "); }
                        self.out.push_str(p);
                    }
                    self.out.push_str(" => ");
                    self.format_expr(body, level);
                    self.out.push(')');
                }
            }
            Expr::RecordConstruct { type_name, fields } => {
                self.out.push_str(type_name);
                self.out.push('(');
                for (i, (name, val)) in fields.iter().enumerate() {
                    if i > 0 { self.out.push_str(", "); }
                    self.out.push_str(&format!("{}: ", name));
                    self.format_expr(val, level);
                }
                self.out.push(')');
            }
            Expr::FieldAccess { object, field } => {
                self.format_expr(object, level);
                self.out.push('.');
                self.out.push_str(field);
            }
            Expr::MethodCall { object, method, args } => {
                self.format_expr(object, level);
                self.out.push('.');
                self.out.push_str(method);
                self.out.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.out.push_str(", "); }
                    self.format_expr(arg, level);
                }
                self.out.push(')');
            }
            Expr::ListLit(elems) => {
                self.out.push('[');
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 { self.out.push_str(", "); }
                    self.format_expr(elem, level);
                }
                self.out.push(']');
            }
            Expr::ListComp { expr, var, iterable, cond } => {
                self.out.push('[');
                self.format_expr(expr, level);
                self.out.push_str(" for ");
                self.out.push_str(var);
                self.out.push_str(" in ");
                self.format_expr(iterable, level);
                if let Some(c) = cond {
                    self.out.push_str(" if ");
                    self.format_expr(c, level);
                }
                self.out.push(']');
            }
            Expr::MapLit(entries) => {
                self.out.push('{');
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 { self.out.push_str(", "); }
                    self.format_expr(k, level);
                    self.out.push_str(": ");
                    self.format_expr(v, level);
                }
                self.out.push('}');
            }
            Expr::Index { object, index } => {
                self.format_expr(object, level);
                self.out.push('[');
                self.format_expr(index, level);
                self.out.push(']');
            }
            Expr::Break => self.out.push_str("break"),
            Expr::OptionNone => self.out.push_str("None"),
            Expr::OptionSome(inner) => {
                self.out.push_str("Some(");
                self.format_expr(inner, level);
                self.out.push(')');
            }
            Expr::ResultOk(inner) => {
                self.out.push_str("Ok(");
                self.format_expr(inner, level);
                self.out.push(')');
            }
            Expr::ResultErr(inner) => {
                self.out.push_str("Err(");
                self.format_expr(inner, level);
                self.out.push(')');
            }
            Expr::Try(inner) => {
                self.format_expr(inner, level);
                self.out.push('?');
            }
            Expr::OptionalChain { object, field } => {
                self.format_expr(object, level);
                self.out.push_str("?.");
                self.out.push_str(field);
            }
            Expr::OptionalMethodCall { object, method, args } => {
                self.format_expr(object, level);
                self.out.push_str("?.");
                self.out.push_str(method);
                self.out.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.out.push_str(", "); }
                    self.format_expr(arg, level);
                }
                self.out.push(')');
            }
            Expr::Assert { cond, message } => {
                self.out.push_str("assert ");
                self.format_expr(cond, level);
                if let Some(msg) = message {
                    self.out.push_str(&format!(", \"{}\"", msg));
                }
            }
            Expr::AssertEq { left, right, message } => {
                self.out.push_str("assert_eq ");
                self.format_expr(left, level);
                self.out.push_str(", ");
                self.format_expr(right, level);
                if let Some(msg) = message {
                    self.out.push_str(&format!(", \"{}\"", msg));
                }
            }
            Expr::AssertNe { left, right, message } => {
                self.out.push_str("assert_ne ");
                self.format_expr(left, level);
                self.out.push_str(", ");
                self.format_expr(right, level);
                if let Some(msg) = message {
                    self.out.push_str(&format!(", \"{}\"", msg));
                }
            }
        }
    }

    fn format_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard => self.out.push('_'),
            Pattern::Variant { name, bindings } => {
                self.out.push_str(name);
                for b in bindings {
                    self.out.push(' ');
                    self.out.push_str(b);
                }
            }
            Pattern::IntLit(n) => {
                self.out.push_str(&n.to_string());
            }
            Pattern::FloatLit(f) => {
                self.out.push_str(&f.to_string());
            }
            Pattern::BoolLit(b) => {
                self.out.push_str(if *b { "true" } else { "false" });
            }
            Pattern::StringLit(s) => {
                self.out.push('"');
                self.out.push_str(s);
                self.out.push('"');
            }
            Pattern::Range(start, end) => {
                self.out.push_str(&start.to_string());
                self.out.push_str("..");
                self.out.push_str(&end.to_string());
            }
            Pattern::Or(alternatives) => {
                for (i, alt) in alternatives.iter().enumerate() {
                    if i > 0 { self.out.push_str(" | "); }
                    self.format_pattern(alt);
                }
            }
        }
    }
}

fn format_type_expr(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named(n) => n.clone(),
        TypeExpr::Generic(name, args) => {
            let args_str: Vec<String> = args.iter().map(format_type_expr).collect();
            format!("{}[{}]", name, args_str.join(", "))
        }
        TypeExpr::Fn { params, ret } => {
            let params_str: Vec<String> = params.iter().map(format_type_expr).collect();
            format!("({} -> {})", params_str.join(", "), format_type_expr(ret))
        }
    }
}

fn format_binop(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Eq => "==",
        BinOp::NotEq => "!=",
        BinOp::Lt => "<",
        BinOp::Gt => ">",
        BinOp::LtEq => "<=",
        BinOp::GtEq => ">=",
        BinOp::And => "and",
        BinOp::Or => "or",
        BinOp::Pipe => "|",
    }
}

fn binop_precedence(op: BinOp) -> u8 {
    match op {
        BinOp::Pipe => 1,
        BinOp::Or => 3,
        BinOp::And => 5,
        BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => 7,
        BinOp::Add | BinOp::Sub => 9,
        BinOp::Mul | BinOp::Div | BinOp::Mod => 11,
    }
}

fn binop_needs_parens(child: &Expr, parent_op: BinOp) -> bool {
    if let Expr::BinOp { op: child_op, .. } = child {
        binop_precedence(*child_op) < binop_precedence(parent_op)
    } else {
        false
    }
}
