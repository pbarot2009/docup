use crate::ast::{
    BlockNode, CalloutNode, ChartNode, CodeBlockNode, DocumentNode, FootnoteDefNode, HeadingNode,
    ImageNode, IncludeNode, InlineKind, InlineNode, ItemChild, ItemNode, ListNode, MathBlockNode,
    MermaidNode, MetaNode, ParagraphNode, QuoteChild, QuoteNode, RawNode, RowNode, TableNode,
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

/// Parse DocUP source and print it in the stable layout.
///
/// The result parses again to the same markup, so `fmt` then `fmt --check`
/// agrees. Invalid source returns the parse error and is left unchanged.
pub fn format_source(src: &str) -> Result<String, ParseError> {
    let mut parser = Parser::new(src.as_bytes())?;
    let doc = parser.parse_document()?;
    Ok(format_document(&doc))
}

/// Print a document node in the stable layout.
pub fn format_document(doc: &DocumentNode) -> String {
    let mut out = String::new();
    let mut wrote = false;

    if let Some(ref meta) = doc.meta {
        format_meta(&mut out, meta);
        wrote = true;
    }

    for block in &doc.blocks {
        if wrote {
            out.push_str("\n\n");
        }
        format_block(&mut out, block, 0);
        wrote = true;
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
        let pos_a = PREFERRED_META_KEYS.iter().position(|k| k == a);
        let pos_b = PREFERRED_META_KEYS.iter().position(|k| k == b);
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
    match block {
        BlockNode::Heading(h) => format_heading(w, h, depth),
        BlockNode::Paragraph(p) => format_paragraph(w, p, depth),
        BlockNode::CodeBlock(cb) => format_codeblock(w, cb, depth),
        BlockNode::HR(_) => {
            w.push_str(&indent_str(depth));
            w.push_str("hr {}");
        }
        BlockNode::List(l) => format_list(w, l, depth),
        BlockNode::Quote(q) => format_quote(w, q, depth),
        BlockNode::Image(img) => format_image(w, img, depth),
        BlockNode::Table(t) => format_table(w, t, depth),
        BlockNode::Callout(c) => format_callout(w, c, depth),
        BlockNode::Raw(r) => format_raw(w, r, depth),
        BlockNode::Math(m) => format_math(w, m, depth),
        BlockNode::Mermaid(m) => format_mermaid(w, m, depth),
        BlockNode::Chart(c) => format_chart(w, c, depth),
        BlockNode::TOC(_) => {
            w.push_str(&indent_str(depth));
            w.push_str("toc {}");
        }
        BlockNode::Footnote(f) => format_footnote(w, f, depth),
        BlockNode::Include(inc) => format_include(w, inc, depth),
    }
}

fn format_heading(w: &mut String, h: &HeadingNode, depth: usize) {
    let ind = indent_str(depth);
    let mut head = format!("h({})", h.level);
    if !h.attrs.is_empty() {
        head.pop();
        let mut keys: Vec<&String> = h.attrs.keys().collect();
        keys.sort_by(|a, b| attr_order(a, b));
        for key in keys {
            head.push_str(", ");
            head.push_str(key);
            head.push_str(": \"");
            head.push_str(&escape_du_string(&h.attrs[key]));
            head.push('"');
        }
        head.push(')');
    }

    let body = format_inlines(&h.children);
    write_prose_block(w, &ind, &head, &body);
}

fn format_paragraph(w: &mut String, p: &ParagraphNode, depth: usize) {
    let ind = indent_str(depth);
    let body = format_inlines(&p.children);
    write_prose_block(w, &ind, "p", &body);
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
    if !cb.src.is_empty() {
        attrs.push(format!("src: \"{}\"", escape_du_string(&cb.src)));
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

    // A src attribute loads the file. The raw body is omitted so the two
    // sources cannot disagree after a format.
    if !cb.src.is_empty() {
        return;
    }

    write_raw_scope(w, &ind, &cb.raw_code);
}

fn format_list(w: &mut String, list: &ListNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    if list.ordered {
        w.push_str("list(ordered: true) {\n");
    } else {
        w.push_str("list {\n");
    }

    for (i, item) in list.items.iter().enumerate() {
        if i > 0 {
            w.push('\n');
        }
        format_item(w, item, depth + 1);
    }

    if !list.items.is_empty() {
        w.push('\n');
    }
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

    let has_nested = item.children.iter().any(|c| matches!(c, ItemChild::List(_)));
    if !has_nested {
        let inlines = inline_only(&item.children);
        let body = format_inlines(&inlines);
        write_prose_block(w, &ind, tag, &body);
        return;
    }

    w.push_str(&ind);
    w.push_str(tag);
    w.push_str(" {\n");

    let mut pending = Vec::new();
    let mut wrote = false;
    for child in &item.children {
        match child {
            ItemChild::Inline(node) => pending.push(node.clone()),
            ItemChild::List(nested) => {
                if !pending.is_empty() {
                    let body = format_inlines(&pending);
                    if !body.is_empty() {
                        w.push_str(&wrap_prose(&body, depth + 1, MAX_LINE_WIDTH));
                        w.push('\n');
                        wrote = true;
                    }
                    pending.clear();
                }
                if wrote {
                    w.push('\n');
                }
                format_list(w, nested, depth + 1);
                w.push('\n');
                wrote = true;
            }
        }
    }
    if !pending.is_empty() {
        let body = format_inlines(&pending);
        if !body.is_empty() {
            if wrote {
                w.push('\n');
            }
            w.push_str(&wrap_prose(&body, depth + 1, MAX_LINE_WIDTH));
            w.push('\n');
        }
    }

    w.push_str(&ind);
    w.push('}');
}

fn format_quote(w: &mut String, quote: &QuoteNode, depth: usize) {
    let ind = indent_str(depth);
    let has_nested = quote
        .children
        .iter()
        .any(|c| matches!(c, QuoteChild::Quote(_)));

    if !has_nested {
        let inlines: Vec<InlineNode> = quote
            .children
            .iter()
            .filter_map(|c| match c {
                QuoteChild::Inline(n) => Some(n.clone()),
                QuoteChild::Quote(_) => None,
            })
            .collect();
        let body = format_inlines(&inlines);
        write_prose_block(w, &ind, "quote", &body);
        return;
    }

    w.push_str(&ind);
    w.push_str("quote {\n");

    let mut pending = Vec::new();
    let mut wrote = false;
    for child in &quote.children {
        match child {
            QuoteChild::Inline(node) => pending.push(node.clone()),
            QuoteChild::Quote(nested) => {
                if !pending.is_empty() {
                    let body = format_inlines(&pending);
                    if !body.is_empty() {
                        w.push_str(&wrap_prose(&body, depth + 1, MAX_LINE_WIDTH));
                        w.push('\n');
                        wrote = true;
                    }
                    pending.clear();
                }
                if wrote {
                    w.push('\n');
                }
                format_quote(w, nested, depth + 1);
                w.push('\n');
                wrote = true;
            }
        }
    }
    if !pending.is_empty() {
        let body = format_inlines(&pending);
        if !body.is_empty() {
            if wrote {
                w.push('\n');
            }
            w.push_str(&wrap_prose(&body, depth + 1, MAX_LINE_WIDTH));
            w.push('\n');
        }
    }

    w.push_str(&ind);
    w.push('}');
}

fn format_image(w: &mut String, img: &ImageNode, depth: usize) {
    w.push_str(&indent_str(depth));
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

fn format_table(w: &mut String, table: &TableNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("table {\n");
    for (i, row) in table.rows.iter().enumerate() {
        if i > 0 {
            w.push('\n');
        }
        format_row(w, row, depth + 1);
    }
    if !table.rows.is_empty() {
        w.push('\n');
    }
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
    for cell in &row.cells {
        let body = format_inlines(&cell.children);
        write_prose_block(w, &indent_str(depth + 1), "cell", &body);
        w.push('\n');
    }
    w.push_str(&ind);
    w.push('}');
}

fn format_callout(w: &mut String, callout: &CalloutNode, depth: usize) {
    let ind = indent_str(depth);
    let head = format!("callout(type: \"{}\")", callout.kind.as_str());
    let body = format_inlines(&callout.children);
    write_prose_block(w, &ind, &head, &body);
}

fn format_raw(w: &mut String, raw: &RawNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("raw");
    write_raw_scope(w, &ind, &raw.html);
}

fn format_math(w: &mut String, math: &MathBlockNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("math");
    write_raw_scope(w, &ind, &math.latex);
}

fn format_mermaid(w: &mut String, diagram: &MermaidNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    w.push_str("mermaid");
    if !diagram.caption.is_empty() {
        w.push_str("(caption: \"");
        w.push_str(&escape_du_string(&diagram.caption));
        w.push_str("\")");
    }
    write_raw_scope(w, &ind, &diagram.source);
}

fn format_chart(w: &mut String, chart: &ChartNode, depth: usize) {
    let ind = indent_str(depth);
    w.push_str(&ind);
    if chart.as_graph {
        w.push_str("graph");
    } else {
        w.push_str("chart");
    }
    let mut attrs = Vec::new();
    let default_type = if chart.as_graph { "line" } else { "bar" };
    if chart.kind.as_str() != default_type {
        attrs.push(format!("type: \"{}\"", chart.kind.as_str()));
    }
    if !chart.title.is_empty() {
        attrs.push(format!("title: \"{}\"", escape_du_string(&chart.title)));
    }
    if !attrs.is_empty() {
        w.push('(');
        w.push_str(&attrs.join(", "));
        w.push(')');
    }
    w.push_str(" {\n");
    for (i, row) in chart.rows.iter().enumerate() {
        if i > 0 {
            w.push('\n');
        }
        format_row(w, row, depth + 1);
    }
    if !chart.rows.is_empty() {
        w.push('\n');
    }
    w.push_str(&ind);
    w.push('}');
}

fn format_footnote(w: &mut String, note: &FootnoteDefNode, depth: usize) {
    let ind = indent_str(depth);
    let head = format!("footnote(id: \"{}\")", escape_du_string(&note.id));
    let body = format_inlines(&note.children);
    write_prose_block(w, &ind, &head, &body);
}

fn format_include(w: &mut String, inc: &IncludeNode, depth: usize) {
    w.push_str(&indent_str(depth));
    w.push_str("include \"");
    w.push_str(&escape_du_string(&inc.path));
    w.push('"');
}

fn write_raw_scope(w: &mut String, ind: &str, body: &str) {
    let escaped = escape_raw_block_content(body.trim_end_matches([' ', '\t', '\r', '\n']));
    w.push_str(" {!\n");
    if !escaped.is_empty() {
        w.push_str(&escaped);
        w.push('\n');
    }
    w.push_str(ind);
    w.push_str("!}");
}

/// One-line form when it fits, otherwise a wrapped body.
/// Empty bodies stay `name {}` so the second pass prints the same text.
fn write_prose_block(w: &mut String, ind: &str, name: &str, body: &str) {
    if body.is_empty() {
        w.push_str(ind);
        w.push_str(name);
        w.push_str(" {}");
        return;
    }
    let one = format!("{ind}{name} {{ {body} }}");
    if char_len(&one) <= MAX_LINE_WIDTH && !body.contains('\n') {
        w.push_str(&one);
        return;
    }
    w.push_str(ind);
    w.push_str(name);
    w.push_str(" {\n");
    w.push_str(&wrap_prose(body, ind_depth(ind) + 1, MAX_LINE_WIDTH));
    w.push('\n');
    w.push_str(ind);
    w.push('}');
}

fn inline_only(children: &[ItemChild]) -> Vec<InlineNode> {
    children
        .iter()
        .filter_map(|c| match c {
            ItemChild::Inline(n) => Some(n.clone()),
            ItemChild::List(_) => None,
        })
        .collect()
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
        InlineKind::Text(t) => w.push_str(&escape_prose_text(t)),
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
            w.push_str(&escape_inline_code(code));
            w.push('}');
        }
        InlineKind::Math(math) => {
            w.push_str("m{");
            w.push_str(&escape_inline_math(math));
            w.push('}');
        }
        InlineKind::Link { url, children } => {
            w.push_str("link(\"");
            w.push_str(&escape_du_string(url));
            w.push_str("\"){");
            w.push_str(&format_inlines(children));
            w.push('}');
        }
        InlineKind::FootnoteRef(id) => {
            w.push_str("fn(\"");
            w.push_str(&escape_du_string(id));
            w.push_str("\")");
        }
    }
}

fn escape_raw_block_content(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 2 < chars.len() && chars[i + 1] == '!' && chars[i + 2] == '}' {
            out.push_str("\\\\!}");
            i += 3;
            continue;
        }
        if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '}' {
            out.push_str("\\!}");
            i += 2;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn escape_inline_code(s: &str) -> String {
    escape_balanced_literal(s, false)
}

fn escape_inline_math(s: &str) -> String {
    escape_balanced_literal(s, true)
}

/// Reprint a code or math literal so the lexer reads the same bytes back.
///
/// Code unescapes `\{`, `\}`, and `\\`. Math keeps those sequences, so a
/// stored backslash has to be doubled only for code.
fn escape_balanced_literal(s: &str, is_math: bool) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut depth: usize = 1;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' {
            let next = chars.get(i + 1).copied();
            if is_math {
                if matches!(next, Some('{') | Some('}') | Some('\\')) {
                    out.push('\\');
                    out.push(next.unwrap());
                    i += 2;
                    continue;
                }
                out.push('\\');
                i += 1;
                continue;
            }
            out.push_str("\\\\");
            i += 1;
            continue;
        }
        if c == '{' {
            depth += 1;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '}' {
            if depth <= 1 {
                out.push('\\');
                out.push('}');
            } else {
                depth -= 1;
                out.push('}');
            }
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn escape_prose_text(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                out.push_str("\\\\");
                i += 1;
            }
            '{' => {
                out.push_str("\\{");
                i += 1;
            }
            '}' => {
                out.push_str("\\}");
                i += 1;
            }
            '!' => {
                out.push('!');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

fn wrap_prose(text: &str, depth: usize, max_width: usize) -> String {
    let ind = indent_str(depth);
    let words = split_atoms(text);
    if words.is_empty() {
        return String::new();
    }

    let limit = max_width.max(ind.chars().count() + 1);
    let mut lines = Vec::new();
    let mut current = ind.clone();

    for word in words {
        if current.chars().count() == ind.chars().count() {
            current.push_str(&word);
            continue;
        }
        if current.chars().count() + 1 + word.chars().count() <= limit {
            current.push(' ');
            current.push_str(&word);
        } else {
            lines.push(std::mem::take(&mut current));
            current = ind.clone();
            current.push_str(&word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.join("\n")
}

/// Split prose on whitespace, but keep `code{}` and `m{}` in one piece.
/// A newline inside those spans would change the stored literal.
fn split_atoms(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_whitespace() {
            if !cur.is_empty() {
                words.push(std::mem::take(&mut cur));
            }
            i += 1;
            continue;
        }
        if let Some(end) = atomic_inline(&chars, i) {
            for ch in &chars[i..end] {
                cur.push(*ch);
            }
            i = end;
            continue;
        }
        cur.push(chars[i]);
        i += 1;
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

fn atomic_inline(chars: &[char], i: usize) -> Option<usize> {
    let (tag_len, math) = if starts_with_at(chars, i, "code{") {
        (5, false)
    } else if starts_with_at(chars, i, "m{") {
        (2, true)
    } else {
        return None;
    };
    consume_balanced(chars, i + tag_len - 1, math).map(|end| end)
}

fn consume_balanced(chars: &[char], open: usize, _math: bool) -> Option<usize> {
    if open >= chars.len() || chars[open] != '{' {
        return None;
    }
    let mut depth = 1;
    let mut i = open + 1;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next == '{' || next == '}' || next == '\\' {
                i += 2;
                continue;
            }
        }
        if chars[i] == '{' {
            depth += 1;
        } else if chars[i] == '}' {
            depth -= 1;
            i += 1;
            if depth == 0 {
                return Some(i);
            }
            continue;
        }
        i += 1;
    }
    None
}

fn starts_with_at(chars: &[char], i: usize, pat: &str) -> bool {
    let pat: Vec<char> = pat.chars().collect();
    if i + pat.len() > chars.len() {
        return false;
    }
    chars[i..i + pat.len()] == pat[..]
}

fn indent_str(depth: usize) -> String {
    INDENT.repeat(depth)
}

fn ind_depth(ind: &str) -> usize {
    ind.len() / INDENT.len()
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn attr_order(a: &str, b: &str) -> std::cmp::Ordering {
    fn rank(k: &str) -> u8 {
        match k {
            "id" => 0,
            "class" => 1,
            _ => 2,
        }
    }
    rank(a).cmp(&rank(b)).then_with(|| a.cmp(b))
}

fn escape_du_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

fn format_line_ranges(lines: &[usize]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut sorted = lines.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    let mut ranges = Vec::new();
    let mut start = sorted[0];
    let mut end = sorted[0];
    for &num in &sorted[1..] {
        if num == end + 1 {
            end = num;
        } else {
            ranges.push(range_text(start, end));
            start = num;
            end = num;
        }
    }
    ranges.push(range_text(start, end));
    ranges.join(",")
}

fn range_text(start: usize, end: usize) -> String {
    if start == end {
        format!("{start}")
    } else {
        format!("{start}-{end}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(src: &str) -> String {
        let once = format_source(src).unwrap_or_else(|e| panic!("parse failed: {e:?}\n{src}"));
        let twice = format_source(&once).unwrap_or_else(|e| panic!("reparse failed: {e:?}\n{once}"));
        assert_eq!(once, twice, "formatter is not stable\n{once}");
        once
    }

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
    fn test_format_raw_codeblock_escape() {
        let src = r#"codeblock(lang: "zig") {!
print("\nvalue: {\!}\n", .{val});
!}
"#;
        let formatted = round_trip(src);
        assert!(formatted.contains(r#"{\!}"#), "{formatted}");
    }

    #[test]
    fn test_format_codeblock_src_omits_body() {
        let src = r#"codeblock(lang: "rust", src: "src/main.rs", line_numbers: true)"#;
        let formatted = format_source(src).expect("must parse");
        assert_eq!(
            formatted,
            "codeblock(lang: \"rust\", src: \"src/main.rs\", line_numbers: true)\n"
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
        round_trip(src);
    }

    #[test]
    fn test_unicode_prose_does_not_panic() {
        let src = "p { Width is 80 — and infinity is ∞, so wrapping must stay on char bounds. }\n";
        let formatted = round_trip(src);
        assert!(formatted.contains('—'), "{formatted}");
        assert!(formatted.contains('∞'), "{formatted}");
    }

    #[test]
    fn test_inline_code_is_not_split() {
        let src = "p { Before code{let answer = 42; let name = \"docup\";} after the span, and more words so the line has to wrap cleanly. }\n";
        let formatted = round_trip(src);
        assert!(formatted.contains("code{let answer = 42; let name = \"docup\";}"), "{formatted}");
    }

    #[test]
    fn test_literal_braces_round_trip() {
        let src = "p { Use \\{ and \\} so braces stay text, and b{bold} still works. }\n";
        let formatted = round_trip(src);
        assert!(formatted.contains("\\{"), "{formatted}");
        assert!(formatted.contains("\\}"), "{formatted}");
    }

    #[test]
    fn test_nested_list_and_quote() {
        let src = r#"
list {
item { outer
list(ordered: true) {
item { inner }
}
}
}
quote { hello quote { nested } }
"#;
        round_trip(src);
    }

    #[test]
    fn test_diagram_round_trip() {
        let src = r#"
mermaid(caption: "Flow") {!
flowchart LR
  A-->B
!}
chart(type: "pie", title: "Share") {
  row { cell { docs } cell { 5 } }
}
graph(title: "Time") {
  row { cell { lex } cell { 4 } }
}
"#;
        let formatted = round_trip(src);
        assert!(formatted.contains("mermaid(caption: \"Flow\")"), "{formatted}");
        assert!(formatted.contains("chart(type: \"pie\""), "{formatted}");
        assert!(formatted.contains("graph(title: \"Time\")"), "{formatted}");
    }

    #[test]
    fn test_raw_unicode_not_corrupted() {
        let src = "raw {!\n<p>café — ∞</p>\n!}\n";
        let formatted = round_trip(src);
        assert!(formatted.contains("café — ∞"), "{formatted}");
    }
}
