use std::collections::HashMap;

use crate::ast::{
    trim_inline_edges, BlockNode, CalloutKind, CalloutNode, CellNode, CodeBlockNode, DocumentNode,
    FootnoteDefNode, HRNode, HeadingNode, ImageNode, IncludeNode, InlineKind, InlineNode,
    ItemChild, ItemNode, ListNode, MathBlockNode, MetaNode, ParagraphNode, QuoteChild, QuoteNode,
    RawNode, RowNode, TOCNode, TableNode,
};
use crate::errors::{LexError, ParseError};
use crate::lexer::{Lexer, Token, TokenType};

/// max_inline_depth bounds nested inline elements (e.g. b{i{b{...}}}) to prevent
/// adversarial or deeply nested input from overflowing the stack.
const MAX_INLINE_DEPTH: usize = 64;

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError::new(err.line, err.col, err.message)
    }
}

pub struct Parser<'a> {
    lex: Lexer<'a>,
    cur: Token,
    depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a [u8]) -> Result<Self, ParseError> {
        let mut lex = Lexer::new(src);
        let cur = lex.next_token()?;
        Ok(Self { lex, cur, depth: 0 })
    }

    fn next(&mut self) -> Result<(), ParseError> {
        self.cur = self.lex.next_token()?;
        Ok(())
    }

    fn errorf(&self, msg: impl Into<String>) -> ParseError {
        ParseError::new(self.cur.line, self.cur.col, msg)
    }

    fn expect(&mut self, expected: TokenType, what: &str) -> Result<Token, ParseError> {
        if self.cur.token_type != expected {
            return Err(self.errorf(format!("expected {what}, got {:?}", self.cur.value)));
        }
        let tok = self.cur.clone();
        self.next()?;
        Ok(tok)
    }

    /// ParseDocument parses the complete token stream into a DocumentNode.
    pub fn parse_document(&mut self) -> Result<DocumentNode, ParseError> {
        let mut doc = DocumentNode::new();
        while self.cur.token_type != TokenType::Eof {
            if self.cur.token_type != TokenType::Ident {
                return Err(self.errorf(format!(
                    "expected a top-level block (meta, h, p, codeblock, hr, list, quote, image, table, callout, raw, math, toc, footnote, include), got {:?}",
                    self.cur.value
                )));
            }
            match self.cur.value.as_str() {
                "meta" => {
                    if doc.meta.is_some() {
                        return Err(self.errorf("duplicate meta block"));
                    }
                    doc.meta = Some(self.parse_meta()?);
                }
                "h" => {
                    let h = self.parse_heading()?;
                    doc.blocks.push(BlockNode::Heading(h));
                }
                "p" => {
                    let p = self.parse_paragraph()?;
                    doc.blocks.push(BlockNode::Paragraph(p));
                }
                "codeblock" => {
                    let cb = self.parse_code_block()?;
                    doc.blocks.push(BlockNode::CodeBlock(cb));
                }
                "hr" => {
                    let hr = self.parse_hr()?;
                    doc.blocks.push(BlockNode::HR(hr));
                }
                "list" => {
                    let list = self.parse_list()?;
                    doc.blocks.push(BlockNode::List(list));
                }
                "quote" => {
                    let q = self.parse_quote()?;
                    doc.blocks.push(BlockNode::Quote(q));
                }
                "image" => {
                    let img = self.parse_image()?;
                    doc.blocks.push(BlockNode::Image(img));
                }
                "table" => {
                    let tbl = self.parse_table()?;
                    doc.blocks.push(BlockNode::Table(tbl));
                }
                "callout" => {
                    let callout = self.parse_callout()?;
                    doc.blocks.push(BlockNode::Callout(callout));
                }
                "raw" => {
                    let raw = self.parse_raw()?;
                    doc.blocks.push(BlockNode::Raw(raw));
                }
                "math" => {
                    let math = self.parse_math()?;
                    doc.blocks.push(BlockNode::Math(math));
                }
                "toc" => {
                    let toc = self.parse_toc()?;
                    doc.blocks.push(BlockNode::TOC(toc));
                }
                "footnote" => {
                    let footnote = self.parse_footnote()?;
                    doc.blocks.push(BlockNode::Footnote(footnote));
                }
                "include" => {
                    let include = self.parse_include()?;
                    doc.blocks.push(BlockNode::Include(include));
                }
                _ => {
                    return Err(self.errorf(format!("unknown block type {:?}", self.cur.value)));
                }
            }
        }
        Ok(doc)
    }

    fn parse_include(&mut self) -> Result<IncludeNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'include'
        let path = if self.cur.token_type == TokenType::LParen {
            self.next()?;
            let p = self.expect(TokenType::String, "include file path")?.value;
            self.expect(TokenType::RParen, "')' to close include(...)")?;
            p
        } else if self.cur.token_type == TokenType::String {
            let p = self.cur.value.clone();
            self.next()?;
            p
        } else {
            return Err(self.errorf(format!(
                "expected file path string after include, got {:?}",
                self.cur.value
            )));
        };
        Ok(IncludeNode { line, col, path })
    }

    fn parse_meta(&mut self) -> Result<MetaNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'meta'
        self.expect(TokenType::LBrace, "'{'")?;

        let mut fields = HashMap::new();
        while self.cur.token_type != TokenType::RBrace {
            let key = self.expect(TokenType::Ident, "metadata key")?;
            self.expect(TokenType::Colon, "':'")?;
            let val = self.expect(TokenType::String, "string value")?;
            fields.insert(key.value, val.value);
            if self.cur.token_type == TokenType::Comma {
                self.next()?;
            }
        }
        self.expect(TokenType::RBrace, "'}'")?;
        Ok(MetaNode::new(line, col, fields))
    }

    /// parse_attrs parses an optional attribute list: `(key: "val", "posArg", flag: true)`
    fn parse_attrs(&mut self) -> Result<(HashMap<String, String>, Vec<String>), ParseError> {
        let mut attrs = HashMap::new();
        let mut positional = Vec::new();
        if self.cur.token_type != TokenType::LParen {
            return Ok((attrs, positional));
        }
        self.next()?; // consume '('
        while self.cur.token_type != TokenType::RParen {
            if self.cur.token_type == TokenType::String {
                positional.push(self.cur.value.clone());
                self.next()?;
            } else if self.cur.token_type == TokenType::Ident {
                let key = self.cur.value.clone();
                self.next()?;
                self.expect(TokenType::Colon, "':'")?;
                let val = self.expect_attr_value()?;
                attrs.insert(key, val);
            } else {
                return Err(self.errorf(format!(
                    "unexpected token {:?} in attribute list",
                    self.cur.value
                )));
            }
            if self.cur.token_type == TokenType::Comma {
                self.next()?;
            }
        }
        self.expect(TokenType::RParen, "')'")?;
        Ok((attrs, positional))
    }

    fn expect_attr_value(&mut self) -> Result<String, ParseError> {
        if self.cur.token_type == TokenType::String {
            let v = self.cur.value.clone();
            self.next()?;
            return Ok(v);
        }
        if self.cur.token_type == TokenType::Ident
            && (self.cur.value == "true" || self.cur.value == "false")
        {
            let v = self.cur.value.clone();
            self.next()?;
            return Ok(v);
        }
        Err(self.errorf(format!(
            "expected string or boolean value, got {:?}",
            self.cur.value
        )))
    }

    fn parse_heading(&mut self) -> Result<HeadingNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'h'
        self.expect(TokenType::LParen, "'(' after h")?;
        let level_tok = self.expect(TokenType::Number, "heading level")?;
        let level: usize = level_tok
            .value
            .parse()
            .map_err(|_| self.errorf(format!("invalid heading level {:?}", level_tok.value)))?;

        let mut attrs = HashMap::new();
        while self.cur.token_type == TokenType::Comma {
            self.next()?;
            let key = self.expect(TokenType::Ident, "attribute name")?;
            self.expect(TokenType::Colon, "':'")?;
            let val = self.expect(TokenType::String, "attribute value")?;
            attrs.insert(key.value, val.value);
        }
        self.expect(TokenType::RParen, "')'")?;
        if self.cur.token_type != TokenType::LBrace {
            return Err(self.errorf(format!("expected '{{', got {:?}", self.cur.value)));
        }
        let children = self.parse_prose_block()?;
        Ok(HeadingNode {
            line,
            col,
            level,
            attrs,
            children,
        })
    }

    fn parse_paragraph(&mut self) -> Result<ParagraphNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'p'
        if self.cur.token_type != TokenType::LBrace {
            return Err(self.errorf(format!("expected '{{', got {:?}", self.cur.value)));
        }
        let children = self.parse_prose_block()?;
        Ok(ParagraphNode {
            line,
            col,
            children,
        })
    }

    fn parse_callout(&mut self) -> Result<CalloutNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'callout'
        let (attrs, _) = self.parse_attrs()?;
        let kind_str = attrs.get("type").map(|s| s.as_str()).unwrap_or("note");
        let kind = match CalloutKind::parse(kind_str) {
            Some(k) => k,
            None => {
                return Err(self.errorf(format!(
                    "invalid callout type {:?}, expected note, tip, warning, or danger",
                    kind_str
                )));
            }
        };
        if self.cur.token_type != TokenType::LBrace {
            return Err(self.errorf(format!(
                "expected '{{' after callout, got {:?}",
                self.cur.value
            )));
        }
        let children = self.parse_prose_block()?;
        Ok(CalloutNode {
            line,
            col,
            kind,
            children,
        })
    }

    fn parse_raw(&mut self) -> Result<RawNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'raw'
        if self.cur.token_type != TokenType::RawScopeOpen {
            return Err(self.errorf(format!(
                "expected '{{!' to open raw scope, got {:?}",
                self.cur.value
            )));
        }
        let (raw, _, _) = self.lex.read_raw_until_bang_brace()?;
        self.next()?;
        Ok(RawNode {
            line,
            col,
            html: raw,
        })
    }

    fn parse_math(&mut self) -> Result<MathBlockNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'math'
        if self.cur.token_type != TokenType::RawScopeOpen {
            return Err(self.errorf(format!(
                "expected '{{!' to open math scope, got {:?}",
                self.cur.value
            )));
        }
        let (raw, _, _) = self.lex.read_raw_until_bang_brace()?;
        self.next()?;
        Ok(MathBlockNode {
            line,
            col,
            latex: raw,
        })
    }

    fn parse_toc(&mut self) -> Result<TOCNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'toc'
        self.expect(TokenType::LBrace, "'{' after toc")?;
        self.expect(TokenType::RBrace, "'}' to close toc{}")?;
        Ok(TOCNode { line, col })
    }

    fn parse_footnote(&mut self) -> Result<FootnoteDefNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'footnote'
        let (attrs, _) = self.parse_attrs()?;
        let id = match attrs.get("id") {
            Some(id) if !id.trim().is_empty() => id.clone(),
            _ => return Err(self.errorf("footnote missing 'id' attribute")),
        };
        if self.cur.token_type != TokenType::LBrace {
            return Err(self.errorf(format!(
                "expected '{{' after footnote, got {:?}",
                self.cur.value
            )));
        }
        let children = self.parse_prose_block()?;
        Ok(FootnoteDefNode {
            line,
            col,
            id,
            children,
        })
    }

    fn parse_prose_block(&mut self) -> Result<Vec<InlineNode>, ParseError> {
        let children = self.parse_prose_until_rbrace()?;
        self.next()?; // resync token stream past '}'
        Ok(children)
    }

    fn parse_prose_until_rbrace(&mut self) -> Result<Vec<InlineNode>, ParseError> {
        let mut children = Vec::new();
        loop {
            let text = collapse_whitespace(&self.lex.read_text_run());
            if !text.is_empty() {
                children.push(InlineNode::text(text));
            }
            if self.lex.at_eof() {
                return Err(ParseError::new(
                    self.lex.line,
                    self.lex.col,
                    "unterminated block, expected '}'",
                ));
            }
            if let Some(ident) = self.lex.at_inline_start() {
                let inline = self.parse_inline_raw(&ident)?;
                children.push(inline);
                continue;
            }
            self.lex.consume_rbrace()?;
            trim_inline_edges(&mut children);
            return Ok(children);
        }
    }

    fn parse_inline_raw(&mut self, ident: &str) -> Result<InlineNode, ParseError> {
        let line = self.lex.line;
        let col = self.lex.col;

        self.depth += 1;
        let res = self.parse_inline_raw_inner(ident, line, col);
        self.depth -= 1;
        res
    }

    fn parse_inline_raw_inner(
        &mut self,
        ident: &str,
        line: usize,
        col: usize,
    ) -> Result<InlineNode, ParseError> {
        if self.depth > MAX_INLINE_DEPTH {
            return Err(ParseError::new(
                line,
                col,
                format!("inline elements nested too deeply (limit {MAX_INLINE_DEPTH})"),
            ));
        }

        if ident == "link" {
            return self.parse_link_raw(line, col);
        }

        if ident == "fn" {
            return self.parse_fn_raw(line, col);
        }

        self.lex.consume_ident_only(ident);
        self.lex.consume_lbrace()?;

        if ident == "code" || ident == "m" {
            let is_math = ident == "m";
            let raw = self.lex.read_balanced_braces(is_math);
            self.lex.consume_rbrace()?;
            let kind = if is_math {
                InlineKind::Math(raw)
            } else {
                InlineKind::Code(raw)
            };
            return Ok(InlineNode::new(line, col, kind));
        }

        let children = self.parse_prose_until_rbrace()?;
        let kind = match ident {
            "b" => InlineKind::Bold(children),
            "i" => InlineKind::Italic(children),
            "strike" => InlineKind::Strike(children),
            _ => {
                return Err(ParseError::new(
                    line,
                    col,
                    format!("unknown inline element {ident}"),
                ));
            }
        };
        Ok(InlineNode::new(line, col, kind))
    }

    fn parse_link_raw(&mut self, line: usize, col: usize) -> Result<InlineNode, ParseError> {
        self.lex.consume_ident_only("link");
        self.lex.consume_raw_byte(b'(', "'(' after link")?;
        self.lex.skip_raw_spaces();
        let url = self.lex.read_raw_string()?;
        self.lex.skip_raw_spaces();

        while self.lex.peek_char() == b',' {
            self.lex.consume_raw_byte(b',', "','")?;
            self.lex.skip_raw_spaces();
            let attr_name = self.lex.read_raw_ident();
            if attr_name.is_empty() {
                return Err(ParseError::new(
                    self.lex.line,
                    self.lex.col,
                    "expected attribute name in link(...)",
                ));
            }
            self.lex.skip_raw_spaces();
            self.lex.consume_raw_byte(b':', "':' in link attribute")?;
            self.lex.skip_raw_spaces();
            let _ = self.lex.read_raw_string()?;
            self.lex.skip_raw_spaces();
        }

        self.lex.consume_raw_byte(b')', "')' to close link(...)")?;
        self.lex.skip_raw_spaces();
        self.lex.consume_lbrace()?;
        let children = self.parse_prose_until_rbrace()?;
        Ok(InlineNode::new(
            line,
            col,
            InlineKind::Link { url, children },
        ))
    }

    fn parse_fn_raw(&mut self, line: usize, col: usize) -> Result<InlineNode, ParseError> {
        self.lex.consume_ident_only("fn");
        self.lex.consume_raw_byte(b'(', "'(' after fn")?;
        self.lex.skip_raw_spaces();
        let id = if self.lex.peek_char() == b'"' {
            self.lex.read_raw_string()?
        } else {
            let ident = self.lex.read_raw_ident();
            if ident.is_empty() {
                return Err(ParseError::new(
                    self.lex.line,
                    self.lex.col,
                    "expected footnote identifier in fn(...)",
                ));
            }
            ident
        };
        self.lex.skip_raw_spaces();
        self.lex.consume_raw_byte(b')', "')' to close fn(...)")?;
        Ok(InlineNode::new(line, col, InlineKind::FootnoteRef(id)))
    }

    fn parse_code_block(&mut self) -> Result<CodeBlockNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'codeblock'
        let (mut attrs, _) = self.parse_attrs()?;
        if self.cur.token_type != TokenType::RawScopeOpen {
            return Err(self.errorf(format!(
                "expected '{{!' to open raw code scope, got {:?}",
                self.cur.value
            )));
        }
        let (raw, _, _) = self.lex.read_raw_until_bang_brace()?;
        self.next()?;
        let language = attrs.remove("lang").unwrap_or_default();
        let file = attrs.remove("file").unwrap_or_default();
        let line_numbers = attrs
            .get("line_numbers")
            .map(|v| v == "true")
            .unwrap_or(false);
        let highlight_lines = attrs
            .get("highlight")
            .map(|s| parse_line_ranges(s))
            .unwrap_or_default();
        Ok(CodeBlockNode {
            line,
            col,
            language,
            file,
            raw_code: raw,
            line_numbers,
            highlight_lines,
        })
    }

    fn parse_hr(&mut self) -> Result<HRNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'hr'
        self.expect(TokenType::LBrace, "'{' after hr")?;
        self.expect(TokenType::RBrace, "'}' to close hr{}")?;
        Ok(HRNode { line, col })
    }

    fn parse_list(&mut self) -> Result<ListNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'list'
        let (attrs, _) = self.parse_attrs()?;
        self.expect(TokenType::LBrace, "'{' after list")?;

        let mut items = Vec::new();
        while self.cur.token_type != TokenType::RBrace {
            if self.cur.token_type != TokenType::Ident
                || (self.cur.value != "item" && self.cur.value != "task")
            {
                return Err(self.errorf(format!(
                    "expected 'item' or 'task' inside list, got {:?}",
                    self.cur.value
                )));
            }
            let item = if self.cur.value == "task" {
                self.parse_task()?
            } else {
                self.parse_item()?
            };
            items.push(item);
        }
        self.expect(TokenType::RBrace, "'}' to close list")?;
        let ordered = attrs.get("ordered").map(|v| v == "true").unwrap_or(false);
        Ok(ListNode {
            line,
            col,
            ordered,
            items,
        })
    }

    fn parse_item(&mut self) -> Result<ItemNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'item'
        if self.cur.token_type != TokenType::LBrace {
            return Err(self.errorf(format!(
                "expected '{{' after item, got {:?}",
                self.cur.value
            )));
        }
        let children = self.parse_item_body()?;
        Ok(ItemNode {
            line,
            col,
            done: None,
            children,
        })
    }

    fn parse_task(&mut self) -> Result<ItemNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'task'
        let (attrs, _) = self.parse_attrs()?;
        let done = attrs.get("done").map(|v| v == "true").unwrap_or(false);
        if self.cur.token_type != TokenType::LBrace {
            return Err(self.errorf(format!(
                "expected '{{' after task(...), got {:?}",
                self.cur.value
            )));
        }
        let children = self.parse_item_body()?;
        Ok(ItemNode {
            line,
            col,
            done: Some(done),
            children,
        })
    }

    fn parse_item_body(&mut self) -> Result<Vec<ItemChild>, ParseError> {
        let mut children = Vec::new();
        loop {
            let text = collapse_whitespace(&self.lex.read_text_run_until_block(&["list"]));
            if !text.is_empty() {
                children.push(ItemChild::Inline(InlineNode::text(text)));
            }
            if self.lex.at_eof() {
                return Err(ParseError::new(
                    self.lex.line,
                    self.lex.col,
                    "unterminated item, expected '}'",
                ));
            }
            if let Some(ident) = self.lex.at_inline_start() {
                let inline = self.parse_inline_raw(&ident)?;
                children.push(ItemChild::Inline(inline));
                continue;
            }
            if self.lex.at_raw_ident("list") {
                self.depth += 1;
                if self.depth > MAX_INLINE_DEPTH {
                    return Err(ParseError::new(
                        self.lex.line,
                        self.lex.col,
                        format!("lists nested too deeply (limit {MAX_INLINE_DEPTH})"),
                    ));
                }
                let nested_res = self.parse_nested_list();
                self.depth -= 1;
                let nested = nested_res?;
                children.push(ItemChild::List(nested));

                if self.cur.token_type != TokenType::RBrace {
                    return Err(self.errorf(format!(
                        "expected '}}' to close item after nested list, got {:?}",
                        self.cur.value
                    )));
                }
                self.next()?;
                trim_item_children_boundary(&mut children);
                return Ok(children);
            }
            self.lex.consume_rbrace()?;
            trim_item_children_boundary(&mut children);
            self.next()?; // resync token stream past '}'
            return Ok(children);
        }
    }

    fn parse_nested_list(&mut self) -> Result<ListNode, ParseError> {
        self.next()?; // self.cur becomes the 'list' token
        self.parse_list()
    }

    fn parse_quote(&mut self) -> Result<QuoteNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'quote'
        if self.cur.token_type != TokenType::LBrace {
            return Err(self.errorf(format!(
                "expected '{{' after quote, got {:?}",
                self.cur.value
            )));
        }
        let children = self.parse_quote_body()?;
        Ok(QuoteNode {
            line,
            col,
            children,
        })
    }

    fn parse_quote_body(&mut self) -> Result<Vec<QuoteChild>, ParseError> {
        let mut children = Vec::new();
        loop {
            let text = collapse_whitespace(&self.lex.read_text_run_until_block(&["quote"]));
            if !text.is_empty() {
                children.push(QuoteChild::Inline(InlineNode::text(text)));
            }
            if self.lex.at_eof() {
                return Err(ParseError::new(
                    self.lex.line,
                    self.lex.col,
                    "unterminated quote, expected '}'",
                ));
            }
            if let Some(ident) = self.lex.at_inline_start() {
                let inline = self.parse_inline_raw(&ident)?;
                children.push(QuoteChild::Inline(inline));
                continue;
            }
            if self.lex.at_raw_ident("quote") {
                self.depth += 1;
                if self.depth > MAX_INLINE_DEPTH {
                    return Err(ParseError::new(
                        self.lex.line,
                        self.lex.col,
                        format!("quotes nested too deeply (limit {MAX_INLINE_DEPTH})"),
                    ));
                }
                if let Err(e) = self.next() {
                    self.depth -= 1;
                    return Err(e);
                }
                let nested_res = self.parse_quote();
                self.depth -= 1;
                let nested = nested_res?;
                children.push(QuoteChild::Quote(nested));

                if self.cur.token_type != TokenType::RBrace {
                    return Err(self.errorf(format!(
                        "expected '}}' to close quote after nested quote, got {:?}",
                        self.cur.value
                    )));
                }
                self.next()?;
                trim_quote_children_boundary(&mut children);
                return Ok(children);
            }
            self.lex.consume_rbrace()?;
            trim_quote_children_boundary(&mut children);
            self.next()?; // resync token stream past '}'
            return Ok(children);
        }
    }

    fn parse_image(&mut self) -> Result<ImageNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'image'
        if self.cur.token_type != TokenType::LParen {
            return Err(self.errorf(format!(
                "expected '(' after image, got {:?}",
                self.cur.value
            )));
        }
        self.next()?; // consume '('
        let src_tok = self.expect(TokenType::String, "image source URL")?;
        let mut alt = String::new();
        while self.cur.token_type == TokenType::Comma {
            self.next()?;
            let key = self.expect(TokenType::Ident, "attribute name")?;
            self.expect(TokenType::Colon, "':'")?;
            let val = self.expect(TokenType::String, "attribute value")?;
            if key.value == "alt" {
                alt = val.value;
            }
        }
        self.expect(TokenType::RParen, "')'")?;
        Ok(ImageNode {
            line,
            col,
            src: src_tok.value,
            alt,
        })
    }

    fn parse_table(&mut self) -> Result<TableNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'table'
        let _ = self.parse_attrs()?;
        self.expect(TokenType::LBrace, "'{' after table")?;

        let mut rows = Vec::new();
        while self.cur.token_type != TokenType::RBrace {
            if self.cur.token_type != TokenType::Ident || self.cur.value != "row" {
                return Err(self.errorf(format!(
                    "expected 'row' inside table, got {:?}",
                    self.cur.value
                )));
            }
            let row = self.parse_row()?;
            rows.push(row);
        }
        self.expect(TokenType::RBrace, "'}' to close table")?;
        if rows.is_empty() {
            return Err(ParseError::new(
                line,
                col,
                "table must contain at least one row",
            ));
        }
        Ok(TableNode { line, col, rows })
    }

    fn parse_row(&mut self) -> Result<RowNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'row'
        let (attrs, _) = self.parse_attrs()?;
        self.expect(TokenType::LBrace, "'{' after row")?;

        let mut cells = Vec::new();
        while self.cur.token_type != TokenType::RBrace {
            if self.cur.token_type != TokenType::Ident || self.cur.value != "cell" {
                return Err(self.errorf(format!(
                    "expected 'cell' inside row, got {:?}",
                    self.cur.value
                )));
            }
            let cell = self.parse_cell()?;
            cells.push(cell);
        }
        self.expect(TokenType::RBrace, "'}' to close row")?;
        let header = attrs.get("header").map(|v| v == "true").unwrap_or(false);
        Ok(RowNode {
            line,
            col,
            header,
            cells,
        })
    }

    fn parse_cell(&mut self) -> Result<CellNode, ParseError> {
        let line = self.cur.line;
        let col = self.cur.col;
        self.next()?; // consume 'cell'
        if self.cur.token_type != TokenType::LBrace {
            return Err(self.errorf(format!(
                "expected '{{' after cell, got {:?}",
                self.cur.value
            )));
        }
        let children = self.parse_prose_block()?;
        Ok(CellNode {
            line,
            col,
            children,
        })
    }
}

