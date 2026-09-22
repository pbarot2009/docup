use std::collections::HashMap;

/// Root AST node representing a fully parsed .du document[span_1](start_span)[span_1](end_span).
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

/// Metadata block at the start of a document: `meta { key: "value", ... }`[span_2](start_span)[span_2](end_span).
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

/// Category/severity kind for callout/admonition blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutKind {
    Note,
    Tip,
    Warning,
    Danger,
}

impl CalloutKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CalloutKind::Note => "note",
            CalloutKind::Tip => "tip",
            CalloutKind::Warning => "warning",
            CalloutKind::Danger => "danger",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "note" | "info" => Some(CalloutKind::Note),
            "tip" | "hint" => Some(CalloutKind::Tip),
            "warning" | "caution" => Some(CalloutKind::Warning),
            "danger" | "error" => Some(CalloutKind::Danger),
            _ => None,
        }
    }
}

/// Callout/admonition block: `callout(type: "warning") { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalloutNode {
    pub line: usize,
    pub col: usize,
    pub kind: CalloutKind,
    pub children: Vec<InlineNode>,
}

/// Raw unescaped HTML escape hatch block: `raw {! <div>...</div> !}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawNode {
    pub line: usize,
    pub col: usize,
    pub html: String,
}

/// Standalone display LaTeX math formula: `math {! \int_{0}^{1} x dx !}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathBlockNode {
    pub line: usize,
    pub col: usize,
    pub latex: String,
}

/// Table of contents placeholder block: `toc {}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TOCNode {
    pub line: usize,
    pub col: usize,
}

/// Footnote definition block: `footnote(id: "note1") { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootnoteDefNode {
    pub line: usize,
    pub col: usize,
    pub id: String,
    pub children: Vec<InlineNode>,
}

/// Enumeration of all top-level block constructs supported in DocUP[span_3](start_span)[span_3](end_span).
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
    Callout(CalloutNode),
    Raw(RawNode),
    Math(MathBlockNode),
    TOC(TOCNode),
    Footnote(FootnoteDefNode),
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
            BlockNode::Callout(_) => "Callout",
            BlockNode::Raw(_) => "Raw",
            BlockNode::Math(_) => "Math",
            BlockNode::TOC(_) => "TOC",
            BlockNode::Footnote(_) => "Footnote",
        }
    }
}

/// Heading block: `h(1) { ... }` or `h(2, id: "sub", class: "sec") { ... }`[span_4](start_span)[span_4](end_span).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingNode {
    pub line: usize,
    pub col: usize,
    pub level: usize,
    pub attrs: HashMap<String, String>,
    pub children: Vec<InlineNode>,
}

/// Paragraph block: `p { ... }`[span_5](start_span)[span_5](end_span).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphNode {
    pub line: usize,
    pub col: usize,
    pub children: Vec<InlineNode>,
}

/// Horizontal rule: `hr {}`[span_6](start_span)[span_6](end_span).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HRNode {
    pub line: usize,
    pub col: usize,
}

/// Fenced raw code block: `codeblock(lang: "rust", file: "main.rs", line_numbers: true, highlight: "1,3-5") {! ... !}`[span_7](start_span)[span_7](end_span).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlockNode {
    pub line: usize,
    pub col: usize,
    pub language: String,
    pub file: String,
    pub raw_code: String,
    pub line_numbers: bool,
    pub highlight_lines: Vec<usize>,
}

/// Ordered or unordered list: `list { ... }` or `list(ordered: true) { ... }`[span_8](start_span)[span_8](end_span).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListNode {
    pub line: usize,
    pub col: usize,
    pub ordered: bool,
    pub items: Vec<ItemNode>,
}

/// List item node: `item { ... }` or `task(done: true/false) { ... }`[span_9](start_span)[span_9](end_span).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemNode {
    pub line: usize,
    pub col: usize,
    /// `None` for regular items, `Some(true/false)` for task list checkboxes[span_10](start_span)[span_10](end_span).
    pub done: Option<bool>,
    pub children: Vec<ItemChild>,
}

/// Elements permitted inside a list item body (prose or a nested list).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemChild {
    Inline(InlineNode),
    List(ListNode),
}

/// Blockquote block: `quote { ... }`[span_11](start_span)[span_11](end_span).
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

/// Image block: `image("url", alt: "alt text")`[span_12](start_span)[span_12](end_span).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageNode {
    pub line: usize,
    pub col: usize,
    pub src: String,
    pub alt: String,
}

/// Table block: `table { row { cell { ... } } }`[span_13](start_span)[span_13](end_span).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableNode {
    pub line: usize,
    pub col: usize,
    pub rows: Vec<RowNode>,
}

/// Table row node: `row { ... }` or `row(header: true) { ... }`[span_14](start_span)[span_14](end_span).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowNode {
    pub line: usize,
    pub col: usize,
    pub header: bool,
    pub cells: Vec<CellNode>,
}

/// Individual table cell: `cell { ... }`[span_15](start_span)[span_15](end_span).
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
    Math(String),
    FootnoteRef(String),
}

/// An inline syntax tree element with position tracking[span_16](start_span)[span_16](end_span).
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
            InlineKind::Math(_) => "Math",
            InlineKind::FootnoteRef(_) => "FootnoteRef",
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
