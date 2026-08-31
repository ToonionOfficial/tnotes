use crate::keymap::schema::KeymapConfig;

pub fn load_default_keymap() -> KeymapConfig {
    #[cfg(target_os = "macos")]
    const DEFAULT_JSON: &str = include_str!("../../assets/keymaps/default-macos.json");

    #[cfg(not(target_os = "macos"))]
    const DEFAULT_JSON: &str = include_str!("../../assets/keymaps/default-linux.json");

    serde_json::from_str(DEFAULT_JSON).unwrap_or_else(|err| {
        eprintln!("Failed to parse embedded default keymap: {err}");
        Vec::new()
    })
}
