pub mod ast;

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
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at offset {}: {}", self.offset, self.msg)
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

    fn peek_offset(&self) -> usize {
        self.tokens.get(self.pos).map(|s| s.offset).unwrap_or(0)
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
        ParseError {
            msg,
            offset: self.peek_offset(),
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
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
        let type_name = match self.peek().clone() {
            Token::Ident(n) => { self.advance(); n }
            _ => return Err(self.error("expected type name after impl".into())),
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
        Ok(Item::ImplBlock { type_name, methods })
    }

    fn parse_type_or_enum(&mut self) -> Result<Item, ParseError> {
        self.expect(&Token::Type)?;
        let name = match self.peek().clone() {
            Token::Ident(n) => { self.advance(); n }
            _ => return Err(self.error("expected type name".into())),
        };

        // If next is '{', it's a record type
        if self.peek() == &Token::LBrace {
            return self.parse_record_body(name).map(Item::TypeDef);
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

    fn parse_record_body(&mut self, name: String) -> Result<TypeDef, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while self.peek() != &Token::RBrace {
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
        self.expect(&Token::RBrace)?;
        Ok(TypeDef { name, fields })
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

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        match self.peek().clone() {
            Token::Ident(name) => {
                self.advance();
                Ok(TypeExpr::Named(name))
            }
            _ => Err(self.error("expected type name".into())),
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
            stmts.push(self.parse_stmt()?);
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
                self.expect(&Token::DotDot)?;
                let end = self.parse_expr(0)?;
                self.skip_newlines();
                let body = self.parse_block()?;
                Ok(Stmt::ForIn { var, start, end, body })
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
                // Could be: binding (x := ...), assignment (x = ...), or expression
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
                    _ => {
                        // Backtrack and parse as expression
                        self.pos = saved;
                        let expr = self.parse_expr(0)?;
                        Ok(Stmt::Expr(expr))
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
            if self.peek() == &Token::Dot {
                if let Some(Token::Ident(_)) = self.tokens.get(self.pos + 1).map(|s| &s.token) {
                    let dot_bp = 15; // Higher than any infix op
                    if dot_bp >= min_bp {
                        self.advance(); // consume '.'
                        let field = match self.peek().clone() {
                            Token::Ident(f) => { self.advance(); f }
                            _ => unreachable!(),
                        };
                        // Check if this is a method call: field followed by '('
                        if self.peek() == &Token::LParen {
                            self.advance(); // consume '('
                            let mut args = Vec::new();
                            if self.peek() != &Token::RParen {
                                args.push(self.parse_expr(0)?);
                                while self.peek() == &Token::Comma {
                                    self.advance();
                                    args.push(self.parse_expr(0)?);
                                }
                            }
                            self.expect(&Token::RParen)?;
                            lhs = Expr::MethodCall {
                                object: Box::new(lhs),
                                method: field,
                                args,
                            };
                        } else {
                            lhs = Expr::FieldAccess {
                                object: Box::new(lhs),
                                field,
                            };
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
                Token::Pipe => BinOp::Pipe,
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
                let mut elements = Vec::new();
                if self.peek() != &Token::RBracket {
                    elements.push(self.parse_expr(0)?);
                    while self.peek() == &Token::Comma {
                        self.advance();
                        if self.peek() == &Token::RBracket {
                            break; // trailing comma
                        }
                        elements.push(self.parse_expr(0)?);
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::ListLit(elements))
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
            Token::If => {
                self.advance();
                let cond = self.parse_expr(0)?;
                self.skip_newlines();
                let then_block = self.parse_block()?;
                self.skip_newlines();
                let else_block = if self.peek() == &Token::Else {
                    self.advance();
                    self.skip_newlines();
                    Some(self.parse_block()?)
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
                // Bare lambda: ident => expr (without parens)
                if self.peek() == &Token::FatArrow {
                    self.advance(); // consume '=>'
                    let body = self.parse_expr(0)?;
                    return Ok(Expr::Lambda {
                        params: vec![name],
                        body: Box::new(body),
                    });
                }
                // Check for function call or record construction
                if self.peek() == &Token::LParen {
                    let saved = self.pos;
                    self.advance(); // consume '('

                    // Check if this is record construction: Name(field: expr, ...)
                    // Peek: Ident followed by Colon
                    if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        if let Token::Ident(_) = self.peek() {
                            if let Some(Token::Colon) = self.tokens.get(self.pos + 1).map(|s| &s.token) {
                                // Record construction
                                let mut fields = Vec::new();
                                while self.peek() != &Token::RParen {
                                    let field_name = match self.peek().clone() {
                                        Token::Ident(f) => { self.advance(); f }
                                        _ => return Err(self.error("expected field name".into())),
                                    };
                                    self.expect(&Token::Colon)?;
                                    let val = self.parse_expr(0)?;
                                    fields.push((field_name, val));
                                    if self.peek() == &Token::Comma {
                                        self.advance();
                                    }
                                }
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
                    let mut args = Vec::new();
                    if self.peek() != &Token::RParen {
                        args.push(self.parse_expr(0)?);
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expr(0)?);
                        }
                    }
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
                _ => {
                    self.pos = saved;
                    return Ok(None);
                }
            }
        }

        // Consume '=>'
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
        let pattern = self.parse_pattern()?;
        self.expect(&Token::Arrow)?;
        let body = self.parse_expr(0)?;
        Ok(MatchArm { pattern, body })
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
                match &f.body.stmts[0] {
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
                match &f.body.stmts[0] {
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
                match &f.body.stmts[0] {
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
                match &f.body.stmts[0] {
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
                match &f.body.stmts[0] {
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
                match &f.body.stmts[0] {
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
