use crate::config::settings::Settings;
use crate::file_utils::{FileFilter, FileInfo};
use crate::errors::{Error, Result};
use crate::progress::Progress;
use regex::{Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path,PathBuf};
use std::time::{Duration, Instant};
use std::fs;

#[cfg(feature = "display")]
use crate::Display;

/// 扫描结果数据结构（用于序列化/反序列化）
#[derive(Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "display", derive(Display))]
pub struct ScanResult {
    #[cfg_attr(feature = "display", display(summary, name="保存位置"))]
    pub path: PathBuf,

    #[cfg_attr(feature = "display", display(summary, name="总文件数"))]
    pub total_files_count: usize,
    
    #[cfg_attr(feature = "display", display(summary, name="重复文件数"))]
    pub duplicate_count: usize,
    
    #[cfg_attr(feature = "display", display(details, name="重复文件详情"))]
    pub duplicate_files: HashMap<String, Vec<FileInfo>>,
    
    #[cfg_attr(feature = "display", display(summary, name="扫描耗时"))]
    pub scan_time: Duration,
}

impl ScanResult {
    pub fn new(save_path: PathBuf, total_files_count: usize, duplicate_files: HashMap<String, Vec<FileInfo>>, start_time: Instant) -> Self {
        ScanResult {
            path: save_path,
            total_files_count,
            duplicate_count: duplicate_files.values().map(Vec::len).sum(),
            duplicate_files,
            scan_time: start_time.elapsed(),
        }
    }

    /// 保存扫描结果到临时文件
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = Path::new(&self.path).parent() {
            fs::create_dir_all(parent)?; // 递归创建所有缺失的目录
        }
        fs::write(&self.path, json)?;
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
    pub fn delete(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path).map_err(|e| Error::FileProcessing(format!("删除临时文件失败: {}", e)))
        } else {
            Err(Error::FileProcessing("扫描结果文件不存在".to_string()))
        }
    }
}

/// 文件扫描器
pub struct FileScanner {
    settings: Settings,
}

impl FileScanner {
    /// 创建新的文件扫描器
    pub fn new(settings: Settings) -> Self {
        FileScanner {
            settings,
        }
    }

    /// 执行文件扫描
    pub fn scan(&mut self) -> Option<ScanResult> {
        self.scan_with_progress(&Progress::none())
    }

    /// 带进度显示的文件扫描
    pub fn scan_with_progress(&mut self, progress: &Progress) -> Option<ScanResult> {
        let start_time = Instant::now();
        progress.set_message("开始扫描微信缓存文件...");
        
        let cache_path = self.settings.wechat.cache_path.as_ref()?;
        progress.set_message("收集文件元数据...");
        let all_files = FileInfo::collect_from(&cache_path)?;
        let all_files_count = &all_files.len();

        if all_files_count == &0 {
            progress.finish("无重复文件");
            return None;
        }

        let pattern = self.settings.wechat.cache_patterns.as_ref();
        let regex = Regex::new(pattern).ok()?;
        progress.set_message("查找重复文件...");
        let duplicate_files = all_files.duplicates_by_pattern(&regex);

        let save_path = self.settings
            .cleaning
            .scan_result_save_path
            .as_ref()?;

        let result = ScanResult::new(
            save_path.clone(),
            *all_files_count,
            duplicate_files,
            start_time,
        );

        progress.finish("扫描完成");
        Some(result)
    }
}
