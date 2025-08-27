//! 文件清理模块
//!
//! 提供重复文件的清理功能，支持自动清理模式和安全的删除操作。

use crate::config::settings::{CleaningMode, Settings};
use crate::file_utils::{self, FileInfo, HasSize};
use crate::progressor::{Progress, ProgressTracker, ProgressReporter, ProgressCallback};
use crate::scanner::ScanResult;
use crate::errors::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[cfg(feature = "display")]
use crate::Display;

/// 清理结果数据结构（用于序列化/反序列化）
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "display", derive(Display))]
pub struct CleaningResult {
    #[cfg_attr(feature = "display", display(summary, name="删除文件数"))]
    pub files_deleted: usize,
    
    #[cfg_attr(feature = "display", display(summary, name="释放空间"))]
    pub freed_space: u64,
    
    #[cfg_attr(feature = "display", display(details, name="删除详情"))]
    pub deleted_files: HashMap<PathBuf, Vec<FileInfo>>,
    
    #[cfg_attr(feature = "display", display(summary, name="清理耗时"))]
    pub clean_time: Duration,
    
    // 预览信息（仅在预览模式下填充）
    pub preview_info: Option<CleaningPreview>,
}

/// 清理预览信息
#[derive(Debug, Clone)]
#[cfg_attr(feature = "display", derive(Display))]
pub struct CleaningPreview {
    #[cfg_attr(feature = "display", display(summary, name="预计删除文件数"))]
    pub estimated_files_count: usize,
    
    #[cfg_attr(feature = "display", display(summary, name="预计释放空间"))]
    pub estimated_freed_space: u64,
    
    #[cfg_attr(feature = "display", display(details, name="文件分组详情"))]
    pub file_groups: HashMap<PathBuf, PreviewGroup>,
}

/// 预览组，表示一个文件夹中的文件清理情况
#[derive(Debug, Clone)]
#[cfg_attr(feature = "display", derive(Display))]
pub struct PreviewGroup {
    #[cfg_attr(feature = "display", display(details, name="保留文件"))]
    pub file_to_keep: FileInfo,
    
    #[cfg_attr(feature = "display", display(summary, details, name="删除文件列表"))]
    pub files_to_delete: Vec<FileInfo>,
}

impl CleaningResult {
    pub fn new(deleted_files: HashMap<PathBuf, Vec<FileInfo>>, start_time: Instant) -> Self {
        let files_deleted = deleted_files.values().map(Vec::len).sum();
        let freed_space = deleted_files.values()
            .flat_map(|files| files.iter())
            .map(|f| f.size())
            .sum();
            
        CleaningResult {
            files_deleted,
            freed_space,
            deleted_files,
            clean_time: start_time.elapsed(),
            preview_info: None,
        }
    }
    
    /// 创建预览结果
    pub fn new_preview(preview: CleaningPreview, start_time: Instant) -> Self {
        CleaningResult {
            files_deleted: 0,
            freed_space: 0,
            deleted_files: HashMap::new(),
            clean_time: start_time.elapsed(),
            preview_info: Some(preview),
        }
    }
    
    /// 检查是否为预览模式
    pub fn is_preview(&self) -> bool {
        self.preview_info.is_some()
    }
    
    /// 获取预览信息
    pub fn preview(&self) -> Option<&CleaningPreview> {
        self.preview_info.as_ref()
    }
}

/// 文件清理器
pub struct FileCleaner {
    settings: Settings,
    progress_tracker: Option<ProgressTracker>,
}

impl FileCleaner {
    /// 创建新的文件清理器
    pub fn new(settings: Settings) -> Self {
        FileCleaner {
            settings,
            progress_tracker: None,
        }
    }

    /// 设置进度回调
    pub fn with_progress_callback(mut self, callback: impl ProgressCallback + 'static) -> Self {
        self.progress_tracker = Some(ProgressTracker::new(
            Progress::new(),
            callback,
        ));
        self
    }

    /// 执行文件清理（支持预览模式）
    pub fn clean(&mut self, scan_result: &ScanResult, mode: CleaningMode, preview_only: bool) -> Result<CleaningResult> {
        let start_time = Instant::now();
        
        match mode {
            CleaningMode::Auto => {
                if preview_only {
                    self.progress_tracker.report_msg("生成清理预览...");
                    let preview = self.generate_preview(scan_result)?;
                    self.progress_tracker.report_complete();
                    Ok(CleaningResult::new_preview(preview, start_time))
                } else {
                    self.progress_tracker.report_msg("开始清理重复文件...");
                    let deleted_files = self.clean_auto(scan_result)?;
                    self.progress_tracker.report_complete();
                    Ok(CleaningResult::new(deleted_files, start_time))
                }
            }
            CleaningMode::Interactive => Err(Error::InvalidOperation(
                "交互模式需要用户界面支持".to_string(),
            )),
        }
    }
    
    /// 基于预览结果执行清理（用于确认后的实际清理）
    pub fn confirm_and_execute(&mut self, preview_result: &CleaningResult) -> Result<CleaningResult> {
        let start_time = Instant::now();
        
        if let Some(preview) = &preview_result.preview_info {
            self.progress_tracker.report_msg("开始执行清理操作...");
            let deleted_files = self.execute_cleaning_from_preview(preview)?;
            self.progress_tracker.report_complete();
            Ok(CleaningResult::new(deleted_files, start_time))
        } else {
            Err(Error::InvalidOperation("缺少预览信息无法执行清理".to_string()))
        }
    }

