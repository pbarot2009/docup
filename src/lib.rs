pub mod ast;
pub mod cmd;
pub mod codegen;
pub mod errors;
pub mod highlight;
pub mod lexer;
pub mod parser;
pub mod sema;

// Re-export primary compiler types and entry points for library consumers
pub use ast::{
    BlockNode, DocumentNode, HeadingNode, InlineKind, InlineNode, ItemChild, ItemNode, ListNode,
    MetaNode, ParagraphNode, QuoteChild, QuoteNode, RowNode, TableNode,
};
pub use cmd::{VERSION, run};
pub use codegen::{PAGE_CSS, generate};
pub use errors::{DocupError, LexError, ParseError, PositionedError, SemaError, source_snippet};
pub use highlight::{highlight_code, normalize_lang_name};
pub use lexer::{Lexer, Token, TokenType};
pub use parser::Parser;
pub use sema::analyze;
