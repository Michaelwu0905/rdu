use owo_colors::OwoColorize;
use ratatui::style::Color;

// 文件大小阈值常量
const GB: u64 = 1_000_000_000;
const MB_100: u64 = 100_000_000;
const MB_10: u64 = 10_000_000;
const MB_1: u64 = 1_000_000;

/// 根据百分比生成进度条字符串
/// BAR_WIDTH 默认为 20 个字符
pub fn create_progress_bar(percentage: f64) -> String {
    const BAR_WIDTH: usize = 20;
    let filled = ((percentage / 100.0) * BAR_WIDTH as f64).round() as usize;
    let filled = filled.min(BAR_WIDTH); // 确保不超过最大宽度

    let filled_part = "█".repeat(filled);
    let empty_part = "░".repeat(BAR_WIDTH - filled);

    format!("{}{}", filled_part, empty_part)
}

/// 对整行应用颜色，保持列对齐
/// 格式: "大小 │ 进度条 │ 占比 │ 文件名"
pub fn colorize_line(line: &str, size: u64) -> String {
    apply_color(line, size)
}

/// 应用颜色的内部函数
fn apply_color(text: &str, size: u64) -> String {
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

/// 根据文件大小返回 ratatui 颜色
/// 用于 TUI 模式
pub fn get_color_for_size(size: u64) -> Color {
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
