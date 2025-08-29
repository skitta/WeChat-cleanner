//! 文件清理模块
//!
//! 提供重复文件的清理功能，支持自动清理模式和安全的删除操作。
use crate::config::settings::{CleaningMode, CleaningSettings};
use crate::errors::{Error, Result};
use crate::file_utils::{FileGrouper, FileInfo, FileProcessor, HasSize};
use crate::progress::Progress;
use crate::scanner::ScanResult;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[cfg(feature = "display")]
use crate::Display;

/// 清理结果数据结构（用于序列化/反序列化）
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "display", derive(Display))]
pub struct CleaningResult {
    #[cfg_attr(feature = "display", display(summary, name = "删除文件数"))]
    pub files_deleted: usize,

    #[cfg_attr(feature = "display", display(summary, name = "释放空间"))]
    pub freed_space: u64,

    #[cfg_attr(feature = "display", display(summary, name = "清理耗时"))]
    pub clean_time: Duration,
}

/// 清理预览信息
#[derive(Debug, Clone)]
#[cfg_attr(feature = "display", derive(Display))]
pub struct CleaningPreview {
    #[cfg_attr(feature = "display", display(summary, name = "预计删除文件数"))]
    pub estimated_files_count: usize,

    #[cfg_attr(feature = "display", display(summary, name = "预计释放空间"))]
    pub estimated_freed_space: u64,

    #[cfg_attr(feature = "display", display(details, name = "文件分组详情"))]
    pub file_groups: HashMap<PathBuf, PreviewGroup>,
}

// 手动实现 Display trait 作为备用
impl std::fmt::Display for CleaningPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "预计删除文件数: {}, 预计释放空间: {} 字节",
            self.estimated_files_count, self.estimated_freed_space
        )
    }
}

/// 预览组，表示一个文件夹中的文件清理情况
#[derive(Debug, Clone)]
#[cfg_attr(feature = "display", derive(Display))]
pub struct PreviewGroup {
    #[cfg_attr(feature = "display", display(details, name = "保留文件"))]
    pub file_to_keep: FileInfo,

    #[cfg_attr(feature = "display", display(summary, details, name = "删除文件列表"))]
    pub files_to_delete: Vec<FileInfo>,
}

impl CleaningPreview {
    /// 从ScanResult加载清理预览
    pub fn from(scan_result: &ScanResult) -> Option<Self> {

        let mut groups = HashMap::new();
        let mut total_count = 0;
        let mut total_size = 0;

        for files in scan_result.duplicate_files.values().cloned() {
            if files.is_empty() {
                continue;
            }
            for (parent, mut group) in files.group_by_parent() {
                if group.len() > 1 {
                    group.sort_by_key(|f| f.modified);
                    let to_delete = group.iter().skip(1).cloned().collect::<Vec<_>>();

                    if !to_delete.is_empty() {
                        total_count += to_delete.len();
                        total_size += to_delete.iter().map(|f| f.size()).sum::<u64>();

                        groups.insert(
                            parent.to_path_buf(),
                            PreviewGroup {
                                file_to_keep: group[0].clone(),
                                files_to_delete: to_delete,
                            },
                        );
                    }
                }
            }
        }
        if total_count == 0 { None } else {
            Some(CleaningPreview {
                estimated_files_count: total_count,
                estimated_freed_space: total_size,
                file_groups: groups,
            })
        }
    }
}

/// 文件清理器
pub struct FileCleaner {
    pub preview: CleaningPreview,
    scan_result: ScanResult,
}

impl FileCleaner {
    /// 创建新的文件清理器
    pub fn new(settings: CleaningSettings) -> Result<Self> {
        let result_path = settings
            .scan_result_save_path
            .as_ref()
            .ok_or(Error::Config(
                "扫描结果保存路径不合法".to_string(),
            ))?;

        let scan_result = ScanResult::load(result_path)?;

        let preview = CleaningPreview::from(&scan_result).ok_or(Error::FileProcessing("找到扫描结果，但无可清理文件".to_string()))?;

        Ok(FileCleaner { preview, scan_result })
    }

    /// 执行文件清理（支持预览模式）
    pub fn clean(&self, mode: CleaningMode) -> Option<CleaningResult> {
        self.clean_with_progress(mode, &Progress::none())
    }

    /// 带进度显示的文件清理
    pub fn clean_with_progress(&self, mode: CleaningMode, progress: &Progress) -> Option<CleaningResult> {
        if self.preview.estimated_files_count == 0 {
            progress.set_message("没有需要清理的文件");
            return None;
        }

        match mode {
            CleaningMode::Auto => self.execute_deletion(progress).ok(),
            CleaningMode::Interactive => {
                progress.set_message("交互模式需要用户界面支持");
                return None;
            }
        }
    }

    fn execute_deletion(&self, progress: &Progress) -> Result<CleaningResult> {
        let start_time = Instant::now();
        let total = self.preview.file_groups.len();

        progress.set_message("执行清理中...");
        let mut deleted_files = HashMap::new();
        for (idx, (parent, group)) in self.preview.file_groups.iter().enumerate() {
            let deleted = group.files_to_delete.delete()?;
            if !deleted.is_empty() {
                deleted_files.insert(parent.clone(), deleted);
            }
            progress.update(idx + 1, total, &format!("清理进度: {}/{}", idx + 1, total));
        }
        progress.finish("清理完成");
        
        //清除完成后删除扫描记录
        self.scan_result.delete()?;
        
        Ok(CleaningResult {
            files_deleted: deleted_files.values().map(Vec::len).sum(),
            freed_space: deleted_files
                .values()
                .flat_map(|files| files.iter())
                .map(|f| f.size())
                .sum(),
            clean_time: start_time.elapsed(),
        })
    }
}
