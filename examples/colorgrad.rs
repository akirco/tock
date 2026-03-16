use figlet_rs::FIGfont;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};
use colorgrad;

pub fn get_horizontal_gradient_clock(time_str: &str) -> Paragraph<'static> {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert(time_str).unwrap();
    let ascii_art = figure.to_string();

    // 找出最长的一行，用来计算水平比例
    let max_width = ascii_art.lines().map(|l| l.chars().count()).max().unwrap_or(1);

    // 生成一个内置的彩虹渐变（也可以自定义色标）
    let gradient = colorgrad::rainbow();

    let mut text_lines = Vec::new();

    for line in ascii_art.lines() {
        let mut spans = Vec::new();

        // 遍历这行的每个字符
        for (j, ch) in line.chars().enumerate() {
            // 计算水平位置比例
            let ratio = j as f64 / max_width as f64;

            // 从渐变带中取颜色
            let color = gradient.at(ratio).to_rgba8();

            let ratatui_color = Color::Rgb(color[0], color[1], color[2]);

            // 将单个字符作为一个带颜色的 Span
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(ratatui_color),
            ));
        }

        // 把所有的字符拼接成一行
        text_lines.push(Line::from(spans));
    }

    Paragraph::new(Text::from(text_lines))
}