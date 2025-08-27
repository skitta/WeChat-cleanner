//! 微信缓存清理工具命令行界面
//!
//! 提供扫描、预览和清理重复文件的命令行工具。
//! 采用模块化设计和函数式编程风格，提供用户友好的交互体验。
//!
//! # 主要功能
//! - 扫描微信缓存目录中的重复文件
//! - 预览清理操作，显示将要删除的文件
//! - 执行安全的文件清理操作
//! - 配置管理和显示
//!
//! # 性能特性
//! - 实时进度显示
//! - 交互式确认机制
//! - 详细的统计信息

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use core::config::ConfigManager;
use core::scanner::{FileScanner, ScanResult};
use core::cleaner::FileCleaner;
use core::progressor::Progress;
use core::config::settings::{CleaningMode, Settings};
use std::io::{self, Write};

use core::display::Display;

/// 应用错误类型
type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 简单的显示函数
fn display_result(result: &ScanResult, verbose: bool, save_path: Option<&std::path::Path>) {
    if verbose {
        println!("{}", result.display_details());
    } else {
        println!("{}", result.display_summary());
    }
    
    if let Some(path) = save_path {
        println!("扫描结果已保存到: {}", path.display());
    }
}

fn display_stats(result: &core::cleaner::CleaningResult) {
    println!("{}", result.display_summary());
}

fn display_preview(preview: &core::cleaner::CleaningPreview, verbose: bool) {
    if verbose {
        println!("{}", preview.display_details());
    } else {
        println!("{}", preview.display_summary());
    }
}

fn display_config(settings: &Settings, verbose: bool) {
    println!("当前配置:");
    println!("  微信缓存路径: {:?}", settings.wechat.cache_path);
    println!("  默认清理模式: {:?}", settings.cleaning.default_mode);
    println!("  最小文件大小: {} 字节", settings.cleaning.min_file_size);
    
    if verbose {
        println!("  缓存文件模式: {:?}", settings.wechat.cache_patterns);
        println!("  保留原始文件: {}", settings.cleaning.preserve_originals);
    }
}

/// 进度条配置
struct ProgressConfig {
    template: &'static str,
    tick_strings: &'static [&'static str],
    tick_interval: std::time::Duration,
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

/// CLI 操作类型
struct CliOperations {
    config_manager: ConfigManager,
}

impl CliOperations {
    /// 创建新的 CLI 操作实例
    fn new() -> AppResult<Self> {
        let config_manager = ConfigManager::new()?;
        Ok(Self { config_manager })
    }

    /// 获取应用设置
    fn settings(&self) -> &Settings {
        self.config_manager.settings()
    }

    /// 解析清理模式
    fn parse_cleaning_mode(&self, mode: &str) -> CleaningMode {
        match mode.to_lowercase().as_str() {
            "auto" => CleaningMode::Auto,
            _ => {
                eprintln!("无效的清理模式: {}，使用默认的 auto 模式", mode);
                CleaningMode::Auto
            }
        }
    }

    /// 创建配置好的进度条
    fn create_progress_bar(&self, message: &str) -> AppResult<ProgressBar> {
        let config = ProgressConfig::default();
        let pb = ProgressBar::new_spinner();
        
        pb.set_style(
            ProgressStyle::default_spinner()
                .template(config.template)?
                .tick_strings(config.tick_strings),
        );
        
        pb.set_message(message.to_string());
        pb.enable_steady_tick(config.tick_interval);
        
        Ok(pb)
    }

    /// 加载扫描结果
    fn load_scan_result(&self) -> AppResult<Option<ScanResult>> {
        let result_path = self.settings()
            .cleaning
            .scan_result_save_path
            .as_ref()
            .ok_or("扫描结果文件路径未配置")?;
            
        match ScanResult::load(result_path) {
            Ok(results) => Ok(Some(results)),
            Err(_) => {
                println!("请先执行扫描命令: wechat-cleaner scan");
                Ok(None)
            }
        }
    }

    /// 获取用户确认
    fn get_user_confirmation(&self, prompt: &str) -> AppResult<bool> {
        print!("{}\n是否继续? (y/n): ", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().to_lowercase() == "y")
    }
}

/// 扫描操作处理器
struct ScanHandler<'a> {
    ops: &'a CliOperations,
}

impl<'a> ScanHandler<'a> {
    fn new(ops: &'a CliOperations) -> Self {
        Self { ops }
    }

    /// 执行文件扫描
    fn execute(&self, verbose: bool) -> AppResult<()> {
        let pb = self.ops.create_progress_bar("正在扫描微信缓存文件...")?;
        let mut scanner = self.create_scanner(pb.clone())?;
        
        let result = scanner.scan()?;
        
        if result.duplicate_count == 0 {
            println!("未发现重复文件");
            return Ok(());
        }
        
        self.save_and_display_results(&result, verbose)?;
        Ok(())
    }

    /// 创建配置好的扫描器
    fn create_scanner(&self, pb: ProgressBar) -> AppResult<FileScanner> {
        let settings = self.ops.settings().clone();
        let pb_clone = pb.clone();
        
        let scanner = FileScanner::new(settings)
            .with_progress_callback(move |progress: &Progress| {
                if progress.is_completed() {
                    pb_clone.finish_with_message(progress.display(|_,_,f| -> String {
                        format!("{}: 完成!", f)
                    }));
                } else {
                    pb_clone.set_message(progress.display(|current, total, msg| -> String {
                        if total > 0 {
                            format!("{}: {}/{}", msg, current, total)
                        } else {
                            msg.to_string()
                        }
                    }));
                }
            });
            
        Ok(scanner)
    }

