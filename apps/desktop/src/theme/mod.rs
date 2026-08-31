use gpui::{Rgba, rgb};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Rgba,
    pub foreground: Rgba,
    pub card: Rgba,
    pub card_foreground: Rgba,
    pub primary: Rgba,
    pub primary_foreground: Rgba,
    pub secondary: Rgba,
    pub muted_foreground: Rgba,
    pub subtle_foreground: Rgba,
    pub border: Rgba,
    pub success: Rgba,
    pub destructive: Rgba,
    pub favorite: Rgba,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: rgb(0x141318),
            foreground: rgb(0xe6e1e9),
            card: rgb(0x201f24),
            card_foreground: rgb(0xe6e1e9),
            primary: rgb(0xcabeff),
            primary_foreground: rgb(0x32285f),
            secondary: rgb(0x2a2930),
            muted_foreground: rgb(0xc6c2cd),
            subtle_foreground: rgb(0x938f99),
            border: rgb(0x302e36),
            success: rgb(0xa6e3a1),
            destructive: rgb(0xffb4ab),
            favorite: rgb(0xffc107),
        }
    }
}
