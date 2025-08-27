use crate::config::settings::{CleaningMode, Settings};
use crate::core::file_utils::{self, FileInfo, HasSize};
use crate::core::progressor::{Progress, ProgressTracker, ProgressReporter, ProgressCallback};
use crate::core::scanner::ScanResult;
use crate::errors::{Error, Result};

/// 预览清理结果
#[derive(Debug, Clone)]
pub struct CleaningPreview {
    /// 将要被删除的文件列表，按文件夹分组
    pub files_to_delete: Vec<PreviewGroup>,
    /// 预计释放的空间大小
    pub estimated_freed_space: u64,
    /// 预计删除的文件数量
    pub estimated_files_count: usize,
}

/// 预览组，表示一个文件夹中的文件清理情况
#[derive(Debug, Clone)]
pub struct PreviewGroup {
    /// 父文件夹路径
    pub parent_path: std::path::PathBuf,
    /// 将要保留的文件
    pub file_to_keep: FileInfo,
    /// 将要删除的文件列表
    pub files_to_delete: Vec<FileInfo>,
}

/// 文件清理器
pub struct FileCleaner {
    settings: Settings,
    pub freed_space: u64,
    pub files_deleted: usize,
    progress_callback: Option<ProgressTracker>,
}

impl FileCleaner {
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
    pub fn with_progress_callback(mut self, callback: impl ProgressCallback + 'static) -> Self {
        self.progress_callback = Some(ProgressTracker::new(Progress::new(), callback));
        self
    }

    /// 清理文件组
    pub fn clean_group(&mut self, files: &[FileInfo], mode: CleaningMode) -> Result<usize> {
        match mode {
            CleaningMode::Auto => self.clean_group_auto(files),
            CleaningMode::Interactive => Err(Error::InvalidOperation(
                "交互模式需要用户界面支持".to_string(),
            )),
        }
    }

