package src

import (
	"html"
	"strings"
)

// Syntax highlighting is implemented entirely in Go with no external
// dependencies or JS libraries, matching DocUP's standard-library-only
// philosophy. It is intentionally simple: a single generic tokenizer
// (strings, comments, numbers, keywords, punctuation) driven by a small
// per-language keyword table, rather than a full grammar per language.
// This covers the common case well without pulling in a highlighting
// engine.

type keywordSet map[string]bool

func newKeywordSet(words ...string) keywordSet {
	s := make(keywordSet, len(words))
	for _, w := range words {
		s[w] = true
	}
	return s
}

// languageProfile describes how comments/strings are delimited and which
// identifiers count as keywords/types for a given language.
type languageProfile struct {
	lineComment  string
	blockComment [2]string // start, end; empty if unsupported
	keywords     keywordSet
	types        keywordSet
	hashComment  bool // '#' starts a line comment (shell, python, ruby)
	singleQuote  bool // '...' is also a valid string literal
	backtick     bool // `...` is also a valid string literal (Go, JS)
}

var languageProfiles = map[string]languageProfile{
	"go": {
		lineComment:  "//",
		blockComment: [2]string{"/*", "*/"},
		backtick:     true,
		keywords: newKeywordSet(
			"break", "case", "chan", "const", "continue", "default", "defer",
			"else", "fallthrough", "for", "func", "go", "goto", "if", "import",
			"interface", "map", "package", "range", "return", "select",
			"struct", "switch", "type", "var",
		),
		types: newKeywordSet(
			"bool", "byte", "complex64", "complex128", "error", "float32",
			"float64", "int", "int8", "int16", "int32", "int64", "rune",
			"string", "uint", "uint8", "uint16", "uint32", "uint64", "uintptr",
			"nil", "true", "false", "any",
		),
	},
	"python": {
		hashComment: true,
		singleQuote: true,
		keywords: newKeywordSet(
			"and", "as", "assert", "async", "await", "break", "class",
			"continue", "def", "del", "elif", "else", "except", "finally",
			"for", "from", "global", "if", "import", "in", "is", "lambda",
			"nonlocal", "not", "or", "pass", "raise", "return", "try",
			"while", "with", "yield",
		),
		types: newKeywordSet("None", "True", "False", "self", "int", "str", "float", "bool", "list", "dict", "set", "tuple"),
	},
	"javascript": {
		lineComment:  "//",
		blockComment: [2]string{"/*", "*/"},
		singleQuote:  true,
		backtick:     true,
		keywords: newKeywordSet(
			"async", "await", "break", "case", "catch", "class", "const",
			"continue", "debugger", "default", "delete", "do", "else",
			"export", "extends", "finally", "for", "function", "if",
			"import", "in", "instanceof", "let", "new", "of", "return",
			"static", "super", "switch", "this", "throw", "try", "typeof",
			"var", "void", "while", "with", "yield",
		),
		types: newKeywordSet("true", "false", "null", "undefined"),
	},
	"typescript": {
		lineComment:  "//",
		blockComment: [2]string{"/*", "*/"},
		singleQuote:  true,
		backtick:     true,
		keywords: newKeywordSet(
			"async", "await", "break", "case", "catch", "class", "const",
			"continue", "debugger", "default", "delete", "do", "else",
			"enum", "export", "extends", "finally", "for", "function", "if",
			"implements", "import", "in", "instanceof", "interface", "let",
			"new", "of", "private", "protected", "public", "readonly",
			"return", "static", "super", "switch", "this", "throw", "try",
			"type", "typeof", "var", "void", "while", "with", "yield",
		),
		types: newKeywordSet("true", "false", "null", "undefined", "string", "number", "boolean", "any", "unknown", "never"),
	},
	"json": {
		types: newKeywordSet("true", "false", "null"),
	},
	"bash": {
		hashComment: true,
		singleQuote: true,
		keywords: newKeywordSet(
			"if", "then", "else", "elif", "fi", "for", "while", "do", "done",
			"case", "esac", "function", "in", "return", "exit", "export",
			"local", "readonly", "shift", "break", "continue",
		),
	},
	"rust": {
		lineComment:  "//",
		blockComment: [2]string{"/*", "*/"},
		keywords: newKeywordSet(
			"as", "break", "const", "continue", "crate", "dyn", "else",
			"enum", "extern", "fn", "for", "if", "impl", "in", "let", "loop",
			"match", "mod", "move", "mut", "pub", "ref", "return", "self",
			"Self", "static", "struct", "super", "trait", "type", "unsafe",
			"use", "where", "while", "async", "await",
		),
		types: newKeywordSet("true", "false", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "str", "String", "Vec", "Option", "Result"),
	},
	"c": {
		lineComment:  "//",
		blockComment: [2]string{"/*", "*/"},
		keywords: newKeywordSet(
			"break", "case", "const", "continue", "default", "do", "else",
			"enum", "extern", "for", "goto", "if", "return", "sizeof",
			"static", "struct", "switch", "typedef", "union", "while",
		),
		types: newKeywordSet("int", "char", "float", "double", "void", "long", "short", "unsigned", "signed", "NULL"),
	},
	"cpp": {
		lineComment:  "//",
		blockComment: [2]string{"/*", "*/"},
		keywords: newKeywordSet(
			"break", "case", "catch", "class", "const", "continue", "default",
			"delete", "do", "else", "enum", "explicit", "export", "extern",
			"for", "friend", "goto", "if", "namespace", "new", "operator",
			"private", "protected", "public", "return", "sizeof", "static",
			"struct", "switch", "template", "this", "throw", "try", "typedef",
			"typename", "union", "using", "virtual", "while",
		),
		types: newKeywordSet("int", "char", "float", "double", "void", "long", "short", "unsigned", "signed", "bool", "true", "false", "nullptr", "auto", "std"),
	},
	"html": {
		blockComment: [2]string{"<!--", "-->"},
		singleQuote:  true,
	},
	"css": {
		blockComment: [2]string{"/*", "*/"},
		singleQuote:  true,
	},
}

