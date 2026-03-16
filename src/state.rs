use crate::models;
use ratatui::widgets::TableState;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Clock,
    Stopwatch,
    Countdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    Normal,
    Typing { is_new_row: bool },
}

pub struct AppState {
    pub mode: AppMode,
    pub is_running: bool,
    pub sw_start: Option<Instant>,
    pub sw_elapsed: Duration,
    pub cd_target: Option<Instant>,
    pub cd_remaining: Duration,
    pub cd_initial: Duration,
    pub cd_name: String,
    pub show_panel: bool,
    pub table_state: TableState,
    pub edit_mode: EditMode,
    pub input_buffer: String,
    pub data: models::AppData,
    pub items_dirty: bool,
}

impl AppState {
    pub fn new(stopwatch: bool, time_str: Option<&str>) -> Self {
        let (mode, cd_initial) = if let Some(t_str) = time_str {
            (AppMode::Countdown, super::util::parse_duration(t_str))
        } else if stopwatch {
            (AppMode::Stopwatch, Duration::ZERO)
        } else {
            (AppMode::Clock, Duration::ZERO)
        };

        let mut table_state = TableState::default();
        table_state.select_first();
        table_state.select_first_column();

        Self {
            mode,
            is_running: false,
            sw_start: None,
            sw_elapsed: Duration::ZERO,
            cd_target: None,
            cd_remaining: cd_initial,
            cd_initial,
            cd_name: String::new(),
            show_panel: false,
            table_state,
            edit_mode: EditMode::Normal,
            input_buffer: String::new(),
            data: crate::data::load_data(),
            items_dirty: true,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.items_dirty = true;
    }

    pub fn get_items(&self) -> Vec<Vec<String>> {
        match self.mode {
            AppMode::Clock => self
                .data
                .alarms
                .iter()
                .map(|a| {
                    vec![
                        a.time.format("%H:%M:%S").to_string(),
                        String::from(if a.enabled { "✔" } else { "✗" }),
                        a.repeat.to_string(),
                        a.note.clone(),
                        if a.enabled {
                            self.get_alarm_next_duration(a)
                                .map(super::util::format_duration_short)
                                .unwrap_or_default()
                        } else {
                            String::new()
                        },
                    ]
                })
                .collect(),
            AppMode::Stopwatch => self
                .data
                .history
                .iter()
                .map(|r| {
                    vec![
                        r.lap.to_string(),
                        super::util::format_duration(Duration::from_millis(r.time_millis)),
                    ]
                })
                .collect(),
            AppMode::Countdown => self
                .data
                .presets
                .iter()
                .map(|p| vec![p.name.clone(), p.duration.to_string()])
                .collect(),
        }
    }

    pub fn get_headers(&self) -> &'static [&'static str] {
        match self.mode {
            AppMode::Clock => &["Time", "Enabled", "Repeat", "Note", "Next"],
            AppMode::Stopwatch => &["Lap", "Time"],
            AppMode::Countdown => &["Name", "Seconds"],
        }
    }

    pub fn data_len(&self) -> usize {
        match self.mode {
            AppMode::Clock => self.data.alarms.len(),
            AppMode::Stopwatch => self.data.history.len(),
            AppMode::Countdown => self.data.presets.len(),
        }
    }

