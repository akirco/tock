use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, TableState};
use ratatui::Frame;

// 定义状态机，用于区分当前操作状态
#[derive(PartialEq)]
enum AppMode {
    Normal,
    Typing {
        is_new_row: bool, // 是否正在“新增数据行”的流程中
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut table_state = TableState::default();
    table_state.select_first();
    table_state.select_first_column();

    let mut items = vec![
        vec!["Eggplant".into(), "1 medium".into(), "25 kcal, 6g carbs, 1g protein".into()],
        vec!["Tomato".into(), "2 large".into(), "44 kcal, 10g carbs, 2g protein".into()],
        vec!["Zucchini".into(), "1 medium".into(), "33 kcal, 7g carbs, 2g protein".into()],
    ];

    let mut mode = AppMode::Normal;
    let mut input_buffer = String::new();

    ratatui::run(|terminal| loop {
        terminal.draw(|frame| render(frame, &mut table_state, &items, &mode, &input_buffer))?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => table_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => table_state.select_previous(),
                    KeyCode::Char('l') | KeyCode::Right => table_state.select_next_column(),
                    KeyCode::Char('h') | KeyCode::Left => table_state.select_previous_column(),
                    KeyCode::Char('g') => table_state.select_first(),
                    KeyCode::Char('G') => table_state.select_last(),

                    KeyCode::Char('a') => {
                        // 1. 插入一行空数据
                        items.push(vec![String::new(), String::new(), String::new()]);
                        // 2. 跳转到最新行的第一列
                        table_state.select(Some(items.len() - 1));
                        table_state.select_column(Some(0));
                        input_buffer.clear();
                        mode = AppMode::Typing { is_new_row: true };
                    }
                    KeyCode::Char('e') => {
                        if let (Some(r), Some(c)) = (table_state.selected(), table_state.selected_column()) {
                            if r < items.len() && c < items[r].len() {
                                // 提取原本的数据到缓冲区用于编辑
                                input_buffer = items[r][c].clone();
                                mode = AppMode::Typing { is_new_row: false };
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(i) = table_state.selected() {
                            if !items.is_empty() {
                                items.remove(i);
                                if i >= items.len() && !items.is_empty() {
                                    table_state.select(Some(items.len() - 1));
                                } else if items.is_empty() {
                                    table_state.select(None);
                                }
                            }
                        }
                    }
                    _ => {}
                },
                AppMode::Typing { is_new_row } => match key.code {
                    KeyCode::Enter => {
                        if let (Some(r), Some(c)) = (table_state.selected(), table_state.selected_column()) {
                            // 将输入框内容保存回原数据
                            items[r][c] = input_buffer.clone();

                            if is_new_row && c < 2 {
                                // 如果是“新增行”流程，且没到最后一列 -> 自动跳到下一列继续输入
                                table_state.select_column(Some(c + 1));
                                input_buffer.clear();
                            } else {
                                // 新增到最后一列，或者只是单独编辑一个单元格 -> 结束编辑
                                mode = AppMode::Normal;
                            }
                        }
                    }
                    KeyCode::Esc => {
                        // 如果在新增流程中按了 Esc，意味着放弃新增，直接删掉这一行
                        if is_new_row {
                            if let Some(r) = table_state.selected() {
                                items.remove(r);
                                table_state.select(if items.is_empty() { None } else { Some(r.saturating_sub(1)) });
                            }
                        }
                        mode = AppMode::Normal;
                    }
                    KeyCode::Backspace => {
                        input_buffer.pop();
                    }
                    KeyCode::Char(ch) => {
                        input_buffer.push(ch);
                    }
                    _ => {}
                },
            }
        }
    })
}

fn render(
    frame: &mut Frame,
    table_state: &mut TableState,
    items: &[Vec<String>],
    mode: &AppMode,
    input_buffer: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    let header = Row::new(["Ingredient", "Quantity", "Macros"])
        .style(Style::new().bold())
        .bottom_margin(1);

    let (sel_r, sel_c) = (table_state.selected(), table_state.selected_column());
    let is_typing = matches!(mode, AppMode::Typing { .. });

    // 构建视图行时，动态劫持正在编辑的那个单元格
    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(r_idx, item)| {
            let cells: Vec<String> = item
                .iter()
                .enumerate()
                .map(|(c_idx, s)| {
                    // 如果这个单元格正好是被选中的并且在输入模式下：直接渲染 input_buffer + 光标
                    if is_typing && Some(r_idx) == sel_r && Some(c_idx) == sel_c {
                        format!("{}█", input_buffer)
                    } else {
                        // 否则渲染底层真实数据
                        s.clone()
                    }
                })
                .collect();
            Row::new(cells)
        })
        .collect();

    let widths =[
        Constraint::Percentage(30),
        Constraint::Percentage(20),
        Constraint::Percentage(50),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .style(Color::White)
        .row_highlight_style(Style::new().on_black().bold())
        .column_highlight_style(Color::Gray)
        // 被选中单元格的高亮样式
        .cell_highlight_style(Style::new().reversed().yellow())
        .highlight_symbol("❱ ");

    frame.render_stateful_widget(table, chunks[0], table_state);

    // 底部状态提示条
    let help_text = match mode {
        AppMode::Normal => " 'a' Add Row | 'd' Delete Row | 'e' Edit Cell | 'q' Quit | h/j/k/l Navigate ",
        AppMode::Typing { is_new_row: true } => " [Adding Row] Type text. Press <Enter> to move to next column, <Esc> to abort. ",
        AppMode::Typing { is_new_row: false } => " [Editing Cell] Type text. Press <Enter> to save, <Esc> to cancel. ",
    };

    let help_widget = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(if is_typing { " Typing Mode " } else { " Normal Mode " })
            .border_style(Style::default().fg(if is_typing { Color::Yellow } else { Color::DarkGray })),
    );

    frame.render_widget(help_widget, chunks[1]);
}