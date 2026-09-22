/// LanguageProfile describes how comments and strings are delimited and which
/// identifiers count as keywords or types for a given language[span_1](start_span)[span_1](end_span).
#[derive(Debug, Clone, Default)]
pub struct LanguageProfile {
    pub line_comment: Option<&'static str>,
    pub block_comment: Option<(&'static str, &'static str)>,
    pub keywords: &'static [&'static str],
    pub types: &'static [&'static str],
    pub hash_comment: bool,
    pub single_quote: bool,
    pub backtick: bool,
}

fn get_profile(lang: &str) -> (LanguageProfile, bool) {
    match normalize_lang_name(lang).as_str() {
        "go" => (
            LanguageProfile {
                line_comment: Some("//"),
                block_comment: Some(("/*", "*/")),
                backtick: true,
                keywords: &[
                    "break",
                    "case",
                    "chan",
                    "const",
                    "continue",
                    "default",
                    "defer",
                    "else",
                    "fallthrough",
                    "for",
                    "func",
                    "go",
                    "goto",
                    "if",
                    "import",
                    "interface",
                    "map",
                    "package",
                    "range",
                    "return",
                    "select",
                    "struct",
                    "switch",
                    "type",
                    "var",
                ],
                types: &[
                    "bool",
                    "byte",
                    "complex64",
                    "complex128",
                    "error",
                    "float32",
                    "float64",
                    "int",
                    "int8",
                    "int16",
                    "int32",
                    "int64",
                    "rune",
                    "string",
                    "uint",
                    "uint8",
                    "uint16",
                    "uint32",
                    "uint64",
                    "uintptr",
                    "nil",
                    "true",
                    "false",
                    "any",
                ],
                ..Default::default()
            },
            true,
        ),
        "python" => (
            LanguageProfile {
                hash_comment: true,
                single_quote: true,
                keywords: &[
                    "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                    "del", "elif", "else", "except", "finally", "for", "from", "global", "if",
                    "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
                    "return", "try", "while", "with", "yield",
                ],
                types: &[
                    "None", "True", "False", "self", "int", "str", "float", "bool", "list", "dict",
                    "set", "tuple",
                ],
                ..Default::default()
            },
            true,
        ),
        "javascript" => (
            LanguageProfile {
                line_comment: Some("//"),
                block_comment: Some(("/*", "*/")),
                single_quote: true,
                backtick: true,
                keywords: &[
                    "async",
                    "await",
                    "break",
                    "case",
                    "catch",
                    "class",
                    "const",
                    "continue",
                    "debugger",
                    "default",
                    "delete",
                    "do",
                    "else",
                    "export",
                    "extends",
                    "finally",
                    "for",
                    "function",
                    "if",
                    "import",
                    "in",
                    "instanceof",
                    "let",
                    "new",
                    "of",
                    "return",
                    "static",
                    "super",
                    "switch",
                    "this",
                    "throw",
                    "try",
                    "typeof",
                    "var",
                    "void",
                    "while",
                    "with",
                    "yield",
                ],
                types: &["true", "false", "null", "undefined"],
                ..Default::default()
            },
            true,
        ),
        "typescript" => (
            LanguageProfile {
                line_comment: Some("//"),
                block_comment: Some(("/*", "*/")),
                single_quote: true,
                backtick: true,
                keywords: &[
                    "async",
                    "await",
                    "break",
                    "case",
                    "catch",
                    "class",
                    "const",
                    "continue",
                    "debugger",
                    "default",
                    "delete",
                    "do",
                    "else",
                    "enum",
                    "export",
                    "extends",
                    "finally",
                    "for",
                    "function",
                    "if",
                    "implements",
                    "import",
                    "in",
                    "instanceof",
                    "interface",
                    "let",
                    "new",
                    "of",
                    "private",
                    "protected",
                    "public",
                    "readonly",
                    "return",
                    "static",
                    "super",
                    "switch",
                    "this",
                    "throw",
                    "try",
                    "type",
                    "typeof",
                    "var",
                    "void",
                    "while",
                    "with",
                    "yield",
                ],
                types: &[
                    "true",
                    "false",
                    "null",
                    "undefined",
                    "string",
                    "number",
                    "boolean",
                    "any",
                    "unknown",
                    "never",
                ],
                ..Default::default()
            },
            true,
        ),
        "json" => (
            LanguageProfile {
                types: &["true", "false", "null"],
                ..Default::default()
            },
            true,
        ),
        "bash" => (
            LanguageProfile {
                hash_comment: true,
                single_quote: true,
                keywords: &[
                    "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case",
                    "esac", "function", "in", "return", "exit", "export", "local", "readonly",
                    "shift", "break", "continue",
                ],
                ..Default::default()
            },
            true,
        ),
        "rust" => (
            LanguageProfile {
                line_comment: Some("//"),
                block_comment: Some(("/*", "*/")),
                keywords: &[
                    "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
                    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
                    "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
                    "type", "unsafe", "use", "where", "while", "async", "await",
                ],
                types: &[
                    "true", "false", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32",
                    "f64", "bool", "str", "String", "Vec", "Option", "Result",
                ],
                ..Default::default()
            },
            true,
        ),
        "c" => (
            LanguageProfile {
                line_comment: Some("//"),
                block_comment: Some(("/*", "*/")),
                keywords: &[
                    "break", "case", "const", "continue", "default", "do", "else", "enum",
                    "extern", "for", "goto", "if", "return", "sizeof", "static", "struct",
                    "switch", "typedef", "union", "while",
                ],
                types: &[
                    "int", "char", "float", "double", "void", "long", "short", "unsigned",
                    "signed", "NULL",
                ],
                ..Default::default()
            },
            true,
        ),
        "cpp" => (
            LanguageProfile {
                line_comment: Some("//"),
                block_comment: Some(("/*", "*/")),
                keywords: &[
                    "break",
                    "case",
                    "catch",
                    "class",
                    "const",
                    "continue",
                    "default",
                    "delete",
                    "do",
                    "else",
                    "enum",
                    "explicit",
                    "export",
                    "extern",
                    "for",
                    "friend",
                    "goto",
                    "if",
                    "namespace",
                    "new",
                    "operator",
                    "private",
                    "protected",
                    "public",
                    "return",
                    "sizeof",
                    "static",
                    "struct",
                    "switch",
                    "template",
                    "this",
                    "throw",
                    "try",
                    "typedef",
                    "typename",
                    "union",
                    "using",
                    "virtual",
                    "while",
                ],
                types: &[
                    "int", "char", "float", "double", "void", "long", "short", "unsigned",
                    "signed", "bool", "true", "false", "nullptr", "auto", "std",
                ],
                ..Default::default()
            },
            true,
        ),
        "html" => (
            LanguageProfile {
                block_comment: Some(("<!--", "-->")),
                single_quote: true,
                ..Default::default()
            },
            true,
        ),
        "css" => (
            LanguageProfile {
                block_comment: Some(("/*", "*/")),
                single_quote: true,
                ..Default::default()
            },
            true,
        ),
        _ => (
            // Generic fallback profile: covers common C-like & scripting languages[span_2](start_span)[span_2](end_span)
            LanguageProfile {
                line_comment: Some("//"),
                block_comment: Some(("/*", "*/")),
                hash_comment: true,
                single_quote: true,
                ..Default::default()
            },
            false,
        ),
    }
}

