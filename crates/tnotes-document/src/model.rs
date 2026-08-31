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

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Marks {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub code: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SubList {
    Bullet(Vec<ListItem>),
    Ordered(Vec<ListItem>),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    pub content: RichText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_list: Option<Box<SubList>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskItem {
    pub checked: bool,
    pub content: RichText,
}
