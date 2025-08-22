use crate::config::settings::{CleaningMode, Settings};
use crate::core::file_utils::{self, FileInfo};
use crate::core::progressor::{Progress, ProgressTracker, ProgressReporter};
use crate::core::scanner::ScanResult;
use crate::errors::{Error, Result};

/// 文件清理器
pub struct FileCleaner<F> {
    settings: Settings,
    pub freed_space: u64,
    pub files_deleted: usize,
    progress_callback: Option<ProgressTracker<F>>,
}

impl<F> FileCleaner<F>
where
    F: Fn(&Progress) + Send + Sync + 'static,
{
    /// 创建新的文件清理器
    pub fn new(settings: Settings) -> Self {
        FileCleaner {
            settings,
            freed_space: 0,
            files_deleted: 0,
            progress_callback: None,
        }
    }

    /// 设置进度回调函数
    pub fn with_progress_callback(mut self, callback: F) -> Self {
        self.progress_callback = Some(ProgressTracker::new(Progress::new(), callback));
        self
    }

    /// 清理文件组
    pub fn clean_group(&mut self, files: &[FileInfo], mode: CleaningMode) -> Result<usize> {
        match mode {
            CleaningMode::Auto => self.clean_group_auto(files),
            CleaningMode::Smart => self.clean_group_smart(files),
            CleaningMode::Interactive => Err(Error::InvalidOperation(
                "交互模式需要用户界面支持".to_string(),
            )),
        }
    }

    /// 自动清理模式
    fn clean_group_auto(&mut self, files: &[FileInfo]) -> Result<usize> {
        // 按修改时间排序 (最早的文件在最前面)
        let mut sorted = files.to_vec();
        sorted.sort_by_key(|f| f.modified);

        // 保留最早的文件，删除其他
        let to_keep = sorted
            .first()
            .ok_or_else(|| Error::InvalidOperation("文件组为空".to_string()))?
            .path
            .clone();

        let mut deleted = 0;

        for file in sorted.iter() {
            if file.path != to_keep {
                if self.delete_file(file)? {
                    deleted += 1;
                }
            }
        }

        Ok(deleted)
    }

    /// 智能清理模式
    fn clean_group_smart(&mut self, files: &[FileInfo]) -> Result<usize> {
        // 尝试找到原始文件（没有自动标记）
        let original_files: Vec<_> = files.iter().collect();

        let to_keep = if !original_files.is_empty() {
            // 保留最早的原始文件
            let mut sorted_originals = original_files.clone();
            sorted_originals.sort_by_key(|f| f.modified);
            sorted_originals.first().unwrap().path.clone()
        } else {
            // 没有原始文件，保留最早的文件
            let mut sorted = files.to_vec();
            sorted.sort_by_key(|f| f.modified);
            sorted.first().unwrap().path.clone()
        };

        let mut deleted = 0;

        for file in files {
            if file.path != to_keep {
                if self.delete_file(file)? {
                    deleted += 1;
                }
            }
        }

        Ok(deleted)
    }

    /// 删除单个文件
    pub fn delete_file(&mut self, file: &FileInfo) -> Result<bool> {
        // 检查文件大小是否满足最小要求
        if file.size < self.settings.cleaning.min_file_size {
            return Ok(false);
        }

        // 设置文件权限（解决只读问题）
        if let Err(e) = file_utils::set_file_permissions(&file.path, 0o644) {
            log::warn!("无法设置文件权限 {}: {}", file.path.display(), e);
        }

        // 删除文件
        match std::fs::remove_file(&file.path) {
            Ok(_) => {
                self.freed_space += file.size;
                self.files_deleted += 1;
                Ok(true)
            }
            Err(e) => {
                log::error!("删除文件失败 {}: {}", file.path.display(), e);
                Err(Error::FileProcessing(format!(
                    "删除文件失败: {} - {}",
                    file.path.display(),
                    e
                )))
            }
        }
    }

    /// 批量清理所有重复文件
    pub fn clean_all_duplicates(
        &mut self,
        scanner: &ScanResult,
        mode: CleaningMode,
    ) -> Result<()> {
        let total_groups = scanner.duplicate_files.len();
        
        self.progress_callback.report_msg("正在清理重复文件...");

        let mut current_group = 0;
        for files in scanner.duplicate_files.values() {
            current_group += 1;
            self.progress_callback.report_msg(format!(
                "正在清理第 {} 组文件 ({} 个文件)",
                current_group,
                files.len()
            ));
            match self.clean_group(files, mode) {
                Ok(deleted) => {
                    log::info!("清理文件组: 删除 {} 个文件", deleted);
                }
                Err(e) => {
                    log::error!("清理文件组失败: {}", e);
                }
            }
            self.progress_callback.report_progress(current_group, total_groups);
        }

        // 报告完成阶段
        self.progress_callback.report_complete();

        Ok(())
    }
}