fn parse_line_ranges(s: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if let Some((start_s, end_s)) = part.split_once('-') {
            if let (Ok(start), Ok(end)) = (
                start_s.trim().parse::<usize>(),
                end_s.trim().parse::<usize>(),
            ) {
                for l in start..=end {
                    lines.push(l);
                }
            }
        } else if let Ok(num) = part.parse::<usize>() {
            lines.push(num);
        }
    }
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn trim_item_children_boundary(children: &mut [ItemChild]) {
    if let Some(ItemChild::Inline(first)) = children.first_mut() {
        first.trim_start_space();
    }
    if let Some(ItemChild::Inline(last)) = children.last_mut() {
        last.trim_end_space();
    }
}

fn trim_quote_children_boundary(children: &mut [QuoteChild]) {
    if let Some(QuoteChild::Inline(first)) = children.first_mut() {
        first.trim_start_space();
    }
    if let Some(QuoteChild::Inline(last)) = children.last_mut() {
        last.trim_end_space();
    }
}

#[inline]
fn is_space(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\r' || c == b'\n'
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    let mut prev_space = false;
    for b in s.bytes() {
        if is_space(b) {
            if !prev_space {
                out.push(b' ');
            }
            prev_space = true;
        } else {
            prev_space = false;
            out.push(b);
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