    /// 自动清理模式
    fn clean_group_auto(&mut self, files: &[FileInfo]) -> Result<usize> {
        use std::collections::HashMap;
        
        if files.is_empty() {
            return Err(Error::InvalidOperation("文件组为空".to_string()));
        }

        // 按父文件夹分组
        let mut groups: HashMap<std::path::PathBuf, Vec<&FileInfo>> = HashMap::new();
        for file in files {
            let parent = file.path.parent()
                .unwrap_or_else(|| std::path::Path::new("/"))
                .to_path_buf();
            groups.entry(parent).or_default().push(file);
        }

        let mut deleted = 0;

        // 处理每个父文件夹中的文件
        for (_parent_path, mut group_files) in groups {
            if group_files.len() <= 1 {
                continue; // 只有一个文件，无需清理
            }

            // 在每个父文件夹中，按修改时间排序，保留最早的文件
            group_files.sort_by_key(|f| f.modified);

            // 删除其他文件
            for file in group_files.iter().skip(1) {
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
        if file.size() < self.settings.cleaning.min_file_size {
            return Ok(false);
        }

        // 设置文件权限（解决只读问题）
        if let Err(e) = file_utils::set_file_permissions(&file.path, 0o644) {
            log::warn!("无法设置文件权限 {}: {}", file.path.display(), e);
        }

        // 删除文件
        match std::fs::remove_file(&file.path) {
            Ok(_) => {
                self.freed_space += file.size();
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

    /// 预览清理操作，返回将要被删除的文件列表
    pub fn preview_cleaning(&self, scanner: &ScanResult, mode: CleaningMode) -> Result<CleaningPreview> {
        let mut preview_groups = Vec::new();
        let mut estimated_freed_space = 0u64;
        let mut estimated_files_count = 0usize;

        for files in scanner.duplicate_files.values() {
            match mode {
                CleaningMode::Auto => {
                    let group_previews = self.preview_group_auto(files)?;
                    for group in group_previews {
                        estimated_freed_space += group.files_to_delete.iter()
                            .filter(|f| f.size() >= self.settings.cleaning.min_file_size)
                            .map(|f| f.size())
                            .sum::<u64>();
                        estimated_files_count += group.files_to_delete.iter()
                            .filter(|f| f.size() >= self.settings.cleaning.min_file_size)
                            .count();
                        preview_groups.push(group);
                    }
                }
                CleaningMode::Interactive => {
                    return Err(Error::InvalidOperation(
                        "交互模式需要用户界面支持".to_string(),
                    ));
                }
            }
        }

        Ok(CleaningPreview {
            files_to_delete: preview_groups,
            estimated_freed_space,
            estimated_files_count,
        })
    }

    /// 预览自动清理模式的结果
    fn preview_group_auto(&self, files: &[FileInfo]) -> Result<Vec<PreviewGroup>> {
        use std::collections::HashMap;
        
        if files.is_empty() {
            return Ok(Vec::new());
        }

        // 按父文件夹分组
        let mut groups: HashMap<std::path::PathBuf, Vec<&FileInfo>> = HashMap::new();
        for file in files {
            let parent = file.path.parent()
                .unwrap_or_else(|| std::path::Path::new("/"))
                .to_path_buf();
            groups.entry(parent).or_default().push(file);
        }

        let mut preview_groups = Vec::new();

        // 处理每个父文件夹中的文件
        for (parent_path, mut group_files) in groups {
            if group_files.len() <= 1 {
                continue; // 只有一个文件，无需清理
            }

            // 在每个父文件夹中，按修改时间排序，保留最早的文件
            group_files.sort_by_key(|f| f.modified);
            
            let file_to_keep = group_files[0].clone();
            let files_to_delete: Vec<FileInfo> = group_files.iter()
                .skip(1)
                .map(|&f| f.clone())
                .collect();

            if !files_to_delete.is_empty() {
                preview_groups.push(PreviewGroup {
                    parent_path,
                    file_to_keep,
                    files_to_delete,
                });
            }
        }

        Ok(preview_groups)
    }

    /// 基于预览结果执行清理
    pub fn clean_from_preview(&mut self, preview: &CleaningPreview) -> Result<()> {
        let total_groups = preview.files_to_delete.len();
        
        self.progress_callback.report_msg("正在根据预览结果清理重复文件...");

        let mut current_group = 0;
        for group in &preview.files_to_delete {
            current_group += 1;
            self.progress_callback.report_msg(format!(
                "正在清理文件夹 {} ({} 个文件)",
                group.parent_path.display(),
                group.files_to_delete.len()
            ));

            let mut deleted_in_group = 0;
            for file in &group.files_to_delete {
                match self.delete_file(file) {
                    Ok(true) => deleted_in_group += 1,
                    Ok(false) => {
                        log::info!("跳过文件 {} (大小不符合要求)", file.path.display());
                    }
                    Err(e) => {
                        log::error!("删除文件 {} 失败: {}", file.path.display(), e);
                    }
                }
            }

            log::info!("清理文件夹 {}: 删除 {} 个文件", 
                group.parent_path.display(), deleted_in_group);
            
            self.progress_callback.report_progress(current_group, total_groups);
        }

        // 报告完成阶段
        self.progress_callback.report_complete();

        Ok(())
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

impl CleaningPreview {
    /// 显示清理预览的详细信息
    pub fn display_summary(&self) -> String {
        format!(
            "预览清理结果:\n  - 预计删除 {} 个文件\n  - 预计释放空间 {:.2} MB\n  - 涉及 {} 个文件夹",
            self.estimated_files_count,
            self.estimated_freed_space as f64 / (1024.0 * 1024.0),
            self.files_to_delete.len()
        )
    }

    /// 显示详细的文件列表
    pub fn display_details(&self) -> String {
        let mut result = self.display_summary();
        result.push_str("\n\n详细文件列表:\n");

        for (i, group) in self.files_to_delete.iter().enumerate() {
            result.push_str(&format!(
                "\n[{}] 文件夹: {}\n",
                i + 1,
                group.parent_path.display()
            ));
            result.push_str(&format!(
                "  保留: {} ({})\n",
                group.file_to_keep.path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                format_size(group.file_to_keep.size())
            ));
            result.push_str("  删除:\n");
            for file in &group.files_to_delete {
                result.push_str(&format!(
                    "    - {} ({})\n",
                    file.path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    format_size(file.size())
                ));
            }
        }

        result
    }
}

/// 格式化文件大小显示
fn format_size(size: u64) -> String {
    if size >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if size >= 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else if size >= 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else {
        format!("{} B", size)
    }
}

