use crate::models;
use crate::sound::SoundPlayer;
use ratatui::widgets::TableState;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Clock,
    Stopwatch,
    Countdown,
}

impl AppMode {
    pub fn next(&self) -> Self {
        match self {
            AppMode::Clock => AppMode::Stopwatch,
            AppMode::Stopwatch => AppMode::Countdown,
            AppMode::Countdown => AppMode::Clock,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            AppMode::Clock => "󰀠 Clock",
            AppMode::Stopwatch => " Stopwatch",
            AppMode::Countdown => "󱦟 Countdown",
        }
    }

    pub fn space_key_desc(&self) -> &'static str {
        match self {
            AppMode::Clock => "Stop Alarm",
            AppMode::Stopwatch => "Start/Stop",
            AppMode::Countdown => "Start/Stop Timer",
        }
    }
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
    pub sound_player: Option<Arc<SoundPlayer>>,
    pub alarm_sound: Option<String>,
    pub countdown_sound: Option<String>,
    pub last_alarm_triggered: Option<u64>,
    pub countdown_played: bool,
    pub sound_start_time: Option<Instant>,
    pub current_alarm_duration: u64,
    pub current_alarm_repeat: u32,
}

impl AppState {
    pub fn new(sound_player: Option<Arc<SoundPlayer>>, alarm_sound: Option<String>, countdown_sound: Option<String>) -> Self {
        let mut table_state = TableState::default();
        table_state.select_first();
        table_state.select_first_column();

        Self {
            mode: AppMode::Clock,
            is_running: false,
            sw_start: None,
            sw_elapsed: Duration::ZERO,
            cd_target: None,
            cd_remaining: Duration::ZERO,
            cd_initial: Duration::ZERO,
            cd_name: String::new(),
            show_panel: false,
            table_state,
            edit_mode: EditMode::Normal,
            input_buffer: String::new(),
            data: crate::data::load_data(),
            items_dirty: true,
            sound_player,
            alarm_sound,
            countdown_sound,
            last_alarm_triggered: None,
            countdown_played: false,
            sound_start_time: None,
            current_alarm_duration: 60,
            current_alarm_repeat: 0,
        }
    }

