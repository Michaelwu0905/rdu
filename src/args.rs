use std::path::PathBuf;

use clap::Parser;

/// 命令行参数结构
/// 使用 clap 自动处理 --help 和输入参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// 要分析的路径 (默认为当前目录)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// 启用 TUI 交互模式
    #[arg(long)]
    pub tui: bool,
}

impl Args {
    /// 解析命令行参数
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
