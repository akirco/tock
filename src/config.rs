use directories::ProjectDirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub font: Option<String>,
    pub bg: Option<String>,
    pub fg: Option<String>,
    pub subtitle_fg: Option<String>,
    pub panel_ratio: Option<u8>,
    pub panel_bg: Option<String>,
    pub panel_fg: Option<String>,
    pub panel_border: Option<String>,
    pub panel_border_sides: Option<String>,
    pub panel_border_style: Option<String>,
    pub alarm_sound: Option<String>,
    pub countdown_sound: Option<String>,
    pub color: Option<ColorConfig>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ColorConfig {
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

pub fn load_config() -> Config {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "clock") {
        let config_file = proj_dirs.config_dir().join("config.toml");
        if config_file.exists()
            && let Ok(contents) = fs::read_to_string(config_file)
                && let Ok(config) = toml::from_str::<Config>(&contents) {
                    return config;
                }
    }
    Config::default()
}
