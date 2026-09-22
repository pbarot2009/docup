use std::fmt;

/// PositionedError is implemented by any compiler-stage error that can
/// point at a specific line and column in the source code.
pub trait PositionedError: std::error::Error {
    fn position(&self) -> (usize, usize);
}

/// LexError represents a failure encountered during lexical scanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl LexError {
    pub fn new(line: usize, col: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            col,
            message: message.into(),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "lex error at {}:{}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for LexError {}

impl PositionedError for LexError {
    fn position(&self) -> (usize, usize) {
        (self.line, self.col)
    }
}

/// ParseError represents a failure encountered during syntactic parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl ParseError {
    pub fn new(line: usize, col: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            col,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse error at {}:{}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for ParseError {}

impl PositionedError for ParseError {
    fn position(&self) -> (usize, usize) {
        (self.line, self.col)
    }
}

/// SemaError represents a semantic analysis violation (e.g. invalid heading level,
/// duplicate metadata, malformed tables).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemaError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl SemaError {
    pub fn new(line: usize, col: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            col,
            message: message.into(),
        }
    }
}

impl fmt::Display for SemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "semantic error at {}:{}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for SemaError {}

impl PositionedError for SemaError {
    fn position(&self) -> (usize, usize) {
        (self.line, self.col)
    }
}

/// DocupError is a unified diagnostic error type encompassing all stages
/// of the DocUP pipeline.
#[derive(Debug)]
pub enum DocupError {
    Lex(LexError),
    Parse(ParseError),
    Sema(SemaError),
    General(String),
}

impl DocupError {
    /// Returns the (line, col) position of the error if available.
    pub fn position(&self) -> Option<(usize, usize)> {
        match self {
            DocupError::Lex(e) => Some(e.position()),
            DocupError::Parse(e) => Some(e.position()),
            DocupError::Sema(e) => Some(e.position()),
            DocupError::General(_) => None,
        }
    }
}

impl fmt::Display for DocupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocupError::Lex(e) => write!(f, "{e}"),
            DocupError::Parse(e) => write!(f, "{e}"),
            DocupError::Sema(e) => write!(f, "{e}"),
            DocupError::General(msg) => write!(f, "error: {msg}"),
        }
    }
}

impl std::error::Error for DocupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DocupError::Lex(e) => Some(e),
            DocupError::Parse(e) => Some(e),
            DocupError::Sema(e) => Some(e),
            DocupError::General(_) => None,
        }
    }
}

impl From<LexError> for DocupError {
    fn from(err: LexError) -> Self {
        DocupError::Lex(err)
    }
}

impl From<ParseError> for DocupError {
    fn from(err: ParseError) -> Self {
        DocupError::Parse(err)
    }
}

impl From<SemaError> for DocupError {
    fn from(err: SemaError) -> Self {
        DocupError::Sema(err)
    }
}

/// source_snippet extracts the offending line of source with a caret (`^`)
/// pointing at the exact column for terminal diagnostic reporting.
pub fn source_snippet(src: &[u8], line: usize, mut col: usize) -> String {
    let text_content = String::from_utf8_lossy(src);
    let lines: Vec<&str> = text_content.split('\n').collect();

    if line < 1 || line > lines.len() {
        return String::new();
    }

    let mut line_text = lines[line - 1];
    if line_text.ends_with('\r') {
        line_text = &line_text[..line_text.len() - 1];
    }

    if col < 1 {
        col = 1;
    }

    let char_count = line_text.chars().count();
    let caret_pos = if (col - 1) > char_count {
        char_count
    } else {
        col - 1
    };

    let gutter = format!("{line} | ");
    let pad = " ".repeat(gutter.len() + caret_pos);

    format!("{gutter}{line_text}\n{pad}^")
}
