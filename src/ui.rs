use figlet_rs::FIGlet;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
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
}

pub fn draw(f: &mut Frame, data: &UiData) {
    let area = f.area();

    // 1. Draw global background
    f.render_widget(
        Block::default().style(Style::default().bg(data.bg_color)),
        area,
    );

    // 2. Vertical centering layout - footer always at bottom
    let footer_height = 1;
    let center_content_height = 10;

    let main_area_height = area.height.saturating_sub(footer_height);
    let main_area = Rect::new(area.x, area.y, area.width, main_area_height);

    let vertical_chunks = if data.show_panel {
        let remaining = main_area_height - center_content_height;
        let panel_height = (remaining * 50) / 100;
        let top_space = remaining - panel_height;

        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(top_space),
                Constraint::Length(center_content_height),
                Constraint::Length(panel_height),
            ])
            .split(main_area)
    } else {
        let remaining = main_area_height - center_content_height;
        let top_space = remaining / 2;
        let bottom_space = remaining - top_space;

        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(top_space),
                Constraint::Length(center_content_height),
                Constraint::Length(bottom_space),
            ])
            .split(main_area)
    };

    // 3. Generate ASCII art
    let figure = data.font.convert(data.time_str).unwrap();
    let ascii_art = figure.to_string();

    // 4. Draw center area (ASCII art + Subtitle)
    let center_text = format!("{}\n{}", ascii_art, data.subtitle_str);
    let center_paragraph = Paragraph::new(center_text)
        .style(Style::default().fg(data.clock_color).bg(data.bg_color))
        .alignment(Alignment::Center);

    f.render_widget(center_paragraph, vertical_chunks[1]);

    // 5. Draw panel (if visible)
    if data.show_panel {
        let panel_block = Block::default()
            .title("Panel")
            .borders(ratatui::widgets::Borders::ALL)
            .style(Style::default().fg(data.clock_color).bg(data.bg_color));
        f.render_widget(panel_block, vertical_chunks[2]);
    }

    // 6. Draw footer hints (always at bottom)
    let footer_area = Rect::new(area.x, area.height - 1, area.width, 1);
    let footer_paragraph = Paragraph::new(data.footer_str)
        .style(Style::default().fg(Color::DarkGray).bg(data.bg_color))
        .alignment(Alignment::Right);

    f.render_widget(footer_paragraph, footer_area);
}
