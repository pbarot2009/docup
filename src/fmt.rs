use crate::ast::{
    BlockNode, CalloutNode, CodeBlockNode, DocumentNode, FootnoteDefNode, HeadingNode, ImageNode,
    IncludeNode, InlineKind, InlineNode, ItemChild, ItemNode, ListNode, MathBlockNode, MetaNode,
    ParagraphNode, QuoteChild, QuoteNode, RawNode, RowNode, TableNode,
};
use crate::errors::ParseError;
use crate::parser::Parser;

const PREFERRED_META_KEYS: &[&str] = &[
    "title",
    "author",
    "version",
    "theme",
    "lang",
    "description",
    "keywords",
    "canonical",
    "image",
    "stylesheet",
];

const INDENT: &str = "    ";
const MAX_LINE_WIDTH: usize = 80;

/// Parses source DocUP code and produces canonically formatted DocUP markup.
pub fn format_source(src: &str) -> Result<String, ParseError> {
    let mut parser = Parser::new(src.as_bytes())?;
    let doc = parser.parse_document()?;
    Ok(format_document(&doc))
}

/// Serializes an AST DocumentNode into canonical DocUP markup.
pub fn format_document(doc: &DocumentNode) -> String {
    let mut out = String::new();
    let mut has_previous_block = false;

    if let Some(ref meta) = doc.meta {
        format_meta(&mut out, meta);
        has_previous_block = true;
    }

    for block in &doc.blocks {
        if has_previous_block {
            out.push_str("\n\n");
        }
        format_block(&mut out, block, 0);
        has_previous_block = true;
    }

    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }

    out
}

fn format_meta(w: &mut String, meta: &MetaNode) {
    w.push_str("meta {\n");

    let mut keys: Vec<&String> = meta.fields.keys().collect();
    keys.sort_by(|a, b| {
        let pos_a = PREFERRED_META_KEYS.iter().position(|k| k == *a);
        let pos_b = PREFERRED_META_KEYS.iter().position(|k| k == *b);
        match (pos_a, pos_b) {
            (Some(ia), Some(ib)) => ia.cmp(&ib),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(b),
        }
    });

    for (i, key) in keys.iter().enumerate() {
        let val = &meta.fields[*key];
        w.push_str(INDENT);
        w.push_str(key);
        w.push_str(": \"");
        w.push_str(&escape_du_string(val));
        w.push('"');
        if i + 1 < keys.len() {
            w.push(',');
        }
        w.push('\n');
    }

    w.push('}');
}

fn format_block(w: &mut String, block: &BlockNode, depth: usize) {
    let ind = indent_str(depth);
    match block {
        BlockNode::Heading(h) => format_heading(w, h, depth),
        BlockNode::Paragraph(p) => format_paragraph(w, p, depth),
        BlockNode::CodeBlock(cb) => format_codeblock(w, cb, depth),
        BlockNode::HR(_) => {
            w.push_str(&ind);
            w.push_str("hr {}");
        }
        BlockNode::List(l) => format_list(w, l, depth),
        BlockNode::Quote(q) => format_quote(w, q, depth),
        BlockNode::Image(img) => format_image(w, img, depth),
        BlockNode::Table(t) => format_table(w, t, depth),
        BlockNode::Callout(c) => format_callout(w, c, depth),
        BlockNode::Raw(r) => format_raw(w, r, depth),
        BlockNode::Math(m) => format_math(w, m, depth),
        BlockNode::TOC(_) => {
            w.push_str(&ind);
            w.push_str("toc {}");
        }
        BlockNode::Footnote(f) => format_footnote(w, f, depth),
        BlockNode::Include(inc) => format_include(w, inc, depth),
    }
}

fn format_heading(w: &mut String, h: &HeadingNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str(&format!("h({})", h.level));

    if !h.attrs.is_empty() {
        w.truncate(w.len() - 1);
        let mut sorted_keys: Vec<&String> = h.attrs.keys().collect();
        sorted_keys.sort_by(|a, b| match (a.as_str(), b.as_str()) {
            ("id", _) => std::cmp::Ordering::Less,
            (_, "id") => std::cmp::Ordering::Greater,
            ("class", _) => std::cmp::Ordering::Less,
            (_, "class") => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        });

        for key in sorted_keys {
            let val = &h.attrs[key];
            w.push_str(&format!(", {key}: \"{}\"", escape_du_string(val)));
        }
        w.push(')');
    }

    w.push_str(" { ");
    w.push_str(&format_inlines(&h.children));
    w.push_str(" }");
}

