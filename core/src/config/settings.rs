// src/config/settings.rs
use serde::{Deserialize, Serialize};
use std::path::{PathBuf};

use crate::file_utils::WechatCacheResolver;

/// 配置合并策略
pub trait Merge {
    /// 将其他配置合并到当前配置中
    fn merge(&mut self, other: Self);
}

/// 应用程序设置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub wechat: WechatSettings,
    pub scanner: ScannerSettings,
    pub cleaner: CleanerSettings,
}

/// 微信相关设置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WechatSettings {
    /// 微信缓存路径
    #[serde(default = "default_wechat_cache_path")]
    pub cache_path: Option<PathBuf>,

    /// 用于识别微信自动生成副本的文件名模式
    #[serde(default = "default_cache_patterns")]
    pub cache_patterns: String,
}

/// 扫描设置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScannerSettings {
    /// 扫描结果保存位置
    #[serde(default = "default_scan_result_save_path")]
    pub save_path: PathBuf,
}

/// 清理设置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CleanerSettings {
    /// 默认清理模式
    #[serde(default = "default_cleaning_mode")]
    pub mode: CleaningMode,
}

/// 清理模式
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CleaningMode {
    /// 自动模式：保留每组中最早的文件
    Auto,
    /// 交互模式：用户手动选择
    Interactive,
}

// 默认值函数
fn default_wechat_cache_path() -> Option<PathBuf> {
    WechatCacheResolver::find_wechat_dirs()
}

fn default_cache_patterns() -> String {
    r"\(\d+\)\.[a-zA-Z0-9]+$".to_string()
}

fn default_cleaning_mode() -> CleaningMode {
    CleaningMode::Auto
}

fn default_scan_result_save_path() -> PathBuf {
    dirs::cache_dir().unwrap_or(PathBuf::from("."))
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            wechat: WechatSettings {
                cache_path: default_wechat_cache_path(),
                cache_patterns: default_cache_patterns(),
            },
            scanner: ScannerSettings {
                save_path: default_scan_result_save_path(),
            },
            cleaner: CleanerSettings {
                mode: default_cleaning_mode(),
            },
        }
    }
}

// 实现 Merge trait 为各个配置结构
impl Merge for Settings {
    fn merge(&mut self, other: Self) {
        self.wechat.merge(other.wechat);
        self.scanner.merge(other.scanner);
        self.cleaner.merge(other.cleaner);
    }
}

impl Merge for WechatSettings {
    fn merge(&mut self, other: Self) {
        // 如果 other 中有新的路径，则更新
        if let Some(path) = other.cache_path {
            if !path.exists() {
                println!("配置的微信缓存路径不存在，将使用默认路径")
            } else {
                self.cache_path = Some(path);
            }
        }
        
        // 如果 other 中有非空的模式列表，则更新
        if !other.cache_patterns.is_empty() {
            self.cache_patterns = other.cache_patterns;
        }
    }
}

impl Merge for ScannerSettings {
    fn merge(&mut self, other: Self) {
        // 清理模式直接更新（枚举类型没有“空”状态）
        if !other.save_path.exists() {
            println!("配置的扫描数据保存路径不存在, 将使用默认路径")
        } else {
            self.save_path = other.save_path;
        }
    }
}

impl Merge for CleanerSettings {
    fn merge(&mut self, other: Self) {
        // 清理模式直接更新（枚举类型没有“空”状态）
        self.mode = other.mode;
    }
}