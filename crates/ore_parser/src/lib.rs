pub mod ast;
pub mod fmt;

use ast::*;
use ore_lexer::Token;
use ore_lexer::Spanned;

pub fn parse(tokens: Vec<Spanned>) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub offset: usize,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at {}:{}: {}", self.line, self.col, self.msg)
    }
}

struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Spanned>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).map(|s| &s.token).unwrap_or(&Token::Eof)
    }

    fn peek_line(&self) -> usize {
        self.tokens.get(self.pos).map(|s| s.line).unwrap_or(0)
    }

    fn advance(&mut self) -> &Token {
        let tok = self.tokens.get(self.pos).map(|s| &s.token).unwrap_or(&Token::Eof);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        if self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("expected {:?}, got {:?}", expected, self.peek())))
        }
    }

    fn error(&self, msg: String) -> ParseError {
        let spanned = self.tokens.get(self.pos);
        ParseError {
            msg,
            offset: spanned.map(|s| s.offset).unwrap_or(0),
            line: spanned.map(|s| s.line).unwrap_or(0),
            col: spanned.map(|s| s.col).unwrap_or(0),
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }

    /// Skip newlines, indents, and dedents (for inside brackets/braces)
    fn skip_whitespace_tokens(&mut self) {
        loop {
            match self.peek() {
                Token::Newline | Token::Indent | Token::Dedent => { self.advance(); }
                _ => break,
            }
        }
    }

    // ── Program ──

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut items = Vec::new();
        self.skip_newlines();

        while self.peek() != &Token::Eof {
            items.push(self.parse_item()?);
            self.skip_newlines();
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        match self.peek() {
            Token::Fn => Ok(Item::FnDef(self.parse_fn_def()?)),
            Token::Type => self.parse_type_or_enum(),
            Token::Impl => self.parse_impl_block(),
            Token::Trait => self.parse_trait_def(),
            Token::Use => self.parse_use(),
            _ => Err(self.error(format!("expected item, got {:?}", self.peek()))),
        }
    }

    fn parse_use(&mut self) -> Result<Item, ParseError> {
        self.expect(&Token::Use)?;
        match self.peek().clone() {
            Token::StringLit(path) => {
                self.advance();
                Ok(Item::Use { path })
            }
            Token::Ident(name) => {
                self.advance();
                // Bare identifier becomes "name.ore"
                Ok(Item::Use { path: format!("{}.ore", name) })
            }
            _ => Err(self.error("expected string path or module name after use".into())),
        }
    }

    fn parse_impl_block(&mut self) -> Result<Item, ParseError> {
        self.expect(&Token::Impl)?;
        let first_name = match self.peek().clone() {
            Token::Ident(n) => { self.advance(); n }
            _ => return Err(self.error("expected type or trait name after impl".into())),
        };

        // Check for "impl Trait for Type"
        let (trait_name, type_name) = if self.peek() == &Token::For {
            self.advance(); // consume 'for'
            let tn = match self.peek().clone() {
                Token::Ident(n) => { self.advance(); n }
                _ => return Err(self.error("expected type name after 'for'".into())),
            };
            (Some(first_name), tn)
        } else {
            (None, first_name)
        };

        self.skip_newlines();
        self.expect(&Token::Indent)?;
        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek() == &Token::Dedent || self.peek() == &Token::Eof {
                break;
            }
            methods.push(self.parse_fn_def()?);
        }
        if self.peek() == &Token::Dedent {
            self.advance();
        }

        if let Some(trait_name) = trait_name {
            Ok(Item::ImplTrait { trait_name, type_name, methods })
        } else {
            Ok(Item::ImplBlock { type_name, methods })
        }
    }

    fn parse_trait_def(&mut self) -> Result<Item, ParseError> {
        self.expect(&Token::Trait)?;
        let name = match self.peek().clone() {
            Token::Ident(n) => { self.advance(); n }
            _ => return Err(self.error("expected trait name".into())),
        };
        self.skip_newlines();
        self.expect(&Token::Indent)?;
        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek() == &Token::Dedent || self.peek() == &Token::Eof {
                break;
            }
            methods.push(self.parse_trait_method()?);
        }
        if self.peek() == &Token::Dedent {
            self.advance();
        }
        Ok(Item::TraitDef(TraitDef { name, methods }))
    }

    /// Parse a trait method signature: fn name params -> RetType (no body)
    fn parse_trait_method(&mut self) -> Result<TraitMethod, ParseError> {
        self.expect(&Token::Fn)?;
        let name = match self.peek().clone() {
            Token::Ident(n) => { self.advance(); n }
            _ => return Err(self.error("expected method name".into())),
        };
        let mut params = Vec::new();
        let mut ret_type = None;
        loop {
            match self.peek() {
                Token::Arrow => {
                    self.advance();
                    ret_type = Some(self.parse_type_expr()?);
                    break;
                }
                Token::Newline | Token::Dedent | Token::Eof => break,
                Token::Ident(_) => {
                    params.push(self.parse_param()?);
                }
                _ => break,
            }
        }
        Ok(TraitMethod { name, params, ret_type })
    }

    fn parse_type_or_enum(&mut self) -> Result<Item, ParseError> {
        self.expect(&Token::Type)?;
        let name = match self.peek().clone() {
            Token::Ident(n) => { self.advance(); n }
            _ => return Err(self.error("expected type name".into())),
        };

        // Parse optional type parameters
        let type_params = self.parse_optional_type_params()?;

        // If next is '{', it's a record type
        if self.peek() == &Token::LBrace {
            return self.parse_record_body(name, type_params).map(Item::TypeDef);
        }

        // Otherwise it's an enum (indented variants)
        self.skip_newlines();
        self.expect(&Token::Indent)?;
        let mut variants = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek() == &Token::Dedent || self.peek() == &Token::Eof {
                break;
            }
            variants.push(self.parse_variant()?);
        }
        if self.peek() == &Token::Dedent {
            self.advance();
        }
        Ok(Item::EnumDef(EnumDef { name, variants }))
    }

    fn parse_variant(&mut self) -> Result<Variant, ParseError> {
        let name = match self.peek().clone() {
            Token::Ident(n) => { self.advance(); n }
            _ => return Err(self.error("expected variant name".into())),
        };
        let mut fields = Vec::new();
        if self.peek() == &Token::LParen {
            self.advance();
            while self.peek() != &Token::RParen {
                let field_name = match self.peek().clone() {
                    Token::Ident(n) => { self.advance(); n }
                    _ => return Err(self.error("expected field name".into())),
                };
                self.expect(&Token::Colon)?;
                let ty = self.parse_type_expr()?;
                fields.push(FieldDef { name: field_name, ty });
                if self.peek() == &Token::Comma {
                    self.advance();
                }
            }
            self.expect(&Token::RParen)?;
        }
        Ok(Variant { name, fields })
    }

    // ── Type Definitions ──

    fn parse_record_body(&mut self, name: String, type_params: Vec<TypeParam>) -> Result<TypeDef, ParseError> {
        self.expect(&Token::LBrace)?;
        self.skip_whitespace_tokens();
        let mut fields = Vec::new();
        while self.peek() != &Token::RBrace {
            self.skip_whitespace_tokens();
            let field_name = match self.peek().clone() {
                Token::Ident(n) => { self.advance(); n }
                _ => return Err(self.error("expected field name".into())),
            };
            self.expect(&Token::Colon)?;
            let ty = self.parse_type_expr()?;
            fields.push(FieldDef { name: field_name, ty });
            self.skip_whitespace_tokens();
            if self.peek() == &Token::Comma {
                self.advance();
                self.skip_whitespace_tokens();
            }
        }
        self.skip_whitespace_tokens();
        self.expect(&Token::RBrace)?;
        Ok(TypeDef { name, type_params, fields })
    }

    // ── Function Definitions ──

    fn parse_fn_def(&mut self) -> Result<FnDef, ParseError> {
        self.expect(&Token::Fn)?;

        let name = match self.peek().clone() {
            Token::Ident(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("expected function name".into())),
        };

        // Parse optional type parameters: fn name[T, U]
        let type_params = self.parse_optional_type_params()?;

        // Parse parameters: name:Type pairs before -> or newline
        let mut params = Vec::new();
        let mut ret_type = None;

        loop {
            match self.peek() {
                Token::Arrow => {
                    self.advance();
                    ret_type = Some(self.parse_type_expr()?);
                    break;
                }
                Token::Newline | Token::Indent | Token::Eof => break,
                Token::Ident(_) => {
                    params.push(self.parse_param()?);
                }
                _ => break,
            }
        }

        self.skip_newlines();
        let body = self.parse_block()?;

        Ok(FnDef {
            name,
            type_params,
            params,
            ret_type,
            body,
        })
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let name = match self.peek().clone() {
            Token::Ident(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("expected parameter name".into())),
        };
        self.expect(&Token::Colon)?;
        let ty = self.parse_type_expr()?;
        Ok(Param { name, ty })
    }

    /// Parse optional type parameters: [T, U, V]
    fn parse_optional_type_params(&mut self) -> Result<Vec<TypeParam>, ParseError> {
        if self.peek() != &Token::LBracket {
            return Ok(Vec::new());
        }
        self.advance(); // consume '['
        let mut params = Vec::new();
        if self.peek() != &Token::RBracket {
            params.push(self.parse_type_param()?);
            while self.peek() == &Token::Comma {
                self.advance();
                params.push(self.parse_type_param()?);
            }
        }
        self.expect(&Token::RBracket)?;
        Ok(params)
    }

    fn parse_type_param(&mut self) -> Result<TypeParam, ParseError> {
        let name = match self.peek().clone() {
            Token::Ident(n) => { self.advance(); n }
            _ => return Err(self.error("expected type parameter name".into())),
        };
        // Check for trait bound: T: TraitName
        let bound = if self.peek() == &Token::Colon {
            self.advance(); // consume ':'
            match self.peek().clone() {
                Token::Ident(b) => { self.advance(); Some(b) }
                _ => return Err(self.error("expected trait name after ':'".into())),
            }
        } else {
            None
        };
        Ok(TypeParam { name, bound })
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        let base = match self.peek().clone() {
            Token::Ident(name) => {
                self.advance();
                // Check for generic type: Name[Type, Type]
                if self.peek() == &Token::LBracket {
                    self.advance(); // consume '['
                    let mut args = Vec::new();
                    args.push(self.parse_type_expr()?);
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_type_expr()?);
                    }
                    self.expect(&Token::RBracket)?;
                    TypeExpr::Generic(name, args)
                } else {
                    TypeExpr::Named(name)
                }
            }
            Token::LParen => {
                // Function type: (Type, Type -> RetType) or (Type -> RetType)
                self.advance(); // consume '('
                let mut types = Vec::new();
                if self.peek() != &Token::RParen {
                    types.push(self.parse_type_expr()?);
                    while self.peek() == &Token::Comma {
                        self.advance();
                        types.push(self.parse_type_expr()?);
                    }
                }
                // Expect '->' then return type
                self.expect(&Token::Arrow)?;
                let ret = self.parse_type_expr()?;
                self.expect(&Token::RParen)?;
                TypeExpr::Fn { params: types, ret: Box::new(ret) }
            }
            _ => return Err(self.error("expected type name".into())),
        };
        // Check for optional suffix: Type?
        if self.peek() == &Token::QuestionMark {
            self.advance();
            Ok(TypeExpr::Generic("Option".to_string(), vec![base]))
        } else {
            Ok(base)
        }
    }

    // ── Blocks ──

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect(&Token::Indent)?;
        let mut stmts = Vec::new();

        loop {
            self.skip_newlines();
            if self.peek() == &Token::Dedent || self.peek() == &Token::Eof {
                break;
            }
            let line = self.peek_line();
            let stmt = self.parse_stmt()?;
            stmts.push(SpannedStmt { stmt, line });
        }

        if self.peek() == &Token::Dedent {
            self.advance();
        }

        Ok(Block { stmts })
    }

    // ── Statements ──

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Return => {
                self.advance();
                if matches!(self.peek(), Token::Newline | Token::Dedent | Token::Eof) {
                    Ok(Stmt::Return(None))
                } else {
                    let expr = self.parse_expr(0)?;
                    Ok(Stmt::Return(Some(expr)))
                }
            }
            Token::For => {
                self.advance();
                let var = match self.peek().clone() {
                    Token::Ident(n) => { self.advance(); n }
                    _ => return Err(self.error("expected variable name after for".into())),
                };
                self.expect(&Token::In)?;
                let start = self.parse_expr(3)?; // Parse at higher precedence to stop at ..
                if self.peek() == &Token::DotDot {
                    // Range loop: for x in start..end
                    self.advance();
                    let end = self.parse_expr(0)?;
                    self.skip_newlines();
                    let body = self.parse_block()?;
                    Ok(Stmt::ForIn { var, start, end, body })
                } else {
                    // Collection iteration: for x in list
                    self.skip_newlines();
                    let body = self.parse_block()?;
                    Ok(Stmt::ForEach { var, iterable: start, body })
                }
            }
            Token::While => {
                self.advance();
                let cond = self.parse_expr(0)?;
                self.skip_newlines();
                let body = self.parse_block()?;
                Ok(Stmt::While { cond, body })
            }
            Token::Loop => {
                self.advance();
                self.skip_newlines();
                let body = self.parse_block()?;
                Ok(Stmt::Loop { body })
            }
            Token::Break => {
                self.advance();
                Ok(Stmt::Break)
            }
            Token::Continue => {
                self.advance();
                Ok(Stmt::Continue)
            }
            Token::Ident(name) if name == "print" => {
                self.advance();
                let expr = self.parse_expr(0)?;
                Ok(Stmt::Expr(Expr::Print(Box::new(expr))))
            }
            Token::Ident(name) if name == "sleep" => {
                self.advance();
                let expr = self.parse_expr(0)?;
                Ok(Stmt::Expr(Expr::Sleep(Box::new(expr))))
            }
            Token::Spawn => {
                self.advance();
                let expr = self.parse_expr(0)?;
                Ok(Stmt::Spawn(expr))
            }
            Token::Mut => {
                self.advance();
                let name = match self.peek().clone() {
                    Token::Ident(n) => { self.advance(); n }
                    _ => return Err(self.error("expected variable name after mut".into())),
                };
                self.expect(&Token::ColonEq)?;
                let value = self.parse_expr(0)?;
                Ok(Stmt::Let { name, mutable: true, value })
            }
            Token::Ident(_) => {
                // Could be: binding (x := ...), assignment (x = ...),
                // index assign (x[i] = ...), field assign (x.f = ...), or expression
                let saved = self.pos;
                let name = match self.peek().clone() {
                    Token::Ident(n) => { self.advance(); n }
                    _ => unreachable!(),
                };

                match self.peek() {
                    Token::ColonEq => {
                        self.advance();
                        let value = self.parse_expr(0)?;
                        Ok(Stmt::Let { name, mutable: false, value })
                    }
                    Token::Eq => {
                        self.advance();
                        let value = self.parse_expr(0)?;
                        Ok(Stmt::Assign { name, value })
                    }
                    Token::PlusEq | Token::MinusEq | Token::StarEq | Token::SlashEq | Token::PercentEq => {
                        let op = match self.peek() {
                            Token::PlusEq => BinOp::Add,
                            Token::MinusEq => BinOp::Sub,
                            Token::StarEq => BinOp::Mul,
                            Token::SlashEq => BinOp::Div,
                            Token::PercentEq => BinOp::Mod,
                            _ => unreachable!(),
                        };
                        self.advance();
                        let rhs = self.parse_expr(0)?;
                        let value = Expr::BinOp {
                            left: Box::new(Expr::Ident(name.clone())),
                            op,
                            right: Box::new(rhs),
                        };
                        Ok(Stmt::Assign { name, value })
                    }
                    _ => {
                        // Backtrack and parse as expression, then check for assignment
                        self.pos = saved;
                        let expr = self.parse_expr(0)?;
                        // Check for index/field assignment: expr[idx] = val, expr.field = val
                        if self.peek() == &Token::Eq {
                            self.advance();
                            let value = self.parse_expr(0)?;
                            match expr {
                                Expr::Index { object, index } => {
                                    Ok(Stmt::IndexAssign { object: *object, index: *index, value })
                                }
                                Expr::FieldAccess { object, field } => {
                                    Ok(Stmt::FieldAssign { object: *object, field, value })
                                }
                                _ => Err(self.error("invalid assignment target".into())),
                            }
                        } else {
                            Ok(Stmt::Expr(expr))
                        }
                    }
                }
            }
            _ => {
                let expr = self.parse_expr(0)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    // ── Pratt Expression Parser ──

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Try operator (?) - highest precedence postfix
            if self.peek() == &Token::QuestionMark {
                let try_bp = 16;
                if try_bp >= min_bp {
                    self.advance(); // consume '?'
                    lhs = Expr::Try(Box::new(lhs));
                    continue;
                }
            }

            // Indexing: expr[expr]
            if self.peek() == &Token::LBracket {
                let idx_bp = 15;
                if idx_bp >= min_bp {
                    self.advance(); // consume '['
                    let index = self.parse_expr(0)?;
                    self.expect(&Token::RBracket)?;
                    lhs = Expr::Index {
                        object: Box::new(lhs),
                        index: Box::new(index),
                    };
                    continue;
                }
            }

            // Field access / method call (highest precedence postfix)
            if self.peek() == &Token::Dot || self.peek() == &Token::QuestionDot {
                let optional = self.peek() == &Token::QuestionDot;
                if let Some(Token::Ident(_)) = self.tokens.get(self.pos + 1).map(|s| &s.token) {
                    let dot_bp = 15; // Higher than any infix op
                    if dot_bp >= min_bp {
                        self.advance(); // consume '.' or '?.'
                        let field = match self.peek().clone() {
                            Token::Ident(f) => { self.advance(); f }
                            _ => unreachable!(),
                        };
                        // Check if this is a method call: field followed by '('
                        if self.peek() == &Token::LParen {
                            self.advance(); // consume '('
                            self.skip_whitespace_tokens();
                            let mut args = Vec::new();
                            if self.peek() != &Token::RParen {
                                args.push(self.parse_expr(0)?);
                                self.skip_whitespace_tokens();
                                while self.peek() == &Token::Comma {
                                    self.advance();
                                    self.skip_whitespace_tokens();
                                    if self.peek() == &Token::RParen {
                                        break; // trailing comma
                                    }
                                    args.push(self.parse_expr(0)?);
                                    self.skip_whitespace_tokens();
                                }
                            }
                            self.skip_whitespace_tokens();
                            self.expect(&Token::RParen)?;
                            if optional {
                                lhs = Expr::OptionalMethodCall {
                                    object: Box::new(lhs),
                                    method: field,
                                    args,
                                };
                            } else {
                                lhs = Expr::MethodCall {
                                    object: Box::new(lhs),
                                    method: field,
                                    args,
                                };
                            }
                        } else {
                            if optional {
                                lhs = Expr::OptionalChain {
                                    object: Box::new(lhs),
                                    field,
                                };
                            } else {
                                lhs = Expr::FieldAccess {
                                    object: Box::new(lhs),
                                    field,
                                };
                            }
                        }
                        continue;
                    }
                }
            }

            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                Token::EqEq => BinOp::Eq,
                Token::BangEq => BinOp::NotEq,
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,
                Token::And => BinOp::And,
                Token::Or => BinOp::Or,
                Token::Pipe => {
                    // Check for `| each|` syntax: parallel pipeline
                    if let Some(next) = self.tokens.get(self.pos + 1) {
                        if matches!(&next.token, Token::Ident(n) if n == "each") {
                            if let Some(after) = self.tokens.get(self.pos + 2) {
                                if matches!(&after.token, Token::Pipe) {
                                    // `| each| f` desugars to `.par_map(f)`
                                    let (l_bp, _r_bp) = (1u8, 2u8); // same as Pipe
                                    if l_bp < min_bp {
                                        break;
                                    }
                                    self.advance(); // consume first `|`
                                    self.advance(); // consume `each`
                                    self.advance(); // consume second `|`
                                    let rhs = self.parse_expr(2)?;
                                    lhs = Expr::MethodCall {
                                        object: Box::new(lhs),
                                        method: "par_map".to_string(),
                                        args: vec![rhs],
                                    };
                                    continue;
                                }
                            }
                        }
                    }
                    // Check for `| else default` syntax: Option/Result fallback
                    if let Some(next) = self.tokens.get(self.pos + 1) {
                        if matches!(&next.token, Token::Else) {
                            let l_bp = 1u8;
                            if l_bp < min_bp {
                                break;
                            }
                            self.advance(); // consume `|`
                            self.advance(); // consume `else`
                            let default_expr = self.parse_expr(2)?;
                            lhs = Expr::MethodCall {
                                object: Box::new(lhs),
                                method: "unwrap_or".to_string(),
                                args: vec![default_expr],
                            };
                            continue;
                        }
                    }
                    BinOp::Pipe
                }
                Token::Colon => {
                    // Check next token after colon
                    let next = self.tokens.get(self.pos + 1).map(|s| &s.token);
                    if matches!(next, None | Some(Token::Dedent) | Some(Token::Eof)) {
                        break;
                    }

                    let l_bp = 2;
                    if l_bp < min_bp {
                        break;
                    }

                    // If next is Newline (then Indent), it's a pattern match block
                    if matches!(next, Some(Token::Newline)) {
                        // Check if there's an Indent after the Newline
                        let after_nl = self.tokens.get(self.pos + 2).map(|s| &s.token);
                        if matches!(after_nl, Some(Token::Indent)) {
                            self.advance(); // consume ':'
                            self.skip_newlines();
                            let arms = self.parse_match_arms()?;
                            lhs = Expr::Match {
                                subject: Box::new(lhs),
                                arms,
                            };
                            continue;
                        }
                    }

                    self.advance(); // consume ':'
                    // Inline conditional: cond : trueExpr [: elseExpr]
                    // Parse then_expr stopping at ':' (use min_bp=3 so ':' at bp=2 stops)
                    let then_expr = self.parse_expr(3)?;
                    let else_expr = if self.peek() == &Token::Colon {
                        self.advance();
                        Some(Box::new(self.parse_expr(3)?))
                    } else {
                        None
                    };
                    lhs = Expr::ColonMatch {
                        cond: Box::new(lhs),
                        then_expr: Box::new(then_expr),
                        else_expr,
                    };
                    continue;
                }
                _ => break,
            };

            let (l_bp, r_bp) = infix_binding_power(op);
            if l_bp < min_bp {
                break;
            }

            self.advance(); // consume operator
            let rhs = self.parse_expr(r_bp)?;

            // Comparison chaining: `a < b < c` → `a < b and b < c`
            if matches!(op, BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq) {
                let next_op = match self.peek() {
                    Token::Lt => Some(BinOp::Lt),
                    Token::Gt => Some(BinOp::Gt),
                    Token::LtEq => Some(BinOp::LtEq),
                    Token::GtEq => Some(BinOp::GtEq),
                    _ => None,
                };
                if let Some(op2) = next_op {
                    self.advance(); // consume second comparison operator
                    let rhs2 = self.parse_expr(r_bp)?;
                    let first_cmp = Expr::BinOp {
                        op,
                        left: Box::new(lhs),
                        right: Box::new(rhs.clone()),
                    };
                    let second_cmp = Expr::BinOp {
                        op: op2,
                        left: Box::new(rhs),
                        right: Box::new(rhs2),
                    };
                    lhs = Expr::BinOp {
                        op: BinOp::And,
                        left: Box::new(first_cmp),
                        right: Box::new(second_cmp),
                    };
                    continue;
                }
            }

            lhs = Expr::BinOp {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Token::LBracket => {
                self.advance(); // consume '['
                self.skip_whitespace_tokens();
                let mut elements = Vec::new();
                if self.peek() != &Token::RBracket {
                    elements.push(self.parse_expr(0)?);
                    self.skip_whitespace_tokens();
                    while self.peek() == &Token::Comma {
                        self.advance();
                        self.skip_whitespace_tokens();
                        if self.peek() == &Token::RBracket {
                            break; // trailing comma
                        }
                        elements.push(self.parse_expr(0)?);
                        self.skip_whitespace_tokens();
                    }
                }
                self.skip_whitespace_tokens();
                self.expect(&Token::RBracket)?;
                Ok(Expr::ListLit(elements))
            }
            Token::LBrace => {
                self.advance(); // consume '{'
                self.skip_whitespace_tokens();
                let mut entries = Vec::new();
                if self.peek() != &Token::RBrace {
                    // Parse key at bp=3 so ':' (bp=2) is not consumed as colon-match
                    let key = self.parse_expr(3)?;
                    self.expect(&Token::Colon)?;
                    let value = self.parse_expr(0)?;
                    entries.push((key, value));
                    self.skip_whitespace_tokens();
                    while self.peek() == &Token::Comma {
                        self.advance();
                        self.skip_whitespace_tokens();
                        if self.peek() == &Token::RBrace {
                            break; // trailing comma
                        }
                        let key = self.parse_expr(3)?;
                        self.expect(&Token::Colon)?;
                        let value = self.parse_expr(0)?;
                        entries.push((key, value));
                        self.skip_whitespace_tokens();
                    }
                }
                self.skip_whitespace_tokens();
                self.expect(&Token::RBrace)?;
                Ok(Expr::MapLit(entries))
            }
            Token::Int(n) => {
                self.advance();
                Ok(Expr::IntLit(n))
            }
            Token::Float(f) => {
                self.advance();
                Ok(Expr::FloatLit(f))
            }
            Token::Break => {
                self.advance();
                Ok(Expr::Break)
            }
            Token::True => {
                self.advance();
                Ok(Expr::BoolLit(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::BoolLit(false))
            }
            Token::StringLit(s) => {
                self.advance();
                Ok(Expr::StringLit(s))
            }
            Token::StringStart(s) => {
                self.advance();
                let mut parts = Vec::new();
                if !s.is_empty() {
                    parts.push(StringPart::Lit(s));
                }
                // Parse the first interpolated expression
                let expr = self.parse_expr(0)?;
                parts.push(StringPart::Expr(expr));
                // Continue with StringMid/StringEnd
                loop {
                    match self.peek().clone() {
                        Token::StringMid(s) => {
                            self.advance();
                            if !s.is_empty() {
                                parts.push(StringPart::Lit(s));
                            }
                            let expr = self.parse_expr(0)?;
                            parts.push(StringPart::Expr(expr));
                        }
                        Token::StringEnd(s) => {
                            self.advance();
                            if !s.is_empty() {
                                parts.push(StringPart::Lit(s));
                            }
                            break;
                        }
                        _ => return Err(self.error("expected string continuation".into())),
                    }
                }
                Ok(Expr::StringInterp(parts))
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_expr(PREFIX_BP)?;
                Ok(Expr::UnaryMinus(Box::new(expr)))
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_expr(PREFIX_BP)?;
                Ok(Expr::UnaryNot(Box::new(expr)))
            }
            Token::LParen => {
                // Could be: (expr), (param => body), (p1, p2 => body)
                let saved = self.pos;
                self.advance(); // consume '('

                // Try to parse as lambda: look for ident [, ident]* =>
                if let Some(lambda) = self.try_parse_lambda()? {
                    return Ok(lambda);
                }

                // Not a lambda, backtrack and parse as grouped expression
                self.pos = saved;
                self.advance(); // consume '('
                let expr = self.parse_expr(0)?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::None_ => {
                self.advance();
                Ok(Expr::OptionNone)
            }
            Token::Some => {
                self.advance();
                self.expect(&Token::LParen)?;
                let inner = self.parse_expr(0)?;
                self.expect(&Token::RParen)?;
                Ok(Expr::OptionSome(Box::new(inner)))
            }
            Token::Ok_ => {
                self.advance();
                self.expect(&Token::LParen)?;
                let inner = self.parse_expr(0)?;
                self.expect(&Token::RParen)?;
                Ok(Expr::ResultOk(Box::new(inner)))
            }
            Token::Err_ => {
                self.advance();
                self.expect(&Token::LParen)?;
                let inner = self.parse_expr(0)?;
                self.expect(&Token::RParen)?;
                Ok(Expr::ResultErr(Box::new(inner)))
            }
            Token::Match => {
                self.advance();
                let subject = self.parse_expr(0)?;
                self.skip_newlines();
                self.expect(&Token::Indent)?;
                let mut arms = Vec::new();
                while self.peek() != &Token::Dedent && self.peek() != &Token::Eof {
                    self.skip_newlines();
                    if self.peek() == &Token::Dedent {
                        break;
                    }
                    let mut pattern = self.parse_pattern()?;
                    // Check for or-patterns: `pattern | pattern`
                    if self.peek() == &Token::Pipe {
                        let mut alternatives = vec![pattern];
                        while self.peek() == &Token::Pipe {
                            self.advance();
                            alternatives.push(self.parse_pattern()?);
                        }
                        pattern = Pattern::Or(alternatives);
                    }
                    let guard = if self.peek() == &Token::If {
                        self.advance();
                        Some(Box::new(self.parse_expr(0)?))
                    } else {
                        None
                    };
                    self.expect(&Token::Arrow)?;
                    let body = self.parse_expr(0)?;
                    arms.push(MatchArm { pattern, guard, body });
                    self.skip_newlines();
                }
                if self.peek() == &Token::Dedent {
                    self.advance();
                }
                Ok(Expr::Match { subject: Box::new(subject), arms })
            }
            Token::If => {
                self.advance();
                let cond = self.parse_expr(0)?;
                self.skip_newlines();
                let then_block = self.parse_block()?;
                self.skip_newlines();
                let else_block = if self.peek() == &Token::Else {
                    self.advance();
                    self.skip_newlines();
                    if self.peek() == &Token::If {
                        // else if — parse the if as a single expression in a block
                        let line = self.peek_line();
                        let nested_if = self.parse_expr(0)?;
                        Some(Block { stmts: vec![SpannedStmt { stmt: Stmt::Expr(nested_if), line }] })
                    } else {
                        Some(self.parse_block()?)
                    }
                } else {
                    None
                };
                Ok(Expr::IfElse {
                    cond: Box::new(cond),
                    then_block,
                    else_block,
                })
            }
            Token::Ident(name) if name == "print" => {
                // Allow 'print' as an expression (e.g. inside lambdas)
                self.advance();
                let expr = self.parse_expr(0)?;
                Ok(Expr::Print(Box::new(expr)))
            }
            Token::Ident(name) => {
                self.advance();
                // Bare lambda: ident [ident...] => expr (without parens)
                if self.peek() == &Token::FatArrow {
                    self.advance(); // consume '=>'
                    let body = self.parse_expr(0)?;
                    return Ok(Expr::Lambda {
                        params: vec![name],
                        body: Box::new(body),
                    });
                }
                // Multi-param bare lambda: a b => expr, a b c => expr, etc.
                if let Token::Ident(_) = self.peek() {
                    // Look ahead to see if there's a => after consecutive idents
                    let saved = self.pos;
                    let mut params = vec![name.clone()];
                    while let Token::Ident(p) = self.peek().clone() {
                        self.advance();
                        params.push(p);
                        if self.peek() == &Token::FatArrow {
                            self.advance(); // consume '=>'
                            let body = self.parse_expr(0)?;
                            return Ok(Expr::Lambda {
                                params,
                                body: Box::new(body),
                            });
                        }
                    }
                    // Not a lambda — restore position
                    self.pos = saved;
                }
                // Check for function call or record construction
                if self.peek() == &Token::LParen {
                    let saved = self.pos;
                    self.advance(); // consume '('

                    // Check if this is record construction: Name(field: expr, ...)
                    // Peek: Ident followed by Colon
                    self.skip_whitespace_tokens();
                    if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        if let Token::Ident(_) = self.peek() {
                            if let Some(Token::Colon) = self.tokens.get(self.pos + 1).map(|s| &s.token) {
                                // Record construction
                                let mut fields = Vec::new();
                                self.skip_whitespace_tokens();
                                while self.peek() != &Token::RParen {
                                    self.skip_whitespace_tokens();
                                    let field_name = match self.peek().clone() {
                                        Token::Ident(f) => { self.advance(); f }
                                        _ => return Err(self.error("expected field name".into())),
                                    };
                                    self.expect(&Token::Colon)?;
                                    let val = self.parse_expr(0)?;
                                    fields.push((field_name, val));
                                    self.skip_whitespace_tokens();
                                    if self.peek() == &Token::Comma {
                                        self.advance();
                                        self.skip_whitespace_tokens();
                                    }
                                }
                                self.skip_whitespace_tokens();
                                self.expect(&Token::RParen)?;
                                return Ok(Expr::RecordConstruct {
                                    type_name: name,
                                    fields,
                                });
                            }
                        }
                    }

                    // Regular function call
                    self.pos = saved;
                    self.advance(); // consume '('
                    self.skip_whitespace_tokens();
                    let mut args = Vec::new();
                    if self.peek() != &Token::RParen {
                        args.push(self.parse_expr(0)?);
                        self.skip_whitespace_tokens();
                        while self.peek() == &Token::Comma {
                            self.advance();
                            self.skip_whitespace_tokens();
                            if self.peek() == &Token::RParen {
                                break; // trailing comma
                            }
                            args.push(self.parse_expr(0)?);
                            self.skip_whitespace_tokens();
                        }
                    }
                    self.skip_whitespace_tokens();
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call {
                        func: Box::new(Expr::Ident(name)),
                        args,
                    })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            _ => Err(self.error(format!("expected expression, got {:?}", self.peek()))),
        }
    }

    /// Try to parse lambda inside parens. Already consumed '('.
    /// Supports two forms:
    ///   (a, b => body)   — FatArrow inside parens
    ///   (a, b) => body   — FatArrow after closing paren
    /// Returns None if it doesn't look like a lambda (caller should backtrack).
    fn try_parse_lambda(&mut self) -> Result<Option<Expr>, ParseError> {
        let saved = self.pos;

        // Collect identifiers separated by commas
        let mut params = Vec::new();
        match self.peek().clone() {
            Token::Ident(name) => {
                self.advance();
                params.push(name);
            }
            _ => {
                self.pos = saved;
                return Ok(None);
            }
        }

        loop {
            match self.peek() {
                Token::Comma => {
                    self.advance();
                    match self.peek().clone() {
                        Token::Ident(name) => {
                            self.advance();
                            params.push(name);
                        }
                        _ => {
                            self.pos = saved;
                            return Ok(None);
                        }
                    }
                }
                Token::FatArrow => break,
                Token::RParen => {
                    // Check for (params) => body form
                    self.advance(); // consume ')'
                    if self.peek() == &Token::FatArrow {
                        self.advance(); // consume '=>'
                        let body = self.parse_expr(0)?;
                        return Ok(Some(Expr::Lambda {
                            params,
                            body: Box::new(body),
                        }));
                    }
                    // Not a lambda, backtrack
                    self.pos = saved;
                    return Ok(None);
                }
                _ => {
                    self.pos = saved;
                    return Ok(None);
                }
            }
        }

        // Consume '=>' (for the (params => body) form)
        self.expect(&Token::FatArrow)?;
        let body = self.parse_expr(0)?;
        self.expect(&Token::RParen)?;
        Ok(Some(Expr::Lambda {
            params,
            body: Box::new(body),
        }))
    }

    /// Parse match arms inside an indented block.
    /// Each arm: Pattern -> expr
    fn parse_match_arms(&mut self) -> Result<Vec<MatchArm>, ParseError> {
        self.expect(&Token::Indent)?;
        let mut arms = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek() == &Token::Dedent || self.peek() == &Token::Eof {
                break;
            }
            arms.push(self.parse_match_arm()?);
        }
        if self.peek() == &Token::Dedent {
            self.advance();
        }
        Ok(arms)
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let mut pattern = self.parse_pattern()?;
        // Check for or-patterns: `pattern | pattern | pattern`
        if self.peek() == &Token::Pipe {
            let mut alternatives = vec![pattern];
            while self.peek() == &Token::Pipe {
                self.advance();
                alternatives.push(self.parse_pattern()?);
            }
            pattern = Pattern::Or(alternatives);
        }
        // Check for guard: `pattern if condition -> body`
        let guard = if self.peek() == &Token::If {
            self.advance();
            Some(Box::new(self.parse_expr(0)?))
        } else {
            None
        };
        self.expect(&Token::Arrow)?;
        let body = self.parse_expr(0)?;
        Ok(MatchArm { pattern, guard, body })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek().clone() {
            Token::Ident(name) if name == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::None_ => {
                self.advance();
                Ok(Pattern::Variant { name: "None".to_string(), bindings: vec![] })
            }
            Token::Some => {
                self.advance();
                let mut bindings = Vec::new();
                while let Token::Ident(b) = self.peek().clone() {
                    if self.peek() == &Token::Arrow {
                        break;
                    }
                    self.advance();
                    bindings.push(b);
                }
                Ok(Pattern::Variant { name: "Some".to_string(), bindings })
            }
            Token::Ok_ => {
                self.advance();
                let mut bindings = Vec::new();
                while let Token::Ident(b) = self.peek().clone() {
                    if self.peek() == &Token::Arrow {
                        break;
                    }
                    self.advance();
                    bindings.push(b);
                }
                Ok(Pattern::Variant { name: "Ok".to_string(), bindings })
            }
            Token::Err_ => {
                self.advance();
                let mut bindings = Vec::new();
                while let Token::Ident(b) = self.peek().clone() {
                    if self.peek() == &Token::Arrow {
                        break;
                    }
                    self.advance();
                    bindings.push(b);
                }
                Ok(Pattern::Variant { name: "Err".to_string(), bindings })
            }
            Token::Ident(name) => {
                self.advance();
                // If followed by identifiers (not Arrow), these are bindings
                let mut bindings = Vec::new();
                while let Token::Ident(b) = self.peek().clone() {
                    if self.peek() == &Token::Arrow {
                        break;
                    }
                    self.advance();
                    bindings.push(b);
                }
                Ok(Pattern::Variant { name, bindings })
            }
            Token::Int(n) => {
                self.advance();
                // Check for range pattern: 1..10
                if self.peek() == &Token::DotDot {
                    self.advance();
                    match self.peek().clone() {
                        Token::Int(end) => {
                            self.advance();
                            Ok(Pattern::Range(n, end))
                        }
                        Token::Minus => {
                            self.advance();
                            if let Token::Int(end) = self.peek().clone() {
                                self.advance();
                                Ok(Pattern::Range(n, -end))
                            } else {
                                Err(self.error("expected integer after '-' in range pattern".into()))
                            }
                        }
                        _ => Err(self.error("expected integer after '..' in range pattern".into())),
                    }
                } else {
                    Ok(Pattern::IntLit(n))
                }
            }
            Token::Float(f) => {
                self.advance();
                Ok(Pattern::FloatLit(f))
            }
            Token::True => {
                self.advance();
                Ok(Pattern::BoolLit(true))
            }
            Token::False => {
                self.advance();
                Ok(Pattern::BoolLit(false))
            }
            Token::StringLit(s) => {
                self.advance();
                Ok(Pattern::StringLit(s))
            }
            Token::Minus => {
                // Negative literal: -3 or range: -3..10
                self.advance();
                match self.peek().clone() {
                    Token::Int(n) => {
                        self.advance();
                        let neg = -n;
                        // Check for range pattern: -3..10
                        if self.peek() == &Token::DotDot {
                            self.advance();
                            match self.peek().clone() {
                                Token::Int(end) => {
                                    self.advance();
                                    Ok(Pattern::Range(neg, end))
                                }
                                Token::Minus => {
                                    self.advance();
                                    if let Token::Int(end) = self.peek().clone() {
                                        self.advance();
                                        Ok(Pattern::Range(neg, -end))
                                    } else {
                                        Err(self.error("expected integer after '-' in range pattern".into()))
                                    }
                                }
                                _ => Err(self.error("expected integer after '..' in range pattern".into())),
                            }
                        } else {
                            Ok(Pattern::IntLit(neg))
                        }
                    }
                    Token::Float(f) => {
                        self.advance();
                        Ok(Pattern::FloatLit(-f))
                    }
                    _ => Err(self.error("expected number after '-' in pattern".into())),
                }
            }
            _ => Err(self.error(format!("expected pattern, got {:?}", self.peek()))),
        }
    }
}

