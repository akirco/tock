use colorgrad::Gradient;
use figlet_rs::FIGlet;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

// 生成渐变色时钟 Paragraph 的核心函数
pub fn get_horizontal_gradient_clock(time_str: &str) -> Paragraph<'static> {
    // 1. 生成 ASCII 艺术字 (使用 FIGlet)
    let standard_font = FIGlet::standard().unwrap();
    let figure = standard_font.convert(time_str).unwrap();
    let ascii_art = figure.to_string();

    // 2. 显式添加 `|l: &str|` 解决闭包类型推断错误
    let max_width = ascii_art
        .lines()
        .map(|l: &str| l.chars().count())
        .max()
        .unwrap_or(1);

    // 3. 使用 preset 模块下的彩虹色
    let gradient = colorgrad::preset::rainbow();

    let mut text_lines = Vec::new();

    for line in ascii_art.lines() {
        let mut spans = Vec::new();

        for (j, ch) in line.chars().enumerate() {
            let ratio = j as f32 / max_width as f32;

            // 取出 [R, G, B, A]
            let color = gradient.at(ratio).to_rgba8();
            let ratatui_color = Color::Rgb(color[0], color[1], color[2]);

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(ratatui_color),
            ));
        }

        text_lines.push(Line::from(spans));
    }

    Paragraph::new(Text::from(text_lines))
}


fn main() {

  ratatui::run(|terminal| loop {
  let p = get_horizontal_gradient_clock("12:34:56");

    // 如果想要在这个 example 里直接看到效果，可以将这段代码接入你的 Ratatui 渲染流程
    terminal.draw(|f| {
        f.render_widget(p, f.area());
    }).unwrap();

  });

    println!("编译成功！函数已可以正常使用。");
}