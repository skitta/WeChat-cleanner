// src/config/mod.rs
pub mod settings;

use crate::errors::{Error, Result};
use settings::{Settings, Merge};
use std::path::PathBuf;

/// 配置管理器
pub struct ConfigManager {
    settings: Settings,
}

impl ConfigManager {
    /// 创建配置管理器并加载配置
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            settings: Settings::default(),
        };

        manager.load()?;
        Ok(manager)
    }
    
    /// 获取当前配置
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// 加载配置
    fn load(&mut self) -> Result<()> {
        
        // 1. 加载内置默认配置
        self.settings = Settings::default();

        // 2. 加载用户级配置文件
        if let Some(user_path) = Self::user_config_path() {
            if user_path.exists() {
                self.merge_from_file(&user_path)?;
            }
        }

        Ok(())
    }

    /// 从文件合并配置
    fn merge_from_file(&mut self, path: &PathBuf) -> Result<()> {
        let config_str = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("读取配置文件失败: {}", e)))?;

        let file_settings: Settings = toml::from_str(&config_str)
            .map_err(|e| Error::Config(format!("解析配置文件失败: {}", e)))?;

        self.settings.merge(file_settings);
        
        Ok(())
    }

    /// 获取用户级配置文件路径
    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("wechat-cleaner/config.toml"))
    }
}
