// src/config/settings.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::core::file_utils::WechatCacheResolver;

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
    pub ui: UiSettings,
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
    pub default_mode: CleaningMode,

    /// 是否保留原始文件（没有(1)、(2)等后缀的文件）
    #[serde(default = "default_preserve_originals")]
    pub preserve_originals: bool,

    /// 最小文件大小（字节），小于此值的文件将被忽略
    #[serde(default = "default_min_file_size")]
    pub min_file_size: u64,

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
    /// 智能模式：优先保留原始文件，删除自动生成的副本
    Smart,
    /// 交互模式：用户手动选择
    Interactive,
}

/// UI设置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UiSettings {
    /// 主题设置
    #[serde(default = "default_theme")]
    pub theme: ThemeSettings,

    /// 快捷键设置
    #[serde(default = "default_keybindings")]
    pub keybindings: Keybindings,
}

/// 主题设置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThemeSettings {
    /// 主色调
    #[serde(default = "default_primary_color")]
    pub primary_color: String,

    /// 次要色调
    #[serde(default = "default_secondary_color")]
    pub secondary_color: String,

    /// 高亮色调
    #[serde(default = "default_highlight_color")]
    pub highlight_color: String,
}

/// 快捷键设置
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Keybindings {
    /// 退出程序
    #[serde(default = "default_quit_key")]
    pub quit: char,

    /// 上移
    #[serde(default = "default_up_key")]
    pub up: char,

    /// 下移
    #[serde(default = "default_down_key")]
    pub down: char,

    /// 左移
    #[serde(default = "default_left_key")]
    pub left: char,

    /// 右移
    #[serde(default = "default_right_key")]
    pub right: char,

    /// 确认选择
    #[serde(default = "default_confirm_key")]
    pub confirm: char,

    /// 删除文件
    #[serde(default = "default_delete_key")]
    pub delete: char,
}

// 默认值函数
fn default_wechat_cache_path() -> Option<PathBuf> {
    WechatCacheResolver::find_wechat_dirs().ok()
}

fn default_cache_patterns() -> String {
        r"\(\d+\)\.[a-zA-Z0-9]+$".to_string()
}

fn default_cleaning_mode() -> CleaningMode {
    CleaningMode::Smart
}

fn default_preserve_originals() -> bool {
    true
}

fn default_min_file_size() -> u64 {
    1024 // 1KB
}

fn default_scan_result_save_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| {p.join("wechat-cleaner/scan_result.json")})
}

fn default_theme() -> ThemeSettings {
    ThemeSettings {
        primary_color: default_primary_color(),
        secondary_color: default_secondary_color(),
        highlight_color: default_highlight_color(),
    }
}

fn default_primary_color() -> String {
    "blue".to_string()
}

fn default_secondary_color() -> String {
    "green".to_string()
}

fn default_highlight_color() -> String {
    "yellow".to_string()
}

fn default_keybindings() -> Keybindings {
    Keybindings {
        quit: default_quit_key(),
        up: default_up_key(),
        down: default_down_key(),
        left: default_left_key(),
        right: default_right_key(),
        confirm: default_confirm_key(),
        delete: default_delete_key(),
    }
}

fn default_quit_key() -> char {
    'q'
}

fn default_up_key() -> char {
    'k'
}

fn default_down_key() -> char {
    'j'
}

fn default_left_key() -> char {
    'h'
}

fn default_right_key() -> char {
    'l'
}

fn default_confirm_key() -> char {
    '\n' // Enter键
}

fn default_delete_key() -> char {
    'd'
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            wechat: WechatSettings {
                cache_path: default_wechat_cache_path(),
                cache_patterns: default_cache_patterns(),
            },
            cleaning: CleaningSettings {
                default_mode: default_cleaning_mode(),
                preserve_originals: default_preserve_originals(),
                min_file_size: default_min_file_size(),
                scan_result_save_path: default_scan_result_save_path()
            },
            ui: UiSettings {
                theme: default_theme(),
                keybindings: default_keybindings(),
            },
        }
    }
}

// 实现 Merge trait 为各个配置结构
impl Merge for Settings {
    fn merge(&mut self, other: Self) {
        self.wechat.merge(other.wechat);
        self.cleaning.merge(other.cleaning);
        self.ui.merge(other.ui);
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
        self.default_mode = other.default_mode;
        
        // 布尔值直接更新
        self.preserve_originals = other.preserve_originals;
        
        // 只有当新值大于 0 时才更新最小文件大小
        if other.min_file_size > 0 {
            self.min_file_size = other.min_file_size;
        }
    }
}

impl Merge for UiSettings {
    fn merge(&mut self, other: Self) {
        self.theme.merge(other.theme);
        self.keybindings.merge(other.keybindings);
    }
}

impl Merge for ThemeSettings {
    fn merge(&mut self, other: Self) {
        // 只有当新颜色非空且不等于默认值时才更新
        if !other.primary_color.is_empty() && other.primary_color != default_primary_color() {
            self.primary_color = other.primary_color;
        }
        
        if !other.secondary_color.is_empty() && other.secondary_color != default_secondary_color() {
            self.secondary_color = other.secondary_color;
        }
        
        if !other.highlight_color.is_empty() && other.highlight_color != default_highlight_color() {
            self.highlight_color = other.highlight_color;
        }
    }
}

