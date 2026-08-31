# TNotes Document Parser & Serializer (`tnotes-document`)

This crate defines the **Document AST** and provides bidirectional translation between the HTML produced/consumed by TNotes clients (such as TipTap on mobile/web) and the native `Document` model used by the GPUI desktop editor.

---

## 1. Architecture Overview

```text
       +-------------------------------------------------------+
       |                  Server / SQLite DB                   |
       |             (stores note.body as HTML string)         |
       +-------------------------------------------------------+
                ▲                                       ▲
  Syncs raw HTML|                           Syncs raw HTML
                ▼                                       ▼
+-------------------------------+       +-------------------------------+
|         Mobile / Web          |       |         Desktop (GPUI)        |
|  TipTap / TenTap / WebView    |       |   tnotes-document translator  |
|  (Native DOM / HTML input)    |       |                               |
+-------------------------------+       |   HTML -> Document AST        |
                                        |   (GPUI renders AST blocks)   |
                                        |   Document AST -> HTML        |
                                        +-------------------------------+
```

The HTML parser is **not** a general web scraper; it is a **strict format translator** defining the contract between TNotes storage (HTML) and native desktop views.

---

## 2. Supported HTML Elements & AST Mapping

### Block Elements

| HTML Representation | AST Enum Variant | Description |
| :--- | :--- | :--- |
| `<p>...</p>` | `Block::Paragraph(RichText)` | Standard body paragraph |
| `<h1>...</h1>` | `Block::Heading { level: 1, content }` | Title heading |
| `<h2>...</h2>` | `Block::Heading { level: 2, content }` | Heading |
| `<h3>...</h3>` | `Block::Heading { level: 3, content }` | Subheading |
| `<blockquote>...</blockquote>` | `Block::Quote(RichText)` | Blockquote |
| `<pre><code class="language-xyz">...</code></pre>` | `Block::CodeBlock { language, code }` | Code block with optional language |
| `<hr>` | `Block::Divider` | Horizontal rule divider |

### Lists & Checklists

| HTML Representation | AST Enum Variant | Description |
| :--- | :--- | :--- |
| `<ul><li>...</li></ul>` | `Block::BulletList(Vec<ListItem>)` | Bulleted list (supports nesting) |
| `<ol><li>...</li></ol>` | `Block::OrderedList(Vec<ListItem>)` | Numbered list |
| `<ul data-type="taskList"><li data-type="taskItem" data-checked="true\|false">...</li></ul>` | `Block::TaskList(Vec<TaskItem>)` | Interactive checklist item |

### Inline Formatting (Marks)

Marks are boolean flags combined inside a single `Span` to avoid deeply nested trees for inline styles:

| HTML Tag | Mark Field | Description |
| :--- | :--- | :--- |
| `<strong>`, `<b>` | `marks.bold = true` | Bold text |
| `<em>`, `<i>` | `marks.italic = true` | Italic text |
| `<u>` | `marks.underline = true` | Underlined text |
| `<s>`, `<del>`, `<strike>` | `marks.strike = true` | Strikethrough text |
| `<code>` | `marks.code = true` | Inline code chip |
| `<a href="...">` | `link = Some(href)` | Clickable link URL |
| `<br>` | `Span { text: "\n", .. }` | Hard line break |

---

## 3. AST Definitions (`src/ast.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Block {
    Paragraph(RichText),
    Heading {
        level: u8,
        content: RichText,
    },
    BulletList(Vec<ListItem>),
    OrderedList(Vec<ListItem>),
    TaskList(Vec<TaskItem>),
    Quote(RichText),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Divider,
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct RichText {
    pub spans: Vec<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
    pub marks: Marks,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Marks {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub code: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    pub content: RichText,
    pub children: Vec<ListItem>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskItem {
    pub checked: bool,
    pub content: RichText,
}
```

---

## 4. Error Handling (`src/error.rs`)

```rust
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("Failed to parse HTML structure")]
    InvalidHtml,

    #[error("Unsupported block element: <{0}>")]
    UnsupportedBlock(String),

    #[error("Malformed block attributes in <{tag}>: {reason}")]
    MalformedAttributes { tag: String, reason: String },
}
```

---

## 5. Parser Implementation (`src/parser.rs`)

Key principles:
1. **Parser Engine**: Uses `tl` for fast, lightweight HTML DOM traversal.
2. **Transparent Wrappers**: Containers like `<div>`, `<article>`, and `<section>` are automatically unwrapped to parse their inner blocks.
3. **No Silent Data Loss**: Unknown block tags (like `<table>`) are rejected in strict mode rather than being silently converted to paragraphs.
4. **Span Normalization**: Merges adjacent spans with identical formatting (e.g. `<strong>Hello</strong><strong> world</strong>` $\to$ `Span("Hello world", bold)`).

```rust
use tl::{HTMLTag, Node, NodeHandle, VDomGuard};

