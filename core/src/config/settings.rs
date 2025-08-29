// src/config/settings.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub cleaning: CleaningSettings,
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

/// 清理设置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CleaningSettings {
    /// 默认清理模式
    #[serde(default = "default_cleaning_mode")]
    pub mode: CleaningMode,

    /// 临时文件保存位置
    #[serde(default = "default_scan_result_save_path")]
    pub scan_result_save_path: Option<PathBuf>,
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
    WechatCacheResolver::find_wechat_dirs().ok()
}

fn default_cache_patterns() -> String {
        r"\(\d+\)\.[a-zA-Z0-9]+$".to_string()
}

fn default_cleaning_mode() -> CleaningMode {
    CleaningMode::Auto
}

fn default_scan_result_save_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| {p.join("wechat-cleaner/scan_result.json")})
}


impl Default for Settings {
    fn default() -> Self {
        Settings {
            wechat: WechatSettings {
                cache_path: default_wechat_cache_path(),
                cache_patterns: default_cache_patterns(),
            },
            cleaning: CleaningSettings {
                mode: default_cleaning_mode(),
                scan_result_save_path: default_scan_result_save_path()
            },
        }
    }
}

// 实现 Merge trait 为各个配置结构
impl Merge for Settings {
    fn merge(&mut self, other: Self) {
        self.wechat.merge(other.wechat);
        self.cleaning.merge(other.cleaning);
    }
}

impl Merge for WechatSettings {
    fn merge(&mut self, other: Self) {
        // 如果 other 中有新的路径，则更新
        if other.cache_path.is_some() {
            self.cache_path = other.cache_path;
        }
        
        // 如果 other 中有非空的模式列表，则更新
        if !other.cache_patterns.is_empty() {
            self.cache_patterns = other.cache_patterns;
        }
    }
}

impl Merge for CleaningSettings {
    fn merge(&mut self, other: Self) {
        // 清理模式直接更新（枚举类型没有“空”状态）
        self.mode = other.mode;
        
        self.scan_result_save_path = other.scan_result_save_path;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        assert!(settings.wechat.cache_path.is_none());
        assert!(
            settings
                .wechat
                .cache_patterns
                .contains(&r"\(\d+\)\.[a-zA-Z0-9]+$".to_string())
        );
        assert_eq!(settings.cleaning.mode, CleaningMode::Auto);
    }

    #[test]
    fn test_serialize_deserialize() {
        let settings = Settings::default();
        let toml_str = toml::to_string(&settings).unwrap();

        let deserialized: Settings = toml::from_str(&toml_str).unwrap();

        assert_eq!(
            deserialized.wechat.cache_patterns,
            settings.wechat.cache_patterns
        );
        assert_eq!(
            deserialized.cleaning.mode,
            settings.cleaning.mode
        );
    }
    
    #[test]
    fn test_merge_wechat_settings() {
        let mut base = WechatSettings {
            cache_path: None,
            cache_patterns: "old_pattern".to_string(),
        };
        
        let other = WechatSettings {
            cache_path: Some(PathBuf::from("/new/path")),
            cache_patterns: "new_pattern".to_string(),
        };
        
        base.merge(other);
        
        assert_eq!(base.cache_path, Some(PathBuf::from("/new/path")));
        assert_eq!(base.cache_patterns, "new_pattern".to_string());
    }
    
    #[test]
    fn test_merge_cleaning_settings() {
        let mut base = CleaningSettings {
            mode: CleaningMode::Auto,
            scan_result_save_path: Some(PathBuf::from("/first/temp"))
        };
        
        let other = CleaningSettings {
            mode: CleaningMode::Auto,
            scan_result_save_path: Some(PathBuf::from("/second/temp"))
        };
        
        base.merge(other);
        
        assert_eq!(base.mode, CleaningMode::Auto);
        assert_eq!(base.scan_result_save_path, Some(PathBuf::from("/second/temp")))
    }
}
