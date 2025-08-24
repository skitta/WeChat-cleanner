// src/config/mod.rs
pub mod settings;

use crate::errors::{Error, Result};
use settings::{Settings, Merge};
use std::path::PathBuf;

/// 配置管理器
pub struct ConfigManager {
    settings: Settings,
    config_path: Option<PathBuf>,
    /// 配置加载历史，用于追踪配置来源
    config_sources: Vec<ConfigSource>,
}

/// 配置来源信息
#[derive(Debug, Clone)]
pub struct ConfigSource {
    pub path: PathBuf,
    pub source_type: ConfigSourceType,
    pub loaded_at: std::time::SystemTime,
}

/// 配置来源类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSourceType {
    /// 内置默认配置
    Default,
    /// 用户级配置文件
    User,
}

impl ConfigManager {
    /// 创建配置管理器并加载配置
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            settings: Settings::default(),
            config_path: None,
            config_sources: Vec::new(),
        };

        manager.load()?;
        Ok(manager)
    }
    
    /// 获取配置来源列表
    pub fn config_sources(&self) -> &[ConfigSource] {
        &self.config_sources
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
        // 清空之前的配置来源
        self.config_sources.clear();
        
        // 1. 加载内置默认配置
        self.settings = Settings::default();
        self.add_config_source(ConfigSource {
            path: PathBuf::from("<built-in>"),
            source_type: ConfigSourceType::Default,
            loaded_at: std::time::SystemTime::now(),
        });

        // 2. 加载用户级配置文件
        if let Some(user_path) = Self::user_config_path() {
            if user_path.exists() {
                self.merge_from_file(&user_path, ConfigSourceType::User)?;
                self.config_path = Some(user_path);
            }
        }

        // 3. 验证配置
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
    fn merge_from_file(&mut self, path: &PathBuf, source_type: ConfigSourceType) -> Result<()> {
        let config_str = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("读取配置文件失败: {}", e)))?;

        let file_settings: Settings = toml::from_str(&config_str)
            .map_err(|e| Error::Config(format!("解析配置文件失败: {}", e)))?;

        self.merge_settings(file_settings);
        
        // 记录配置来源
        self.add_config_source(ConfigSource {
            path: path.clone(),
            source_type,
            loaded_at: std::time::SystemTime::now(),
        });
        
        Ok(())
    }

    /// 合并配置设置
    fn merge_settings(&mut self, new_settings: Settings) {
        self.settings.merge(new_settings);
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
    
    /// 添加配置来源记录
    fn add_config_source(&mut self, source: ConfigSource) {
        self.config_sources.push(source);
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
        manager.merge_from_file(&config_path, ConfigSourceType::User).unwrap();
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
    
    #[test]
    fn test_config_sources_tracking() {
        let manager = ConfigManager::new().unwrap();
        let sources = manager.config_sources();
        
        // 应该至少有一个默认配置来源
        assert!(!sources.is_empty());
        assert_eq!(sources[0].source_type, ConfigSourceType::Default);
        assert_eq!(sources[0].path, PathBuf::from("<built-in>"));
    }
    
    #[test]
    fn test_merge_priority() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        
        // 创建简单的配置文件
        let mut file = File::create(&config_path).unwrap();
        file.write_all(
            b"[wechat]\ncache_patterns = [\"test_pattern\"]\n\n[cleaning]\ndefault_mode = \"auto\"\npreserve_originals = false\nmin_file_size = 2048\n\n[ui.theme]\nprimary_color = \"red\"\nsecondary_color = \"green\"\nhighlight_color = \"yellow\"\n\n[ui.keybindings]\nquit = 'x'\nup = 'w'\ndown = 's'\nleft = 'a'\nright = 'd'\nconfirm = ' '\ndelete = 'r'\n"
        ).unwrap();
        
        let mut manager = ConfigManager::new().unwrap();
        let original_size = manager.settings.cleaning.min_file_size;
        
        // 加载文件配置
        manager.merge_from_file(&config_path, ConfigSourceType::User).unwrap();
        
        // 文件配置应该覆盖默认配置
        assert_ne!(manager.settings.cleaning.min_file_size, original_size);
        assert_eq!(manager.settings.cleaning.min_file_size, 2048);
        assert_eq!(manager.settings.ui.theme.primary_color, "red");
        assert_eq!(manager.settings.ui.keybindings.quit, 'x');
    }
}
