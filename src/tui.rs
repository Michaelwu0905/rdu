use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use humansize::{format_size, DECIMAL};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use crate::app::App;
use crate::utils::{create_progress_bar, get_color_for_size};

/// 运行 TUI 模式
pub fn run_tui(root_path: std::path::PathBuf) -> Result<(), io::Error> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建应用状态
    let mut app = App::new(root_path);

    // 主循环
    let res = run_app(&mut terminal, &mut app);

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("错误: {:?}", err);
    }

    Ok(())
}

/// 应用主循环
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.quit();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous();
                    }
                    KeyCode::Enter => {
                        app.enter_directory();
                    }
                    KeyCode::Backspace => {
                        app.go_up();
                    }
                    KeyCode::Char('r') => {
                        app.refresh();
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

/// UI 渲染函数
fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 头部
            Constraint::Min(0),    // 列表
            Constraint::Length(3), // 底部
        ])
        .split(f.area());

    // 头部 - 显示当前路径
    let header = Paragraph::new(format!("Path: {}", app.current_path.display())).block(
        Block::default()
            .borders(Borders::ALL)
            .title("RDU - Rust Disk Usage"),
    );
    f.render_widget(header, chunks[0]);

    // 列表 - 显示文件/目录
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|item| {
            let percentage = if app.total_size > 0 {
                (item.size as f64 / app.total_size as f64) * 100.0
            } else {
                0.0
            };

            let bar = create_progress_bar(percentage);
            let human_size = format_size(item.size, DECIMAL);
            let color = get_color_for_size(item.size);

            let dir_indicator = if item.is_dir { "📁 " } else { "📄 " };
            let content = format!(
                "{}{:<15} {} {:>6.1}%  {}",
                dir_indicator, human_size, bar, percentage, item.name
            );

            ListItem::new(Line::from(Span::styled(
                content,
                Style::default().fg(color),
            )))
        })
        .collect();

    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_style(
            Style::default()
                .bg(ratatui::style::Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items_list, chunks[1], &mut app.list_state);

    // 底部 - 显示快捷键提示
    let help = Paragraph::new(
        "↑/↓ or j/k: Navigate | Enter: Open | Backspace: Up | r: Refresh | q/Esc: Quit",
    )
    .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[2]);
}