const PREFIX_BP: u8 = 13;

fn infix_binding_power(op: BinOp) -> (u8, u8) {
    match op {
        BinOp::Pipe => (1, 2),
        BinOp::Or => (3, 4),
        BinOp::And => (5, 6),
        BinOp::Eq | BinOp::NotEq => (7, 8),
        BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => (7, 8),
        BinOp::Add | BinOp::Sub => (9, 10),
        BinOp::Mul | BinOp::Div | BinOp::Mod => (11, 12),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ore_lexer::lex;

    fn parse_src(src: &str) -> Program {
        let tokens = lex(src).expect("lex failed");
        parse(tokens).expect("parse failed")
    }

    #[test]
    fn test_simple_fn_with_binding() {
        let prog = parse_src("fn main\n  x := 42\n");
        assert_eq!(prog.items.len(), 1);
        match &prog.items[0] {
            Item::FnDef(f) => {
                assert_eq!(f.name, "main");
                assert_eq!(f.body.stmts.len(), 1);
                match &f.body.stmts[0].stmt {
                    Stmt::Let { name, mutable, value } => {
                        assert_eq!(name, "x");
                        assert!(!mutable);
                        assert_eq!(*value, Expr::IntLit(42));
                    }
                    _ => panic!("expected let"),
                }
            }
            _ => panic!("expected FnDef"),
        }
    }

    #[test]
    fn test_arithmetic() {
        let prog = parse_src("fn main\n  x := 1 + 2 * 3\n");
        match &prog.items[0] {
            Item::FnDef(f) => {
                match &f.body.stmts[0].stmt {
                    Stmt::Let { value, .. } => {
                        // Should be 1 + (2 * 3) due to precedence
                        match value {
                            Expr::BinOp { op: BinOp::Add, left, right } => {
                                assert_eq!(**left, Expr::IntLit(1));
                                match right.as_ref() {
                                    Expr::BinOp { op: BinOp::Mul, .. } => {}
                                    _ => panic!("expected mul on right"),
                                }
                            }
                            _ => panic!("expected add"),
                        }
                    }
                    _ => panic!("expected let"),
                }
            }
            _ => panic!("expected FnDef"),
        }
    }

    #[test]
    fn test_print() {
        let prog = parse_src("fn main\n  print 42\n");
        match &prog.items[0] {
            Item::FnDef(f) => {
                match &f.body.stmts[0].stmt {
                    Stmt::Expr(Expr::Print(inner)) => {
                        assert_eq!(**inner, Expr::IntLit(42));
                    }
                    _ => panic!("expected print"),
                }
            }
            _ => panic!("expected FnDef"),
        }
    }

    #[test]
    fn test_fn_with_params() {
        let prog = parse_src("fn add a:Int b:Int -> Int\n  a + b\n");
        match &prog.items[0] {
            Item::FnDef(f) => {
                assert_eq!(f.name, "add");
                assert_eq!(f.params.len(), 2);
                assert_eq!(f.params[0].name, "a");
                assert_eq!(f.params[0].ty, TypeExpr::Named("Int".into()));
                assert_eq!(f.ret_type, Some(TypeExpr::Named("Int".into())));
            }
            _ => panic!("expected FnDef"),
        }
    }

    #[test]
    fn test_function_call() {
        let prog = parse_src("fn main\n  foo(1, 2)\n");
        match &prog.items[0] {
            Item::FnDef(f) => {
                match &f.body.stmts[0].stmt {
                    Stmt::Expr(Expr::Call { func, args }) => {
                        assert_eq!(**func, Expr::Ident("foo".into()));
                        assert_eq!(args.len(), 2);
                    }
                    _ => panic!("expected call"),
                }
            }
            _ => panic!("expected FnDef"),
        }
    }

    #[test]
    fn test_unary_minus() {
        let prog = parse_src("fn main\n  x := -42\n");
        match &prog.items[0] {
            Item::FnDef(f) => {
                match &f.body.stmts[0].stmt {
                    Stmt::Let { value, .. } => {
                        match value {
                            Expr::UnaryMinus(inner) => {
                                assert_eq!(**inner, Expr::IntLit(42));
                            }
                            _ => panic!("expected unary minus"),
                        }
                    }
                    _ => panic!("expected let"),
                }
            }
            _ => panic!("expected FnDef"),
        }
    }

    #[test]
    fn test_if_else() {
        let prog = parse_src("fn main\n  if true\n    1\n  else\n    2\n");
        match &prog.items[0] {
            Item::FnDef(f) => {
                match &f.body.stmts[0].stmt {
                    Stmt::Expr(Expr::IfElse { cond, then_block, else_block }) => {
                        assert_eq!(**cond, Expr::BoolLit(true));
                        assert_eq!(then_block.stmts.len(), 1);
                        assert!(else_block.is_some());
                    }
                    _ => panic!("expected if/else"),
                }
            }
            _ => panic!("expected FnDef"),
        }
    }
}
