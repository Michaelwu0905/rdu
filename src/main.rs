mod app;
mod args;
mod cli;
mod scanner;
mod tui;
mod utils;

use args::Args;

fn main() {
    // 解析参数
    let args = Args::parse_args();
    let root_path = args.path;

    // 根据参数选择运行模式
    if args.tui {
        // TUI 模式
        if let Err(e) = tui::run_tui(root_path) {
            eprintln!("TUI 错误: {}", e);
            std::process::exit(1);
        }
    } else {
        // CLI 模式
        cli::run_cli(root_path);
    }
}
