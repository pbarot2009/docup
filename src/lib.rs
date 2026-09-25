pub mod ast;
pub mod cmd;
pub mod codegen;
pub mod errors;
pub mod fmt;
pub mod highlight;
pub mod init;
pub mod lexer;
pub mod parser;
pub mod sema;
pub mod theme;

// Re-export primary compiler types and entry points for library consumers
pub use ast::{
    BlockNode, DocumentNode, HeadingNode, InlineKind, InlineNode, ItemChild, ItemNode, ListNode,
    MetaNode, ParagraphNode, QuoteChild, QuoteNode, RowNode, TableNode,
};
pub use cmd::{run, VERSION};
pub use codegen::{generate, PAGE_CSS};
pub use errors::{source_snippet, DocupError, LexError, ParseError, PositionedError, SemaError};
pub use fmt::{format_document, format_source};
pub use highlight::{highlight_code, normalize_lang_name};
pub use init::{scaffold_init, scaffold_new, ProjectConfig, SUPPORTED_THEMES};
pub use lexer::{Lexer, Token, TokenType};
pub use parser::Parser;
pub use sema::analyze;
pub use theme::ThemeKind;