fn format_paragraph(w: &mut String, p: &ParagraphNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("p {\n");
    let inner_text = format_inlines(&p.children);
    let wrapped = wrap_text(&inner_text, depth + 1, MAX_LINE_WIDTH);
    w.push_str(&wrapped);
    w.push('\n');
    w.push_str(&ind);
    w.push('}');
}

fn format_codeblock(w: &mut String, cb: &CodeBlockNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("codeblock");

    let mut attrs = Vec::new();
    if !cb.language.is_empty() {
        attrs.push(format!("lang: \"{}\"", escape_du_string(&cb.language)));
    }
    if !cb.file.is_empty() {
        attrs.push(format!("file: \"{}\"", escape_du_string(&cb.file)));
    }
    if cb.line_numbers {
        attrs.push("line_numbers: true".to_string());
    }
    if !cb.highlight_lines.is_empty() {
        attrs.push(format!(
            "highlight: \"{}\"",
            format_line_ranges(&cb.highlight_lines)
        ));
    }

    if !attrs.is_empty() {
        w.push('(');
        w.push_str(&attrs.join(", "));
        w.push(')');
    }

    w.push_str(" {!\n");
    w.push_str(cb.raw_code.trim_end());
    w.push('\n');
    w.push_str(&ind);
    w.push_str("!}");
}

fn format_list(w: &mut String, l: &ListNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    if l.ordered {
        w.push_str("list(ordered: true) {\n");
    } else {
        w.push_str("list {\n");
    }

    for (i, item) in l.items.iter().enumerate() {
        if i > 0 {
            w.push('\n');
        }
        format_item(w, item, depth + 1);
    }

    w.push('\n');
    w.push_str(&ind);
    w.push('}');
}

fn format_item(w: &mut String, item: &ItemNode, depth: usize) {
    let ind = indent_str(depth);
    let tag = match item.done {
        Some(true) => "task(done: true)",
        Some(false) => "task(done: false)",
        None => "item",
    };

    let has_nested_list = item
        .children
        .iter()
        .any(|c| matches!(c, ItemChild::List(_)));

    if !has_nested_list {
        let inlines: Vec<InlineNode> = item
            .children
            .iter()
            .filter_map(|c| match c {
                ItemChild::Inline(n) => Some(n.clone()),
                ItemChild::List(_) => None,
            })
            .collect();

        let inline_text = format_inlines(&inlines);
        if ind.len() + tag.len() + 5 + inline_text.len() <= MAX_LINE_WIDTH
            && !inline_text.contains('\n')
        {
            w.push_str(&ind);
            w.push_str(tag);
            w.push_str(" { ");
            w.push_str(&inline_text);
            w.push_str(" }");
            return;
        }

        w.push_str(&ind);
        w.push_str(tag);
        w.push_str(" {\n");
        let wrapped = wrap_text(&inline_text, depth + 1, MAX_LINE_WIDTH);
        w.push_str(&wrapped);
        w.push('\n');
        w.push_str(&ind);
        w.push('}');
    } else {
        w.push_str(&ind);
        w.push_str(tag);
        w.push_str(" {\n");

        let mut pending_inlines = Vec::new();
        for child in &item.children {
            match child {
                ItemChild::Inline(node) => {
                    pending_inlines.push(node.clone());
                }
                ItemChild::List(nested) => {
                    if !pending_inlines.is_empty() {
                        let text = format_inlines(&pending_inlines);
                        let wrapped = wrap_text(&text, depth + 1, MAX_LINE_WIDTH);
                        w.push_str(&wrapped);
                        w.push('\n');
                        pending_inlines.clear();
                    }
                    format_list(w, nested, depth + 1);
                    w.push('\n');
                }
            }
        }

        if !pending_inlines.is_empty() {
            let text = format_inlines(&pending_inlines);
            let wrapped = wrap_text(&text, depth + 1, MAX_LINE_WIDTH);
            w.push_str(&wrapped);
            w.push('\n');
        }

        if w.ends_with("\n\n") {
            w.pop();
        }

        w.push_str(&ind);
        w.push('}');
    }
}

