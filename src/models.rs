use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Repeat {
    #[default]
    Daily,
    Weekday,
    Weekend,
    Once,
}

impl Repeat {
    pub fn next(&self) -> Self {
        match self {
            Repeat::Daily => Repeat::Weekday,
            Repeat::Weekday => Repeat::Weekend,
            Repeat::Weekend => Repeat::Once,
            Repeat::Once => Repeat::Daily,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Repeat::Daily => "daily",
            Repeat::Weekday => "weekday",
            Repeat::Weekend => "weekend",
            Repeat::Once => "once",
        }
    }
}


impl std::fmt::Display for Repeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn parse_time_from_str<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&s, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(&s, "%H:%M"))
        .map_err(serde::de::Error::custom)
}

fn serialize_time_to_str<S>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&time.format("%H:%M:%S").to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    #[serde(
        deserialize_with = "parse_time_from_str",
        serialize_with = "serialize_time_to_str"
    )]
    pub time: NaiveTime,
    pub enabled: bool,
    pub repeat: Repeat,
    pub note: String,
}

impl Default for Alarm {
    fn default() -> Self {
        Self {
            time: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
            enabled: false,
            repeat: Repeat::Daily,
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
