//! Code generation for `deriving(...)` clauses on type and enum definitions.
//!
//! Generates synthetic AST `FnDef` nodes that the existing LLVM codegen pipeline
//! can declare and compile like any other function.
//!
//! Supported derived traits:
//!   - `Debug`     — generates `TypeName_debug(self) -> Str`
//!   - `Eq`        — generates `TypeName_eq(self, other) -> Bool`
//!   - `Clone`     — generates `TypeName_clone(self) -> Self`
//!   - `Serialize` — generates `TypeName_toJson(self) -> Str`

use ore_parser::ast::*;

/// Known derivable trait names.
const KNOWN_TRAITS: &[&str] = &["Debug", "Eq", "Clone", "Serialize"];

/// Check if an enum type needs a derived Eq function (generated as LLVM IR).
pub fn enum_needs_eq(ed: &EnumDef) -> bool {
    ed.deriving.iter().any(|t| t == "Eq")
}

/// Validate that all trait names in a deriving clause are known.
pub fn validate_deriving(traits: &[String]) -> Result<(), String> {
    for t in traits {
        if !KNOWN_TRAITS.contains(&t.as_str()) {
            return Err(format!(
                "unknown derived trait '{}'. Supported: {}",
                t,
                KNOWN_TRAITS.join(", ")
            ));
        }
    }
    Ok(())
}

/// Generate synthetic FnDef nodes for all derived traits on a record type.
pub fn generate_record_derives(td: &TypeDef) -> Result<Vec<FnDef>, String> {
    if td.deriving.is_empty() {
        return Ok(Vec::new());
    }
    validate_deriving(&td.deriving)?;

    let mut fns = Vec::new();
    for trait_name in &td.deriving {
        match trait_name.as_str() {
            "Debug" => fns.push(gen_record_debug(td)),
            "Eq" => fns.push(gen_record_eq(td)),
            "Clone" => fns.push(gen_record_clone(td)),
            "Serialize" => fns.push(gen_record_serialize(td)),
            _ => {}
        }
    }
    Ok(fns)
}

/// Generate synthetic FnDef nodes for all derived traits on an enum type.
pub fn generate_enum_derives(ed: &EnumDef) -> Result<Vec<FnDef>, String> {
    if ed.deriving.is_empty() {
        return Ok(Vec::new());
    }
    validate_deriving(&ed.deriving)?;

    let mut fns = Vec::new();
    for trait_name in &ed.deriving {
        match trait_name.as_str() {
            "Debug" => fns.push(gen_enum_debug(ed)),
            // Eq for enums is generated directly as LLVM IR (see CodeGen::compile_enum_eq)
            // because nested match expressions cause LLVM backend issues with large structs.
            "Eq" => {}
            "Clone" => fns.push(gen_enum_clone(ed)),
            "Serialize" => fns.push(gen_enum_serialize(ed)),
            _ => {}
        }
    }
    Ok(fns)
}

// ── Helpers ──

fn spanned(stmt: Stmt) -> SpannedStmt {
    SpannedStmt { stmt, line: 0 }
}

fn ident(name: &str) -> Expr {
    Expr::Ident(name.to_string())
}

fn str_lit(s: &str) -> Expr {
    Expr::StringLit(s.to_string())
}

fn field_access(obj: &str, field: &str) -> Expr {
    Expr::FieldAccess {
        object: Box::new(ident(obj)),
        field: field.to_string(),
    }
}

