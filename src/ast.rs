use std::collections::HashMap;

/// Root AST node representing a fully parsed .du document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentNode {
    pub meta: Option<MetaNode>,
    pub blocks: Vec<BlockNode>,
}

impl DocumentNode {
    pub fn new() -> Self {
        Self {
            meta: None,
            blocks: Vec::new(),
        }
    }
}

impl Default for DocumentNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata block at the start of a document: `meta { key: "value", ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaNode {
    pub line: usize,
    pub col: usize,
    pub fields: HashMap<String, String>,
}

impl MetaNode {
    pub fn new(line: usize, col: usize, fields: HashMap<String, String>) -> Self {
        Self { line, col, fields }
    }
}

/// Enumeration of all top-level block constructs supported in DocUP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockNode {
    Heading(HeadingNode),
    Paragraph(ParagraphNode),
    CodeBlock(CodeBlockNode),
    HR(HRNode),
    List(ListNode),
    Quote(QuoteNode),
    Image(ImageNode),
    Table(TableNode),
}

impl BlockNode {
    pub fn kind(&self) -> &'static str {
        match self {
            BlockNode::Heading(_) => "Heading",
            BlockNode::Paragraph(_) => "Paragraph",
            BlockNode::CodeBlock(_) => "CodeBlock",
            BlockNode::HR(_) => "HR",
            BlockNode::List(_) => "List",
            BlockNode::Quote(_) => "Quote",
            BlockNode::Image(_) => "Image",
            BlockNode::Table(_) => "Table",
        }
    }
}

/// Heading block: `h(1) { ... }` or `h(2, id: "sub", class: "sec") { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingNode {
    pub line: usize,
    pub col: usize,
    pub level: usize,
    pub attrs: HashMap<String, String>,
    pub children: Vec<InlineNode>,
}

/// Paragraph block: `p { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphNode {
    pub line: usize,
    pub col: usize,
    pub children: Vec<InlineNode>,
}

/// Horizontal rule: `hr {}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HRNode {
    pub line: usize,
    pub col: usize,
}

/// Fenced raw code block: `codeblock(lang: "rust", file: "main.rs") {! ... !}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlockNode {
    pub line: usize,
    pub col: usize,
    pub language: String,
    pub file: String,
    pub raw_code: String,
}

/// Ordered or unordered list: `list { ... }` or `list(ordered: true) { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListNode {
    pub line: usize,
    pub col: usize,
    pub ordered: bool,
    pub items: Vec<ItemNode>,
}

/// List item node: `item { ... }` or `task(done: true/false) { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemNode {
    pub line: usize,
    pub col: usize,
    /// `None` for regular items, `Some(true/false)` for task list checkboxes.
    pub done: Option<bool>,
    pub children: Vec<ItemChild>,
}

/// Elements permitted inside a list item body (prose or a nested list).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemChild {
    Inline(InlineNode),
    List(ListNode),
}

/// Blockquote block: `quote { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteNode {
    pub line: usize,
    pub col: usize,
    pub children: Vec<QuoteChild>,
}

/// Elements permitted inside a blockquote body (prose or a nested blockquote).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuoteChild {
    Inline(InlineNode),
    Quote(QuoteNode),
}

/// Image block: `image("url", alt: "alt text")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageNode {
    pub line: usize,
    pub col: usize,
    pub src: String,
    pub alt: String,
}

/// Table block: `table { row { cell { ... } } }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableNode {
    pub line: usize,
    pub col: usize,
    pub rows: Vec<RowNode>,
}

/// Table row node: `row { ... }` or `row(header: true) { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowNode {
    pub line: usize,
    pub col: usize,
    pub header: bool,
    pub cells: Vec<CellNode>,
}

/// Individual table cell: `cell { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellNode {
    pub line: usize,
    pub col: usize,
    pub children: Vec<InlineNode>,
}

/// Specific variant of an inline element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineKind {
    Text(String),
    Bold(Vec<InlineNode>),
    Italic(Vec<InlineNode>),
    Code(String),
    Link {
        url: String,
        children: Vec<InlineNode>,
    },
    Strike(Vec<InlineNode>),
}

/// An inline syntax tree element with position tracking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineNode {
    pub line: usize,
    pub col: usize,
    pub kind: InlineKind,
}

impl InlineNode {
    pub fn new(line: usize, col: usize, kind: InlineKind) -> Self {
        Self { line, col, kind }
    }

    pub fn text(value: impl Into<String>) -> Self {
        Self {
            line: 0,
            col: 0,
            kind: InlineKind::Text(value.into()),
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match &self.kind {
            InlineKind::Text(_) => "Text",
            InlineKind::Bold(_) => "Bold",
            InlineKind::Italic(_) => "Italic",
            InlineKind::Code(_) => "InlineCode",
            InlineKind::Link { .. } => "Link",
            InlineKind::Strike(_) => "Strike",
        }
    }

    /// Trims a single leading space if this node is a Text inline.
    pub fn trim_start_space(&mut self) {
        if let InlineKind::Text(ref mut s) = self.kind {
            if s.starts_with(' ') {
                s.remove(0);
            }
        }
    }

    /// Trims a single trailing space if this node is a Text inline.
    pub fn trim_end_space(&mut self) {
        if let InlineKind::Text(ref mut s) = self.kind {
            if s.ends_with(' ') {
                s.pop();
            }
        }
    }
}

/// Trims leading whitespace from the first inline node and trailing whitespace
/// from the last inline node in a sequence if they are Text nodes.
pub fn trim_inline_edges(nodes: &mut [InlineNode]) {
    if let Some(first) = nodes.first_mut() {
        first.trim_start_space();
    }
    if let Some(last) = nodes.last_mut() {
        last.trim_end_space();
    }
}