    pub fn switch_mode(&mut self) {
        self.mode = self.mode.next();
        self.is_running = false;
        self.sw_start = None;
        self.sw_elapsed = Duration::ZERO;
        self.cd_target = None;
        self.cd_remaining = self.cd_initial;
        self.mark_dirty();
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
                        a.alarm_duration.to_string(),
                        a.alarm_repeat.to_string(),
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
                .map(|p| vec![
                    p.name.clone(),
                    p.duration.to_string(),
                    p.alarm_duration.to_string(),
                    p.alarm_repeat.to_string(),
                ])
                .collect(),
        }
    }

    pub fn get_headers(&self) -> &'static [&'static str] {
        match self.mode {
            AppMode::Clock => &["Time", "Enabled", "Repeat", "Note", "Duration(s)", "Repeat", "Next"],
            AppMode::Stopwatch => &["Lap", "Time"],
            AppMode::Countdown => &["Name", "Seconds", "Duration(s)", "Repeat"],
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
                    4 => a.alarm_duration.to_string(),
                    5 => a.alarm_repeat.to_string(),
                    6 => {
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
                    2 => p.alarm_duration.to_string(),
                    3 => p.alarm_repeat.to_string(),
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
                    if let Err(e) = crate::data::save_data(&self.data) {
                        eprintln!("Failed to save data: {}", e);
                    }
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
                let alarm_info = if !self.data.alarms.is_empty() {
                    if let Some((note, d)) = self.get_next_alarm_info() {
                        if note.is_empty() {
                            Some(format!(
                                "{} | Next: {}",
                                date_str,
                                super::util::format_duration_short(d)
                            ))
                        } else {
                            Some(format!(
                                "{} | {} | Next: {}",
                                date_str,
                                note,
                                super::util::format_duration_short(d)
                            ))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                let subtitle = if let Some(start_time) = self.sound_start_time {
                    let elapsed = start_time.elapsed();
                    let remaining = self.current_alarm_duration.saturating_sub(elapsed.as_secs());
                    if remaining > 0 {
                        format!("🔔 Ringing... ({}s)", remaining)
                    } else {
                        alarm_info.unwrap_or(date_str)
                    }
                } else {
                    alarm_info.unwrap_or(date_str)
                };

                (now.format("%H : %M : %S").to_string(), subtitle)
            }
            AppMode::Stopwatch => {
                let total = if self.is_running {
                    self.sw_elapsed + self.sw_start.map(|i| i.elapsed()).unwrap_or(Duration::ZERO)
                } else {
                    self.sw_elapsed
                };
                let status = if self.is_running { "(Running)" } else { "(Paused)" };

                let subtitle = if let Some(start_time) = self.sound_start_time {
                    let elapsed = start_time.elapsed();
                    let remaining = self.current_alarm_duration.saturating_sub(elapsed.as_secs());
                    if remaining > 0 {
                        format!("🔔 Ringing... ({}s) | STOPWATCH {}", remaining, status)
                    } else {
                        format!("STOPWATCH {}", status)
                    }
                } else {
                    format!("STOPWATCH {}", status)
                };

                (
                    super::util::format_duration(total),
                    subtitle,
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

                let subtitle = if let Some(start_time) = self.sound_start_time {
                    let elapsed = start_time.elapsed();
                    let remaining = self.current_alarm_duration.saturating_sub(elapsed.as_secs());
                    if remaining > 0 {
                        let base = if self.cd_name.is_empty() { status } else { format!("{} - {}", self.cd_name, status) };
                        format!("🔔 Ringing... ({}s) | {}", remaining, base)
                    } else {
                        if self.cd_name.is_empty() { status } else { format!("{} - {}", self.cd_name, status) }
                    }
                } else {
                    if self.cd_name.is_empty() { status } else { format!("{} - {}", self.cd_name, status) }
                };

                (super::util::format_duration(self.cd_remaining), subtitle)
            }
        }
    }

    pub fn update_countdown(&mut self) {
        // Check if sound should stop based on alarm_duration
        if let Some(start_time) = self.sound_start_time {
            let stop_duration = Duration::from_secs(self.current_alarm_duration);
            if start_time.elapsed() > stop_duration {
                self.stop_sound();
                self.sound_start_time = None;
            }
        }

        if self.is_running && self.mode == AppMode::Countdown
            && let Some(target) = self.cd_target {
                let now = Instant::now();
                if now >= target {
                    self.is_running = false;
                    self.cd_remaining = Duration::ZERO;
                    if !self.countdown_played {
                        self.countdown_played = true;

                        // Get alarm settings from selected preset
                        if let Some(r) = self.table_state.selected()
                            && r < self.data.presets.len() {
                                let preset = &self.data.presets[r];
                                self.current_alarm_duration = preset.alarm_duration;
                                self.current_alarm_repeat = preset.alarm_repeat;
                            }

                        self.sound_start_time = Some(Instant::now());
                        if let Some(ref sound) = self.countdown_sound
                            && let Some(ref player) = self.sound_player {
                                player.play(sound);
                            }
                    }
                } else {
                    self.cd_remaining = target.duration_since(now);
                }
            }
    }

    pub fn check_alarms(&mut self) {
        if self.mode != AppMode::Clock {
            return;
        }

        let now = chrono::Local::now();
        let current_time = now.time();
        let today = now.date_naive();

        for (idx, alarm) in self.data.alarms.iter().enumerate() {
            if !alarm.enabled {
                continue;
            }

            let alarm_datetime = today.and_time(alarm.time);

            if alarm_datetime.time() <= current_time {
                continue;
            }

            let time_diff = alarm_datetime.signed_duration_since(now.naive_local());
            let seconds_until = time_diff.num_seconds();

            // Window of 3 seconds to catch the alarm
            if (0..=3).contains(&seconds_until) {
                let alarm_id = idx as u64;
                if self.last_alarm_triggered != Some(alarm_id) {
                    self.last_alarm_triggered = Some(alarm_id);
                    self.current_alarm_duration = alarm.alarm_duration;
                    self.current_alarm_repeat = alarm.alarm_repeat;
                    self.sound_start_time = Some(Instant::now());
                    if let Some(ref sound) = self.alarm_sound
                        && let Some(ref player) = self.sound_player {
                            player.play(sound);
                        }
                }
            }
        }

        // Reset triggered state if no alarms about to go off
        let now2 = chrono::Local::now();
        let current_time2 = now2.time();
        let mut about_to_trigger = false;
        for alarm in &self.data.alarms {
            if alarm.enabled {
                let alarm_datetime = today.and_time(alarm.time);
                let diff = alarm_datetime.time().signed_duration_since(current_time2);
                if diff.num_seconds() >= 0 && diff.num_seconds() <= 3 {
                    about_to_trigger = true;
                    break;
                }
            }
        }
        if !about_to_trigger {
            self.last_alarm_triggered = None;
        }
    }

    pub fn stop_sound(&mut self) {
        if let Some(ref player) = self.sound_player {
            player.stop();
        }
        self.sound_start_time = None;
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
