use std::path::PathBuf;

use humansize::{format_size, DECIMAL};

use crate::scanner::{calculate_total_size, scan_directory};
use crate::utils::{colorize_line, create_progress_bar};

/// 运行 CLI 模式
pub fn run_cli(root_path: PathBuf) {
    println!("正在分析目录: {:?} (启用多线程加速...)", root_path);

    // 扫描目录
    let items = scan_directory(&root_path);

    if items.is_empty() {
        println!("目录为空或无法访问");
        return;
    }

    // 计算总大小
    let total_size = calculate_total_size(&items);

    // 打印表头
    println!(
        "{:>12} │ {:<20} │ {:>7} │ {}",
        "大小", "进度条", "占比", "文件/目录名"
    );
    println!("{}", "─".repeat(70));

    // 打印每个项目
    for item in items {
        let human_size = format_size(item.size, DECIMAL);

        // 计算百分比
        let percentage = if total_size > 0 {
            (item.size as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        // 生成进度条
        let bar = create_progress_bar(percentage);

        // 格式化各列（不带颜色）
        let size_col = format!("{:>12}", human_size);
        let bar_col = format!("{:<20}", bar);
        let pct_col = format!("{:>6.1}%", percentage);
        let name_col = &item.name;

        // 对大小和文件名列着色
        let colored_size = colorize_line(&size_col, item.size);
        let colored_name = colorize_line(name_col, item.size);

        println!(
            "{} │ {} │ {} │ {}",
            colored_size, bar_col, pct_col, colored_name
        );
    }
}
