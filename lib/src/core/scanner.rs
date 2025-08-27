use crate::config::settings::Settings;
use crate::core::file_utils::{FileFilter, FileInfo};
use crate::errors::{Error, Result};
use crate::core::progressor::{Progress, ProgressTracker, ProgressReporter, ProgressCallback};
use regex::{Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path,PathBuf};
use std::time::{Duration, Instant};
use std::fs;

/// 扫描结果数据结构（用于序列化/反序列化）
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScanResult {
    pub total_files_count: usize,
    pub duplicate_count: usize,
    pub duplicate_files: HashMap<String, Vec<FileInfo>>,
    pub scan_time: Duration,
}

impl ScanResult {
    pub fn new(total_files_count: usize, duplicate_files: HashMap<String, Vec<FileInfo>>, start_time: Instant) -> Self {
        ScanResult {
            total_files_count,
            duplicate_count: duplicate_files.values().map(Vec::len).sum(),
            duplicate_files,
            scan_time: start_time.elapsed(),
        }
    }

    /// 保存扫描结果到临时文件
    pub fn save(&self, file: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = Path::new(file).parent() {
            fs::create_dir_all(parent)?; // 递归创建所有缺失的目录
        }
        fs::write(file, json)?;
        Ok(())
    }

    /// 从临时文件加载扫描结果
    pub fn load(file: &PathBuf) -> Result<Self> {
        if !file.exists() {
            return Err(Error::FileProcessing("未找到扫描结果文件，请先执行扫描命令".to_string()));
        };

        let json = fs::read_to_string(file)?;
        serde_json::from_str(&json).map_err(|e| Error::Json(e))
    }

    /// 删除扫描结果文件
    pub fn delete(&self, file: &PathBuf) -> Result<()> {
        if file.exists() {
            fs::remove_file(file).map_err(|e| Error::FileProcessing(format!("删除临时文件失败: {}", e)))
        } else {
            Err(Error::FileProcessing("扫描结果文件不存在".to_string()))
        }
    }
}

/// 文件扫描器
pub struct FileScanner {
    settings: Settings,
    progress_tracker: Option<ProgressTracker>,
}

impl FileScanner {
    /// 创建新的文件扫描器
    pub fn new(settings: Settings) -> Self {
        FileScanner {
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

    /// 解析微信缓存路径
    fn resolve_cache_path(&self) -> Result<PathBuf> {
        match &self.settings.wechat.cache_path {
            Some(p) => {
                if p.exists() { Ok(p.clone()) } else {
                    Err(Error::Config("配置的微信缓存路径非法".to_string()))
                }
            },
            None => Err(Error::Config("配置的微信缓存路径不存在".to_string()))
        }
    }

    /// 执行文件扫描
    pub fn scan(&mut self) -> Result<ScanResult> {
        let start_time = Instant::now();
        self.progress_tracker.report_msg("开始扫描微信缓存文件...");
        
        let cache_path = self.resolve_cache_path()?;
        self.progress_tracker.report_msg("收集文件元数据...");
        let all_files = FileInfo::collect_from(&cache_path)?;
        let all_files_count = &all_files.len();

        let pattern = self.settings.wechat.cache_patterns.clone();
        let regex = Regex::new(&pattern)?;
        self.progress_tracker.report_msg("查找重复文件...");
        let duplicate_files = all_files.duplicates_by_pattern(&regex);
        
        Ok(ScanResult::new(*all_files_count, duplicate_files, start_time))
    }
}
