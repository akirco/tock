use crate::{cli::Cli, config::load_config, data, models, ui};
use chrono::Local;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use figlet_rs::FIGlet;
use ratatui::{backend::CrosstermBackend, style::Color, widgets::{Borders, TableState}, Terminal};
use std::{io, str::FromStr, time::{Duration, Instant}};

#[derive(PartialEq, Clone)]
pub enum AppMode {
    Clock,
    Stopwatch,
    Countdown,
}

#[derive(PartialEq, Clone)]
pub(crate) enum EditMode {
    Normal,
    Typing { is_new_row: bool },
}

pub struct AppState {
    pub mode: AppMode,
    pub is_running: bool,
    // Stopwatch states
    pub sw_start: Option<Instant>,
    pub sw_elapsed: Duration,
    // Countdown states
    pub cd_target: Option<Instant>,
    pub cd_remaining: Duration,
    pub cd_initial: Duration,
    pub cd_name: String,
    // Panel visibility
    pub show_panel: bool,
    // Table editing state
    pub table_state: TableState,
    pub edit_mode: EditMode,
    pub input_buffer: String,
    // Data
    pub data: models::AppData,
}

impl AppState {
    pub fn new(stopwatch: bool, time_str: Option<&String>) -> Self {
        // Determine mode based on provided arguments
        let (mode, cd_initial) = if let Some(t_str) = time_str {
            (AppMode::Countdown, parse_duration(t_str))
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
            data: data::load_data(),
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
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        match self.mode {
            AppMode::Stopwatch => {
                if self.sw_elapsed > Duration::ZERO || self.sw_start.is_some() {
                    let total = if self.is_running {
                        self.sw_elapsed + self.sw_start.map(|i| i.elapsed()).unwrap_or(Duration::ZERO)
                    } else {
                        self.sw_elapsed
                    };
                    let lap = self.data.history.len() as u32 + 1;
                    self.data.history.push(models::StopwatchRecord {
                        lap,
                        time_millis: total.as_millis() as u64,
                    });
                    self.table_state.select(Some(self.data.history.len() - 1));
                    data::save_data(&self.data).ok();
                }
                self.sw_elapsed = Duration::ZERO;
                self.sw_start = if self.is_running { Some(Instant::now()) } else { None };
            }
            AppMode::Countdown => {
                self.cd_remaining = self.cd_initial;
                self.cd_target = if self.is_running { Some(Instant::now() + self.cd_initial) } else { None };
            }
            _ => {}
        }
    }

    /// Calculate the string content to be displayed for the current frame
    pub fn tick(&mut self) -> (String, String) {
        match self.mode {
            AppMode::Clock => {
                let now = Local::now();
                let date_str = now.format("%A, %B %d, %Y").to_string();
                let next = self.get_next_alarm_info();
                let subtitle = if let Some((note, d)) = next {
                    if note.is_empty() {
                        format!("{} | Next: {}", date_str, format_duration_short(d))
                    } else {
                        format!("{} | {} | Next: {}", date_str, note, format_duration_short(d))
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
                let status = if self.is_running { "(Running)" } else { "(Paused)" };
                (format_duration(total), format!("STOPWATCH {}", status))
            }
            AppMode::Countdown => {
                if self.is_running
                    && let Some(target) = self.cd_target {
                        let now = Instant::now();
                        if now >= target {
                            self.is_running = false;
                            self.cd_remaining = Duration::ZERO;
                        } else {
                            self.cd_remaining = target.duration_since(now);
                        }
                    }
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
                (format_duration(self.cd_remaining), subtitle)
            }
        }
    }
}

// Helper function: Format Duration to HH:MM:SS.mmm
fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    let ms = d.subsec_millis();
    format!("{:02} : {:02} : {:02} . {:03}", h, m, s, ms)
}

fn format_duration_short(d: Duration) -> String {
    let total_secs = d.as_secs();
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    if h > 0 {
        format!("{:02}h {:02}m", h, m)
    } else {
        format!("{:02}m", m)
    }
}

// Helper function: Parse user input like "1h30m15s" to Duration
fn parse_duration(s: &str) -> Duration {
    let (mut total_secs, mut current_num, mut has_digits) = (0, 0, false);
    for c in s.chars() {
        if let Some(d) = c.to_digit(10) {
            current_num = current_num * 10 + d as u64;
            has_digits = true;
        } else {
            match c {
                'h' | 'H' => total_secs += current_num * 3600,
                'm' | 'M' => total_secs += current_num * 60,
                's' | 'S' => total_secs += current_num,
                _ => {}
            }
            current_num = 0;
            has_digits = false;
        }
    }
    if has_digits { total_secs += current_num; }
    if total_secs == 0 { total_secs = 300; } // Default to 5 minutes if parsing fails or equals 0
    Duration::from_secs(total_secs)
}

fn parse_border_sides(s: &str) -> Borders {
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

fn parse_border_style(s: &str) -> ratatui::widgets::BorderType {
    match s.to_lowercase().as_str() {
        "rounded" => ratatui::widgets::BorderType::Rounded,
        "double" => ratatui::widgets::BorderType::Double,
        "thick" => ratatui::widgets::BorderType::Thick,
        _ => ratatui::widgets::BorderType::Plain,
    }
}

impl AppState {
    fn get_alarm_next_duration(&self, alarm: &models::Alarm) -> Option<Duration> {
        if self.mode != AppMode::Clock || !alarm.enabled {
            return None;
        }
        
        let now = Local::now();
        let current_time = now.time();
        let now_naive = now.naive_local();
        
        if let Ok(alarm_time) = chrono::NaiveTime::parse_from_str(&alarm.time, "%H:%M:%S")
            .or_else(|_| chrono::NaiveTime::parse_from_str(&alarm.time, "%H:%M")) {
            let mut alarm_datetime = now.date_naive().and_time(alarm_time);
            
            if alarm_datetime.time() <= current_time {
                alarm_datetime += chrono::Duration::days(1);
            }
            
            let duration = alarm_datetime.signed_duration_since(now_naive);
            if duration.num_seconds() > 0 {
                return Some(Duration::from_secs(duration.num_seconds() as u64));
            }
        }
        
        None
    }

    fn get_next_alarm_info(&self) -> Option<(String, Duration)> {
        if self.mode != AppMode::Clock {
            return None;
        }
        
        let now = Local::now();
        let current_time = now.time();
        let now_naive = now.naive_local();
        
        let mut min_info: Option<(String, chrono::Duration)> = None;
        
        for alarm in &self.data.alarms {
            if !alarm.enabled {
                continue;
            }
            
            if let Ok(alarm_time) = chrono::NaiveTime::parse_from_str(&alarm.time, "%H:%M:%S")
                .or_else(|_| chrono::NaiveTime::parse_from_str(&alarm.time, "%H:%M")) {
                let mut alarm_datetime = now.date_naive().and_time(alarm_time);
                
                if alarm_datetime.time() <= current_time {
                    alarm_datetime += chrono::Duration::days(1);
                }
                
                let duration = alarm_datetime.signed_duration_since(now_naive);
                if duration.num_seconds() > 0 {
                    if let Some((_, min_dur)) = min_info {
                        if duration < min_dur {
                            min_info = Some((alarm.note.clone(), duration));
                        }
                    } else {
                        min_info = Some((alarm.note.clone(), duration));
                    }
                }
            }
        }
        
        min_info.map(|(note, d)| (note, Duration::from_secs(d.num_seconds() as u64)))
    }

    pub fn get_items(&self) -> Vec<Vec<String>> {
        match self.mode {
            AppMode::Clock => {
                self.data.alarms.iter().map(|a| vec![
                    a.time.clone(),
                    if a.enabled { "✔".to_string() } else { "✗".to_string() },
                    a.repeat.clone(),
                    a.note.clone(),
                    if a.enabled { self.get_alarm_next_duration(a).map(format_duration_short).unwrap_or_default() } else { String::new() },
                ]).collect()
            }
            AppMode::Stopwatch => self.data.history.iter().map(|r| vec![
                r.lap.to_string(),
                format_duration(Duration::from_millis(r.time_millis)),
            ]).collect(),
            AppMode::Countdown => self.data.presets.iter().map(|p| vec![
                p.name.clone(),
                p.duration.to_string(),
            ]).collect(),
        }
    }

    pub fn get_headers(&self) -> Vec<&'static str> {
        match self.mode {
            AppMode::Clock => vec!["Time", "Enabled", "Repeat", "Note", "Next"],
            AppMode::Stopwatch => vec!["Lap", "Time"],
            AppMode::Countdown => vec!["Name", "Seconds"],
        }
    }
}

pub fn run() -> Result<(), io::Error> {
    let cli = Cli::parse();
    let config = load_config();

    // Resolve parameters: CLI args > Config file > Default values
    let time_choice = cli.time.or(config.time);
    let stopwatch_choice = cli.stopwatch || config.stopwatch.unwrap_or(false);
    let font_choice = cli.font.or(config.font).unwrap_or_else(|| "standard".to_string());
    let bg_str = cli.bg.or(config.bg).unwrap_or_else(|| "reset".to_string());
    let fg_str = cli.fg.or(config.fg).unwrap_or_else(|| "cyan".to_string());
    let panel_ratio = cli.panel_ratio.or(config.panel_ratio).unwrap_or(50);

    // Panel styling
    let panel_bg_str = cli.panel_bg.or(config.panel_bg).unwrap_or_else(|| "reset".to_string());
    let panel_fg_str = cli.panel_fg.or(config.panel_fg).unwrap_or_else(|| "cyan".to_string());
    let panel_border_str = cli.panel_border.or(config.panel_border).unwrap_or_else(|| "cyan".to_string());
    let panel_border_sides_str = cli.panel_border_sides.or(config.panel_border_sides).unwrap_or_else(|| "vertical".to_string());
    let panel_border_style_str = cli.panel_border_style.or(config.panel_border_style).unwrap_or_else(|| "rounded".to_string());
    let user_panel_title = cli.panel_title.or(config.panel_title);

    let bg_color = Color::from_str(&bg_str).unwrap_or(Color::Reset);
    let clock_color = Color::from_str(&fg_str).unwrap_or(Color::Cyan);
    let panel_bg = Color::from_str(&panel_bg_str).unwrap_or(Color::Reset);
    let panel_fg = Color::from_str(&panel_fg_str).unwrap_or(Color::Cyan);
    let panel_border = Color::from_str(&panel_border_str).unwrap_or(Color::Cyan);
    let panel_border_sides = parse_border_sides(&panel_border_sides_str);
    let panel_border_style = parse_border_style(&panel_border_style_str);

    // Initialize App state machine
    let mut app_state = AppState::new(stopwatch_choice, time_choice.as_ref());

    // Default panel title based on mode
    let default_title = match app_state.mode {
        AppMode::Clock => " 󰀠 ",
        AppMode::Stopwatch => "  ",
        AppMode::Countdown => " 󱦟 ",
    };
    let panel_title = user_panel_title.unwrap_or_else(|| default_title.to_string());

    let font = match font_choice.to_lowercase().as_str() {
        "standard" => FIGlet::standard().expect("Failed to load standard font"),
        "small"    => FIGlet::small().expect("Failed to load small font"),
        "big"      => FIGlet::big().expect("Failed to load big font"),
        "slant"    => FIGlet::slant().expect("Failed to load slant font"),
        path       => FIGlet::from_file(path).unwrap_or_else(|_| panic!("Failed to load font file: {}", path)),
    };

    let footer_str = if app_state.mode == AppMode::Clock {
        format!("Clock (Font: {}) - [p] Toggle Panel | Press 'ESC' or 'Ctrl+C' to exit", font_choice)
    } else {
        format!("Timer (Font: {}) - [Space] Play/Pause | [r] Reset | [p] Panel | [q] Exit", font_choice)
    };

    // --- Terminal Initialization ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // --- Main Event Loop ---
    loop {
        let (time_str, subtitle_str) = app_state.tick();
        let items = app_state.get_items();
        let headers = app_state.get_headers();

        terminal.draw(|f| ui::draw(f, &mut ui::UiData {
            font: &font,
            time_str: &time_str,
            subtitle_str: &subtitle_str,
            footer_str: &footer_str,
            bg_color,
            clock_color,
            show_panel: app_state.show_panel,
            panel_ratio,
            panel_bg,
            panel_fg,
            panel_border,
            panel_border_sides,
            panel_border_style,
            panel_title: panel_title.as_str(),
            mode: app_state.mode.clone(),
            items: &items,
            headers: &headers,
            table_state: &mut app_state.table_state,
            edit_mode: &app_state.edit_mode,
            input_buffer: &app_state.input_buffer,
        }))?;

        // Polling rate set to 50ms for smooth timer UI updates
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app_state.show_panel {
                    match app_state.edit_mode.clone() {
                        EditMode::Normal => {
                            match key.code {
                                KeyCode::Esc => break,
                                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                                KeyCode::Enter => {
                                    if let AppMode::Countdown = app_state.mode
                                        && let Some(r) = app_state.table_state.selected()
                                            && r < app_state.data.presets.len() {
                                                let preset = &app_state.data.presets[r];
                                                app_state.cd_initial = Duration::from_secs(preset.duration);
                                                app_state.cd_remaining = app_state.cd_initial;
                                                app_state.cd_name = preset.name.clone();
                                                app_state.cd_target = None;
                                                app_state.is_running = false;
                                            }
                                }
                                KeyCode::Down | KeyCode::Up | KeyCode::Left | KeyCode::Right => {
                                    match key.code {
                                        KeyCode::Down => app_state.table_state.select_next(),
                                        KeyCode::Up => app_state.table_state.select_previous(),
                                        KeyCode::Right => app_state.table_state.select_next_column(),
                                        KeyCode::Left => app_state.table_state.select_previous_column(),
                                        _ => {}
                                    }
                                }
                                KeyCode::Char('g') => app_state.table_state.select_first(),
                                KeyCode::Char('G') => app_state.table_state.select_last(),
                                KeyCode::Char('a') => {
                                    match app_state.mode {
                                        AppMode::Clock => {
                                            app_state.data.alarms.push(models::Alarm::default());
                                            let len = app_state.data.alarms.len();
                                            app_state.table_state.select(Some(len - 1));
                                            app_state.table_state.select_column(Some(0));
                                            app_state.input_buffer.clear();
                                            app_state.edit_mode = EditMode::Typing { is_new_row: true };
                                        }
                                        AppMode::Countdown => {
                                            app_state.data.presets.push(models::CountdownPreset::default());
                                            let len = app_state.data.presets.len();
                                            app_state.table_state.select(Some(len - 1));
                                            app_state.table_state.select_column(Some(0));
                                            app_state.input_buffer.clear();
                                            app_state.edit_mode = EditMode::Typing { is_new_row: true };
                                        }
                                        AppMode::Stopwatch => {}
                                    }
                                }
                                KeyCode::Char('e') => {
                                    if let (Some(r), Some(c)) = (app_state.table_state.selected(), app_state.table_state.selected_column())
                                        && r < items.len() && c < items[r].len() {
                                            app_state.input_buffer = items[r][c].clone();
                                            app_state.edit_mode = EditMode::Typing { is_new_row: false };
                                        }
                                }
                                KeyCode::Char('d') => {
                                    if let Some(r) = app_state.table_state.selected() {
                                        match app_state.mode {
                                            AppMode::Clock if r < app_state.data.alarms.len() => {
                                                app_state.data.alarms.remove(r);
                                            }
                                            AppMode::Countdown if r < app_state.data.presets.len() => {
                                                app_state.data.presets.remove(r);
                                            }
                                            AppMode::Stopwatch if r < app_state.data.history.len() => {
                                                app_state.data.history.remove(r);
                                            }
                                            _ => {}
                                        }
                                        data::save_data(&app_state.data).ok();
                                        if r >= items.len().saturating_sub(1) && !items.is_empty() {
                                            app_state.table_state.select(Some(items.len().saturating_sub(1)));
                                        }
                                    }
                                }
                                KeyCode::Char(' ') => app_state.toggle_pause(),
                                KeyCode::Char('r') | KeyCode::Char('R') => app_state.reset(),
                                KeyCode::Char('p') | KeyCode::Char('P') => app_state.show_panel = !app_state.show_panel,
                                _ => {}
                            }
                        }
                        EditMode::Typing { is_new_row } => {
                            match key.code {
                                KeyCode::Enter => {
                                    if let (Some(r), Some(c)) = (app_state.table_state.selected(), app_state.table_state.selected_column()) {
                                        let col_count = headers.len();

                                        let is_valid = !app_state.input_buffer.trim().is_empty();

                                        if is_valid {
                                            match app_state.mode {
                                                AppMode::Clock if r < app_state.data.alarms.len() => {
                                                    let alarm = &mut app_state.data.alarms[r];
                                                    match c {
                                                        0 => alarm.time = app_state.input_buffer.clone(),
                                                        1 => alarm.enabled = app_state.input_buffer == "✔",
                                                        2 => alarm.repeat = app_state.input_buffer.clone(),
                                                        3 => alarm.note = app_state.input_buffer.clone(),
                                                        _ => {}
                                                    }
                                                }
                                                AppMode::Countdown if r < app_state.data.presets.len() => {
                                                    let preset = &mut app_state.data.presets[r];
                                                    match c {
                                                        0 => preset.name = app_state.input_buffer.clone(),
                                                        1 => preset.duration = app_state.input_buffer.parse().unwrap_or(0),
                                                        _ => {}
                                                    }
                                                }
                                                _ => {}
                                            }
                                            data::save_data(&app_state.data).ok();

                                            if is_new_row && c < col_count - 1 {
                                                app_state.table_state.select_column(Some(c + 1));
                                                app_state.input_buffer.clear();
                                            } else {
                                                app_state.edit_mode = EditMode::Normal;
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char(' ') => {
                                    if let (Some(r), Some(c)) = (app_state.table_state.selected(), app_state.table_state.selected_column()) {
                                        if app_state.mode == AppMode::Clock && r < app_state.data.alarms.len() {
                                            if c == 1 {
                                                app_state.data.alarms[r].enabled = !app_state.data.alarms[r].enabled;
                                                app_state.input_buffer = if app_state.data.alarms[r].enabled { "✔".to_string() } else { "✗".to_string() };
                                                data::save_data(&app_state.data).ok();
                                            } else if c == 2 {
                                                let options = ["daily", "weekday", "weekend", "once"];
                                                let current = &app_state.data.alarms[r].repeat;
                                                let idx = options.iter().position(|x| x == current).unwrap_or(0);
                                                let next = options[(idx + 1) % options.len()];
                                                app_state.data.alarms[r].repeat = next.to_string();
                                                app_state.input_buffer = next.to_string();
                                                data::save_data(&app_state.data).ok();
                                            } else {
                                                app_state.input_buffer.push(' ');
                                            }
                                        } else {
                                            app_state.input_buffer.push(' ');
                                        }
                                    }
                                }
                                KeyCode::Esc => {
                                    if is_new_row {
                                        match app_state.mode {
                                            AppMode::Clock if app_state.data.alarms.len() > items.len() => {
                                                app_state.data.alarms.pop();
                                            }
                                            AppMode::Countdown if app_state.data.presets.len() > items.len() => {
                                                app_state.data.presets.pop();
                                            }
                                            _ => {}
                                        }
                                    }
                                    app_state.edit_mode = EditMode::Normal;
                                }
                                KeyCode::Backspace => {
                                    app_state.input_buffer.pop();
                                }
                                KeyCode::Char('l') | KeyCode::Right => {
                                    if let Some(c) = app_state.table_state.selected_column()
                                        && c < headers.len() - 1 {
                                            app_state.table_state.select_column(Some(c + 1));
                                            app_state.input_buffer.clear();
                                        }
                                }
                                KeyCode::Char('h') | KeyCode::Left => {
                                    if let Some(c) = app_state.table_state.selected_column()
                                        && c > 0 {
                                            app_state.table_state.select_column(Some(c - 1));
                                            app_state.input_buffer.clear();
                                        }
                                }
                                KeyCode::Char(ch) => {
                                    app_state.input_buffer.push(ch);
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Char(' ') => app_state.toggle_pause(),
                        KeyCode::Char('r') | KeyCode::Char('R') => app_state.reset(),
                        KeyCode::Char('p') | KeyCode::Char('P') => app_state.show_panel = !app_state.show_panel,
                        _ => {}
                    }
                }
            }
    }

    // --- Cleanup & Exit ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}