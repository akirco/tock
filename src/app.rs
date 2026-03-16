use crate::cli::Cli;
use crate::config::load_config;
use crate::sound::{get_sound_path, SoundPlayer};
use crate::state::AppState;
use crate::util::{parse_border_sides, parse_border_style};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use figlet_rs::FIGlet;
use ratatui::{backend::CrosstermBackend, style::Color, Terminal};
use std::{io, str::FromStr, sync::Arc, time::Duration};

pub fn run() -> Result<(), io::Error> {
    let cli = Cli::parse();
    let config = load_config();

    let font_choice = cli.font.or(config.font).unwrap_or_else(|| "standard".to_string());
    let bg_str = cli.bg.or(config.bg).unwrap_or_else(|| "reset".to_string());
    let fg_str = cli.fg.or(config.fg).unwrap_or_else(|| "cyan".to_string());
    let panel_ratio = cli.panel_ratio.or(config.panel_ratio).unwrap_or(50);

    let panel_bg_str = cli.panel_bg.or(config.panel_bg).unwrap_or_else(|| "reset".to_string());
    let panel_fg_str = cli.panel_fg.or(config.panel_fg).unwrap_or_else(|| "cyan".to_string());
    let panel_border_str = cli.panel_border.or(config.panel_border).unwrap_or_else(|| "cyan".to_string());
    let panel_border_sides_str = cli.panel_border_sides.or(config.panel_border_sides).unwrap_or_else(|| "vertical".to_string());
    let panel_border_style_str = cli.panel_border_style.or(config.panel_border_style).unwrap_or_else(|| "rounded".to_string());

    let alarm_sound_cli = cli.alarm_sound.or(config.alarm_sound).or(Some("alarm".to_string()));
    let countdown_sound_cli = cli.countdown_sound.or(config.countdown_sound).or(Some("alarm".to_string()));
    
    let alarm_sound = alarm_sound_cli.and_then(|s| get_sound_path(&s));
    let countdown_sound = countdown_sound_cli.and_then(|s| get_sound_path(&s));

    if alarm_sound.is_none() {
        eprintln!("Warning: No alarm sound file found in ~/.config/clock/sounds/");
    }
    if countdown_sound.is_none() {
        eprintln!("Warning: No countdown sound file found in ~/.config/clock/sounds/");
    }

    let sound_player = Arc::new(SoundPlayer::new());

    let bg_color = Color::from_str(&bg_str).unwrap_or(Color::Reset);
    let clock_color = Color::from_str(&fg_str).unwrap_or(Color::Cyan);
    let panel_bg = Color::from_str(&panel_bg_str).unwrap_or(Color::Reset);
    let panel_fg = Color::from_str(&panel_fg_str).unwrap_or(Color::Cyan);
    let panel_border = Color::from_str(&panel_border_str).unwrap_or(Color::Cyan);
    let panel_border_sides = parse_border_sides(&panel_border_sides_str);
    let panel_border_style = parse_border_style(&panel_border_style_str);

    let mut app_state = AppState::new(
        Some(sound_player),
        alarm_sound.map(|p| p.to_string_lossy().to_string()),
        countdown_sound.map(|p| p.to_string_lossy().to_string()),
    );

    let font = {
        let font_choice_lower = font_choice.to_lowercase();
        match font_choice_lower.as_str() {
            "standard" => FIGlet::standard().expect("Failed to load standard font"),
            "small" => FIGlet::small().expect("Failed to load small font"),
            "big" => FIGlet::big().expect("Failed to load big font"),
            "slant" => FIGlet::slant().expect("Failed to load slant font"),
            _ => FIGlet::from_file(&font_choice).unwrap_or_else(|_| {
                eprintln!("Warning: Failed to load font file '{}', using standard", font_choice);
                FIGlet::standard().expect("Failed to load standard font")
            }),
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = main_loop(
        &mut terminal,
        &mut app_state,
        &font,
        &font_choice,
        bg_color,
        clock_color,
        panel_ratio,
        panel_bg,
        panel_fg,
        panel_border,
        panel_border_sides,
        panel_border_style,
    );

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

#[allow(clippy::too_many_arguments)]
fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app_state: &mut AppState,
    font: &FIGlet,
    font_choice: &str,
    bg_color: Color,
    clock_color: Color,
    panel_ratio: u8,
    panel_bg: Color,
    panel_fg: Color,
    panel_border: Color,
    panel_border_sides: ratatui::widgets::Borders,
    panel_border_style: ratatui::widgets::BorderType,
) -> Result<(), io::Error> {
    loop {
        app_state.update_countdown();
        app_state.check_alarms();
        let (time_str, subtitle_str) = app_state.tick();
        let headers = app_state.get_headers();
        let mode = app_state.mode;
        let show_panel = app_state.show_panel;
        let items = app_state.get_items();

        let footer_str = format!(
            "{} (Font: {}) | [Tab] Switch Mode | [p] Panel | [Space] Stop Alarm | [q] Exit",
            mode.title(),
            font_choice
        );

        terminal.draw(|f| crate::ui::draw(f, &mut crate::ui::UiData {
            font,
            time_str: &time_str,
            subtitle_str: &subtitle_str,
            footer_str: &footer_str,
            bg_color,
            clock_color,
            show_panel,
            panel_ratio,
            panel_bg,
            panel_fg,
            panel_border,
            panel_border_sides,
            panel_border_style,
            mode,
            items: &items,
            headers,
            table_state: &mut app_state.table_state,
            edit_mode: &app_state.edit_mode,
            input_buffer: &app_state.input_buffer,
        }))?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match crate::handler::handle_key(key.code, key.modifiers, app_state, headers) {
                    crate::handler::Action::Quit => break,
                    crate::handler::Action::Continue => {}
                }
            }
    }

    Ok(())
}