fn format_quote(w: &mut String, q: &QuoteNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("quote {\n");

    let mut pending_inlines = Vec::new();
    let mut wrote_child = false;

    for child in &q.children {
        match child {
            QuoteChild::Inline(node) => {
                pending_inlines.push(node.clone());
            }
            QuoteChild::Quote(nested) => {
                if !pending_inlines.is_empty() {
                    if wrote_child {
                        w.push('\n');
                    }
                    let text = format_inlines(&pending_inlines);
                    let wrapped = wrap_text(&text, depth + 1, MAX_LINE_WIDTH);
                    w.push_str(&wrapped);
                    w.push('\n');
                    pending_inlines.clear();
                    wrote_child = true;
                }
                if wrote_child {
                    w.push('\n');
                }
                format_quote(w, nested, depth + 1);
                w.push('\n');
                wrote_child = true;
            }
        }
    }

    if !pending_inlines.is_empty() {
        if wrote_child {
            w.push('\n');
        }
        let text = format_inlines(&pending_inlines);
        let wrapped = wrap_text(&text, depth + 1, MAX_LINE_WIDTH);
        w.push_str(&wrapped);
        w.push('\n');
    }

    w.push_str(&ind);
    w.push('}');
}

fn format_image(w: &mut String, img: &ImageNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("image(\"");
    w.push_str(&escape_du_string(&img.src));
    w.push('"');
    if !img.alt.is_empty() {
        w.push_str(", alt: \"");
        w.push_str(&escape_du_string(&img.alt));
        w.push('"');
    }
    w.push(')');
}

fn format_table(w: &mut String, t: &TableNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("table {\n");

    for (row_idx, row) in t.rows.iter().enumerate() {
        if row_idx > 0 {
            w.push('\n');
        }
        format_row(w, row, depth + 1);
    }

    w.push('\n');
    w.push_str(&ind);
    w.push('}');
}

fn format_row(w: &mut String, row: &RowNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    if row.header {
        w.push_str("row(header: true) {\n");
    } else {
        w.push_str("row {\n");
    }

    let cell_ind = indent_str(depth + 1);
    for cell in &row.cells {
        let text = format_inlines(&cell.children);
        w.push_str(&cell_ind);
        w.push_str("cell { ");
        w.push_str(&text);
        w.push_str(" }\n");
    }

    w.push_str(&ind);
    w.push('}');
}

fn format_callout(w: &mut String, c: &CalloutNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str(&format!("callout(type: \"{}\") {{\n", c.kind.as_str()));
    let text = format_inlines(&c.children);
    let wrapped = wrap_text(&text, depth + 1, MAX_LINE_WIDTH);
    w.push_str(&wrapped);
    w.push('\n');
    w.push_str(&ind);
    w.push('}');
}

fn format_raw(w: &mut String, r: &RawNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("raw {!\n");
    w.push_str(r.html.trim_end());
    w.push('\n');
    w.push_str(&ind);
    w.push_str("!}");
}

fn format_math(w: &mut String, m: &MathBlockNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("math {!\n");
    w.push_str(m.latex.trim_end());
    w.push('\n');
    w.push_str(&ind);
    w.push_str("!}");
}

fn format_footnote(w: &mut String, f: &FootnoteDefNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str(&format!(
        "footnote(id: \"{}\") {{\n",
        escape_du_string(&f.id)
    ));
    let text = format_inlines(&f.children);
    let wrapped = wrap_text(&text, depth + 1, MAX_LINE_WIDTH);
    w.push_str(&wrapped);
    w.push('\n');
    w.push_str(&ind);
    w.push('}');
}

fn format_include(w: &mut String, inc: &IncludeNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str(&format!("include \"{}\"", escape_du_string(&inc.path)));
}

pub fn format_inlines(inlines: &[InlineNode]) -> String {
    let mut out = String::new();
    for inline in inlines {
        format_inline_into(&mut out, inline);
    }
    out
}

