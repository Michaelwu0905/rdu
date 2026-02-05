use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use humansize::{format_size, DECIMAL};
use owo_colors::OwoColorize;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use rayon::prelude::*;
use std::fs;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;

// 1. 定义命令行参数
// 使用 clap 自动处理 --help 和输入参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 要分析的路径 (默认为当前目录)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// 启用 TUI 交互模式
    #[arg(long)]
    tui: bool,
}

fn main() {
    // 解析参数
    let args = Args::parse();
    let root_path = args.path;

    // 根据参数选择运行模式
    if args.tui {
        // TUI 模式
        if let Err(e) = run_tui(root_path) {
            eprintln!("TUI 错误: {}", e);
            std::process::exit(1);
        }
    } else {
        // CLI 模式
        run_cli(root_path);
    }
}

// CLI 模式的原有逻辑
fn run_cli(root_path: PathBuf) {
    println!("正在分析目录: {:?} (启用多线程加速...)", root_path);

    // 2. 获取第一层级的子目录/文件
    // 我们只针对第一层做并行，每一层内部递归计算
    let entries = match fs::read_dir(&root_path) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("错误: 无法读取目录 - {}", err);
            return;
        }
    };

    // 把目录项收集到一个 Vec 中，以便让 Rayon 进行并行处理
    let paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();

    // 3. 【核心魔法】使用 Rayon 并行计算
    // .par_iter() 替代了 .iter() -> 这会自动利用你所有的 CPU 核心
    let mut sizes: Vec<(u64, PathBuf)> = paths
        .par_iter()
        .map(|path| {
            // 对每个子目录/文件计算大小
            let size = get_dir_size(path);
            (size, path.clone())
        })
        .collect();

    // 4. 排序与输出
    // 按大小降序排列 (大的在上面)
    sizes.sort_by(|a, b| b.0.cmp(&a.0));

    // 计算总大小，用于百分比计算
    let total_size: u64 = sizes.iter().map(|(size, _)| size).sum();

    println!(
        "{:<15} {:<22} {:<8} {}",
        "大小", "进度条", "占比", "文件/目录名"
    );
    println!("{}", "─".repeat(70));

    for (size, path) in sizes {
        // 使用 humansize 库把字节变成易读的格式 (如 1.5 GB)
        let human_size = format_size(size, DECIMAL);
        let name = path.file_name().unwrap_or_default().to_string_lossy();

        // 计算百分比
        let percentage = if total_size > 0 {
            (size as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        // 生成进度条
        let bar = create_progress_bar(percentage);

        // 根据大小着色
        let colored_size = colorize_size(&human_size, size);
        let colored_name = colorize_size(&name, size);

        println!(
            "{:<15} {} {:>6.1}%  {}",
            colored_size, bar, percentage, colored_name
        );
    }
}

// 辅助函数：递归计算指定路径的总大小
fn get_dir_size(path: &PathBuf) -> u64 {
    // 如果是文件，直接返回大小
    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }

    // 如果是目录，使用 WalkDir 递归遍历
    WalkDir::new(path)
        .into_iter()
        // filter_map 会自动忽略那些没有权限访问的文件 (Result::Err)
        // 这体现了 Rust 的 Option/Result 处理优势
        .filter_map(|entry| entry.ok())
        // 只关心文件，不加目录本身的大小（避免某些系统下的干扰）
        .filter(|entry| entry.file_type().is_file())
        // 获取每个文件的大小，如果获取失败（比如文件刚好被删了）就当作0
        .map(|entry| entry.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

// 辅助函数：生成进度条
// 根据百分比生成 20 字符宽的进度条
fn create_progress_bar(percentage: f64) -> String {
    const BAR_WIDTH: usize = 20;
    let filled = ((percentage / 100.0) * BAR_WIDTH as f64).round() as usize;
    let filled = filled.min(BAR_WIDTH); // 确保不超过最大宽度

    let filled_part = "█".repeat(filled);
    let empty_part = "░".repeat(BAR_WIDTH - filled);

    format!("{}{}", filled_part, empty_part)
}

// 辅助函数：根据文件大小着色
// 大文件用红色/黄色，小文件用绿色/青色
fn colorize_size(text: &str, size: u64) -> String {
    const GB: u64 = 1_000_000_000;
    const MB_100: u64 = 100_000_000;
    const MB_10: u64 = 10_000_000;
    const MB_1: u64 = 1_000_000;

    if size >= GB {
        text.bright_red().to_string()
    } else if size >= MB_100 {
        text.yellow().to_string()
    } else if size >= MB_10 {
        text.green().to_string()
    } else if size >= MB_1 {
        text.cyan().to_string()
    } else {
        text.white().to_string()
    }
}

// ============================================================================
// TUI 模式实现
// ============================================================================

// 目录项结构
#[derive(Clone)]
struct DirEntry {
    path: PathBuf,
    name: String,
    size: u64,
    is_dir: bool,
}

// 应用状态
struct App {
    current_path: PathBuf,
    items: Vec<DirEntry>,
    total_size: u64,
    selected: usize,
    list_state: ListState,
    should_quit: bool,
}

impl App {
    fn new(path: PathBuf) -> Self {
        let mut app = App {
            current_path: path.clone(),
            items: Vec::new(),
            total_size: 0,
            selected: 0,
            list_state: ListState::default(),
            should_quit: false,
        };
        app.load_directory(path);
        app
    }

    fn load_directory(&mut self, path: PathBuf) {
        self.current_path = path.clone();
        self.items.clear();
        self.selected = 0;

        // 读取目录内容
        let entries = match fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => return,
        };

        let paths: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

        // 并行计算大小
        let mut items: Vec<DirEntry> = paths
            .par_iter()
            .map(|p| {
                let size = get_dir_size(p);
                let name = p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let is_dir = p.is_dir();
                DirEntry {
                    path: p.clone(),
                    name,
                    size,
                    is_dir,
                }
            })
            .collect();

        // 按大小排序
        items.sort_by(|a, b| b.size.cmp(&a.size));

        self.total_size = items.iter().map(|item| item.size).sum();
        self.items = items;

        // 更新列表状态
        if !self.items.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected = i;
    }

    fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected = i;
    }

    fn enter_directory(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if let Some(selected) = self.list_state.selected() {
            if selected < self.items.len() {
                let item = &self.items[selected];
                if item.is_dir {
                    self.load_directory(item.path.clone());
                }
            }
        }
    }

    fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.load_directory(parent.to_path_buf());
        }
    }

    fn refresh(&mut self) {
        let path = self.current_path.clone();
        self.load_directory(path);
    }
}

