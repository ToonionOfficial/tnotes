use std::fs;
use std::io::Result;
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("tnotes")
}

pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("tnotes")
}

#[allow(dead_code)]
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("tnotes")
}

pub fn keymap_file() -> PathBuf {
    config_dir().join("keymap.json")
}

#[allow(dead_code)]
pub fn database_file() -> PathBuf {
    data_dir().join("tnotes.db")
}

pub fn ensure_dirs() -> Result<()> {
    fs::create_dir_all(config_dir())?;
    fs::create_dir_all(data_dir())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_end_with_tnotes() {
        assert!(config_dir().ends_with("tnotes"));
        assert!(data_dir().ends_with("tnotes"));
        assert_eq!(
            keymap_file().file_name().unwrap().to_str().unwrap(),
            "keymap.json"
        );
        assert_eq!(
            database_file().file_name().unwrap().to_str().unwrap(),
            "tnotes.db"
        );
    }
}