// normalizeLangName maps common aliases (js, ts, py, sh, c++, golang, etc.)
// onto the canonical keys in languageProfiles.
func normalizeLangName(lang string) string {
	switch strings.ToLower(strings.TrimSpace(lang)) {
	case "js", "jsx":
		return "javascript"
	case "ts", "tsx":
		return "typescript"
	case "py":
		return "python"
	case "sh", "shell", "zsh":
		return "bash"
	case "c++":
		return "cpp"
	case "golang":
		return "go"
	default:
		return strings.ToLower(strings.TrimSpace(lang))
	}
}

// HighlightCode renders source code as HTML with <span> tags marking
// syntax categories (keyword, type, string, comment, number). It always
// produces valid, fully-escaped HTML — for languages with no profile, it
// falls back to a generic tokenizer that still highlights strings,
// comments (// and #), and numbers, so unknown languages still get some
// benefit rather than none.
func HighlightCode(source, lang string) string {
	profile, known := languageProfiles[normalizeLangName(lang)]
	if !known {
		// Generic fallback profile: covers the syntax shared by most
		// C-like and scripting languages reasonably well.
		profile = languageProfile{
			lineComment:  "//",
			blockComment: [2]string{"/*", "*/"},
			hashComment:  true,
			singleQuote:  true,
		}
	}
	return highlightWithProfile(source, profile)
}

func highlightWithProfile(src string, prof languageProfile) string {
	var out strings.Builder
	runes := []rune(src)
	n := len(runes)
	i := 0

	flushSpan := func(class string, text string) {
		if text == "" {
			return
		}
		escaped := html.EscapeString(text)
		if class == "" {
			out.WriteString(escaped)
			return
		}
		out.WriteString(`<span class="tok-` + class + `">` + escaped + `</span>`)
	}

	for i < n {
		c := runes[i]

		// Block comment.
		if prof.blockComment[0] != "" && hasPrefixAt(runes, i, prof.blockComment[0]) {
			start := i
			i += len([]rune(prof.blockComment[0]))
			end := prof.blockComment[1]
			for i < n && !hasPrefixAt(runes, i, end) {
				i++
			}
			if i < n {
				i += len([]rune(end))
			}
			flushSpan("comment", string(runes[start:i]))
			continue
		}

		// Line comment.
		if prof.lineComment != "" && hasPrefixAt(runes, i, prof.lineComment) {
			start := i
			for i < n && runes[i] != '\n' {
				i++
			}
			flushSpan("comment", string(runes[start:i]))
			continue
		}
		if prof.hashComment && c == '#' {
			start := i
			for i < n && runes[i] != '\n' {
				i++
			}
			flushSpan("comment", string(runes[start:i]))
			continue
		}

		// String literals.
		if c == '"' || (prof.singleQuote && c == '\'') || (prof.backtick && c == '`') {
			quote := c
			start := i
			i++
			for i < n && runes[i] != quote {
				if runes[i] == '\\' && i+1 < n {
					i += 2
					continue
				}
				i++
			}
			if i < n {
				i++ // closing quote
			}
			flushSpan("string", string(runes[start:i]))
			continue
		}

		// Numbers.
		if isDigitRune(c) {
			start := i
			for i < n && (isDigitRune(runes[i]) || runes[i] == '.' || runes[i] == '_' ||
				runes[i] == 'x' || runes[i] == 'X' || isHexDigitRune(runes[i])) {
				i++
			}
			flushSpan("number", string(runes[start:i]))
			continue
		}

		// Identifiers / keywords / types.
		if isIdentStartRune(c) {
			start := i
			for i < n && isIdentContRune(runes[i]) {
				i++
			}
			word := string(runes[start:i])
			switch {
			case prof.keywords != nil && prof.keywords[word]:
				flushSpan("keyword", word)
			case prof.types != nil && prof.types[word]:
				flushSpan("type", word)
			default:
				flushSpan("", word)
			}
			continue
		}

		// Everything else (punctuation, whitespace) passes through
		// unhighlighted but still escaped.
		flushSpan("", string(c))
		i++
	}

	return out.String()
}

func hasPrefixAt(runes []rune, pos int, prefix string) bool {
	pr := []rune(prefix)
	if pos+len(pr) > len(runes) {
		return false
	}
	for j, r := range pr {
		if runes[pos+j] != r {
			return false
		}
	}
	return true
}

func isDigitRune(r rune) bool      { return r >= '0' && r <= '9' }
func isHexDigitRune(r rune) bool   { return (r >= 'a' && r <= 'f') || (r >= 'A' && r <= 'F') }
func isIdentStartRune(r rune) bool { return r == '_' || (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') }
func isIdentContRune(r rune) bool  { return isIdentStartRune(r) || isDigitRune(r) }
