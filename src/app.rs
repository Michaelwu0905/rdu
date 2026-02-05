use std::path::PathBuf;

use ratatui::widgets::ListState;

use crate::scanner::{calculate_total_size, scan_directory, DirEntry};

/// TUI 应用状态结构
pub struct App {
    pub current_path: PathBuf,
    pub items: Vec<DirEntry>,
    pub total_size: u64,
    pub selected: usize,
    pub list_state: ListState,
    pub should_quit: bool,
}

impl App {
    /// 创建新的应用实例
    pub fn new(path: PathBuf) -> Self {
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

    /// 加载指定目录的内容
    pub fn load_directory(&mut self, path: PathBuf) {
        self.current_path = path.clone();
        self.items = scan_directory(&path);
        self.total_size = calculate_total_size(&self.items);
        self.selected = 0;

        // 更新列表状态
        if !self.items.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    /// 选择下一项
    pub fn next(&mut self) {
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

    /// 选择上一项
    pub fn previous(&mut self) {
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

    /// 进入选中的目录
    pub fn enter_directory(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.items.len() {
                let item = &self.items[selected];
                if item.is_dir {
                    self.load_directory(item.path.clone());
                }
            }
        }
    }

    /// 返回上级目录
    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.load_directory(parent.to_path_buf());
        }
    }

    /// 刷新当前目录
    pub fn refresh(&mut self) {
        let path = self.current_path.clone();
        self.load_directory(path);
    }

    /// 标记退出应用
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
