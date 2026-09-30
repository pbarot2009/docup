use std::fmt;

/// Inclusive source span. Lines and columns are 1-based Unicode scalars.
///
/// `end_*` is the last character in the span, not one-past-the-end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn point(line: usize, col: usize) -> Self {
        let line = line.max(1);
        let col = col.max(1);
        Self {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
        }
    }

    /// Span covering `len` Unicode scalar values starting at `(line, col)`.
    pub fn from_len(line: usize, col: usize, len: usize) -> Self {
        let line = line.max(1);
        let col = col.max(1);
        if len <= 1 {
            return Self::point(line, col);
        }
        Self {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col + len - 1,
        }
    }

    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line: start_line.max(1),
            start_col: start_col.max(1),
            end_line: end_line.max(1),
            end_col: end_col.max(1),
        }
    }

    pub fn start(self) -> (usize, usize) {
        (self.start_line, self.start_col)
    }

    pub fn is_point(self) -> bool {
        self.start_line == self.end_line && self.start_col == self.end_col
    }
}

/// A labeled span, rustc-style. The primary span is the error location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub span: Span,
    pub message: String,
    pub primary: bool,
}

impl Label {
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            primary: true,
        }
    }

    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            primary: false,
        }
    }
}

/// PositionedError is implemented by any compiler-stage error that can
/// point at a specific span in the source code.
pub trait PositionedError: std::error::Error {
    fn position(&self) -> (usize, usize);

    fn span(&self) -> Span {
        let (line, col) = self.position();
        Span::point(line, col)
    }

    fn labels(&self) -> Vec<Label> {
        vec![Label::primary(self.span(), String::new())]
    }

    fn notes(&self) -> Vec<String> {
        Vec::new()
    }

    fn helps(&self) -> Vec<String> {
        Vec::new()
    }

    fn stage(&self) -> &'static str;

    fn raw_message(&self) -> String;
}

macro_rules! impl_compiler_error {
    ($name:ident, $stage:expr, $prefix:expr) => {
        impl $name {
            pub fn new(line: usize, col: usize, message: impl Into<String>) -> Self {
                Self::at_span(Span::point(line, col), message)
            }

            pub fn at_span(span: Span, message: impl Into<String>) -> Self {
                Self {
                    line: span.start_line,
                    col: span.start_col,
                    end_line: span.end_line,
                    end_col: span.end_col,
                    message: message.into(),
                    notes: Vec::new(),
                    helps: Vec::new(),
                    labels: vec![Label::primary(span, String::new())],
                }
            }

            pub fn with_note(mut self, note: impl Into<String>) -> Self {
                self.notes.push(note.into());
                self
            }

            pub fn with_help(mut self, help: impl Into<String>) -> Self {
                self.helps.push(help.into());
                self
            }

            pub fn primary_label(mut self, message: impl Into<String>) -> Self {
                if let Some(first) = self.labels.iter_mut().find(|l| l.primary) {
                    first.message = message.into();
                } else {
                    self.labels.insert(
                        0,
                        Label::primary(
                            Span::new(self.line, self.col, self.end_line, self.end_col),
                            message,
                        ),
                    );
                }
                self
            }

            pub fn label(mut self, span: Span, message: impl Into<String>) -> Self {
                self.labels.push(Label::secondary(span, message));
                self
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{} at {}:{}: {}",
                    $prefix, self.line, self.col, self.message
                )
            }
        }

        impl std::error::Error for $name {}

        impl PositionedError for $name {
            fn position(&self) -> (usize, usize) {
                (self.line, self.col)
            }

            fn span(&self) -> Span {
                Span::new(self.line, self.col, self.end_line, self.end_col)
            }

            fn labels(&self) -> Vec<Label> {
                if self.labels.is_empty() {
                    vec![Label::primary(self.span(), String::new())]
                } else {
                    self.labels.clone()
                }
            }

            fn notes(&self) -> Vec<String> {
                self.notes.clone()
            }

            fn helps(&self) -> Vec<String> {
                self.helps.clone()
            }

            fn stage(&self) -> &'static str {
                $stage
            }

            fn raw_message(&self) -> String {
                self.message.clone()
            }
        }
    };
}

/// LexError represents a failure encountered during lexical scanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub message: String,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
    pub labels: Vec<Label>,
}

impl_compiler_error!(LexError, "lex", "lex error");

/// ParseError represents a failure encountered during syntactic parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub message: String,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
    pub labels: Vec<Label>,
}

impl_compiler_error!(ParseError, "parse", "parse error");

/// SemaError represents a semantic analysis violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemaError {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub message: String,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
    pub labels: Vec<Label>,
}

impl_compiler_error!(SemaError, "semantic", "semantic error");

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

impl PositionedError for DocupError {
    fn position(&self) -> (usize, usize) {
        DocupError::position(self).unwrap_or((1, 1))
    }

