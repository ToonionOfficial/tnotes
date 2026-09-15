use std::collections::HashMap;
use gpui::*;
use twrite::theme::{EditorTheme, SyntaxTheme};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Theme {
    pub background: Hsla,
    pub foreground: Hsla,
    pub card: Hsla,
    pub card_foreground: Hsla,
    pub popover: Hsla,
    pub popover_foreground: Hsla,
    pub primary: Hsla,
    pub primary_foreground: Hsla,
    pub secondary: Hsla,
    pub secondary_foreground: Hsla,
    pub muted: Hsla,
    pub muted_foreground: Hsla,
    pub accent: Hsla,
    pub accent_foreground: Hsla,
    pub destructive: Hsla,
    pub destructive_foreground: Hsla,
    pub border: Hsla,
    pub input: Hsla,
    pub ring: Hsla,
    pub success: Hsla,
    pub is_dark: bool,
}

#[allow(dead_code)]
impl Theme {
    pub fn dark() -> Self {
        Self {
            background: rgb(0x141318).into(),
            foreground: rgb(0xe6e1e9).into(),
            card: rgb(0x201f24).into(),
            card_foreground: rgb(0xe6e1e9).into(),
            popover: rgb(0x201f24).into(),
            popover_foreground: rgb(0xe6e1e9).into(),
            primary: rgb(0xcabeff).into(),
            primary_foreground: rgb(0x32285f).into(),
            secondary: rgb(0x2a2930).into(),
            secondary_foreground: rgb(0xe6e1e9).into(),
            muted: rgb(0x201f24).into(),
            muted_foreground: rgb(0xc6c2cd).into(),
            accent: rgb(0x2a2930).into(),
            accent_foreground: rgb(0xe6e1e9).into(),
            destructive: rgb(0xffb4ab).into(),
            destructive_foreground: rgb(0x690005).into(),
            border: rgb(0x302e36).into(),
            input: rgb(0x302e36).into(),
            ring: rgb(0xcabeff).into(),
            success: rgb(0xa6e3a1).into(),
            is_dark: true,
        }
    }

    pub fn light() -> Self {
        Self {
            background: rgb(0xf8f7fa).into(),
            foreground: rgb(0x1d1b20).into(),
            card: rgb(0xffffff).into(),
            card_foreground: rgb(0x1d1b20).into(),
            popover: rgb(0xffffff).into(),
            popover_foreground: rgb(0x1d1b20).into(),
            primary: rgb(0x65558f).into(),
            primary_foreground: rgb(0xffffff).into(),
            secondary: rgb(0xece6f0).into(),
            secondary_foreground: rgb(0x1d1b20).into(),
            muted: rgb(0xf3eef8).into(),
            muted_foreground: rgb(0x79747e).into(),
            accent: rgb(0xe8def8).into(),
            accent_foreground: rgb(0x1d1b20).into(),
            destructive: rgb(0xba1a1a).into(),
            destructive_foreground: rgb(0xffffff).into(),
            border: rgb(0xe6e0e9).into(),
            input: rgb(0xe6e0e9).into(),
            ring: rgb(0x65558f).into(),
            success: rgb(0x2e7d32).into(),
            is_dark: false,
        }
    }

    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    pub fn to_twrite_theme(&self) -> EditorTheme {
        EditorTheme {
            background: self.background,
            foreground: self.foreground,
            cursor: self.primary,
            selection: self.accent,
            search_match: self.ring,
            line_number: self.muted_foreground,
            line_number_active: self.foreground,
            menu_bg: self.popover,
            menu_border: self.border,
            menu_hover: self.secondary,
            menu_fg: self.popover_foreground,
            menu_hint: self.muted_foreground,
            syntax: SyntaxTheme {
                keyword: self.primary,
                function: self.primary,
                type_name: self.ring,
                string: self.success,
                number: self.primary,
                comment: self.muted_foreground,
                operator: self.muted_foreground,
                punctuation: self.muted_foreground,
                heading1: self.primary,
                heading2: self.primary,
                heading3: self.primary,
                bold: self.foreground,
                italic: self.muted_foreground,
                code: self.primary,
                code_bg: self.secondary,
                link: self.primary,
                custom: HashMap::new(),
                error: self.destructive,
                warning: rgb(0xfbbf24).into(),
            },
        }
    }
}

pub struct ActiveTheme(pub Theme);

impl Global for ActiveTheme {}

pub trait ThemeExt {
    fn theme(&self) -> &Theme;
}

impl ThemeExt for App {
    fn theme(&self) -> &Theme {
        &self.global::<ActiveTheme>().0
    }
}
