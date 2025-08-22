// src/config/settings.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    /// 微信缓存路径，如果为None则自动检测
    pub cache_path: Option<PathBuf>,

    /// 用于识别微信自动生成副本的文件名模式
    #[serde(default = "default_cache_patterns")]
    pub cache_patterns: Vec<String>,
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
fn default_cache_patterns() -> Vec<String> {
    vec![
        r"\(\d+\)\.[a-zA-Z0-9]+$".to_string(),
        //r"-\d{13}\.".to_string(),
    ]
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
                cache_path: None,
                cache_patterns: default_cache_patterns(),
            },
            cleaning: CleaningSettings {
                default_mode: default_cleaning_mode(),
                preserve_originals: default_preserve_originals(),
                min_file_size: default_min_file_size(),
            },
            ui: UiSettings {
                theme: default_theme(),
                keybindings: default_keybindings(),
            },
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
}
