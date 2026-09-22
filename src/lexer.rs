use crate::errors::LexError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Eof,
    Ident,
    String,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    RawScopeOpen, // {!
    Number,       // bare integer literal
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(token_type: TokenType, value: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            token_type,
            value: value.into(),
            line,
            col,
        }
    }
}

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    pub line: usize,
    pub col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Self {
            src,
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn errorf(&self, msg: impl Into<String>) -> LexError {
        LexError::new(self.line, self.col, msg)
    }

    #[inline]
    fn peek(&self) -> u8 {
        if self.pos >= self.src.len() {
            0
        } else {
            self.src[self.pos]
        }
    }

    #[inline]
    fn peek_at(&self, offset: usize) -> u8 {
        if self.pos + offset >= self.src.len() {
            0
        } else {
            self.src[self.pos + offset]
        }
    }

    pub fn advance(&mut self) -> u8 {
        let c = self.src[self.pos];
        self.pos += 1;
        if c == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexError> {
        while self.pos < self.src.len() {
            let c = self.peek();
            match c {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.advance();
                }
                b'/' if self.peek_at(1) == b'/' => {
                    while self.pos < self.src.len() && self.peek() != b'\n' {
                        self.advance();
                    }
                }
                b'/' if self.peek_at(1) == b'*' => {
                    let start_line = self.line;
                    let start_col = self.col;
                    self.advance();
                    self.advance();
                    let mut closed = false;
                    while self.pos < self.src.len() {
                        if self.peek() == b'*' && self.peek_at(1) == b'/' {
                            self.advance();
                            self.advance();
                            closed = true;
                            break;
                        }
                        self.advance();
                    }
                    if !closed {
                        return Err(LexError::new(
                            start_line,
                            start_col,
                            "unterminated block comment",
                        ));
                    }
                }
                _ => return Ok(()),
            }
        }
        Ok(())
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace_and_comments()?;
        if self.pos >= self.src.len() {
            return Ok(Token::new(TokenType::Eof, "", self.line, self.col));
        }

        let start_line = self.line;
        let start_col = self.col;
        let c = self.peek();

        match c {
            b'(' => {
                self.advance();
                Ok(Token::new(TokenType::LParen, "(", start_line, start_col))
            }
            b')' => {
                self.advance();
                Ok(Token::new(TokenType::RParen, ")", start_line, start_col))
            }
            b'{' => {
                if self.peek_at(1) == b'!' {
                    self.advance();
                    self.advance();
                    Ok(Token::new(
                        TokenType::RawScopeOpen,
                        "{!",
                        start_line,
                        start_col,
                    ))
                } else {
                    self.advance();
                    Ok(Token::new(TokenType::LBrace, "{", start_line, start_col))
                }
            }
            b'}' => {
                self.advance();
                Ok(Token::new(TokenType::RBrace, "}", start_line, start_col))
            }
            b',' => {
                self.advance();
                Ok(Token::new(TokenType::Comma, ",", start_line, start_col))
            }
            b':' => {
                self.advance();
                Ok(Token::new(TokenType::Colon, ":", start_line, start_col))
            }
            b'"' => self.read_string(),
            _ if is_ident_start(c) => self.read_ident(),
            b'0'..=b'9' => self.read_number(),
            _ => Err(self.errorf(format!("unexpected character {:?}", c as char))),
        }
    }

    fn read_number(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.col;
        let start = self.pos;
        while self.pos < self.src.len() && self.peek().is_ascii_digit() {
            self.advance();
        }
        let val = String::from_utf8_lossy(&self.src[start..self.pos]).into_owned();
        Ok(Token::new(TokenType::Number, val, start_line, start_col))
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.col;
        self.advance();

        let mut buf = Vec::new();
        loop {
            if self.pos >= self.src.len() {
                return Err(LexError::new(
                    start_line,
                    start_col,
                    "unterminated string literal",
                ));
            }
            let c = self.peek();
            if c == b'"' {
                self.advance();
                break;
            }
            if c == b'\\' && self.peek_at(1) == b'"' {
                self.advance();
                buf.push(self.advance());
                continue;
            }
            buf.push(self.advance());
        }

        let val = String::from_utf8_lossy(&buf).into_owned();
        Ok(Token::new(TokenType::String, val, start_line, start_col))
    }

    fn read_ident(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.col;
        let start = self.pos;
        while self.pos < self.src.len() && is_ident_cont(self.peek()) {
            self.advance();
        }
        let val = String::from_utf8_lossy(&self.src[start..self.pos]).into_owned();
        Ok(Token::new(TokenType::Ident, val, start_line, start_col))
    }

    pub fn read_raw_until_bang_brace(&mut self) -> Result<(String, usize, usize), LexError> {
        let start_line = self.line;
        let start_col = self.col;
        let slice = &self.src[self.pos..];

        let close_idx = slice.windows(2).position(|w| w == b"!}");
        let Some(idx) = close_idx else {
            return Err(LexError::new(
                start_line,
                start_col,
                "unterminated raw scope, expected closing !}",
            ));
        };

        let raw = String::from_utf8_lossy(&self.src[self.pos..self.pos + idx]).into_owned();
        for _ in 0..(idx + 2) {
            self.advance();
        }

        let trimmed = trim_raw_block(&raw);
        Ok((trimmed, start_line, start_col))
    }

    #[inline]
    pub fn peek_char(&self) -> u8 {
        self.peek()
    }

    pub fn at_inline_start(&self) -> Option<String> {
        if !is_ident_start(self.peek()) {
            return None;
        }
        let ident = self.peek_ident();
        if is_brace_inline_keyword(&ident) && self.char_after_ident(&ident) == b'{' {
            return Some(ident);
        }
        if (ident == "link" || ident == "fn") && self.char_after_ident(&ident) == b'(' {
            return Some(ident);
        }
        None
    }

    pub fn consume_ident_only(&mut self, ident: &str) {
        for _ in 0..ident.len() {
            self.advance();
        }
    }

    pub fn consume_lbrace(&mut self) -> Result<(), LexError> {
        if self.pos >= self.src.len() || self.peek() != b'{' {
            return Err(self.errorf("expected '{'"));
        }
        self.advance();
        Ok(())
    }

    pub fn consume_rbrace(&mut self) -> Result<(), LexError> {
        if self.pos >= self.src.len() || self.peek() != b'}' {
            return Err(self.errorf("expected '}'"));
        }
        self.advance();
        Ok(())
    }

    pub fn at_raw_ident(&self, want: &str) -> bool {
        if !is_ident_start(self.peek()) {
            return false;
        }
        self.peek_ident() == want
    }

    pub fn at_eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    pub fn read_raw_string(&mut self) -> Result<String, LexError> {
        if self.pos >= self.src.len() || self.peek() != b'"' {
            return Err(self.errorf("expected string literal"));
        }
        let tok = self.read_string()?;
        Ok(tok.value)
    }

    pub fn skip_raw_spaces(&mut self) {
        while self.pos < self.src.len() && (self.peek() == b' ' || self.peek() == b'\t') {
            self.advance();
        }
    }

    /// Reads raw identifier chars, including hyphens (-) for footnotes and attributes.
    pub fn read_raw_ident(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.src.len() && (is_ident_cont(self.peek()) || self.peek() == b'-') {
            self.advance();
        }
        String::from_utf8_lossy(&self.src[start..self.pos]).into_owned()
    }

    pub fn consume_raw_byte(&mut self, expected: u8, what: &str) -> Result<(), LexError> {
        if self.pos >= self.src.len() || self.peek() != expected {
            return Err(self.errorf(format!("expected {what}")));
        }
        self.advance();
        Ok(())
    }

    pub fn read_balanced_braces(&mut self) -> String {
        let mut buf = Vec::new();
        let mut depth = 1;
        while self.pos < self.src.len() {
            let c = self.peek();
            if c == b'{' {
                depth += 1;
            } else if c == b'}' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            buf.push(self.advance());
        }
        String::from_utf8_lossy(&buf).into_owned()
    }

    pub fn read_text_run(&mut self) -> String {
        self.read_text_run_stopping_at(&[])
    }

    pub fn read_text_run_until_block(&mut self, stop_words: &[&str]) -> String {
        self.read_text_run_stopping_at(stop_words)
    }

    fn read_text_run_stopping_at(&mut self, stop_words: &[&str]) -> String {
        let mut buf = Vec::new();
        while self.pos < self.src.len() {
            let c = self.peek();
            if c == b'}' {
                break;
            }
            if is_ident_start(c) {
                let ident = self.peek_ident();
                if is_brace_inline_keyword(&ident) && self.char_after_ident(&ident) == b'{' {
                    break;
                }
                if (ident == "link" || ident == "fn") && self.char_after_ident(&ident) == b'(' {
                    break;
                }
                if stop_words.contains(&ident.as_str()) && self.starts_block_after_ident(&ident) {
                    break;
                }
            }
            buf.push(self.advance());
        }
        String::from_utf8_lossy(&buf).into_owned()
    }

    fn starts_block_after_ident(&self, ident: &str) -> bool {
        let mut i = self.pos + ident.len();
        while i < self.src.len() && (self.src[i] == b' ' || self.src[i] == b'\t') {
            i += 1;
        }
        if i >= self.src.len() {
            return false;
        }
        self.src[i] == b'(' || self.src[i] == b'{'
    }

    fn peek_ident(&self) -> String {
        let mut i = self.pos;
        while i < self.src.len() && is_ident_cont(self.src[i]) {
            i += 1;
        }
        String::from_utf8_lossy(&self.src[self.pos..i]).into_owned()
    }

    fn char_after_ident(&self, ident: &str) -> u8 {
        let i = self.pos + ident.len();
        if i < self.src.len() {
            self.src[i]
        } else {
            0
        }
    }
}

#[inline]
fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

#[inline]
fn is_ident_cont(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

#[inline]
fn is_brace_inline_keyword(s: &str) -> bool {
    matches!(s, "b" | "i" | "code" | "strike" | "m")
}

fn trim_raw_block(raw: &str) -> String {
    let mut s = raw;
    if let Some(idx) = s.find('\n') {
        let prefix = &s[..idx];
        if prefix
            .bytes()
            .all(|b| b == b' ' || b == b'\t' || b == b'\r')
        {
            s = &s[idx + 1..];
        }
    }
    s.trim_end_matches([' ', '\t', '\r', '\n']).to_string()
}
