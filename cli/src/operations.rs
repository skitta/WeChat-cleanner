//! CLI 操作核心模块
//!
//! 提供 CLI 操作的基础设施，包括进度条配置和用户交互功能。

use indicatif::{ProgressBar, ProgressStyle};
use core::config::ConfigManager;
use core::config::settings::{CleaningMode, Settings};
use std::io::{self, Write};

use crate::AppResult;

/// 进度条配置
pub struct ProgressConfig {
    pub template: &'static str,
    pub tick_strings: &'static [&'static str],
    pub tick_interval: std::time::Duration,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            template: "{spinner:.green} {msg}",
            tick_strings: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            tick_interval: std::time::Duration::from_millis(100),
        }
    }
}

/// CLI 操作核心结构
pub struct CliOperations {
    pub config_manager: ConfigManager,
}

impl CliOperations {
    /// 创建新的 CLI 操作实例
    pub fn new() -> AppResult<Self> {
        let config_manager = ConfigManager::new()?;
        Ok(Self { config_manager })
    }

    /// 获取应用设置
    pub fn settings(&self) -> &Settings {
        self.config_manager.settings()
    }

    /// 解析清理模式
    pub fn parse_cleaning_mode(&self, mode: &str) -> CleaningMode {
        match mode.to_lowercase().as_str() {
            "auto" => CleaningMode::Auto,
            _ => {
                eprintln!("无效的清理模式: {}，使用默认的 auto 模式", mode);
                CleaningMode::Auto
            }
        }
    }

    /// 创建配置好的进度条
    pub fn create_progress_bar(&self) -> AppResult<ProgressBar> {
        let config = ProgressConfig::default();
        let pb = ProgressBar::new_spinner();
        
        pb.set_style(
            ProgressStyle::default_spinner()
                .template(config.template)?
                .tick_strings(config.tick_strings),
        );

        pb.enable_steady_tick(config.tick_interval);
        
        Ok(pb)
    }

    /// 获取用户确认
    pub fn get_user_confirmation(&self, prompt: &str) -> AppResult<bool> {
        println!("{}\n是否继续? (y/n): ", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().to_lowercase() == "y")
    }
}