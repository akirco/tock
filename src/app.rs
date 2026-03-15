use crate::{cli::Cli, config::load_config, ui};
use chrono::Local;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use figlet_rs::FIGlet;
use ratatui::{backend::CrosstermBackend, style::Color, widgets::{BorderType, Borders}, Terminal};
use std::{io, str::FromStr, time::{Duration, Instant}};

#[derive(PartialEq)]
pub enum AppMode {
    Clock,
    Stopwatch,
    Countdown,
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
    // Panel visibility
    pub show_panel: bool,
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

        Self {
            mode,
            is_running: false, // Timers start paused, requiring [Space] to begin
            sw_start: None,
            sw_elapsed: Duration::ZERO,
            cd_target: None,
            cd_remaining: cd_initial,
            cd_initial,
            show_panel: false,
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
                (now.format("%H : %M : %S").to_string(), now.format("%A, %B %d, %Y").to_string())
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
                (format_duration(self.cd_remaining), status)
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

fn parse_border_style(s: &str) -> BorderType {
    match s.to_lowercase().as_str() {
        "rounded" => BorderType::Rounded,
        "double" => BorderType::Double,
        "thick" => BorderType::Thick,
        _ => BorderType::Plain,
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
    let panel_border_sides_str = cli.panel_border_sides.or(config.panel_border_sides).unwrap_or_else(|| "all".to_string());
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
        AppMode::Clock => "Alarm",
        AppMode::Stopwatch => "Stopwatch",
        AppMode::Countdown => "Timer",
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
        format!("Clock (Font: {}) - [p] Toggle Panel | Press 'q' or 'Ctrl+C' to exit", font_choice)
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

        terminal.draw(|f| ui::draw(f, &ui::UiData {
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
        }))?;

        // Polling rate set to 50ms for smooth timer UI updates
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()? {
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

    // --- Cleanup & Exit ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}