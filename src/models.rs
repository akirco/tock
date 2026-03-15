use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub time: String,
    pub enabled: bool,
    pub repeat: String,
    pub note: String,
}

impl Default for Alarm {
    fn default() -> Self {
        Self {
            time: "06:30".to_string(),
            enabled: false,
            repeat: "daily".to_string(),
            note: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountdownPreset {
    pub name: String,
    pub duration: u64,
}

impl Default for CountdownPreset {
    fn default() -> Self {
        Self {
            name: "Brush teeth".to_string(),
            duration: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopwatchRecord {
    pub lap: u32,
    pub time_millis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppData {
    pub alarms: Vec<Alarm>,
    pub presets: Vec<CountdownPreset>,
    pub history: Vec<StopwatchRecord>,
}
