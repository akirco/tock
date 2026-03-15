use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub stopwatch: Option<bool>,
    pub time: Option<String>,
    pub font: Option<String>,
    pub bg_color: Option<String>,
    pub clock_color: Option<String>,
    pub panel_ratio: Option<u8>,
    pub panel_bg: Option<String>,
    pub panel_fg: Option<String>,
    pub panel_border: Option<String>,
    pub panel_border_sides: Option<String>,
    pub panel_border_style: Option<String>,
}

pub fn load_config() -> Config {
    // Automatically match the OS-specific configuration directory:
    // Linux: ~/.config/clock/config.toml
    // macOS: ~/Library/Application Support/clock/config.toml
    // Windows: C:\Users\Username\AppData\Roaming\clock\config\config.toml
    if let Some(proj_dirs) = ProjectDirs::from("", "", "clock") {
        let config_file = proj_dirs.config_dir().join("config.toml");
        if config_file.exists()
            && let Ok(contents) = fs::read_to_string(config_file)
                && let Ok(config) = toml::from_str::<Config>(&contents) {
                    return config;
                }
    }
    Config::default() // Return default empty config if file doesn't exist or parsing fails
}