    fn span(&self) -> Span {
        match self {
            DocupError::Lex(e) => e.span(),
            DocupError::Parse(e) => e.span(),
            DocupError::Sema(e) => e.span(),
            DocupError::General(_) => Span::point(1, 1),
        }
    }

    fn labels(&self) -> Vec<Label> {
        match self {
            DocupError::Lex(e) => e.labels(),
            DocupError::Parse(e) => e.labels(),
            DocupError::Sema(e) => e.labels(),
            DocupError::General(_) => Vec::new(),
        }
    }

    fn notes(&self) -> Vec<String> {
        match self {
            DocupError::Lex(e) => e.notes(),
            DocupError::Parse(e) => e.notes(),
            DocupError::Sema(e) => e.notes(),
            DocupError::General(_) => Vec::new(),
        }
    }

    fn helps(&self) -> Vec<String> {
        match self {
            DocupError::Lex(e) => e.helps(),
            DocupError::Parse(e) => e.helps(),
            DocupError::Sema(e) => e.helps(),
            DocupError::General(_) => Vec::new(),
        }
    }

    fn stage(&self) -> &'static str {
        match self {
            DocupError::Lex(_) => "lex",
            DocupError::Parse(_) => "parse",
            DocupError::Sema(_) => "semantic",
            DocupError::General(_) => "error",
        }
    }

    fn raw_message(&self) -> String {
        match self {
            DocupError::Lex(e) => e.raw_message(),
            DocupError::Parse(e) => e.raw_message(),
            DocupError::Sema(e) => e.raw_message(),
            DocupError::General(msg) => msg.clone(),
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

const TAB_WIDTH: usize = 4;

fn strip_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

fn display_col(line: &str, char_col: usize) -> usize {
    let target = char_col.max(1) - 1;
    let mut display = 0usize;
    for (i, ch) in line.chars().enumerate() {
        if i >= target {
            break;
        }
        if ch == '\t' {
            display += TAB_WIDTH - (display % TAB_WIDTH);
        } else {
            display += 1;
        }
    }
    display
}

fn expand_tabs(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut display = 0usize;
    for ch in line.chars() {
        if ch == '\t' {
            let n = TAB_WIDTH - (display % TAB_WIDTH);
            for _ in 0..n {
                out.push(' ');
            }
            display += n;
        } else {
            out.push(ch);
            display += 1;
        }
    }
    out
}

fn line_at<'a>(lines: &'a [&str], line: usize) -> &'a str {
    if line < 1 || line > lines.len() {
        ""
    } else {
        strip_cr(lines[line - 1])
    }
}

fn underline_for(line: &str, span: Span, line_no: usize, primary: bool) -> (usize, usize, String) {
    let start_col = if line_no == span.start_line {
        span.start_col
    } else {
        1
    };
    let line_chars = line.chars().count().max(1);
    let end_col = if line_no == span.end_line {
        span.end_col
    } else {
        line_chars
    };
    let start_d = display_col(line, start_col);
    let end_d = display_col(line, end_col.saturating_add(1)).max(start_d + 1);
    let mark_len = (end_d - start_d).max(1);
    let ch = if primary { '^' } else { '-' };
    (start_d, mark_len, ch.to_string().repeat(mark_len))
}

/// rustc-style snippet for a single point. Kept for library compatibility.
pub fn source_snippet(src: &[u8], line: usize, col: usize) -> String {
    render_span_snippet(src, Span::point(line, col), None)
}

pub fn render_span_snippet(src: &[u8], span: Span, underline_label: Option<&str>) -> String {
    let label = Label {
        span,
        message: underline_label.unwrap_or("").to_string(),
        primary: true,
    };
    render_labeled_snippet(src, &[label])
}

