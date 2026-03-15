use figlet_rs::FIGlet;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
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
    pub panel_title: String,
}

pub fn draw(f: &mut Frame, data: &UiData) {
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

    // Split main area for vertical centering
    let (content_area, panel_area) = if data.show_panel {
        // Panel mode: two-step layout
        // Step 1: Split main_area into content_container and panel_area
        let panel_ratio = data.panel_ratio.clamp(1, 99) as u16;
        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(100 - panel_ratio), // content container
                Constraint::Percentage(panel_ratio),       // panel
            ])
            .split(main_area);

        let content_container = outer_chunks[0];
        let panel_area = outer_chunks[1];

        // Step 2: Center content in content_container
        let container_height = content_container.height as usize;
        let content_needed = 10; // ASCII art ~6-8 lines + subtitle ~2 lines

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
        // No panel: create three equal sections for vertical centering
        let third = main_area_height / 3;
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(third),
                Constraint::Length(third),
                Constraint::Length(third),
            ])
            .split(main_area);
        (vertical_chunks[1], None)
    };

    // 3. Generate ASCII art
    let figure = data.font.convert(data.time_str).unwrap();
    let ascii_art = figure.to_string();

    // 4. Draw center area (ASCII art + Subtitle)
    let center_text = format!("{}\n{}", ascii_art, data.subtitle_str);
    let center_paragraph = Paragraph::new(center_text)
        .style(Style::default().fg(data.clock_color).bg(data.bg_color))
        .alignment(Alignment::Center);

    f.render_widget(center_paragraph, content_area);

    // 5. Draw panel (if visible)
    if let Some(panel_area) = panel_area {
        let panel_block = Block::default()
            .title(data.panel_title.as_str())
            .title_alignment(Alignment::Center)
            .borders(data.panel_border_sides)
            .border_type(data.panel_border_style)
            .style(Style::default().fg(data.panel_fg).bg(data.panel_bg))
            .border_style(Style::default().fg(data.panel_border));
        f.render_widget(panel_block, panel_area);
    }

    // 6. Draw footer hints (always at bottom)
    let footer_area = Rect::new(area.x, area.height - 1, area.width, 1);
    let footer_paragraph = Paragraph::new(data.footer_str)
        .style(Style::default().fg(Color::DarkGray).bg(data.bg_color))
        .alignment(Alignment::Right);

    f.render_widget(footer_paragraph, footer_area);
}
