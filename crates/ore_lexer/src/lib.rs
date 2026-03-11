#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    Ident(String),
    StringLit(String),
    // String interpolation: "hello {expr} world" becomes
    // StringStart("hello "), <expr tokens>, StringMid(" world") or StringEnd(" world")
    StringStart(String),
    StringMid(String),
    StringEnd(String),

    // Keywords
    Fn,
    Mut,
    If,
    Else,
    True,
    False,
    Return,
    For,
    While,
    Loop,
    Break,
    In,
    Type,
    Impl,
    Trait,
    Some,
    None_,
    Ok_,
    Err_,
    Use,
    Pub,
    Spawn,
    Match,
    Continue,
    Test,
    Assert,
    AssertEq,
    AssertNe,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    ColonEq,     // :=
    Eq,          // =
    EqEq,        // ==
    BangEq,      // !=
    Lt,          // <
    Gt,          // >
    LtEq,        // <=
    GtEq,        // >=
    Arrow,       // ->
    FatArrow,    // =>
    Pipe,        // |
    Colon,       // :
    Dot,         // .
    DotDot,      // ..
    QuestionMark, // ?
    QuestionDot,  // ?.
    PlusEq,      // +=
    MinusEq,     // -=
    StarEq,      // *=
    SlashEq,     // /=
    PercentEq,   // %=
    And,
    Or,
    Not,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,

    // Layout
    Newline,
    Indent,
    Dedent,

    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub offset: usize,
    pub len: usize,
    pub line: usize,
    pub col: usize,
}

pub fn lex(source: &str) -> Result<Vec<Spanned>, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.run()?;
    let tokens = fixup_multiline_pipes(lexer.tokens);
    Ok(tokens)
}

/// Post-processing: enable multi-line pipe continuation.
/// When a line starts with `|` at a deeper indent, remove the Newline + Indent
/// tokens so the pipe is parsed as a continuation of the previous expression.
/// Also removes the matching Dedent.
fn fixup_multiline_pipes(tokens: Vec<Spanned>) -> Vec<Spanned> {
    let len = tokens.len();
    let mut remove = vec![false; len];
    let mut pending_dedent_removes: usize = 0;

    let mut i = 0;
    while i + 2 < len {
        // Pattern: Newline [Indent] Pipe
        if matches!(tokens[i].token, Token::Newline) {
            let mut j = i + 1;
            let mut indent_count = 0;
            while j < len && matches!(tokens[j].token, Token::Indent) {
                indent_count += 1;
                j += 1;
            }
            if j < len && matches!(tokens[j].token, Token::Pipe) {
                // Remove the Newline and all Indent tokens
                remove[i] = true;
                for k in (i + 1)..j {
                    remove[k] = true;
                }
                pending_dedent_removes += indent_count;
                i = j; // skip to Pipe
                continue;
            }
        }
        // Remove matching Dedent tokens
        if pending_dedent_removes > 0 && matches!(tokens[i].token, Token::Dedent) {
            remove[i] = true;
            pending_dedent_removes -= 1;
            i += 1;
            continue;
        }
        i += 1;
    }
    // Handle remaining dedent removes at end
    if pending_dedent_removes > 0 {
        let mut j = i;
        while j < len && pending_dedent_removes > 0 {
            if matches!(tokens[j].token, Token::Dedent) {
                remove[j] = true;
                pending_dedent_removes -= 1;
            }
            j += 1;
        }
    }

    tokens.into_iter().enumerate()
        .filter(|(idx, _)| !remove[*idx])
        .map(|(_, t)| t)
        .collect()
}

#[derive(Debug)]
pub struct LexError {
    pub msg: String,
    pub offset: usize,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lex error at {}:{}: {}", self.line, self.col, self.msg)
    }
}

struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    tokens: Vec<Spanned>,
    indent_stack: Vec<usize>,
    at_line_start: bool,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            src: source.as_bytes(),
            pos: 0,
            tokens: Vec::new(),
            indent_stack: vec![0],
            at_line_start: true,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<u8> {
        self.src.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let ch = self.src.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == b'\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn emit(&mut self, token: Token, start: usize) {
        // Compute line/col from start offset by scanning source up to start
        let (line, col) = self.offset_to_line_col(start);
        self.tokens.push(Spanned {
            token,
            offset: start,
            len: self.pos - start,
            line,
            col,
        });
    }

    fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for &b in &self.src[..offset.min(self.src.len())] {
            if b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    fn lex_error(&self, msg: String) -> LexError {
        let (line, col) = self.offset_to_line_col(self.pos);
        LexError { msg, offset: self.pos, line, col }
    }

    fn lex_error_at(&self, msg: String, offset: usize) -> LexError {
        let (line, col) = self.offset_to_line_col(offset);
        LexError { msg, offset, line, col }
    }

    fn current_indent(&self) -> usize {
        *self.indent_stack.last().unwrap()
    }

    fn run(&mut self) -> Result<(), LexError> {
        while self.pos < self.src.len() {
            if self.at_line_start {
                self.handle_indentation()?;
                self.at_line_start = false;
            }

            match self.peek() {
                None => break,
                Some(b'\n') => {
                    // Emit newline if the last non-layout token isn't already a newline
                    let start = self.pos;
                    self.advance();
                    // Skip multiple blank lines
                    if !self.last_is_newline_or_indent() {
                        self.emit(Token::Newline, start);
                    }
                    self.at_line_start = true;
                }
                Some(b'\r') => {
                    self.advance();
                    // Treat \r\n as single newline
                }
                Some(b' ') | Some(b'\t') => {
                    self.advance();
                }
                Some(b'-') if self.peek2() == Some(b'-') => {
                    // Line comment
                    while self.pos < self.src.len() && self.peek() != Some(b'\n') {
                        self.advance();
                    }
                }
                Some(_) => {
                    self.lex_token()?;
                }
            }
        }

        // Emit remaining dedents
        let end = self.pos;
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.emit(Token::Dedent, end);
        }

        self.emit(Token::Eof, end);
        Ok(())
    }

    fn last_is_newline_or_indent(&self) -> bool {
        self.tokens.last().map_or(true, |t| {
            matches!(t.token, Token::Newline | Token::Indent | Token::Dedent)
        })
    }

    fn handle_indentation(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        let mut indent = 0;
        while self.pos < self.src.len() {
            match self.peek() {
                Some(b' ') => {
                    indent += 1;
                    self.advance();
                }
                Some(b'\t') => {
                    indent += 4; // Tabs = 4 spaces
                    self.advance();
                }
                _ => break,
            }
        }

        // Skip blank lines and comment-only lines
        if self.pos >= self.src.len() || self.peek() == Some(b'\n') || self.peek() == Some(b'\r') {
            return Ok(());
        }
        if self.peek() == Some(b'-') && self.peek2() == Some(b'-') {
            return Ok(());
        }

        let current = self.current_indent();
        if indent > current {
            self.indent_stack.push(indent);
            self.emit(Token::Indent, start);
        } else if indent < current {
            while indent < self.current_indent() {
                self.indent_stack.pop();
                self.emit(Token::Dedent, start);
            }
            if indent != self.current_indent() {
                return Err(self.lex_error_at("inconsistent indentation".to_string(), start));
            }
        }

        Ok(())
    }

    fn lex_token(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        let ch = self.peek().unwrap();

        match ch {
            b'0'..=b'9' => self.lex_number()?,
            b'"' => self.lex_string()?,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_ident_or_keyword(),
            b'+' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::PlusEq, start);
                } else {
                    self.emit(Token::Plus, start);
                }
            }
            b'*' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::StarEq, start);
                } else {
                    self.emit(Token::Star, start);
                }
            }
            b'/' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::SlashEq, start);
                } else {
                    self.emit(Token::Slash, start);
                }
            }
            b'%' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::PercentEq, start);
                } else {
                    self.emit(Token::Percent, start);
                }
            }
            b'(' => { self.advance(); self.emit(Token::LParen, start); }
            b')' => { self.advance(); self.emit(Token::RParen, start); }
            b'{' => { self.advance(); self.emit(Token::LBrace, start); }
            b'}' => { self.advance(); self.emit(Token::RBrace, start); }
            b'[' => { self.advance(); self.emit(Token::LBracket, start); }
            b']' => { self.advance(); self.emit(Token::RBracket, start); }
            b',' => { self.advance(); self.emit(Token::Comma, start); }
            b'?' => {
                self.advance();
                if self.peek() == Some(b'.') {
                    self.advance();
                    self.emit(Token::QuestionDot, start);
                } else {
                    self.emit(Token::QuestionMark, start);
                }
            }
            b'|' => { self.advance(); self.emit(Token::Pipe, start); }
            b'-' => {
                self.advance();
                if self.peek() == Some(b'>') {
                    self.advance();
                    self.emit(Token::Arrow, start);
                } else if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::MinusEq, start);
                } else {
                    self.emit(Token::Minus, start);
                }
            }
            b'=' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::EqEq, start);
                } else if self.peek() == Some(b'>') {
                    self.advance();
                    self.emit(Token::FatArrow, start);
                } else {
                    self.emit(Token::Eq, start);
                }
            }
            b':' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::ColonEq, start);
                } else {
                    self.emit(Token::Colon, start);
                }
            }
            b'!' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::BangEq, start);
                } else {
                    return Err(self.lex_error_at("unexpected '!'".to_string(), start));
                }
            }
            b'<' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::LtEq, start);
                } else {
                    self.emit(Token::Lt, start);
                }
            }
            b'>' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(Token::GtEq, start);
                } else {
                    self.emit(Token::Gt, start);
                }
            }
            b'.' => {
                self.advance();
                if self.peek() == Some(b'.') {
                    self.advance();
                    self.emit(Token::DotDot, start);
                } else {
                    self.emit(Token::Dot, start);
                }
            }
            _ => {
                return Err(self.lex_error_at(format!("unexpected character '{}'", ch as char), start));
            }
        }
        Ok(())
    }

    fn lex_number(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        let mut is_float = false;

        while let Some(ch) = self.peek() {
            match ch {
                b'0'..=b'9' | b'_' => { self.advance(); }
                b'.' if !is_float && self.peek2().map_or(false, |c| c.is_ascii_digit()) => {
                    is_float = true;
                    self.advance();
                }
                _ => break,
            }
        }

        let text: String = self.src[start..self.pos]
            .iter()
            .filter(|&&c| c != b'_')
            .map(|&c| c as char)
            .collect();

        if is_float {
            let val: f64 = text.parse().map_err(|_| self.lex_error_at("invalid float literal".to_string(), start))?;
            self.emit(Token::Float(val), start);
        } else {
            let val: i64 = text.parse().map_err(|_| self.lex_error_at("invalid integer literal".to_string(), start))?;
            self.emit(Token::Int(val), start);
        }
        Ok(())
    }

    fn lex_string(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        self.advance(); // skip opening "
        let mut s = String::new();
        let mut has_interp = false;

        loop {
            match self.peek() {
                None => {
                    return Err(self.lex_error_at("unterminated string".to_string(), start));
                }
                Some(b'"') => {
                    self.advance();
                    break;
                }
                Some(b'{') => {
                    // String interpolation
                    let seg_start = self.pos;
                    self.advance(); // skip '{'
                    if !has_interp {
                        self.emit(Token::StringStart(s.clone()), start);
                    } else {
                        self.emit(Token::StringMid(s.clone()), seg_start);
                    }
                    s.clear();
                    has_interp = true;

                    // Lex tokens inside {} until matching '}'
                    let mut depth = 1;
                    while depth > 0 {
                        match self.peek() {
                            None => {
                                return Err(self.lex_error_at("unterminated interpolation".to_string(), seg_start));
                            }
                            Some(b'}') => {
                                depth -= 1;
                                if depth > 0 {
                                    self.lex_token()?;
                                } else {
                                    self.advance(); // skip closing '}'
                                }
                            }
                            Some(b'{') => {
                                depth += 1;
                                self.lex_token()?;
                            }
                            Some(b' ') | Some(b'\t') => {
                                self.advance();
                            }
                            _ => {
                                self.lex_token()?;
                            }
                        }
                    }
                }
                Some(b'\\') => {
                    self.advance();
                    match self.advance() {
                        Some(b'n') => s.push('\n'),
                        Some(b't') => s.push('\t'),
                        Some(b'r') => s.push('\r'),
                        Some(b'0') => s.push('\0'),
                        Some(b'\\') => s.push('\\'),
                        Some(b'"') => s.push('"'),
                        Some(b'{') => s.push('{'),
                        Some(other) => {
                            return Err(self.lex_error_at(format!("unknown escape '\\{}'", other as char), self.pos - 1));
                        }
                        None => {
                            return Err(self.lex_error("unterminated escape".to_string()));
                        }
                    }
                }
                Some(_) => {
                    s.push(self.advance().unwrap() as char);
                }
            }
        }

        if has_interp {
            self.emit(Token::StringEnd(s), start);
        } else {
            self.emit(Token::StringLit(s), start);
        }
        Ok(())
    }

    fn lex_ident_or_keyword(&mut self) {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = std::str::from_utf8(&self.src[start..self.pos]).unwrap();
        let token = match text {
            "fn" => Token::Fn,
            "mut" => Token::Mut,
            "if" => Token::If,
            "else" => Token::Else,
            "true" => Token::True,
            "false" => Token::False,
            "return" => Token::Return,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "for" => Token::For,
            "while" => Token::While,
            "loop" => Token::Loop,
            "break" => Token::Break,
            "in" => Token::In,
            "type" => Token::Type,
            "impl" => Token::Impl,
            "trait" => Token::Trait,
            "Some" => Token::Some,
            "None" => Token::None_,
            "Ok" => Token::Ok_,
            "Err" => Token::Err_,
            "use" => Token::Use,
            "pub" => Token::Pub,
            "spawn" => Token::Spawn,
            "match" => Token::Match,
            "continue" => Token::Continue,
            "test" => Token::Test,
            "assert" => Token::Assert,
            "assert_eq" => Token::AssertEq,
            "assert_ne" => Token::AssertNe,
            _ => Token::Ident(text.to_string()),
        };
        self.emit(token, start);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(src: &str) -> Vec<Token> {
        lex(src).unwrap().into_iter().map(|s| s.token).collect()
    }

    #[test]
    fn test_integer_literals() {
        assert_eq!(tokens("42"), vec![Token::Int(42), Token::Eof]);
        assert_eq!(tokens("1_000"), vec![Token::Int(1000), Token::Eof]);
    }

    #[test]
    fn test_operators() {
        assert_eq!(
            tokens(":= + - * /"),
            vec![
                Token::ColonEq, Token::Plus, Token::Minus, Token::Star, Token::Slash,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_comparison_ops() {
        assert_eq!(
            tokens("< > <= >= == !="),
            vec![
                Token::Lt, Token::Gt, Token::LtEq, Token::GtEq, Token::EqEq, Token::BangEq,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_keywords() {
        assert_eq!(
            tokens("fn mut if else true false return"),
            vec![
                Token::Fn, Token::Mut, Token::If, Token::Else,
                Token::True, Token::False, Token::Return,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_indent_dedent() {
        let src = "fn main\n  x := 42\n  y := 10\n";
        let toks = tokens(src);
        assert_eq!(
            toks,
            vec![
                Token::Fn, Token::Ident("main".into()), Token::Newline,
                Token::Indent,
                Token::Ident("x".into()), Token::ColonEq, Token::Int(42), Token::Newline,
                Token::Ident("y".into()), Token::ColonEq, Token::Int(10), Token::Newline,
                Token::Dedent,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            tokens("42 -- this is a comment\n10"),
            vec![Token::Int(42), Token::Newline, Token::Int(10), Token::Eof]
        );
    }

    #[test]
    fn test_arrow_and_pipe() {
        assert_eq!(
            tokens("-> | =>"),
            vec![Token::Arrow, Token::Pipe, Token::FatArrow, Token::Eof]
        );
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            tokens("\"hello\""),
            vec![Token::StringLit("hello".into()), Token::Eof]
        );
    }

    #[test]
    fn test_boolean_keywords() {
        assert_eq!(
            tokens("and or not"),
            vec![Token::And, Token::Or, Token::Not, Token::Eof]
        );
    }

    #[test]
    fn test_nested_indent() {
        let src = "a\n  b\n    c\n  d\ne\n";
        let toks = tokens(src);
        assert_eq!(
            toks,
            vec![
                Token::Ident("a".into()), Token::Newline,
                Token::Indent,
                Token::Ident("b".into()), Token::Newline,
                Token::Indent,
                Token::Ident("c".into()), Token::Newline,
                Token::Dedent,
                Token::Ident("d".into()), Token::Newline,
                Token::Dedent,
                Token::Ident("e".into()), Token::Newline,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_float_literal() {
        assert_eq!(tokens("3.14"), vec![Token::Float(3.14), Token::Eof]);
    }

    #[test]
    fn test_string_interpolation() {
        let toks = tokens("\"hello {x}!\"");
        assert_eq!(
            toks,
            vec![
                Token::StringStart("hello ".into()),
                Token::Ident("x".into()),
                Token::StringEnd("!".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_string_multi_interp() {
        let toks = tokens("\"a{x}b{y}c\"");
        assert_eq!(
            toks,
            vec![
                Token::StringStart("a".into()),
                Token::Ident("x".into()),
                Token::StringMid("b".into()),
                Token::Ident("y".into()),
                Token::StringEnd("c".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_string_escaped_brace() {
        let toks = tokens("\"hello \\{world}\"");
        assert_eq!(
            toks,
            vec![Token::StringLit("hello {world}".into()), Token::Eof]
        );
    }
}
