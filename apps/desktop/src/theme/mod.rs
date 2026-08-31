use gpui::{Rgba, rgb};

/// Design tokens matching `apps/mobile/global.css`
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Theme {
    /// `--background`: #141318
    pub background: Rgba,
    /// `--foreground`: #e6e1e9
    pub foreground: Rgba,
    /// `--card`: #201f24
    pub card: Rgba,
    /// `--card-foreground`: #e6e1e9
    pub card_foreground: Rgba,
    /// `--primary`: #cabeff
    pub primary: Rgba,
    /// `--primary-foreground`: #32285f
    pub primary_foreground: Rgba,
    /// `--secondary` / `--accent`: #2a2930
    pub secondary: Rgba,
    /// `--muted-foreground`: #c6c2cd
    pub muted_foreground: Rgba,
    /// Subdued text: #938f99
    pub subtle_foreground: Rgba,
    /// `--border` / `--input`: #302e36
    pub border: Rgba,
    /// `--success`: #a6e3a1
    pub success: Rgba,
    /// `--destructive`: #ffb4ab / #ff6b6b
    pub destructive: Rgba,
    /// Star / favorite gold: #ffc107
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
