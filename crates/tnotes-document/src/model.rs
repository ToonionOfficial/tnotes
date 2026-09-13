use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Document {
    #[serde(default = "default_version")]
    pub version: u32,
    pub blocks: Vec<Block>,
}

fn default_version() -> u32 {
    1
}

impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            blocks: Vec::new(),
        }
    }
}

impl Document {
    pub fn new(blocks: Vec<Block>) -> Self {
        Self {
            version: 1,
            blocks,
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    #[serde(default = "default_block_id")]
    pub id: String,
    #[serde(flatten)]
    pub kind: BlockKind,
}

fn default_block_id() -> String {
    Ulid::generate().to_string()
}

impl Block {
    pub fn new(kind: BlockKind) -> Self {
        Self {
            id: Ulid::generate().to_string(),
            kind,
        }
    }

    pub fn with_id(id: impl Into<String>, kind: BlockKind) -> Self {
        Self {
            id: id.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BlockKind {
    Paragraph(RichText),
    Heading { level: u8, content: RichText },
    BulletList(Vec<ListItem>),
    OrderedList(Vec<ListItem>),
    TaskList(Vec<TaskItem>),
    Quote(RichText),
    CodeBlock { language: Option<String>, code: String },
    Divider,
    Table(TableData),
    Drawing(DrawingData),
    Image(ImageData),
    Audio(AudioData),
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct RichText {
    pub spans: Vec<Span>,
}

impl RichText {
    pub fn new(spans: Vec<Span>) -> Self {
        Self { spans }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            spans: vec![Span::plain(text)],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty() || self.spans.iter().all(|s| s.text.is_empty())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
    #[serde(default)]
    pub marks: Marks,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

impl Span {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            marks: Marks::default(),
            link: None,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Marks {
    #[serde(default, skip_serializing_if = "is_false")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub underline: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub strike: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub code: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
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

impl ListItem {
    pub fn new(content: RichText) -> Self {
        Self {
            content,
            sub_list: None,
        }
    }

    pub fn with_sub_list(content: RichText, sub_list: SubList) -> Self {
        Self {
            content,
            sub_list: Some(Box::new(sub_list)),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskItem {
    pub checked: bool,
    pub content: RichText,
}

impl TaskItem {
    pub fn new(checked: bool, content: RichText) -> Self {
        Self { checked, content }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct DrawingData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    pub strokes: Vec<Stroke>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Stroke {
    pub color: String,
    pub width: f32,
    pub points: Vec<Point>,
}

impl Eq for Stroke {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImageData {
    pub asset_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AudioData {
    pub asset_id: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waveform: Option<Vec<u8>>,
}