    /// 自动清理模式
    fn clean_auto(&mut self, scan_result: &ScanResult) -> Result<HashMap<PathBuf, Vec<FileInfo>>> {
        let mut deleted_files = HashMap::new();
        let total_groups = scan_result.duplicate_files.len();
        
        for (current_group, files) in scan_result.duplicate_files.values().enumerate() {
            if files.is_empty() {
                continue;
            }

            // 按文件夹分组
            let groups = self.group_files_by_parent(files);
            
            for (parent_path, group_files) in groups {
                if group_files.len() <= 1 {
                    continue; // 只有一个文件，无需清理
                }

                // 按修改时间排序，保留最早的文件
                let mut sorted_files = group_files;
                sorted_files.sort_by_key(|f| f.modified);
                
                // 删除除第一个文件外的所有文件
                let files_to_delete: Vec<FileInfo> = sorted_files.iter()
                    .skip(1)
                    .cloned()
                    .collect();

                if !files_to_delete.is_empty() {
                    let deleted = self.delete_files(&files_to_delete)?;
                    if !deleted.is_empty() {
                        deleted_files.insert(parent_path, deleted);
                    }
                }
            }
            
            self.progress_tracker.report_progress(current_group + 1, total_groups);
        }
        
        Ok(deleted_files)
    }

    /// 生成清理预览
    fn generate_preview(&mut self, scan_result: &ScanResult) -> Result<CleaningPreview> {
        let mut file_groups = HashMap::new();
        let mut estimated_files_count = 0;
        let mut estimated_freed_space = 0;
        
        for files in scan_result.duplicate_files.values() {
            if files.is_empty() {
                continue;
            }

            // 按文件夹分组
            let groups = self.group_files_by_parent(files);
            
            for (parent_path, group_files) in groups {
                if group_files.len() <= 1 {
                    continue; // 只有一个文件，无需清理
                }

                // 按修改时间排序，保留最早的文件
                let mut sorted_files = group_files;
                sorted_files.sort_by_key(|f| f.modified);
                
                let file_to_keep = sorted_files[0].clone();
                let files_to_delete: Vec<FileInfo> = sorted_files.iter()
                    .skip(1)
                    .filter(|f| f.size() >= self.settings.cleaning.min_file_size)
                    .cloned()
                    .collect();

                if !files_to_delete.is_empty() {
                    estimated_files_count += files_to_delete.len();
                    estimated_freed_space += files_to_delete.iter().map(|f| f.size()).sum::<u64>();
                    
                    file_groups.insert(parent_path, PreviewGroup {
                        file_to_keep,
                        files_to_delete,
                    });
                }
            }
        }
        
        Ok(CleaningPreview {
            estimated_files_count,
            estimated_freed_space,
            file_groups,
        })
    }
    
    /// 基于预览信息执行清理
    fn execute_cleaning_from_preview(&mut self, preview: &CleaningPreview) -> Result<HashMap<PathBuf, Vec<FileInfo>>> {
        let mut deleted_files = HashMap::new();
        let total_groups = preview.file_groups.len();
        
        for (current_group, (parent_path, group)) in preview.file_groups.iter().enumerate() {
            let group_num = current_group + 1;
            self.progress_tracker.report_msg(format!(
                "正在清理文件夹 {} ({} 个文件)",
                parent_path.display(),
                group.files_to_delete.len()
            ));

            let deleted = self.delete_files(&group.files_to_delete)?;
            if !deleted.is_empty() {
                deleted_files.insert(parent_path.clone(), deleted);
            }
            
            self.progress_tracker.report_progress(group_num, total_groups);
        }
        
        Ok(deleted_files)
    }
    /// 按父文件夹对文件进行分组
    fn group_files_by_parent(&self, files: &[FileInfo]) -> HashMap<PathBuf, Vec<FileInfo>> {
        let mut groups: HashMap<PathBuf, Vec<FileInfo>> = HashMap::new();
        
        for file in files {
            let parent = file.path.parent()
                .unwrap_or_else(|| std::path::Path::new("/"))
                .to_path_buf();
            groups.entry(parent).or_default().push(file.clone());
        }
        
        groups
    }

    /// 删除文件列表
    fn delete_files(&mut self, files: &[FileInfo]) -> Result<Vec<FileInfo>> {
        let mut deleted = Vec::new();
        
        for file in files {
            if self.delete_file(file)? {
                deleted.push(file.clone());
            }
        }
        
        Ok(deleted)
    }

    /// 删除单个文件
    fn delete_file(&mut self, file: &FileInfo) -> Result<bool> {
        // 检查文件大小是否满足最小要求
        if file.size() < self.settings.cleaning.min_file_size {
            return Ok(false);
        }

        // 尝试设置文件权限（解决只读问题）
        if let Err(e) = file_utils::set_file_permissions(&file.path, 0o644) {
            log::warn!("无法设置文件权限 {}: {}", file.path.display(), e);
        }

        // 执行文件删除
        match std::fs::remove_file(&file.path) {
            Ok(_) => {
                log::debug!("成功删除文件: {}", file.path.display());
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
}


