use crate::{data, models, state::{AppMode, AppState, EditMode}, util};
use crossterm::event::{KeyCode, KeyModifiers};

pub enum Action {
    Quit,
    Continue,
}

pub fn handle_key(
    key_code: KeyCode,
    modifiers: KeyModifiers,
    app_state: &mut AppState,
    headers: &[&'static str],
) -> Action {
    if matches!(key_code, KeyCode::Tab) {
        app_state.switch_mode();
        return Action::Continue;
    }

    if app_state.show_panel {
        match app_state.edit_mode {
            EditMode::Normal => handle_normal_mode(key_code, modifiers, app_state, headers),
            EditMode::Typing { is_new_row } => handle_typing_mode(key_code, app_state, headers, is_new_row),
        }
    } else {
        handle_panel_closed(key_code, modifiers, app_state)
    }
}

fn handle_normal_mode(
    key_code: KeyCode,
    modifiers: KeyModifiers,
    app_state: &mut AppState,
    _headers: &[&'static str],
) -> Action {
    match key_code {
        KeyCode::Esc => Action::Quit,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Enter => {
            if let AppMode::Countdown = app_state.mode
                && let Some(r) = app_state.table_state.selected()
                    && r < app_state.data.presets.len() {
                        let preset = &app_state.data.presets[r];
                        app_state.cd_initial = std::time::Duration::from_secs(preset.duration);
                        app_state.cd_remaining = app_state.cd_initial;
                        app_state.cd_name = preset.name.clone();
                        app_state.cd_target = None;
                        app_state.is_running = false;
                    }
            Action::Continue
        }
        KeyCode::Down => {
            app_state.table_state.select_next();
            Action::Continue
        }
        KeyCode::Up => {
            app_state.table_state.select_previous();
            Action::Continue
        }
        KeyCode::Right => {
            app_state.table_state.select_next_column();
            Action::Continue
        }
        KeyCode::Left => {
            app_state.table_state.select_previous_column();
            Action::Continue
        }
        KeyCode::Char('g') => {
            app_state.table_state.select_first();
            Action::Continue
        }
        KeyCode::Char('G') => {
            app_state.table_state.select_last();
            Action::Continue
        }
        KeyCode::Char('a') => {
            handle_add(app_state);
            Action::Continue
        }
        KeyCode::Char('e') => {
            if let (Some(r), Some(c)) = (app_state.table_state.selected(), app_state.table_state.selected_column()) {
                app_state.input_buffer = app_state.get_cell_content(r, c);
                app_state.edit_mode = EditMode::Typing { is_new_row: false };
            }
            Action::Continue
        }
        KeyCode::Char('d') => {
            handle_delete(app_state);
            Action::Continue
        }
        KeyCode::Char(' ') => {
            app_state.toggle_pause();
            Action::Continue
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app_state.reset();
            Action::Continue
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app_state.show_panel = !app_state.show_panel;
            Action::Continue
        }
        _ => Action::Continue,
    }
}

fn handle_typing_mode(
    key_code: KeyCode,
    app_state: &mut AppState,
    headers: &[&'static str],
    is_new_row: bool,
) -> Action {
    match key_code {
        KeyCode::Enter => {
            handle_enter(app_state, headers, is_new_row);
            Action::Continue
        }
        KeyCode::Char(' ') => {
            handle_space(app_state);
            Action::Continue
        }
        KeyCode::Esc => {
            handle_escape(app_state, is_new_row);
            Action::Continue
        }
        KeyCode::Backspace => {
            app_state.input_buffer.pop();
            Action::Continue
        }
        KeyCode::Right => {
            if let Some(c) = app_state.table_state.selected_column()
                && c < headers.len() - 1 {
                    app_state.table_state.select_column(Some(c + 1));
                    app_state.input_buffer.clear();
                }
            Action::Continue
        }
        KeyCode::Left => {
            if let Some(c) = app_state.table_state.selected_column()
                && c > 0 {
                    app_state.table_state.select_column(Some(c - 1));
                    app_state.input_buffer.clear();
                }
            Action::Continue
        }
        KeyCode::Char(ch) => {
            app_state.input_buffer.push(ch);
            Action::Continue
        }
        _ => Action::Continue,
    }
}

fn handle_panel_closed(
    key_code: KeyCode,
    modifiers: KeyModifiers,
    app_state: &mut AppState,
) -> Action {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char(' ') => {
            app_state.stop_sound();
            app_state.countdown_played = false;
            app_state.toggle_pause();
            Action::Continue
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app_state.stop_sound();
            app_state.countdown_played = false;
            app_state.reset();
            Action::Continue
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app_state.show_panel = !app_state.show_panel;
            Action::Continue
        }
        _ => Action::Continue,
    }
}

fn handle_add(app_state: &mut AppState) {
    match app_state.mode {
        AppMode::Clock => {
            app_state.data.alarms.push(models::Alarm::default());
            let len = app_state.data.alarms.len();
            app_state.table_state.select(Some(len - 1));
            app_state.table_state.select_column(Some(0));
            app_state.input_buffer.clear();
            app_state.edit_mode = EditMode::Typing { is_new_row: true };
            app_state.mark_dirty();
        }
        AppMode::Countdown => {
            app_state.data.presets.push(models::CountdownPreset::default());
            let len = app_state.data.presets.len();
            app_state.table_state.select(Some(len - 1));
            app_state.table_state.select_column(Some(0));
            app_state.input_buffer.clear();
            app_state.edit_mode = EditMode::Typing { is_new_row: true };
            app_state.mark_dirty();
        }
        AppMode::Stopwatch => {}
    }
}

fn handle_delete(app_state: &mut AppState) {
    if let Some(r) = app_state.table_state.selected() {
        let len_before = app_state.data_len();
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
            _ => return,
        }
        if let Err(e) = data::save_data(&app_state.data) {
            eprintln!("Failed to save data: {}", e);
        }
        app_state.mark_dirty();
        let len_after = app_state.data_len();
        if len_after < len_before && r >= len_after.saturating_sub(1) && len_after > 0 {
            app_state.table_state.select(Some(len_after - 1));
        }
    }
}

fn handle_enter(app_state: &mut AppState, headers: &[&'static str], is_new_row: bool) {
    if let (Some(r), Some(c)) = (app_state.table_state.selected(), app_state.table_state.selected_column()) {
        let col_count = headers.len();
        let is_valid = !app_state.input_buffer.trim().is_empty();

        if is_valid {
            let input = std::mem::take(&mut app_state.input_buffer);
            
            let saved = match app_state.mode {
                AppMode::Clock if r < app_state.data.alarms.len() => {
                    match c {
                        0 => {
                            if let Some(time) = util::parse_time(&input) {
                                app_state.data.alarms[r].time = time;
                                true
                            } else {
                                app_state.input_buffer = input;
                                return;
                            }
                        }
                        1 => {
                            app_state.data.alarms[r].enabled = input == "✔";
                            true
                        }
                        2 => {
                            if let Some(repeat) = util::parse_repeat(&input) {
                                app_state.data.alarms[r].repeat = repeat;
                                true
                            } else {
                                app_state.input_buffer = input;
                                return;
                            }
                        }
                        3 => {
                            app_state.data.alarms[r].note = input;
                            true
                        }
                        _ => false,
                    }
                }
                AppMode::Countdown if r < app_state.data.presets.len() => {
                    match c {
                        0 => {
                            app_state.data.presets[r].name = input;
                            true
                        }
                        1 => {
                            app_state.data.presets[r].duration = input.parse().unwrap_or(0);
                            true
                        }
                        _ => false,
                    }
                }
                _ => false,
            };

                if saved {
                if let Err(e) = data::save_data(&app_state.data) {
                    eprintln!("Failed to save data: {}", e);
                }
                app_state.mark_dirty();

                if is_new_row && c < col_count - 1 {
                    app_state.table_state.select_column(Some(c + 1));
                } else {
                    app_state.edit_mode = EditMode::Normal;
                }
            }
        }
    }
}

fn handle_space(app_state: &mut AppState) {
    if let (Some(r), Some(c)) = (app_state.table_state.selected(), app_state.table_state.selected_column()) {
        if app_state.mode == AppMode::Clock && r < app_state.data.alarms.len() {
            if c == 1 {
                app_state.data.alarms[r].enabled = !app_state.data.alarms[r].enabled;
                app_state.input_buffer = if app_state.data.alarms[r].enabled {
                    "✔".to_string()
                } else {
                    "✗".to_string()
                };
                if let Err(e) = data::save_data(&app_state.data) {
                    eprintln!("Failed to save data: {}", e);
                }
                app_state.mark_dirty();
            } else if c == 2 {
                let next = app_state.data.alarms[r].repeat.next();
                app_state.data.alarms[r].repeat = next;
                app_state.input_buffer = next.to_string();
                if let Err(e) = data::save_data(&app_state.data) {
                    eprintln!("Failed to save data: {}", e);
                }
                app_state.mark_dirty();
            } else {
                app_state.input_buffer.push(' ');
            }
        } else {
            app_state.input_buffer.push(' ');
        }
    }
}

fn handle_escape(app_state: &mut AppState, is_new_row: bool) {
    if is_new_row {
        match app_state.mode {
            AppMode::Clock if !app_state.data.alarms.is_empty() => {
                app_state.data.alarms.pop();
                app_state.mark_dirty();
            }
            AppMode::Countdown if !app_state.data.presets.is_empty() => {
                app_state.data.presets.pop();
                app_state.mark_dirty();
            }
            _ => {}
        }
    }
    app_state.edit_mode = EditMode::Normal;
}
