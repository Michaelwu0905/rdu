use std::fs;
use std::path::PathBuf;

use rayon::prelude::*;
use walkdir::WalkDir;

/// 目录项结构
#[derive(Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

/// 扫描指定路径下的所有目录项
pub fn scan_directory(path: &PathBuf) -> Vec<DirEntry> {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();

    // 并行计算大小
    let mut items: Vec<DirEntry> = paths
        .par_iter()
        .map(|p| {
            let size = calculate_size(p);
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

    // 按大小降序排序
    items.sort_by(|a, b| b.size.cmp(&a.size));

    items
}

/// 计算指定路径的大小（文件或目录）
/// 如果是文件，直接返回大小；如果是目录，递归遍历所有文件
pub fn calculate_size(path: &PathBuf) -> u64 {
    // 如果是文件，直接返回大小
    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }

    // 如果是目录，使用 WalkDir 递归遍历
    WalkDir::new(path)
        .into_iter()
        // filter_map 会自动忽略那些没有权限访问的文件
        .filter_map(|entry| entry.ok())
        // 只关心文件，不加目录本身的大小
        .filter(|entry| entry.file_type().is_file())
        // 获取每个文件的大小，如果获取失败就当作0
        .map(|entry| entry.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

/// 计算目录项列表的总大小
pub fn calculate_total_size(items: &[DirEntry]) -> u64 {
    items.iter().map(|item| item.size).sum()
}
