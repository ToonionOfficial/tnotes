use tl::{HTMLTag, Node, NodeHandle, VDom};

use crate::ast::{Block, Document, ListItem, Marks, RichText, Span, TaskItem};
use crate::error::ParseError;

#[derive(Debug, Default, Clone)]
pub struct Parser {
    pub strip_empty_blocks: bool,
    pub strict_mode: bool,
}

impl Parser {
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
        dom: &VDom<'_>,
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

    fn parse_block(&self, tag: &HTMLTag, dom: &VDom) -> Result<Option<Block>, ParseError> {
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

    fn parse_rich_text(&self, root_tag: &HTMLTag, dom: &VDom) -> RichText {
        let mut spans = Vec::new();
        self.collect_spans(root_tag, dom, Marks::default(), None, &mut spans);
        RichText {
            spans: self.normalize_spans(spans),
        }
    }

    fn collect_spans(
        &self,
        tag: &HTMLTag,
        dom: &VDom,
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

            if let Some(last) = normalized.last_mut()
                && last.marks == span.marks
                && last.link == span.link
            {
                last.text.push_str(&span.text);
                continue;
            }

            normalized.push(span);
        }

        normalized
    }

    fn parse_code_block(&self, pre_tag: &HTMLTag, dom: &VDom) -> (Option<String>, String) {
        let parser = dom.parser();
        let mut language = None;

        for child_handle in pre_tag.children().top().iter() {
            if let Some(Node::Tag(code_tag)) = child_handle.get(parser)
                && code_tag.name().as_utf8_str() == "code"
            {
                if let Some(class_attr) = code_tag.attributes().get("class").flatten() {
                    let class_str = class_attr.as_utf8_str();
                    for cls in class_str.split_whitespace() {
                        if let Some(lang) = cls.strip_prefix("language-") {
                            language = Some(lang.to_string());
                            break;
                        }
                    }
                }
                return (language, code_tag.inner_text(parser).to_string());
            }
        }

        (None, pre_tag.inner_text(parser).to_string())
    }

    fn parse_list_items(
        &self,
        list_tag: &HTMLTag,
        dom: &VDom,
    ) -> Result<Vec<ListItem>, ParseError> {
        let mut items = Vec::new();
        let parser = dom.parser();

        for child_handle in list_tag.children().top().iter() {
            if let Some(Node::Tag(li_tag)) = child_handle.get(parser)
                && li_tag.name().as_utf8_str() == "li"
            {
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

        Ok(items)
    }

    fn parse_task_items(
        &self,
        list_tag: &HTMLTag,
        dom: &VDom,
    ) -> Result<Vec<TaskItem>, ParseError> {
        let mut items = Vec::new();
        let parser = dom.parser();

        for child_handle in list_tag.children().top().iter() {
            if let Some(Node::Tag(li_tag)) = child_handle.get(parser)
                && li_tag.name().as_utf8_str() == "li"
            {
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

        Ok(items)
    }
}