/// normalize_lang_name maps common aliases to canonical language keys[span_3](start_span)[span_3](end_span).
pub fn normalize_lang_name(lang: &str) -> String {
    let trimmed = lang.trim().to_ascii_lowercase();
    match trimmed.as_str() {
        "js" | "jsx" => "javascript".to_string(),
        "ts" | "tsx" => "typescript".to_string(),
        "py" => "python".to_string(),
        "sh" | "shell" | "zsh" => "bash".to_string(),
        "c++" => "cpp".to_string(),
        "golang" => "go".to_string(),
        _ => trimmed,
    }
}

/// HighlightCode renders source code as HTML with <span> tags marking
/// syntax categories (keyword, type, string, comment, number)[span_4](start_span)[span_4](end_span).
pub fn highlight_code(source: &str, lang: &str) -> String {
    let (profile, _) = get_profile(lang);
    highlight_with_profile(source, &profile)
}

fn highlight_with_profile(src: &str, prof: &LanguageProfile) -> String {
    let mut out = String::with_capacity(src.len() * 2);
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut i = 0;

    let flush_span = |out: &mut String, class: &str, text: &str| {
        if text.is_empty() {
            return;
        }
        if class.is_empty() {
            escape_html_into(text, out);
        } else {
            out.push_str("<span class=\"tok-");
            out.push_str(class);
            out.push_str("\">");
            escape_html_into(text, out);
            out.push_str("</span>");
        }
    };

    while i < n {
        let c = chars[i];

        // Block comments
        if let Some((start_tok, end_tok)) = prof.block_comment {
            if has_prefix_at(&chars, i, start_tok) {
                let start = i;
                let start_chars: Vec<char> = start_tok.chars().collect();
                let end_chars: Vec<char> = end_tok.chars().collect();
                i += start_chars.len();
                while i < n && !has_prefix_at(&chars, i, end_tok) {
                    i += 1;
                }
                if i < n {
                    i += end_chars.len();
                }
                let comment_text: String = chars[start..i].iter().collect();
                flush_span(&mut out, "comment", &comment_text);
                continue;
            }
        }

        // Line comments
        if let Some(line_tok) = prof.line_comment {
            if has_prefix_at(&chars, i, line_tok) {
                let start = i;
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
                let comment_text: String = chars[start..i].iter().collect();
                flush_span(&mut out, "comment", &comment_text);
                continue;
            }
        }

        // Shell/Python style hash comments
        if prof.hash_comment && c == '#' {
            let start = i;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            let comment_text: String = chars[start..i].iter().collect();
            flush_span(&mut out, "comment", &comment_text);
            continue;
        }

        // String literals
        if c == '"' || (prof.single_quote && c == '\'') || (prof.backtick && c == '`') {
            let quote = c;
            let start = i;
            i += 1;
            while i < n && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < n {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            if i < n {
                i += 1; // closing quote
            }
            let string_text: String = chars[start..i].iter().collect();
            flush_span(&mut out, "string", &string_text);
            continue;
        }

        // Numbers
        if is_digit_rune(c) {
            let start = i;
            while i < n
                && (is_digit_rune(chars[i])
                    || chars[i] == '.'
                    || chars[i] == '_'
                    || chars[i] == 'x'
                    || chars[i] == 'X'
                    || is_hex_digit_rune(chars[i]))
            {
                i += 1;
            }
            let num_text: String = chars[start..i].iter().collect();
            flush_span(&mut out, "number", &num_text);
            continue;
        }

        // Identifiers / keywords / types
        if is_ident_start_rune(c) {
            let start = i;
            while i < n && is_ident_cont_rune(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if prof.keywords.contains(&word.as_str()) {
                flush_span(&mut out, "keyword", &word);
            } else if prof.types.contains(&word.as_str()) {
                flush_span(&mut out, "type", &word);
            } else {
                flush_span(&mut out, "", &word);
            }
            continue;
        }

        // Pass-through punctuation / whitespace
        let s = c.to_string();
        flush_span(&mut out, "", &s);
        i += 1;
    }

    out
}

fn has_prefix_at(chars: &[char], pos: usize, prefix: &str) -> bool {
    let prefix_chars: Vec<char> = prefix.chars().collect();
    if pos + prefix_chars.len() > chars.len() {
        return false;
    }
    chars[pos..pos + prefix_chars.len()] == prefix_chars[..]
}

#[inline]
fn is_digit_rune(c: char) -> bool {
    c.is_ascii_digit()
}

#[inline]
fn is_hex_digit_rune(c: char) -> bool {
    c.is_ascii_hexdigit()
}

#[inline]
fn is_ident_start_rune(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}

#[inline]
fn is_ident_cont_rune(c: char) -> bool {
    is_ident_start_rune(c) || is_digit_rune(c)
}

/// Escapes HTML characters `<, >, &, ", '` directly into the destination buffer.
pub fn escape_html_into(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
}

pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    escape_html_into(s, &mut out);
    out
}
