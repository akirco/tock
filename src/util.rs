use chrono::NaiveTime;
use std::time::Duration;

pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    let ms = d.subsec_millis();
    format!("{:02} : {:02} : {:02} . {:03}", h, m, s, ms)
}

pub fn format_duration_short(d: Duration) -> String {
    let total_secs = d.as_secs();
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    if h > 0 {
        format!("{:02}h {:02}m", h, m)
    } else {
        format!("{:02}m", m)
    }
}

pub fn parse_border_sides(s: &str) -> ratatui::widgets::Borders {
    use ratatui::widgets::Borders;
    match s.to_lowercase().as_str() {
        "none" => Borders::NONE,
        "left" => Borders::LEFT,
        "right" => Borders::RIGHT,
        "top" => Borders::TOP,
        "bottom" => Borders::BOTTOM,
        "horizontal" => Borders::LEFT | Borders::RIGHT,
        "vertical" => Borders::TOP | Borders::BOTTOM,
        _ => Borders::ALL,
    }
}

pub fn parse_border_style(s: &str) -> ratatui::widgets::BorderType {
    match s.to_lowercase().as_str() {
        "rounded" => ratatui::widgets::BorderType::Rounded,
        "double" => ratatui::widgets::BorderType::Double,
        "thick" => ratatui::widgets::BorderType::Thick,
        _ => ratatui::widgets::BorderType::Plain,
    }
}

pub fn parse_time(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M"))
        .ok()
}

pub fn parse_repeat(s: &str) -> Option<crate::models::Repeat> {
    use crate::models::Repeat;
    match s.to_lowercase().as_str() {
        "daily" => Some(Repeat::Daily),
        "weekday" => Some(Repeat::Weekday),
        "weekend" => Some(Repeat::Weekend),
        "once" => Some(Repeat::Once),
        _ => None,
    }
}
