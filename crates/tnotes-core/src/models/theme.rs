use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::current_time_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub bg: String,
    pub fg: String,
    pub accent: String,
    pub surface: String,
    pub sidebar_bg: String,
    pub editor_bg: String,
    pub heading: String,
    pub code_bg: String,
    pub code_fg: String,
    pub border: String,
    pub muted: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub success: Option<String>,
    #[serde(default)]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub heading_font_family: String,
    pub code_font_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub sidebar_width: f32,
    pub note_list_width: f32,
    pub content_padding: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSchema {
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    pub name: String,
    pub builtin: bool,
    pub schema: ThemeSchema,
    pub version: u64,
    pub updated_at: i64,
    pub created_at: i64,
    pub device_id: String,
}

impl Theme {
    pub fn new(
        name: impl Into<String>,
        schema: ThemeSchema,
        device_id: impl Into<String>,
        user_id: Option<String>,
    ) -> Self {
        let now = current_time_ms();
        Self {
            id: Ulid::generate().to_string(),
            user_id,
            name: name.into(),
            builtin: false,
            schema,
            version: 1,
            created_at: now,
            updated_at: now,
            device_id: device_id.into(),
        }
    }

    /// Built-in Dark Theme (Nord inspired)
    pub fn builtin_dark() -> Self {
        Self {
            id: "theme_builtin_dark".to_string(),
            user_id: None,
            name: "Default Dark".to_string(),
            builtin: true,
            schema: ThemeSchema {
                colors: ThemeColors {
                    bg: "#1e1e2e".to_string(),
                    fg: "#cdd6f4".to_string(),
                    accent: "#89b4fa".to_string(),
                    surface: "#313244".to_string(),
                    sidebar_bg: "#181825".to_string(),
                    editor_bg: "#1e1e2e".to_string(),
                    heading: "#89dceb".to_string(),
                    code_bg: "#11111b".to_string(),
                    code_fg: "#a6e3a1".to_string(),
                    border: "#45475a".to_string(),
                    muted: "#6c7086".to_string(),
                    error: Some("#f38ba8".to_string()),
                    success: Some("#a6e3a1".to_string()),
                    warning: Some("#f9e2af".to_string()),
                },
                typography: ThemeTypography {
                    font_family: "Inter, system-ui, sans-serif".to_string(),
                    font_size: 15.0,
                    line_height: 1.6,
                    heading_font_family: "Inter, system-ui, sans-serif".to_string(),
                    code_font_family: "JetBrains Mono, monospace".to_string(),
                },
                spacing: ThemeSpacing {
                    sidebar_width: 240.0,
                    note_list_width: 300.0,
                    content_padding: 24.0,
                },
            },
            version: 1,
            created_at: 0,
            updated_at: 0,
            device_id: "system".to_string(),
        }
    }

    /// Built-in Light Theme
    pub fn builtin_light() -> Self {
        Self {
            id: "theme_builtin_light".to_string(),
            user_id: None,
            name: "Default Light".to_string(),
            builtin: true,
            schema: ThemeSchema {
                colors: ThemeColors {
                    bg: "#ffffff".to_string(),
                    fg: "#1e1e2e".to_string(),
                    accent: "#1e66f5".to_string(),
                    surface: "#f2f4f8".to_string(),
                    sidebar_bg: "#f8f9fa".to_string(),
                    editor_bg: "#ffffff".to_string(),
                    heading: "#04a5e5".to_string(),
                    code_bg: "#e6e9ef".to_string(),
                    code_fg: "#40a02b".to_string(),
                    border: "#ccd0da".to_string(),
                    muted: "#8c8fa1".to_string(),
                    error: Some("#d20f39".to_string()),
                    success: Some("#40a02b".to_string()),
                    warning: Some("#df8e1d".to_string()),
                },
                typography: ThemeTypography {
                    font_family: "Inter, system-ui, sans-serif".to_string(),
                    font_size: 15.0,
                    line_height: 1.6,
                    heading_font_family: "Inter, system-ui, sans-serif".to_string(),
                    code_font_family: "JetBrains Mono, monospace".to_string(),
                },
                spacing: ThemeSpacing {
                    sidebar_width: 240.0,
                    note_list_width: 300.0,
                    content_padding: 24.0,
                },
            },
            version: 1,
            created_at: 0,
            updated_at: 0,
            device_id: "system".to_string(),
        }
    }
}
