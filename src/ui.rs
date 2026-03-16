use crate::state::{AppMode, EditMode};
use figlet_rs::FIGlet;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, TableState},
    Frame,
};

pub struct UiData<'a> {
    pub font: &'a FIGlet,
    pub time_str: &'a str,
    pub subtitle_str: &'a str,
    pub footer_str: &'a str,
    pub bg_color: Color,
    pub clock_color: Color,
    pub show_panel: bool,
    pub panel_ratio: u8,
    pub panel_bg: Color,
    pub panel_fg: Color,
    pub panel_border: Color,
    pub panel_border_sides: Borders,
    pub panel_border_style: BorderType,
    pub panel_title: &'a str,
    pub mode: AppMode,
    pub items: &'a [Vec<String>],
    pub headers: &'a [&'static str],
    pub table_state: &'a mut TableState,
    pub edit_mode: &'a EditMode,
    pub input_buffer: &'a str,
}

fn build_layout(area: Rect, show_panel: bool, panel_ratio: u8) -> (Rect, Option<Rect>) {
    if show_panel {
        let panel_ratio = panel_ratio.clamp(1, 99) as u16;
        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(100 - panel_ratio),
                Constraint::Percentage(panel_ratio),
            ])
            .split(area);

        let content_container = outer_chunks[0];
        let panel_area = outer_chunks[1];

        let container_height = content_container.height as usize;
        let content_needed = 10;

        let content_area = if container_height <= content_needed {
            content_container
        } else {
            let remaining = container_height - content_needed;
            let top_space = remaining / 2;

            let inner_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(top_space as u16),
                    Constraint::Length(content_needed as u16),
                    Constraint::Length((remaining - top_space) as u16),
                ])
                .split(content_container);
            inner_chunks[1]
        };
        (content_area, Some(panel_area))
    } else {
        let third = area.height / 3;
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(third),
                Constraint::Length(third),
                Constraint::Length(third),
            ])
            .split(area);
        (vertical_chunks[1], None)
    }
}

pub fn draw(f: &mut Frame, data: &mut UiData) {
    let area = f.area();

    // 1. Draw global background
    f.render_widget(
        Block::default().style(Style::default().bg(data.bg_color)),
        area,
    );

    // 2. Vertical layout - footer always at bottom (1 row)
    let footer_height = 1;
    let main_area_height = area.height.saturating_sub(footer_height);
    let main_area = Rect::new(area.x, area.y, area.width, main_area_height);

    let (content_area, panel_area) = build_layout(main_area, data.show_panel, data.panel_ratio);

    // 3. Generate ASCII art
    let figure = data.font.convert(data.time_str).unwrap();
    let ascii_art = figure.to_string();

    // 4. Draw center area (ASCII art + Subtitle)
    let center_text = format!("{}\n{}", ascii_art, data.subtitle_str);
    let center_paragraph = Paragraph::new(center_text)
        .style(Style::default().fg(data.clock_color).bg(data.bg_color))
        .alignment(Alignment::Center);

    f.render_widget(center_paragraph, content_area);

    if let Some(panel_area) = panel_area {
        let panel_block = Block::default()
            .title(data.panel_title)
            .title_alignment(Alignment::Center)
            .borders(data.panel_border_sides)
            .border_type(data.panel_border_style)
            .style(Style::default().fg(data.panel_fg).bg(data.panel_bg))
            .border_style(Style::default().fg(data.panel_border));
        f.render_widget(panel_block, panel_area);

        draw_table(f, panel_area, data);
    }

    // 6. Draw footer hints (always at bottom)
    let footer_area = Rect::new(area.x, area.height - 1, area.width, 1);
    let footer_paragraph = Paragraph::new(data.footer_str)
        .style(Style::default().fg(Color::DarkGray).bg(data.bg_color))
        .alignment(Alignment::Right);

    f.render_widget(footer_paragraph, footer_area);
}

fn draw_table(f: &mut Frame, area: Rect, data: &mut UiData) {
    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let header = Row::new(
        data.headers
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    )
    .style(Style::new().bold())
    .bottom_margin(1);

    let (sel_r, sel_c) = (
        data.table_state.selected(),
        data.table_state.selected_column(),
    );
    let is_typing = matches!(data.edit_mode, EditMode::Typing { .. });

    let col_count = data.headers.len();
    let widths: Vec<Constraint> = (0..col_count)
        .map(|_| Constraint::Percentage(100 / col_count as u16))
        .collect();

    let rows: Vec<Row> = data
        .items
        .iter()
        .enumerate()
        .map(|(r_idx, item)| {
            let cells: Vec<String> = item
                .iter()
                .enumerate()
                .map(|(c_idx, s)| {
                    if is_typing && Some(r_idx) == sel_r && Some(c_idx) == sel_c {
                        format!("{}█", data.input_buffer)
                    } else {
                        s.to_string()
                    }
                })
                .collect();
            Row::new(cells)
        })
        .collect();

    let cell_style = if is_typing {
        Style::new().reversed().light_blue()
    } else {
        Style::new().reversed().dark_gray()
    };

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .style(Color::White)
        .row_highlight_style(Style::new().on_black().bold())
        .cell_highlight_style(cell_style)
        .highlight_symbol("❱ ");

    f.render_stateful_widget(table, chunks[0], data.table_state);

    let help_text = match data.edit_mode {
        EditMode::Normal => {
            if data.mode == AppMode::Stopwatch {
                " 'd' Delete | ↑↓←→ Navigate | g/G First/Last "
            } else {
                " 'a' Add | 'e' Edit | 'd' Delete | ↑↓←→ Navigate | g/G First/Last "
            }
        }
        EditMode::Typing { is_new_row: true } => {
            " [Adding] Enter: next field | Esc: abort | Space: toggle "
        }
        EditMode::Typing { is_new_row: false } => {
            " [Editing] Enter: save | Esc: cancel | Space: toggle "
        }
    };

    let help_widget = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(help_widget, chunks[1]);
}