use crate::ast::{Block, Document, ListItem, Marks, RichText, Span, TaskItem};
use crate::error::ParseError;

#[derive(Debug, Default, Clone)]
pub struct HtmlParser {
    pub strip_empty_blocks: bool,
    pub strict_mode: bool,
}

impl HtmlParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    pub fn parse(&self, html: &str) -> Result<Document, ParseError> {
        let trimmed = html.trim();
        if trimmed.is_empty() {
            return Ok(Document { blocks: Vec::new() });
        }

        let dom = tl::parse(trimmed, tl::ParserOptions::default())
            .map_err(|_| ParseError::InvalidHtml)?;

        let mut blocks = Vec::new();
        for handle in dom.children() {
            self.parse_top_level_handle(handle, &dom, &mut blocks)?;
        }

        Ok(Document { blocks })
    }

    fn parse_top_level_handle(
        &self,
        handle: &NodeHandle,
        dom: &VDomGuard,
        blocks: &mut Vec<Block>,
    ) -> Result<(), ParseError> {
        let node = handle.get(dom.parser()).ok_or(ParseError::InvalidHtml)?;

        match node {
            Node::Tag(tag) => {
                let tag_name = tag.name().as_utf8_str();
                match tag_name.as_ref() {
                    "div" | "article" | "section" => {
                        for child in tag.children().top().iter() {
                            self.parse_top_level_handle(child, dom, blocks)?;
                        }
                    }
                    _ => {
                        if let Some(block) = self.parse_block(tag, dom)? {
                            blocks.push(block);
                        }
                    }
                }
            }
            Node::Raw(bytes) => {
                let text = bytes.as_utf8_str().trim().to_string();
                if !text.is_empty() {
                    blocks.push(Block::Paragraph(RichText {
                        spans: vec![Span {
                            text,
                            marks: Marks::default(),
                            link: None,
                        }],
                    }));
                }
            }
            Node::Comment(_) => {}
        }

        Ok(())
    }

    fn parse_block(&self, tag: &HTMLTag, dom: &VDomGuard) -> Result<Option<Block>, ParseError> {
        let tag_name = tag.name().as_utf8_str();

        match tag_name.as_ref() {
            "p" => {
                let content = self.parse_rich_text(tag, dom);
                if self.strip_empty_blocks && content.spans.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(Block::Paragraph(content)))
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag_name[1..].parse::<u8>().unwrap_or(1);
                let content = self.parse_rich_text(tag, dom);
                Ok(Some(Block::Heading { level, content }))
            }
            "blockquote" => {
                let content = self.parse_rich_text(tag, dom);
                Ok(Some(Block::Quote(content)))
            }
            "pre" => {
                let (language, code) = self.parse_code_block(tag, dom);
                Ok(Some(Block::CodeBlock { language, code }))
            }
            "hr" => Ok(Some(Block::Divider)),
            "ul" => {
                let is_task_list = tag
                    .attributes()
                    .get("data-type")
                    .flatten()
                    .map(|v| v.as_utf8_str() == "taskList")
                    .unwrap_or(false);

                if is_task_list {
                    Ok(Some(Block::TaskList(self.parse_task_items(tag, dom)?)))
                } else {
                    Ok(Some(Block::BulletList(self.parse_list_items(tag, dom)?)))
                }
            }
            "ol" => Ok(Some(Block::OrderedList(self.parse_list_items(tag, dom)?))),
            unsupported => {
                if self.strict_mode {
                    Err(ParseError::UnsupportedBlock(unsupported.to_string()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn parse_rich_text(&self, root_tag: &HTMLTag, dom: &VDomGuard) -> RichText {
        let mut spans = Vec::new();
        self.collect_spans(root_tag, dom, Marks::default(), None, &mut spans);
        RichText {
            spans: self.normalize_spans(spans),
        }
    }

    fn collect_spans(
        &self,
        tag: &HTMLTag,
        dom: &VDomGuard,
        mut current_marks: Marks,
        mut current_link: Option<String>,
        spans: &mut Vec<Span>,
    ) {
        let name = tag.name().as_utf8_str();
        match name.as_ref() {
            "strong" | "b" => current_marks.bold = true,
            "em" | "i" => current_marks.italic = true,
            "u" => current_marks.underline = true,
            "s" | "del" | "strike" => current_marks.strike = true,
            "code" => current_marks.code = true,
            "a" => {
                if let Some(href) = tag.attributes().get("href").flatten() {
                    current_link = Some(href.as_utf8_str().to_string());
                }
            }
            _ => {}
        }

        for child_handle in tag.children().top().iter() {
            if let Some(child_node) = child_handle.get(dom.parser()) {
                match child_node {
                    Node::Tag(child_tag) => {
                        if child_tag.name().as_utf8_str() == "br" {
                            spans.push(Span {
                                text: "\n".to_string(),
                                marks: current_marks,
                                link: None,
                            });
                        } else {
                            self.collect_spans(
                                child_tag,
                                dom,
                                current_marks,
                                current_link.clone(),
                                spans,
                            );
                        }
                    }
                    Node::Raw(bytes) => {
                        let text = bytes.as_utf8_str().to_string();
                        if !text.is_empty() {
                            spans.push(Span {
                                text,
                                marks: current_marks,
                                link: current_link.clone(),
                            });
                        }
                    }
                    Node::Comment(_) => {}
                }
            }
        }
    }

    fn normalize_spans(&self, raw_spans: Vec<Span>) -> Vec<Span> {
        let mut normalized: Vec<Span> = Vec::with_capacity(raw_spans.len());

        for span in raw_spans {
            if span.text.is_empty() {
                continue;
            }

            if let Some(last) = normalized.last_mut() {
                if last.marks == span.marks && last.link == span.link {
                    last.text.push_str(&span.text);
                    continue;
                }
            }

            normalized.push(span);
        }

        normalized
    }

    fn parse_code_block(&self, pre_tag: &HTMLTag, dom: &VDomGuard) -> (Option<String>, String) {
        let parser = dom.parser();
        let mut code_content = String::new();
        let mut language = None;

        for child_handle in pre_tag.children().top().iter() {
            if let Some(Node::Tag(code_tag)) = child_handle.get(parser) {
                if code_tag.name().as_utf8_str() == "code" {
                    if let Some(class_attr) = code_tag.attributes().get("class").flatten() {
                        let class_str = class_attr.as_utf8_str();
                        for cls in class_str.split_whitespace() {
                            if let Some(lang) = cls.strip_prefix("language-") {
                                language = Some(lang.to_string());
                                break;
                            }
                        }
                    }
                    code_content = code_tag.inner_text(parser).to_string();
                    return (language, code_content);
                }
            }
        }

        (None, pre_tag.inner_text(parser).to_string())
    }

    fn parse_list_items(
        &self,
        list_tag: &HTMLTag,
        dom: &VDomGuard,
    ) -> Result<Vec<ListItem>, ParseError> {
        let mut items = Vec::new();
        let parser = dom.parser();

        for child_handle in list_tag.children().top().iter() {
            if let Some(Node::Tag(li_tag)) = child_handle.get(parser) {
                if li_tag.name().as_utf8_str() == "li" {
                    let content = self.parse_rich_text(li_tag, dom);
                    let mut children = Vec::new();

                    for sub_handle in li_tag.children().top().iter() {
                        if let Some(Node::Tag(sub_tag)) = sub_handle.get(parser) {
                            match sub_tag.name().as_utf8_str().as_ref() {
                                "ul" | "ol" => {
                                    children.extend(self.parse_list_items(sub_tag, dom)?);
                                }
                                _ => {}
                            }
                        }
                    }

                    items.push(ListItem { content, children });
                }
            }
        }

        Ok(items)
    }

    fn parse_task_items(
        &self,
        list_tag: &HTMLTag,
        dom: &VDomGuard,
    ) -> Result<Vec<TaskItem>, ParseError> {
        let mut items = Vec::new();
        let parser = dom.parser();

        for child_handle in list_tag.children().top().iter() {
            if let Some(Node::Tag(li_tag)) = child_handle.get(parser) {
                if li_tag.name().as_utf8_str() == "li" {
                    let checked = li_tag
                        .attributes()
                        .get("data-checked")
                        .flatten()
                        .map(|v| v.as_utf8_str() == "true")
                        .unwrap_or(false);

                    let content = self.parse_rich_text(li_tag, dom);
                    items.push(TaskItem { checked, content });
                }
            }
        }

        Ok(items)
    }
}
```

---

## 6. Serializer Implementation (`src/serializer.rs`)

```rust
use std::fmt::Write;
use crate::ast::{Block, Document, ListItem, RichText, Span, TaskItem};

pub struct HtmlSerializer;

impl HtmlSerializer {
    pub fn serialize(doc: &Document) -> String {
        let mut out = String::new();
        for block in &doc.blocks {
            Self::serialize_block(block, &mut out);
        }
        out
    }

    fn serialize_block(block: &Block, out: &mut String) {
        match block {
            Block::Paragraph(rt) => {
                out.push_str("<p>");
                Self::serialize_rich_text(rt, out);
                out.push_str("</p>");
            }
            Block::Heading { level, content } => {
                let _ = write!(out, "<h{level}>");
                Self::serialize_rich_text(content, out);
                let _ = write!(out, "</h{level}>");
            }
            Block::Quote(rt) => {
                out.push_str("<blockquote>");
                Self::serialize_rich_text(rt, out);
                out.push_str("</blockquote>");
            }
            Block::CodeBlock { language, code } => {
                out.push_str("<pre>");
                if let Some(lang) = language {
                    let _ = write!(out, "<code class=\"language-{lang}\">");
                } else {
                    out.push_str("<code>");
                }
                Self::escape_html(code, out);
                out.push_str("</code></pre>");
            }
            Block::Divider => {
                out.push_str("<hr>");
            }
            Block::BulletList(items) => {
                out.push_str("<ul>");
                for item in items {
                    Self::serialize_list_item(item, out);
                }
                out.push_str("</ul>");
            }
            Block::OrderedList(items) => {
                out.push_str("<ol>");
                for item in items {
                    Self::serialize_list_item(item, out);
                }
                out.push_str("</ol>");
            }
            Block::TaskList(items) => {
                out.push_str("<ul data-type=\"taskList\">");
                for item in items {
                    let checked_str = if item.checked { "true" } else { "false" };
                    let _ = write!(out, "<li data-type=\"taskItem\" data-checked=\"{checked_str}\">");
                    Self::serialize_rich_text(&item.content, out);
                    out.push_str("</li>");
                }
                out.push_str("</ul>");
            }
        }
    }

    fn serialize_list_item(item: &ListItem, out: &mut String) {
        out.push_str("<li>");
        Self::serialize_rich_text(&item.content, out);
        if !item.children.is_empty() {
            out.push_str("<ul>");
            for child in &item.children {
                Self::serialize_list_item(child, out);
            }
            out.push_str("</ul>");
        }
        out.push_str("</li>");
    }

    fn serialize_rich_text(rt: &RichText, out: &mut String) {
        for span in &rt.spans {
            Self::serialize_span(span, out);
        }
    }

    fn serialize_span(span: &Span, out: &mut String) {
        let m = span.marks;

        if let Some(link) = &span.link {
            let _ = write!(out, "<a href=\"{link}\">");
        }
        if m.bold { out.push_str("<strong>"); }
        if m.italic { out.push_str("<em>"); }
        if m.underline { out.push_str("<u>"); }
        if m.strike { out.push_str("<s>"); }
        if m.code { out.push_str("<code>"); }

        Self::escape_html(&span.text, out);

        if m.code { out.push_str("</code>"); }
        if m.strike { out.push_str("</s>"); }
        if m.underline { out.push_str("</u>"); }
        if m.italic { out.push_str("</em>"); }
        if m.bold { out.push_str("</strong>"); }
        if span.link.is_some() { out.push_str("</a>"); }
    }

    fn escape_html(text: &str, out: &mut String) {
        for c in text.chars() {
            match c {
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '&' => out.push_str("&amp;"),
                '"' => out.push_str("&quot;"),
                _ => out.push(c),
            }
        }
    }
}
```

---

## 7. Recommended Test Fixtures Directory

Store real TipTap HTML samples in `tests/fixtures/`:

```text
crates/tnotes-document/tests/
  fixtures/
    paragraph.html
    headings.html
    formatting.html
    lists.html
    task_list.html
    code_block.html
    nested_formatting.html
  roundtrip_tests.rs
```

Example fixture test:

```rust
#[test]
fn test_roundtrip_fixtures() {
    let input_html = include_str!("fixtures/task_list.html");
    let doc = Document::from_html(input_html).expect("Must parse valid fixture");
    let output_html = doc.to_html();
    assert_eq!(input_html.trim(), output_html.trim());
}
```