    pub fn get_cell_content(&self, row: usize, col: usize) -> String {
        match self.mode {
            AppMode::Clock if row < self.data.alarms.len() => {
                let a = &self.data.alarms[row];
                match col {
                    0 => a.time.format("%H:%M:%S").to_string(),
                    1 => String::from(if a.enabled { "✔" } else { "✗" }),
                    2 => a.repeat.to_string(),
                    3 => a.note.clone(),
                    4 => {
                        if a.enabled {
                            self.get_alarm_next_duration(a)
                                .map(super::util::format_duration_short)
                                .unwrap_or_default()
                        } else {
                            String::new()
                        }
                    }
                    _ => String::new(),
                }
            }
            AppMode::Stopwatch if row < self.data.history.len() => {
                let r = &self.data.history[row];
                match col {
                    0 => r.lap.to_string(),
                    1 => super::util::format_duration(Duration::from_millis(r.time_millis)),
                    _ => String::new(),
                }
            }
            AppMode::Countdown if row < self.data.presets.len() => {
                let p = &self.data.presets[row];
                match col {
                    0 => p.name.clone(),
                    1 => p.duration.to_string(),
                    _ => String::new(),
                }
            }
            _ => String::new(),
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.mode {
            AppMode::Stopwatch => {
                if self.is_running {
                    if let Some(start) = self.sw_start {
                        self.sw_elapsed += start.elapsed();
                    }
                } else {
                    self.sw_start = Some(Instant::now());
                }
                self.is_running = !self.is_running;
            }
            AppMode::Countdown => {
                if self.cd_remaining > Duration::ZERO {
                    if self.is_running {
                        if let Some(target) = self.cd_target {
                            self.cd_remaining = target.saturating_duration_since(Instant::now());
                        }
                    } else {
                        self.cd_target = Some(Instant::now() + self.cd_remaining);
                    }
                    self.is_running = !self.is_running;
                }
            }
            AppMode::Clock => {}
        }
    }

    pub fn reset(&mut self) {
        match self.mode {
            AppMode::Stopwatch => {
                if self.sw_elapsed > Duration::ZERO || self.sw_start.is_some() {
                    let total = if self.is_running {
                        self.sw_elapsed
                            + self.sw_start.map(|i| i.elapsed()).unwrap_or(Duration::ZERO)
                    } else {
                        self.sw_elapsed
                    };
                    let lap = self.data.history.len() as u32 + 1;
                    self.data.history.push(models::StopwatchRecord {
                        lap,
                        time_millis: total.as_millis() as u64,
                    });
                    self.table_state.select(Some(self.data.history.len() - 1));
                    self.mark_dirty();
                    crate::data::save_data(&self.data).ok();
                }
                self.sw_elapsed = Duration::ZERO;
                self.sw_start = if self.is_running {
                    Some(Instant::now())
                } else {
                    None
                };
            }
            AppMode::Countdown => {
                self.cd_remaining = self.cd_initial;
                self.cd_target = if self.is_running {
                    Some(Instant::now() + self.cd_initial)
                } else {
                    None
                };
            }
            AppMode::Clock => {}
        }
    }

    pub fn tick(&self) -> (String, String) {
        match self.mode {
            AppMode::Clock => {
                let now = chrono::Local::now();
                let date_str = now.format("%A, %B %d, %Y").to_string();
                let subtitle = if !self.data.alarms.is_empty() {
                    if let Some((note, d)) = self.get_next_alarm_info() {
                        if note.is_empty() {
                            format!(
                                "{} | Next: {}",
                                date_str,
                                super::util::format_duration_short(d)
                            )
                        } else {
                            format!(
                                "{} | {} | Next: {}",
                                date_str,
                                note,
                                super::util::format_duration_short(d)
                            )
                        }
                    } else {
                        date_str
                    }
                } else {
                    date_str
                };
                (now.format("%H : %M : %S").to_string(), subtitle)
            }
            AppMode::Stopwatch => {
                let total = if self.is_running {
                    self.sw_elapsed + self.sw_start.map(|i| i.elapsed()).unwrap_or(Duration::ZERO)
                } else {
                    self.sw_elapsed
                };
                let status = if self.is_running {
                    "(Running)"
                } else {
                    "(Paused)"
                };
                (
                    super::util::format_duration(total),
                    format!("STOPWATCH {}", status),
                )
            }
            AppMode::Countdown => {
                let status = if self.cd_remaining == Duration::ZERO {
                    "TIME'S UP!".to_string()
                } else if self.is_running {
                    "COUNTDOWN (Running)".to_string()
                } else {
                    "COUNTDOWN (Paused)".to_string()
                };
                let subtitle = if self.cd_name.is_empty() {
                    status
                } else {
                    format!("{} - {}", self.cd_name, status)
                };
                (super::util::format_duration(self.cd_remaining), subtitle)
            }
        }
    }

    pub fn update_countdown(&mut self) {
        if self.is_running && self.mode == AppMode::Countdown
            && let Some(target) = self.cd_target {
                let now = Instant::now();
                if now >= target {
                    self.is_running = false;
                    self.cd_remaining = Duration::ZERO;
                } else {
                    self.cd_remaining = target.duration_since(now);
                }
            }
    }

    fn get_alarm_next_duration(&self, alarm: &models::Alarm) -> Option<Duration> {
        if !alarm.enabled {
            return None;
        }

        let now = chrono::Local::now();
        let current_time = now.time();
        let now_naive = now.naive_local();

        let mut alarm_datetime = now.date_naive().and_time(alarm.time);
        if alarm_datetime.time() <= current_time {
            alarm_datetime += chrono::Duration::days(1);
        }

        let duration = alarm_datetime.signed_duration_since(now_naive);
        if duration.num_seconds() > 0 {
            Some(Duration::from_secs(duration.num_seconds() as u64))
        } else {
            None
        }
    }

    fn get_next_alarm_info(&self) -> Option<(&str, Duration)> {
        let now = chrono::Local::now();
        let current_time = now.time();
        let now_naive = now.naive_local();

        let mut min_info: Option<(&str, chrono::Duration)> = None;

        for alarm in &self.data.alarms {
            if !alarm.enabled {
                continue;
            }

            let mut alarm_datetime = now.date_naive().and_time(alarm.time);
            if alarm_datetime.time() <= current_time {
                alarm_datetime += chrono::Duration::days(1);
            }

            let duration = alarm_datetime.signed_duration_since(now_naive);
            if duration.num_seconds() > 0 {
                if let Some((_, min_dur)) = min_info {
                    if duration < min_dur {
                        min_info = Some((&alarm.note, duration));
                    }
                } else {
                    min_info = Some((&alarm.note, duration));
                }
            }
        }

        min_info.map(|(note, d)| (note, Duration::from_secs(d.num_seconds() as u64)))
    }
}
