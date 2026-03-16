use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use std::{error::Error, io};

/// 定义所有的路由页面
#[derive(Copy, Clone, Debug, PartialEq)]
enum Route {
    Home,
    Profile,
    Settings,
}

impl Route {
    // 获取下一个路由（用于 Ctrl + Tab 循环切换）
    fn next(&self) -> Self {
        match self {
            Route::Home => Route::Profile,
            Route::Profile => Route::Settings,
            Route::Settings => Route::Home,
        }
    }

    // 获取页面标题
    fn title(&self) -> &'static str {
        match self {
            Route::Home => "Home",
            Route::Profile => "Profile",
            Route::Settings => "Settings",
        }
    }
}

/// 应用程序的全局状态
struct App {
    route: Route,
    should_quit: bool,
}

impl App {
    fn new() -> App {
        App {
            route: Route::Home,
            should_quit: false,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. 初始化终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. 创建应用状态并运行主循环
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // 3. 恢复终端原状
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

/// 主事件循环
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()>
where
    std::io::Error: From<<B as Backend>::Error>,
{
    loop {
        // 渲染 UI
        terminal.draw(|f| ui(f, &app))?;

        // 处理键盘事件
        if let Event::Key(key) = event::read()? {
            // 确保只在按下时触发（忽略释放事件）
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    // 按 'q' 退出
                    KeyCode::Char('q') => app.should_quit = true,
                    // 按 Ctrl + C 退出
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.should_quit = true;
                    }
                    // 按 Ctrl + Tab 切换到下一个路由
                    KeyCode::Tab if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.route = app.route.next();
                    }
                    // 按普通的 Tab 也可以切换（为了防止部分终端吃掉 Ctrl+Tab）
                    KeyCode::Tab => {
                        app.route = app.route.next();
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// 顶层 UI 构建器
fn ui(f: &mut Frame, app: &App) {
    // 将整个屏幕分为上下两部分：顶部导航栏 (Tabs) 和 底部的页面内容区
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 顶部 Tabs 高度为 3
            Constraint::Min(0),    // 剩下的所有空间给页面内容
        ])
        .split(f.area()); // [修复 Warning]：size() 替换为官方推荐的 area()

    // 渲染顶部的导航栏（类似浏览器的 Tab）
    let titles: Vec<_> = vec![Route::Home, Route::Profile, Route::Settings]
        .iter()
        .map(|r| Line::from(r.title()))
        .collect();

    let active_index = match app.route {
        Route::Home => 0,
        Route::Profile => 1,
        Route::Settings => 2,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Navigation "))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .select(active_index);

    f.render_widget(tabs, chunks[0]);

    // 核心路由分发：根据当前的状态，渲染对应的页面组件
    match app.route {
        Route::Home => render_home_page(f, chunks[1]),
        Route::Profile => render_profile_page(f, chunks[1]),
        Route::Settings => render_settings_page(f, chunks[1]),
    }
}

fn render_home_page(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Home Page ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    let content = Paragraph::new(
        "Welcome to the Home Page!\n\n\
        Press [Ctrl + Tab] or [Tab] to switch routes.\n\
        Press [q] to quit.",
    )
    .block(block);

    f.render_widget(content, area);
}

fn render_profile_page(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Profile Page ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let content = Paragraph::new(
        "This is the Profile Page.\n\n\
        Username: Rustacean\n\
        Role: Developer",
    )
    .block(block);

    f.render_widget(content, area);
}

fn render_settings_page(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Settings Page ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));

    let content = Paragraph::new(
        "Settings Configuration:\n\n\
        [x] Enable Dark Mode\n\
        [ ] Enable Notifications\n\
        [x] Auto-Save",
    )
    .block(block);

    f.render_widget(content, area);
}