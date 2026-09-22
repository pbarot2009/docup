use std::collections::HashMap;

use crate::ast::{
    BlockNode, DocumentNode, InlineKind, InlineNode, ItemChild, ListNode, MetaNode, QuoteChild,
    TableNode,
};
use crate::errors::SemaError;

/// Analyze validates the document AST and returns a SemaError on the first
/// semantic violation found.
pub fn analyze(doc: &DocumentNode) -> Result<(), SemaError> {
    if let Some(ref meta) = doc.meta {
        check_meta_fields(meta)?;
    }

    // Collect all footnote definitions and verify there are no duplicate IDs
    let mut footnote_defs: HashMap<String, (usize, usize)> = HashMap::new();
    for block in &doc.blocks {
        if let BlockNode::Footnote(f) = block {
            if footnote_defs.contains_key(&f.id) {
                return Err(SemaError::new(
                    f.line,
                    f.col,
                    format!("duplicate footnote definition with id \"{}\"", f.id),
                ));
            }
            footnote_defs.insert(f.id.clone(), (f.line, f.col));
        }
    }

    for block in &doc.blocks {
        analyze_block(block, &footnote_defs)?;
    }
    Ok(())
}

fn check_meta_fields(m: &MetaNode) -> Result<(), SemaError> {
    for required in ["title"] {
        if !m.fields.contains_key(required) {
            return Err(SemaError::new(
                m.line,
                m.col,
                format!("meta block is missing required field \"{required}\""),
            ));
        }
    }
    Ok(())
}

fn analyze_block(
    block: &BlockNode,
    footnote_defs: &HashMap<String, (usize, usize)>,
) -> Result<(), SemaError> {
    match block {
        BlockNode::Heading(h) => {
            if h.level < 1 || h.level > 6 {
                return Err(SemaError::new(
                    h.line,
                    h.col,
                    format!("heading level {} out of range, must be 1-6", h.level),
                ));
            }
            analyze_inlines(&h.children, footnote_defs)
        }
        BlockNode::Paragraph(p) => analyze_inlines(&p.children, footnote_defs),
        BlockNode::CodeBlock(cb) => {
            if cb.raw_code.is_empty() {
                return Err(SemaError::new(
                    cb.line,
                    cb.col,
                    "codeblock has empty content",
                ));
            }
            Ok(())
        }
        BlockNode::HR(_) => Ok(()),
        BlockNode::List(l) => analyze_list(l, footnote_defs),
        BlockNode::Quote(q) => analyze_quote_children(&q.children, footnote_defs),
        BlockNode::Image(img) => {
            if img.src.is_empty() {
                return Err(SemaError::new(
                    img.line,
                    img.col,
                    "image is missing a source URL",
                ));
            }
            Ok(())
        }
        BlockNode::Table(t) => analyze_table(t, footnote_defs),
        BlockNode::Callout(c) => analyze_inlines(&c.children, footnote_defs),
        BlockNode::Raw(r) => {
            if r.html.is_empty() {
                return Err(SemaError::new(r.line, r.col, "raw block has empty content"));
            }
            Ok(())
        }
        BlockNode::Math(m) => {
            if m.latex.is_empty() {
                return Err(SemaError::new(
                    m.line,
                    m.col,
                    "math block has empty content",
                ));
            }
            Ok(())
        }
        BlockNode::TOC(_) => Ok(()),
        BlockNode::Footnote(f) => analyze_inlines(&f.children, footnote_defs),
        BlockNode::Include(inc) => {
            if inc.path.trim().is_empty() {
                return Err(SemaError::new(
                    inc.line,
                    inc.col,
                    "include statement has empty path",
                ));
            }
            Ok(())
        }
    }
}

fn analyze_list(
    list: &ListNode,
    footnote_defs: &HashMap<String, (usize, usize)>,
) -> Result<(), SemaError> {
    if list.items.is_empty() {
        return Err(SemaError::new(
            list.line,
            list.col,
            "list must contain at least one item",
        ));
    }
    for item in &list.items {
        analyze_item_children(&item.children, footnote_defs)?;
    }
    Ok(())
}

fn analyze_table(
    t: &TableNode,
    footnote_defs: &HashMap<String, (usize, usize)>,
) -> Result<(), SemaError> {
    if t.rows.is_empty() {
        return Err(SemaError::new(
            t.line,
            t.col,
            "table must contain at least one row",
        ));
    }
    let mut width: Option<usize> = None;
    for row in &t.rows {
        if row.cells.is_empty() {
            return Err(SemaError::new(
                row.line,
                row.col,
                "row must contain at least one cell",
            ));
        }
        match width {
            None => width = Some(row.cells.len()),
            Some(w) if w != row.cells.len() => {
                return Err(SemaError::new(
                    row.line,
                    row.col,
                    format!(
                        "row has {} cells, expected {} to match the table's other rows",
                        row.cells.len(),
                        w
                    ),
                ));
            }
            _ => {}
        }
        for cell in &row.cells {
            analyze_inlines(&cell.children, footnote_defs)?;
        }
    }
    Ok(())
}

fn analyze_item_children(
    children: &[ItemChild],
    footnote_defs: &HashMap<String, (usize, usize)>,
) -> Result<(), SemaError> {
    for child in children {
        match child {
            ItemChild::List(nested) => analyze_list(nested, footnote_defs)?,
            ItemChild::Inline(inline) => analyze_inline(inline, footnote_defs)?,
        }
    }
    Ok(())
}

fn analyze_quote_children(
    children: &[QuoteChild],
    footnote_defs: &HashMap<String, (usize, usize)>,
) -> Result<(), SemaError> {
    for child in children {
        match child {
            QuoteChild::Quote(nested) => analyze_quote_children(&nested.children, footnote_defs)?,
            QuoteChild::Inline(inline) => analyze_inline(inline, footnote_defs)?,
        }
    }
    Ok(())
}

fn analyze_inlines(
    children: &[InlineNode],
    footnote_defs: &HashMap<String, (usize, usize)>,
) -> Result<(), SemaError> {
    for inline in children {
        analyze_inline(inline, footnote_defs)?;
    }
    Ok(())
}

fn analyze_inline(
    inline: &InlineNode,
    footnote_defs: &HashMap<String, (usize, usize)>,
) -> Result<(), SemaError> {
    match &inline.kind {
        InlineKind::Text(_) | InlineKind::Code(_) | InlineKind::Math(_) => Ok(()),
        InlineKind::Bold(children)
        | InlineKind::Italic(children)
        | InlineKind::Strike(children) => analyze_inlines(children, footnote_defs),
        InlineKind::Link { url, children } => {
            if url.is_empty() {
                return Err(SemaError::new(
                    inline.line,
                    inline.col,
                    "link is missing a URL",
                ));
            }
            analyze_inlines(children, footnote_defs)
        }
        InlineKind::FootnoteRef(id) => {
            if !footnote_defs.contains_key(id) {
                return Err(SemaError::new(
                    inline.line,
                    inline.col,
                    format!("unresolved footnote reference \"{id}\""),
                ));
            }
            Ok(())
        }
    }
}
