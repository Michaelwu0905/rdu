# rdu

A fast disk usage analyzer written in Rust with colored output and visual progress bars.

English | [简体中文](README_CN.md)

## Features

- Parallel directory scanning using multi-threading
- Two modes: CLI (default) and interactive TUI
- Color-coded output based on file sizes
- Visual progress bars showing space usage percentage
- Human-readable file sizes (KB, MB, GB)
- Interactive directory navigation (TUI mode)
- Simple command-line interface

## Installation

### From Source

```bash
git clone https://github.com/Michaelwu0905/rdu.git
cd rdu
cargo build --release
```

The binary will be available at `target/release/rdu`.

### Install Locally

```bash
cargo install --path .
```

## Usage

### CLI Mode (Default)

Analyze current directory:
```bash
rdu
```

Analyze specific directory:
```bash
rdu /path/to/directory
```

Analyze home directory:
```bash
rdu ~
```

### TUI Mode (Interactive)

Launch interactive interface:
```bash
rdu --tui
```

Launch TUI in specific directory:
```bash
rdu --tui /path/to/directory
```

#### TUI Keyboard Controls

- `↑/↓` or `j/k`: Navigate up/down
- `Enter`: Enter selected directory
- `Backspace`: Go to parent directory
- `r`: Refresh current directory
- `q` or `Esc`: Quit

## Output Format

```
大小              进度条                    占比       文件/目录名
──────────────────────────────────────────────────────────────────────
271.26 GB ██████████████████░░   88.5%  Library
14.61 GB █░░░░░░░░░░░░░░░░░░░    4.8%  Desktop
3.28 GB ░░░░░░░░░░░░░░░░░░░░    1.1%  Downloads
```

### Color Scheme

- Red: Files larger than 1 GB
- Yellow: Files between 100 MB and 1 GB
- Green: Files between 10 MB and 100 MB
- Cyan: Files between 1 MB and 10 MB
- White: Files smaller than 1 MB

## Dependencies

- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- [walkdir](https://github.com/BurntSushi/walkdir) - Recursive directory traversal
- [rayon](https://github.com/rayon-rs/rayon) - Data parallelism
- [humansize](https://github.com/LeopoldArkham/humansize) - Human-readable file sizes
- [owo-colors](https://github.com/jam1garner/owo-colors) - Terminal colors
- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation

## Requirements

- Rust 1.70 or higher

## License

MIT

## Author

Michael Wu