impl Merge for Keybindings {
    fn merge(&mut self, other: Self) {
        // 只有当新快捷键不是 null 字符且不等于默认值时才更新
        if other.quit != '\0' && other.quit != default_quit_key() {
            self.quit = other.quit;
        }
        
        if other.up != '\0' && other.up != default_up_key() {
            self.up = other.up;
        }
        
        if other.down != '\0' && other.down != default_down_key() {
            self.down = other.down;
        }
        
        if other.left != '\0' && other.left != default_left_key() {
            self.left = other.left;
        }
        
        if other.right != '\0' && other.right != default_right_key() {
            self.right = other.right;
        }
        
        if other.confirm != '\0' && other.confirm != default_confirm_key() {
            self.confirm = other.confirm;
        }
        
        if other.delete != '\0' && other.delete != default_delete_key() {
            self.delete = other.delete;
        }
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
        assert_eq!(settings.cleaning.default_mode, CleaningMode::Smart);
        assert!(settings.cleaning.preserve_originals);
        assert_eq!(settings.cleaning.min_file_size, 1024);
        assert_eq!(settings.ui.theme.primary_color, "blue");
        assert_eq!(settings.ui.keybindings.quit, 'q');
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
            deserialized.cleaning.default_mode,
            settings.cleaning.default_mode
        );
        assert_eq!(
            deserialized.ui.keybindings.quit,
            settings.ui.keybindings.quit
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
            default_mode: CleaningMode::Smart,
            preserve_originals: true,
            min_file_size: 1024,
            scan_result_save_path: Some(PathBuf::from("/first/temp"))
        };
        
        let other = CleaningSettings {
            default_mode: CleaningMode::Auto,
            preserve_originals: false,
            min_file_size: 2048,
            scan_result_save_path: Some(PathBuf::from("/second/temp"))
        };
        
        base.merge(other);
        
        assert_eq!(base.default_mode, CleaningMode::Auto);
        assert!(!base.preserve_originals);
        assert_eq!(base.min_file_size, 2048);
        assert_eq!(base.scan_result_save_path, Some(PathBuf::from("/second/temp")))
    }
    
    #[test]
    fn test_merge_theme_settings() {
        let mut base = ThemeSettings {
            primary_color: "blue".to_string(),
            secondary_color: "green".to_string(),
            highlight_color: "yellow".to_string(),
        };
        
        let other = ThemeSettings {
            primary_color: "red".to_string(),
            secondary_color: "green".to_string(), // 与默认值相同，不应该更新
            highlight_color: "purple".to_string(),
        };
        
        base.merge(other);
        
        assert_eq!(base.primary_color, "red");
        assert_eq!(base.secondary_color, "green"); // 保持原值，因为 other 的值等于默认值
        assert_eq!(base.highlight_color, "purple");
    }
    
    #[test]
    fn test_merge_keybindings() {
        let mut base = Keybindings {
            quit: 'q',
            up: 'k',
            down: 'j',
            left: 'h',
            right: 'l',
            confirm: '\n',
            delete: 'd',
        };
        
        let other = Keybindings {
            quit: 'x',
            up: 'w',
            down: 'j', // 与默认值相同，不应该更新
            left: 'a',
            right: 's',
            confirm: '\n', // 与默认值相同，不应该更新
            delete: 'f',
        };
        
        base.merge(other);
        
        assert_eq!(base.quit, 'x');
        assert_eq!(base.up, 'w');
        assert_eq!(base.down, 'j'); // 保持原值，因为 other 的值等于默认值
        assert_eq!(base.left, 'a');
        assert_eq!(base.right, 's');
        assert_eq!(base.confirm, '\n'); // 保持原值，因为 other 的值等于默认值
        assert_eq!(base.delete, 'f');
    }
    
    #[test]
    fn test_merge_complete_settings() {
        let mut base = Settings::default();
        
        let other = Settings {
            wechat: WechatSettings {
                cache_path: Some(PathBuf::from("/custom/path")),
                cache_patterns: "custom_pattern".to_string(),
            },
            cleaning: CleaningSettings {
                default_mode: CleaningMode::Auto,
                preserve_originals: false,
                min_file_size: 4096,
                scan_result_save_path: Some(PathBuf::from("/second/temp"))
            },
            ui: UiSettings {
                theme: ThemeSettings {
                    primary_color: "purple".to_string(),
                    secondary_color: "orange".to_string(),
                    highlight_color: "cyan".to_string(),
                },
                keybindings: Keybindings {
                    quit: 'x',
                    up: 'w',
                    down: 's',
                    left: 'a',
                    right: 'd',
                    confirm: ' ',
                    delete: 'r',
                },
            },
        };
        
        base.merge(other);
        
        // 验证合并结果
        assert_eq!(base.wechat.cache_path, Some(PathBuf::from("/custom/path")));
        assert_eq!(base.wechat.cache_patterns, "custom_pattern".to_string());
        assert_eq!(base.cleaning.default_mode, CleaningMode::Auto);
        assert!(!base.cleaning.preserve_originals);
        assert_eq!(base.cleaning.min_file_size, 4096);
        assert_eq!(base.ui.theme.primary_color, "purple");
        assert_eq!(base.ui.theme.secondary_color, "orange");
        assert_eq!(base.ui.theme.highlight_color, "cyan");
        assert_eq!(base.ui.keybindings.quit, 'x');
        assert_eq!(base.ui.keybindings.up, 'w');
    }
}