fn bin_op(op: BinOp, left: Expr, right: Expr) -> Expr {
    Expr::BinOp {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn str_concat(left: Expr, right: Expr) -> Expr {
    bin_op(BinOp::Add, left, right)
}

fn method_call(obj: Expr, method: &str, args: Vec<Expr>) -> Expr {
    Expr::MethodCall {
        object: Box::new(obj),
        method: method.to_string(),
        args,
    }
}

fn make_fndef(name: String, params: Vec<Param>, ret_type: Option<TypeExpr>, body: Block) -> FnDef {
    FnDef {
        name,
        type_params: vec![],
        params,
        ret_type,
        context: vec![],
        body,
    }
}

fn self_param(type_name: &str) -> Param {
    Param {
        name: "self".to_string(),
        ty: TypeExpr::Named(type_name.to_string()),
        default: None,
    }
}

fn other_param(type_name: &str) -> Param {
    Param {
        name: "other".to_string(),
        ty: TypeExpr::Named(type_name.to_string()),
        default: None,
    }
}

/// Convert a field value to string representation.
fn val_to_str(expr: Expr, ty: &TypeExpr) -> Expr {
    match ty {
        TypeExpr::Named(n) if n == "Str" => expr,
        _ => method_call(expr, "to_str", vec![]),
    }
}

/// Wrap a string expression in JSON quotes.
fn json_quote_str(expr: Expr) -> Expr {
    str_concat(str_lit("\""), str_concat(expr, str_lit("\"")))
}

/// Convert a value to a JSON string representation.
fn val_to_json(expr: Expr, ty: &TypeExpr) -> Expr {
    match ty {
        TypeExpr::Named(n) if n == "Str" => json_quote_str(expr),
        TypeExpr::Named(n) if n == "Bool" => {
            Expr::IfElse {
                cond: Box::new(expr),
                then_block: Block {
                    stmts: vec![spanned(Stmt::Expr(str_lit("true")))],
                },
                else_block: Some(Block {
                    stmts: vec![spanned(Stmt::Expr(str_lit("false")))],
                }),
            }
        }
        _ => method_call(expr, "to_str", vec![]),
    }
}

// ── Record Derives ──

fn gen_record_debug(td: &TypeDef) -> FnDef {
    // "TypeName(field1: val1, field2: val2)"
    let mut expr = str_lit(&format!("{}(", td.name));

    for (i, field) in td.fields.iter().enumerate() {
        if i > 0 {
            expr = str_concat(expr, str_lit(", "));
        }
        expr = str_concat(expr, str_lit(&format!("{}: ", field.name)));
        expr = str_concat(expr, val_to_str(field_access("self", &field.name), &field.ty));
    }

    expr = str_concat(expr, str_lit(")"));

    make_fndef(
        format!("{}_debug", td.name),
        vec![self_param(&td.name)],
        Some(TypeExpr::Named("Str".to_string())),
        Block {
            stmts: vec![spanned(Stmt::Expr(expr))],
        },
    )
}

fn gen_record_eq(td: &TypeDef) -> FnDef {
    // self.f1 == other.f1 && self.f2 == other.f2 && ...
    let expr = if td.fields.is_empty() {
        Expr::BoolLit(true)
    } else {
        let mut result = bin_op(
            BinOp::Eq,
            field_access("self", &td.fields[0].name),
            field_access("other", &td.fields[0].name),
        );
        for field in &td.fields[1..] {
            let cmp = bin_op(
                BinOp::Eq,
                field_access("self", &field.name),
                field_access("other", &field.name),
            );
            result = bin_op(BinOp::And, result, cmp);
        }
        result
    };

    make_fndef(
        format!("{}_eq", td.name),
        vec![self_param(&td.name), other_param(&td.name)],
        Some(TypeExpr::Named("Bool".to_string())),
        Block {
            stmts: vec![spanned(Stmt::Expr(expr))],
        },
    )
}

fn gen_record_clone(td: &TypeDef) -> FnDef {
    let fields: Vec<(String, Expr)> = td
        .fields
        .iter()
        .map(|f| (f.name.clone(), field_access("self", &f.name)))
        .collect();

    let expr = Expr::RecordConstruct {
        type_name: td.name.clone(),
        fields,
    };

    make_fndef(
        format!("{}_clone", td.name),
        vec![self_param(&td.name)],
        Some(TypeExpr::Named(td.name.clone())),
        Block {
            stmts: vec![spanned(Stmt::Expr(expr))],
        },
    )
}

fn gen_record_serialize(td: &TypeDef) -> FnDef {
    // {"field1": val1, "field2": val2}
    let mut expr = str_lit("{");

    for (i, field) in td.fields.iter().enumerate() {
        if i > 0 {
            expr = str_concat(expr, str_lit(", "));
        }
        expr = str_concat(expr, str_lit(&format!("\"{}\": ", field.name)));
        expr = str_concat(expr, val_to_json(field_access("self", &field.name), &field.ty));
    }

    expr = str_concat(expr, str_lit("}"));

    make_fndef(
        format!("{}_toJson", td.name),
        vec![self_param(&td.name)],
        Some(TypeExpr::Named("Str".to_string())),
        Block {
            stmts: vec![spanned(Stmt::Expr(expr))],
        },
    )
}

// ── Enum Derives ──

fn gen_enum_debug(ed: &EnumDef) -> FnDef {
    let arms: Vec<MatchArm> = ed
        .variants
        .iter()
        .map(|v| {
            let bindings: Vec<String> = v.fields.iter().map(|f| f.name.clone()).collect();

            let body = if v.fields.is_empty() {
                str_lit(&v.name)
            } else {
                let mut expr = str_lit(&format!("{}(", v.name));
                for (i, field) in v.fields.iter().enumerate() {
                    if i > 0 {
                        expr = str_concat(expr, str_lit(", "));
                    }
                    expr = str_concat(expr, str_lit(&format!("{}: ", field.name)));
                    expr = str_concat(expr, val_to_str(ident(&field.name), &field.ty));
                }
                expr = str_concat(expr, str_lit(")"));
                expr
            };

            MatchArm {
                pattern: Pattern::Variant {
                    name: v.name.clone(),
                    bindings,
                },
                guard: None,
                body,
            }
        })
        .collect();

    let expr = Expr::Match {
        subject: Box::new(ident("self")),
        arms,
    };

    make_fndef(
        format!("{}_debug", ed.name),
        vec![self_param(&ed.name)],
        Some(TypeExpr::Named("Str".to_string())),
        Block {
            stmts: vec![spanned(Stmt::Expr(expr))],
        },
    )
}

#[allow(dead_code)]
fn gen_enum_eq(ed: &EnumDef) -> FnDef {
    // Strategy: match self, for each variant match other with same variant, compare fields.
    // Uses nested match expressions.
    let arms: Vec<MatchArm> = ed
        .variants
        .iter()
        .map(|v| {
            // Bind self's fields with prefixed names
            let self_bindings: Vec<String> = v
                .fields
                .iter()
                .map(|f| format!("self_{}", f.name))
                .collect();

            // Inner match on other
            let other_bindings: Vec<String> = v
                .fields
                .iter()
                .map(|f| format!("other_{}", f.name))
                .collect();

            let eq_expr = if v.fields.is_empty() {
                Expr::BoolLit(true)
            } else {
                let mut result = bin_op(
                    BinOp::Eq,
                    ident(&format!("self_{}", v.fields[0].name)),
                    ident(&format!("other_{}", v.fields[0].name)),
                );
                for field in &v.fields[1..] {
                    let cmp = bin_op(
                        BinOp::Eq,
                        ident(&format!("self_{}", field.name)),
                        ident(&format!("other_{}", field.name)),
                    );
                    result = bin_op(BinOp::And, result, cmp);
                }
                result
            };

            let other_match_arm = MatchArm {
                pattern: Pattern::Variant {
                    name: v.name.clone(),
                    bindings: other_bindings,
                },
                guard: None,
                body: eq_expr,
            };

            let wildcard_arm = MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expr::BoolLit(false),
            };

            let inner_match = Expr::Match {
                subject: Box::new(ident("other")),
                arms: vec![other_match_arm, wildcard_arm],
            };

            MatchArm {
                pattern: Pattern::Variant {
                    name: v.name.clone(),
                    bindings: self_bindings,
                },
                guard: None,
                body: inner_match,
            }
        })
        .collect();

    let expr = Expr::Match {
        subject: Box::new(ident("self")),
        arms,
    };

    make_fndef(
        format!("{}_eq", ed.name),
        vec![self_param(&ed.name), other_param(&ed.name)],
        Some(TypeExpr::Named("Bool".to_string())),
        Block {
            stmts: vec![spanned(Stmt::Expr(expr))],
        },
    )
}

fn gen_enum_clone(ed: &EnumDef) -> FnDef {
    // Enums are value types — clone is identity
    make_fndef(
        format!("{}_clone", ed.name),
        vec![self_param(&ed.name)],
        Some(TypeExpr::Named(ed.name.clone())),
        Block {
            stmts: vec![spanned(Stmt::Expr(ident("self")))],
        },
    )
}

fn gen_enum_serialize(ed: &EnumDef) -> FnDef {
    let arms: Vec<MatchArm> = ed
        .variants
        .iter()
        .map(|v| {
            let bindings: Vec<String> = v.fields.iter().map(|f| f.name.clone()).collect();

            let body = if v.fields.is_empty() {
                // {"type": "VariantName"}
                str_lit(&format!("{{\"type\": \"{}\"}}", v.name))
            } else {
                // {"type": "VariantName", "field1": val1, ...}
                let mut expr = str_lit(&format!("{{\"type\": \"{}\", ", v.name));
                for (i, field) in v.fields.iter().enumerate() {
                    if i > 0 {
                        expr = str_concat(expr, str_lit(", "));
                    }
                    expr = str_concat(expr, str_lit(&format!("\"{}\": ", field.name)));
                    expr = str_concat(expr, val_to_json(ident(&field.name), &field.ty));
                }
                expr = str_concat(expr, str_lit("}"));
                expr
            };

            MatchArm {
                pattern: Pattern::Variant {
                    name: v.name.clone(),
                    bindings,
                },
                guard: None,
                body,
            }
        })
        .collect();

    let expr = Expr::Match {
        subject: Box::new(ident("self")),
        arms,
    };

    make_fndef(
        format!("{}_toJson", ed.name),
        vec![self_param(&ed.name)],
        Some(TypeExpr::Named("Str".to_string())),
        Block {
            stmts: vec![spanned(Stmt::Expr(expr))],
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_known_traits() {
        assert!(validate_deriving(&["Debug".into(), "Eq".into()]).is_ok());
        assert!(validate_deriving(&["Clone".into(), "Serialize".into()]).is_ok());
    }

    #[test]
    fn validate_unknown_trait() {
        let result = validate_deriving(&["Unknown".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn record_eq_generates_fndef() {
        let td = TypeDef {
            name: "Point".into(),
            type_params: vec![],
            fields: vec![
                FieldDef { name: "x".into(), ty: TypeExpr::Named("Int".into()) },
                FieldDef { name: "y".into(), ty: TypeExpr::Named("Int".into()) },
            ],
            deriving: vec!["Eq".into()],
        };
        let fns = generate_record_derives(&td).unwrap();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "Point_eq");
        assert_eq!(fns[0].params.len(), 2);
    }

    #[test]
    fn record_all_derives() {
        let td = TypeDef {
            name: "User".into(),
            type_params: vec![],
            fields: vec![
                FieldDef { name: "name".into(), ty: TypeExpr::Named("Str".into()) },
                FieldDef { name: "age".into(), ty: TypeExpr::Named("Int".into()) },
            ],
            deriving: vec!["Debug".into(), "Eq".into(), "Clone".into(), "Serialize".into()],
        };
        let fns = generate_record_derives(&td).unwrap();
        assert_eq!(fns.len(), 4);
        let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"User_debug"));
        assert!(names.contains(&"User_eq"));
        assert!(names.contains(&"User_clone"));
        assert!(names.contains(&"User_toJson"));
    }

    #[test]
    fn enum_debug_generates_fndef() {
        let ed = EnumDef {
            name: "Color".into(),
            variants: vec![
                Variant { name: "Red".into(), fields: vec![] },
                Variant { name: "Green".into(), fields: vec![] },
            ],
            deriving: vec!["Debug".into()],
        };
        let fns = generate_enum_derives(&ed).unwrap();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "Color_debug");
    }

    #[test]
    fn no_deriving_produces_no_fns() {
        let td = TypeDef {
            name: "Empty".into(),
            type_params: vec![],
            fields: vec![],
            deriving: vec![],
        };
        let fns = generate_record_derives(&td).unwrap();
        assert!(fns.is_empty());
    }
}