fn render_labeled_snippet(src: &[u8], labels: &[Label]) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let text = String::from_utf8_lossy(src);
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.is_empty() {
        return String::new();
    }

    let mut annotated: Vec<(usize, &Label)> = labels
        .iter()
        .flat_map(|l| {
            let start = l.span.start_line.clamp(1, lines.len().max(1));
            let end = l.span.end_line.clamp(start, lines.len().max(1));
            (start..=end).map(move |n| (n, l))
        })
        .collect();
    annotated.sort_by_key(|(n, l)| (*n, !l.primary, l.span.start_col));

    let mut wanted: Vec<usize> = annotated.iter().map(|(n, _)| *n).collect();
    wanted.sort_unstable();
    wanted.dedup();

    // Include up to one blank context line between close spans (rustc does this).
    let mut show: Vec<usize> = Vec::new();
    for (i, &n) in wanted.iter().enumerate() {
        if i > 0 {
            let prev = wanted[i - 1];
            if n > prev + 1 && n <= prev + 3 {
                for fill in (prev + 1)..n {
                    show.push(fill);
                }
            }
        }
        show.push(n);
    }

    let max_line = *show.last().unwrap_or(&1);
    let width = max_line.to_string().len().max(1);

    let mut out = String::new();
    out.push_str(&format!("{:>width$} |\n", ""));

    let mut last_printed = 0usize;
    for line_no in show {
        if last_printed > 0 && line_no > last_printed + 1 {
            out.push_str(&format!("{:>width$} |\n", ""));
        }
        last_printed = line_no;

        let raw = line_at(&lines, line_no);
        let shown = expand_tabs(raw);
        out.push_str(&format!("{line_no:>width$} | {shown}\n"));

        let on_line: Vec<&Label> = annotated
            .iter()
            .filter(|(n, _)| *n == line_no)
            .map(|(_, l)| *l)
            .collect();

        if on_line.is_empty() {
            continue;
        }

        // One underline row: place every mark on this line.
        let mut row = vec![' '; expand_tabs(raw).chars().count().max(1) + 8];
        let mut rightmost_primary: Option<(&Label, usize)> = None;
        let mut rightmost_any: Option<(&Label, usize)> = None;

        for &label in &on_line {
            let (start_d, mark_len, mark) = underline_for(raw, label.span, line_no, label.primary);

            while row.len() < start_d + mark_len {
                row.push(' ');
            }
            for (i, ch) in mark.chars().enumerate() {
                let idx = start_d + i;
                if idx >= row.len() {
                    row.resize(idx + 1, ' ');
                }
                if row[idx] != '^' {
                    row[idx] = ch;
                }
            }
            let end = start_d + mark_len;
            if label.primary {
                rightmost_primary = Some((label, end));
            }
            rightmost_any = Some((label, end));
        }

        let mut underline: String = row.into_iter().collect();
        underline = underline.trim_end().to_string();
        let caption = rightmost_primary.or(rightmost_any).and_then(|(l, end)| {
            if l.message.is_empty() {
                None
            } else {
                Some((end, l.message.as_str()))
            }
        });

        out.push_str(&format!("{:>width$} | {underline}", ""));
        if let Some((end, msg)) = caption {
            let pad = end.saturating_sub(underline.chars().count());
            out.push_str(&" ".repeat(pad.max(1)));
            out.push_str(msg);
        }
        out.push('\n');

        // Stack leftover secondary messages that were not used as the caption.
        if let Some((cap_label, _)) = rightmost_primary {
            for label in on_line {
                if std::ptr::eq(label, cap_label) || label.message.is_empty() {
                    continue;
                }

                if !label.primary {
                    let (start_d, _, _) = underline_for(raw, label.span, line_no, false);
                    out.push_str(&format!("{:>width$} | ", ""));
                    out.push_str(&" ".repeat(start_d));
                    out.push_str("|\n");
                    out.push_str(&format!("{:>width$} | ", ""));
                    out.push_str(&" ".repeat(start_d));
                    out.push_str(&label.message);
                    out.push('\n');
                }
            }
        }
    }

    out
}

/// Full rustc-style diagnostic: `error:`, `-->`, snippet, `= note:`, `= help:`.
pub fn render_diagnostic(path: &str, src: &[u8], err: &(dyn PositionedError + 'static)) -> String {
    let labels = err.labels();
    let primary = labels
        .iter()
        .find(|l| l.primary)
        .cloned()
        .unwrap_or_else(|| Label::primary(err.span(), String::new()));
    let (line, col) = primary.span.start();
    let message = err.raw_message();

    let max_line = labels
        .iter()
        .map(|l| l.span.end_line)
        .max()
        .unwrap_or(line)
        .max(line);
    let width = max_line.to_string().len().max(1);

    let mut out = String::new();
    out.push_str(&format!("error: {message}\n"));
    out.push_str(&format!("{:>width$}--> {path}:{line}:{col}\n", ""));
    out.push_str(&render_labeled_snippet(src, &labels));

    let notes = err.notes();
    let helps = err.helps();
    if !notes.is_empty() || !helps.is_empty() {
        out.push_str(&format!("{:>width$} |\n", ""));
        for note in notes {
            out.push_str(&format!("{:>width$} = note: {note}\n", ""));
        }
        for help in helps {
            out.push_str(&format!("{:>width$} = help: {help}\n", ""));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustc_header_and_primary_span() {
        let err = ParseError::at_span(
            Span::from_len(1, 1, 9),
            "expected `{!` to start a raw code body, found `h`",
        )
        .primary_label("missing `{! ... !}` body")
        .label(Span::point(3, 1), "unexpected token")
        .with_help("add `{! ... !}` or `src: \"file.rs\"`");

        let src = b"codeblock(lang: \"rust\")\n\nh(2) { Next }\n";
        let rendered = render_diagnostic("error.du", src, &err);
        assert!(rendered.starts_with("error: expected `{!` to start a raw code body, found `h`"));
        assert!(rendered.contains("--> error.du:1:1"));
        assert!(rendered.contains("codeblock(lang: \"rust\")"));
        assert!(rendered.contains("^^^^^^^^^"));
        assert!(rendered.contains("= help:"));
    }
}
