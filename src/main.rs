use clap::Parser;
use humansize::{format_size, DECIMAL};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use std::fs;
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
}

fn main() {
    // 解析参数
    let args = Args::parse();   // 解析命令行，创建Args实例
    let root_path = args.path;

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
