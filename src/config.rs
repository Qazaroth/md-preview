use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub output_filename: Option<String>,
    pub theme: Option<String>,
    pub save: Option<bool>,
    pub template: Option<String>,
    pub port: Option<u16>,
}

impl Config {
    pub fn load() -> Self {
        find_config_path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }
}

fn find_config_path() -> Option<PathBuf> {
    // Local config takes priority over global
    let local = PathBuf::from(".md-previewer.toml");
    if local.exists() {
        return Some(local);
    }

    dirs::config_dir()
        .map(|p| p.join("md-previewer").join("config.toml"))
        .filter(|p| p.exists())
}
