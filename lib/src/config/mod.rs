// src/config/mod.rs
pub mod settings;

use crate::errors::{Error, Result};
use settings::Settings;
use std::path::PathBuf;

/// 配置管理器
pub struct ConfigManager {
    settings: Settings,
    config_path: Option<PathBuf>,
}

impl ConfigManager {
    /// 创建配置管理器并加载配置
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            settings: Settings::default(),
            config_path: None,
        };

        manager.load()?;
        Ok(manager)
    }

    /// 获取当前配置
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// 获取当前配置的可变引用
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
    }

    /// 加载配置
    pub fn load(&mut self) -> Result<()> {
        // 1. 加载内置默认配置
        self.settings = Settings::default();

        // 3. 加载用户级配置文件
        if let Some(user_path) = Self::user_config_path() {
            if user_path.exists() {
                self.merge_from_file(&user_path)?;
                self.config_path = Some(user_path);
            }
        }

        // 5. 验证配置
        self.validate()?;

        Ok(())
    }

    /// 保存配置到用户配置文件
    pub fn save(&self) -> Result<()> {
        let path = self
            .config_path
            .clone()
            .or_else(Self::user_config_path)
            .ok_or_else(|| Error::Config("无法确定配置文件路径".to_string()))?;

        let toml = toml::to_string_pretty(&self.settings)
            .map_err(|e| Error::Config(format!("TOML序列化失败: {}", e)))?;

        std::fs::write(path, toml)
            .map_err(|e| Error::Config(format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    /// 从文件合并配置
    fn merge_from_file(&mut self, path: &PathBuf) -> Result<()> {
        let config_str = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("读取配置文件失败: {}", e)))?;

        let file_settings: Settings = toml::from_str(&config_str)
            .map_err(|e| Error::Config(format!("解析配置文件失败: {}", e)))?;

        self.merge_settings(file_settings);
        Ok(())
    }

    /// 合并配置设置
    fn merge_settings(&mut self, new_settings: Settings) {
        // 合并微信设置
        if new_settings.wechat.cache_path.is_some() {
            self.settings.wechat.cache_path = new_settings.wechat.cache_path;
        }

        if !new_settings.wechat.cache_patterns.is_empty() {
            self.settings.wechat.cache_patterns = new_settings.wechat.cache_patterns;
        }

        // 合并清理设置
        self.settings.cleaning.default_mode = new_settings.cleaning.default_mode;
        self.settings.cleaning.preserve_originals = new_settings.cleaning.preserve_originals;

        if new_settings.cleaning.min_file_size > 0 {
            self.settings.cleaning.min_file_size = new_settings.cleaning.min_file_size;
        }

        // 合并UI设置
        if !new_settings.ui.theme.primary_color.is_empty() {
            self.settings.ui.theme.primary_color = new_settings.ui.theme.primary_color;
        }

        if !new_settings.ui.theme.secondary_color.is_empty() {
            self.settings.ui.theme.secondary_color = new_settings.ui.theme.secondary_color;
        }

        if !new_settings.ui.theme.highlight_color.is_empty() {
            self.settings.ui.theme.highlight_color = new_settings.ui.theme.highlight_color;
        }

        // 合并快捷键设置
        let keybindings = &mut self.settings.ui.keybindings;
        let new_keybindings = new_settings.ui.keybindings;

        if new_keybindings.quit != '\0' {
            keybindings.quit = new_keybindings.quit;
        }
        if new_keybindings.up != '\0' {
            keybindings.up = new_keybindings.up;
        }
        if new_keybindings.down != '\0' {
            keybindings.down = new_keybindings.down;
        }
        if new_keybindings.left != '\0' {
            keybindings.left = new_keybindings.left;
        }
        if new_keybindings.right != '\0' {
            keybindings.right = new_keybindings.right;
        }
        if new_keybindings.confirm != '\0' {
            keybindings.confirm = new_keybindings.confirm;
        }
        if new_keybindings.delete != '\0' {
            keybindings.delete = new_keybindings.delete;
        }
    }

    /// 验证配置
    fn validate(&self) -> Result<()> {
        let settings = &self.settings;

        // 验证微信缓存路径
        if let Some(path) = &settings.wechat.cache_path {
            if !path.exists() {
                return Err(Error::Config(format!(
                    "微信缓存路径不存在: {}",
                    path.display()
                )));
            }
        }

        // 验证最小文件大小
        if settings.cleaning.min_file_size > 100 * 1024 * 1024 {
            return Err(Error::Config(format!(
                "最小文件大小 {} 过大",
                settings.cleaning.min_file_size
            )));
        }

        // 验证快捷键冲突
        let keys = [
            settings.ui.keybindings.quit,
            settings.ui.keybindings.up,
            settings.ui.keybindings.down,
            settings.ui.keybindings.left,
            settings.ui.keybindings.right,
            settings.ui.keybindings.confirm,
            settings.ui.keybindings.delete,
        ];

        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                if keys[i] == keys[j] {
                    return Err(Error::Config(format!(
                        "快捷键冲突: '{}' 被分配了多个功能",
                        keys[i]
                    )));
                }
            }
        }

        Ok(())
    }

    /// 获取用户级配置文件路径
    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("wechat-cleaner/config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use settings::CleaningMode;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let manager = ConfigManager::new().unwrap();
        let settings = manager.settings();

        assert!(
            settings
                .wechat
                .cache_patterns
                .contains(&r"\(\d+\)\.[a-zA-Z0-9]+$".to_string())
        );
        assert_eq!(settings.cleaning.default_mode, CleaningMode::Smart);
        assert_eq!(settings.cleaning.min_file_size, 1024);
        assert_eq!(settings.ui.theme.primary_color, "blue");
        assert_eq!(settings.ui.keybindings.quit, 'q');
    }

    #[test]
    fn test_load_from_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let mut file = File::create(&config_path).unwrap();
        file.write_all(
            b"
            [wechat]
            cache_path = \"/custom/path\"
            cache_patterns = [\"custom_pattern\"]

            [cleaning]
            default_mode = \"auto\"
            preserve_originals = false
            min_file_size = 2048

            [ui.theme]
            primary_color = \"red\"
            secondary_color = \"green\"
            highlight_color = \"yellow\"

            [ui.keybindings]
            quit = 'x'
            up = 'w'
        ",
        )
        .unwrap();

        let mut manager = ConfigManager::new().unwrap();
        manager.merge_from_file(&config_path).unwrap();
        let settings = manager.settings();

        assert_eq!(
            settings.wechat.cache_path,
            Some(PathBuf::from("/custom/path"))
        );
        assert_eq!(settings.wechat.cache_patterns, vec!["custom_pattern"]);
        assert_eq!(settings.cleaning.default_mode, CleaningMode::Auto);
        assert!(!settings.cleaning.preserve_originals);
        assert_eq!(settings.cleaning.min_file_size, 2048);
        assert_eq!(settings.ui.theme.primary_color, "red");
        assert_eq!(settings.ui.keybindings.quit, 'x');
        assert_eq!(settings.ui.keybindings.up, 'w');
    }

    #[test]
    fn test_config_validation() {
        let mut manager = ConfigManager::new().unwrap();

        // 测试无效路径
        manager.settings_mut().wechat.cache_path = Some(PathBuf::from("/invalid/path"));
        let result = manager.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("微信缓存路径不存在")
        );

        // 测试文件大小过大
        manager.settings_mut().wechat.cache_path = None;
        manager.settings_mut().cleaning.min_file_size = 200 * 1024 * 1024;
        let result = manager.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("最小文件大小"));

        // 测试快捷键冲突
        manager.settings_mut().cleaning.min_file_size = 1024;
        manager.settings_mut().ui.keybindings.quit = 'a';
        manager.settings_mut().ui.keybindings.up = 'a';
        let result = manager.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("快捷键冲突"));
    }

    #[test]
    fn test_save_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let mut manager = ConfigManager::new().unwrap();
        manager.config_path = Some(config_path.clone());

        // 修改一些设置
        manager.settings_mut().ui.theme.primary_color = "purple".to_string();
        manager.settings_mut().cleaning.min_file_size = 8192;

        // 保存配置
        manager.save().unwrap();

        // 验证文件内容
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("primary_color = \"purple\""));
        assert!(content.contains("min_file_size = 8192"));
    }
}