fn format_inline_into(w: &mut String, inline: &InlineNode) {
    match &inline.kind {
        InlineKind::Text(t) => w.push_str(t),
        InlineKind::Bold(children) => {
            w.push_str("b{");
            w.push_str(&format_inlines(children));
            w.push('}');
        }
        InlineKind::Italic(children) => {
            w.push_str("i{");
            w.push_str(&format_inlines(children));
            w.push('}');
        }
        InlineKind::Strike(children) => {
            w.push_str("strike{");
            w.push_str(&format_inlines(children));
            w.push('}');
        }
        InlineKind::Code(code) => {
            w.push_str("code{");
            w.push_str(code);
            w.push('}');
        }
        InlineKind::Math(math) => {
            w.push_str("m{");
            w.push_str(math);
            w.push('}');
        }
        InlineKind::Link { url, children } => {
            w.push_str(&format!("link(\"{}\"){{", escape_du_string(url)));
            w.push_str(&format_inlines(children));
            w.push('}');
        }
        InlineKind::FootnoteRef(id) => {
            w.push_str(&format!("fn(\"{}\")", escape_du_string(id)));
        }
    }
}

fn wrap_text(text: &str, depth: usize, max_width: usize) -> String {
    let ind = indent_str(depth);
    let words = split_prose_words(text);
    if words.is_empty() {
        return ind;
    }

    let mut lines = Vec::new();
    let mut current_line = String::from(&ind);

    for word in words {
        if current_line.len() == ind.len() {
            current_line.push_str(&word);
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(&word);
        } else {
            lines.push(current_line);
            current_line = format!("{ind}{word}");
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines.join("\n")
}

fn split_prose_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Keep code{...} and m{...} together as an atomic word
        if (text[i..].starts_with("code{") || text[i..].starts_with("m{"))
            && (cur.is_empty() || cur.ends_with(' '))
        {
            let is_code = text[i..].starts_with("code{");
            let tag_len = if is_code { 5 } else { 2 };
            cur.push_str(if is_code { "code{" } else { "m{" });
            i += tag_len;
            let mut depth = 1;
            while i < chars.len() && depth > 0 {
                let c = chars[i];
                if c == '{' {
                    depth += 1;
                } else if c == '}' {
                    depth -= 1;
                }
                cur.push(c);
                i += 1;
            }
            continue;
        }

        let c = chars[i];
        if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
            if !cur.is_empty() {
                words.push(cur.clone());
                cur.clear();
            }
        } else {
            cur.push(c);
        }
        i += 1;
    }

    if !cur.is_empty() {
        words.push(cur);
    }

    words
}

fn indent_str(depth: usize) -> String {
    INDENT.repeat(depth)
}

fn escape_du_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out
}

fn format_line_ranges(lines: &[usize]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut ranges = Vec::new();
    let mut start = lines[0];
    let mut end = lines[0];

    for &num in &lines[1..] {
        if num == end + 1 {
            end = num;
        } else {
            if start == end {
                ranges.push(format!("{start}"));
            } else {
                ranges.push(format!("{start}-{end}"));
            }
            start = num;
            end = num;
        }
    }

    if start == end {
        ranges.push(format!("{start}"));
    } else {
        ranges.push(format!("{start}-{end}"));
    }

    ranges.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_meta_and_heading() {
        let src = r#"meta { author: "Prathmesh", title: "Test" } h(1){Hello}"#;
        let formatted = format_source(src).expect("must parse");
        assert_eq!(
            formatted,
            "meta {\n    title: \"Test\",\n    author: \"Prathmesh\"\n}\n\nh(1) { Hello }\n"
        );
    }

    #[test]
    fn test_format_idempotency() {
        let src = r#"meta {
    title: "Title",
    author: "Prathmesh"
}

h(1, id: "intro") { Introduction }

p {
    This is a paragraph with b{bold} and code{let x = 1;}.
}

codeblock(lang: "rust") {!
fn main() {
    println!("Hello");
}
!}
"#;
        let formatted1 = format_source(src).expect("pass 1");
        let formatted2 = format_source(&formatted1).expect("pass 2");
        assert_eq!(formatted1, formatted2);
    }
}