// TUI 主函数
fn run_tui(root_path: PathBuf) -> Result<(), io::Error> {
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

// 应用主循环
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
                        app.should_quit = true;
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

// UI 渲染
fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 头部
            Constraint::Min(0),     // 列表
            Constraint::Length(3),  // 底部
        ])
        .split(f.area());

    // 头部 - 显示当前路径
    let header = Paragraph::new(format!("Path: {}", app.current_path.display()))
        .block(Block::default().borders(Borders::ALL).title("RDU - Rust Disk Usage"));
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

            ListItem::new(Line::from(Span::styled(content, Style::default().fg(color))))
        })
        .collect();

    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items_list, chunks[1], &mut app.list_state);

    // 底部 - 显示快捷键提示
    let help = Paragraph::new("↑/↓ or j/k: Navigate | Enter: Open | Backspace: Up | r: Refresh | q/Esc: Quit")
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[2]);
}

// 根据文件大小获取颜色
fn get_color_for_size(size: u64) -> Color {
    const GB: u64 = 1_000_000_000;
    const MB_100: u64 = 100_000_000;
    const MB_10: u64 = 10_000_000;
    const MB_1: u64 = 1_000_000;

    if size >= GB {
        Color::Red
    } else if size >= MB_100 {
        Color::Yellow
    } else if size >= MB_10 {
        Color::Green
    } else if size >= MB_1 {
        Color::Cyan
    } else {
        Color::White
    }
}
