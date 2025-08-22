use crate::config::settings::Settings;
use crate::core::file_utils::{FileInfo, FileInfosExt, WechatCacheResolver};
use crate::errors::{Error, Result};
use crate::core::progressor::{Progress, ProgressTracker, ProgressReporter};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use walkdir::WalkDir;
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
    /// 保存扫描结果到临时文件
    pub fn save(&self, path: Option<&str>) -> Result<PathBuf> {
        if self.duplicate_count == 0 {
            return Ok(PathBuf::new());
        }
        
        let temp_dir = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            std::env::temp_dir()
        };
        
        let temp_file = temp_dir.join("wechat_cleaner_scan_result.json");
        
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&temp_file, json)?;
        Ok(temp_file)
    }

    /// 从临时文件加载扫描结果
    pub fn load(file: Option<&str>) -> Result<Self> {
        let temp_file = if let Some(p) = file {
            PathBuf::from(p)
        } else {
            std::env::temp_dir().join("wechat_cleaner_scan_result.json")
        };
        
        if !temp_file.exists() {
            return Err(Error::FileProcessing("未找到扫描结果文件，请先执行扫描命令".to_string()));
        };

        let json = fs::read_to_string(&temp_file)?;
        serde_json::from_str(&json).map_err(|e| Error::Json(e))
    }

    /// 删除扫描结果文件
    pub fn delete(&self, file: Option<&str>) -> Result<()> {
        let temp_file = if let Some(p) = file {
            PathBuf::from(p)
        } else {
            std::env::temp_dir().join("wechat_cleaner_scan_result.json")
        };

        if temp_file.exists() {
            fs::remove_file(temp_file).map_err(|e| Error::FileProcessing(format!("删除文件失败: {}", e)))
        } else {
            Err(Error::FileProcessing("扫描结果文件不存在".to_string()))
        }
    }

    pub fn summaries(&self) -> Vec<GroupSummary> {
        let mut summaries: Vec<_> = self
            .duplicate_files
            .iter()
            .map(|(hash, files)| {
                let size = files.first().map(|f| f.size).unwrap_or(0);

                GroupSummary {
                    hash_prefix: format!("{}", &hash[..8]),
                    file_count: files.len(),
                    size
                }
            })
            .collect();

        // 按文件数量排序
        summaries.sort_by(|a, b| b.file_count.cmp(&a.file_count));
        summaries
    }

    pub fn get_files_by_hash_prefix(&self, hash_prefix: &str) -> Option<&Vec<FileInfo>> {
        self.duplicate_files
            .keys()
            .find(|k| k.starts_with(hash_prefix))
            .and_then(|k| self.duplicate_files.get(k))
    }
}

/// 文件扫描器
pub struct FileScanner<F> {
    settings: Settings,
    progress_tracker: Option<ProgressTracker<F>>,
}

impl<F> FileScanner<F>
where 
    F: Fn(&Progress) + Send + Sync + 'static,
{
    /// 创建新的文件扫描器
    pub fn new(settings: Settings) -> Self {
        FileScanner {
            settings,
            progress_tracker: None,

        }
    }

    pub fn with_progress_callback(mut self, callback: F) -> Self {
        self.progress_tracker = Some(ProgressTracker::new(
            Progress::new(),
            callback,
        ));
        self
    }

    /// 执行文件扫描
    pub fn scan(&mut self) -> Result<ScanResult> {
        let start_time = Instant::now();
        self.progress_tracker.report_msg("开始扫描微信缓存文件...");
        let cache_path = self.resolve_cache_path()?;

        // 2. 收集文件元数据
        self.progress_tracker.report_msg("收集文件元数据");
        let all_files = self.collect_file_metadata(&cache_path)?;
        //let mut result = ScanResult::default();
        let total_files_count = all_files.len();

        if total_files_count == 0 {
            return Err(Error::CacheNotFound);
        }

        // 3. 按大小预分组，减少不必要的哈希计算
        let size_groups = self.pre_group_by_size(&all_files);
        
        // 4. 只对可能重复的文件组计算哈希
        self.progress_tracker.report_msg("计算文件哈希");
        let mut files_to_hash = Vec::new();
        for (_, files) in size_groups {
            // 如果该大小组只有一个文件，则跳过哈希计算
            if files.len() > 1 {
                files_to_hash.extend(files);
            }
        }

        // 5. 计算文件哈希值
        let mut file_groups = files_to_hash
            .calculate_hashes()
            .group_by_hash();
        file_groups.retain(|_, files| files.len() > 1); // 只保留重复文件组

        Ok(ScanResult {
            total_files_count,
            duplicate_count: file_groups.values().map(Vec::len).sum(),
            duplicate_files: file_groups,
            scan_time: start_time.elapsed(),
        })
    }

    /// 解析微信缓存路径
    fn resolve_cache_path(&self) -> Result<PathBuf> {
        if let Some(path) = &self.settings.wechat.cache_path {
            if path.exists() {
                return Ok(path.clone());
            }
            return Err(Error::Config(format!(
                "配置的微信缓存路径不存在: {}",
                path.display()
            )));
        }
        WechatCacheResolver::find_wechat_dirs()
    }

    /// 收集文件元数据（并行化）
    fn collect_file_metadata(&self, path: &PathBuf) -> Result<Vec<FileInfo>> {
        let entries: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        // 使用并行处理文件元数据
        let files: Vec<_> = entries
            .into_par_iter()
            .filter_map(|entry| {
                let path = entry.path();

                match FileInfo::new(path) {
                    Ok(info) => Some(info),
                    Err(e) => {
                        // 记录错误但继续处理其他文件
                        log::warn!("跳过文件 {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        Ok(files)
    }

    /// 按文件大小预分组
    fn pre_group_by_size(&self, files: &[FileInfo]) -> HashMap<u64, Vec<FileInfo>> {
        let mut size_groups: HashMap<u64, Vec<FileInfo>> = HashMap::new();
        
        for file in files {
            size_groups.entry(file.size).or_default().push(file.clone());
        }
        
        size_groups
    }
}

/// 分组摘要信息
pub struct GroupSummary {
    pub hash_prefix: String,
    pub file_count: usize,
    pub size: u64,
}