    /// 保存并显示扫描结果
    fn save_and_display_results(&self, result: &ScanResult, verbose: bool) -> AppResult<()> {
        let save_path = self.ops.settings()
            .cleaning
            .scan_result_save_path
            .as_ref()
            .ok_or("无法保存扫描结果，因为路径不合法")?;
            
        result.save(save_path)?;
        
        // 使用简化显示函数
        display_result(result, verbose, Some(save_path));
        
        Ok(())
    }
}

/// 微信缓存清理工具
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 扫描重复文件
    Scan {
        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,
    },
    /// 清理重复文件（包含预览功能）
    Clean {
        /// 清理模式: auto
        #[arg(short, long, default_value = "auto")]
        mode: String,
        
        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,
    },
    /// 显示配置信息
    Config,
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();
    let ops = CliOperations::new()?;

    match &cli.command {
        Some(Commands::Scan { verbose }) => {
            let handler = ScanHandler::new(&ops);
            handler.execute(*verbose)
        }
        Some(Commands::Clean { mode, verbose }) => {
            let handler = CleanerHandler::new(&ops);
            handler.execute_with_preview(mode, *verbose)
        }
        Some(Commands::Config) => {
            let handler = ConfigHandler::new(&ops);
            handler.execute()
        }
        None => {
            // 默认显示帮助信息
            println!("微信缓存清理工具");
            println!("使用 'wechat-cleaner --help' 查看可用命令");
            Ok(())
        }
    }
}

/// 清理操作处理器
struct CleanerHandler<'a> {
    ops: &'a CliOperations,
}

impl<'a> CleanerHandler<'a> {
    fn new(ops: &'a CliOperations) -> Self {
        Self { ops }
    }

    /// 执行带预览的清理操作（新的主要入口）
    fn execute_with_preview(&self, mode: &str, verbose: bool) -> AppResult<()> {
        let cleaning_mode = self.ops.parse_cleaning_mode(mode);
        
        let scan_result = match self.ops.load_scan_result()? {
            Some(result) => result,
            None => return Ok(()), // 没有扫描结果，已显示提示
        };
        
        if scan_result.duplicate_count == 0 {
            println!("未发现重复文件，无需清理");
            return Ok(());
        }
        
        // 生成预览
        let preview_result = self.generate_preview(&scan_result, cleaning_mode)?;
        
        // 显示预览结果
        if let Some(preview) = preview_result.preview() {
            display_preview(preview, verbose);
            
            // 检查是否有文件需要清理
            if preview.estimated_files_count == 0 {
                println!("没有需要清理的文件");
                return Ok(());
            }
            
            // 获取用户确认
            let should_clean = self.ops.get_user_confirmation("预览结果")?;
            
            if should_clean {
                self.execute_from_preview(&preview_result)?;
            } else {
                println!("清理已取消");
            }
        } else {
            println!("预览生成失败");
        }
        
        Ok(())
    }

    /// 生成清理预览
    fn generate_preview(&self, scan_result: &ScanResult, mode: CleaningMode) -> AppResult<core::cleaner::CleaningResult> {
        let pb = self.ops.create_progress_bar("正在生成预览...")?;
        let mut cleaner = self.create_cleaner_with_progress(pb.clone())?;
        
        let result = cleaner.clean(scan_result, mode, true)?; // preview_only = true
        pb.finish();
        
        Ok(result)
    }

    /// 基于预览结果执行清理
    fn execute_from_preview(&self, preview_result: &core::cleaner::CleaningResult) -> AppResult<()> {
        let pb = self.ops.create_progress_bar("正在清理重复文件...")?;
        let mut cleaner = self.create_cleaner_with_progress(pb.clone())?;
        
        let result = cleaner.confirm_and_execute(preview_result)?;
        
        pb.finish();
        self.display_cleanup_results(&result)?;
        self.cleanup_scan_result_file()?;
        
        Ok(())
    }

    /// 创建带进度显示的清理器
    fn create_cleaner_with_progress(&self, pb: ProgressBar) -> AppResult<FileCleaner> {
        let settings = self.ops.settings().clone();
        let pb_clone = pb.clone();
        
        let cleaner = FileCleaner::new(settings)
            .with_progress_callback(move |progress: &Progress| {
                if progress.is_completed() {
                    pb_clone.set_message("已清理完成".to_string());
                } else {
                    pb_clone.set_message(progress.display(|curr, total, msg| {
                        if total > 0 {
                            format!("{}: {}/{}", msg, curr, total)
                        } else {
                            msg.to_string()
                        }
                    }));
                }
            });
            
        Ok(cleaner)
    }

    /// 显示清理结果
    fn display_cleanup_results(&self, result: &core::cleaner::CleaningResult) -> AppResult<()> {
        display_stats(result);
        Ok(())
    }

    /// 清理扫描结果文件
    fn cleanup_scan_result_file(&self) -> AppResult<()> {
        if let Some(result_path) = &self.ops.settings().cleaning.scan_result_save_path {
            let _ = std::fs::remove_file(result_path);
        }
        Ok(())
    }
}

/// 配置操作处理器
struct ConfigHandler<'a> {
    ops: &'a CliOperations,
}

impl<'a> ConfigHandler<'a> {
    fn new(ops: &'a CliOperations) -> Self {
        Self { ops }
    }

    /// 显示当前配置
    fn execute(&self) -> AppResult<()> {
        display_config(self.ops.settings(), true); // 配置默认显示详细信息
        Ok(())
    }
}
