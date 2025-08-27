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
use wechat_cleaner::config::ConfigManager;
use wechat_cleaner::core::scanner::{FileScanner, ScanResult};
use wechat_cleaner::core::cleaner::FileCleaner;
use wechat_cleaner::core::progressor::Progress;
use wechat_cleaner::core::file_utils::HasSize;
use wechat_cleaner::config::settings::{CleaningMode, Settings};
use std::io::{self, Write};

/// 应用错误类型
type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

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
        
        // 显示结果统计
        self.display_scan_summary(result, save_path);
        
        // 显示详细信息（如果需要）
        if verbose {
            self.display_scan_details(result);
        }
        
        Ok(())
    }

    /// 显示扫描摘要
    fn display_scan_summary(&self, result: &ScanResult, save_path: &std::path::Path) {
        println!("总文件数: {}", result.total_files_count);
        println!("发现 {} 份重复文件", result.duplicate_count);
        println!("扫描耗时: {:?}", result.scan_time);
        println!("扫描结果已保存到: {}", save_path.display());
    }

    /// 显示扫描详细信息
    fn display_scan_details(&self, result: &ScanResult) {
        for (hash, files) in &result.duplicate_files {
            println!("\n重复文件组 (哈希: {}):", hash);
            for file in files {
                println!("  - {} (大小: {} 字节, 修改时间: {:?})", 
                         file.path.display(), file.size(), file.modified);
            }
        }
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
    /// 预览清理操作
    Preview {
        /// 清理模式: auto
        #[arg(short, long, default_value = "auto")]
        mode: String,
        
        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,
    },
    /// 清理重复文件
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
        Some(Commands::Preview { mode, verbose }) => {
            let handler = PreviewHandler::new(&ops);
            handler.execute(mode, *verbose)
        }
        Some(Commands::Clean { mode, verbose: _ }) => {
            let handler = CleanerHandler::new(&ops);
            handler.execute(mode)
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

/// 预览操作处理器
struct PreviewHandler<'a> {
    ops: &'a CliOperations,
}

impl<'a> PreviewHandler<'a> {
    fn new(ops: &'a CliOperations) -> Self {
        Self { ops }
    }

    /// 执行预览操作
    fn execute(&self, mode: &str, verbose: bool) -> AppResult<()> {
        let cleaning_mode = self.ops.parse_cleaning_mode(mode);
        
        let scan_result = match self.ops.load_scan_result()? {
            Some(result) => result,
            None => return Ok(()), // 没有扫描结果，已显示提示
        };
        
        if scan_result.duplicate_count == 0 {
            println!("未发现重复文件，无需清理");
            return Ok(());
        }
        
        let preview = self.generate_preview(&scan_result, cleaning_mode)?;
        self.display_and_confirm_preview(&preview, verbose)?;
        
        Ok(())
    }

    /// 生成清理预览
    fn generate_preview(&self, scan_result: &ScanResult, mode: CleaningMode) -> AppResult<wechat_cleaner::core::cleaner::CleaningPreview> {
        let cleaner = FileCleaner::new(self.ops.settings().clone());
        cleaner.preview_cleaning(scan_result, mode)
            .map_err(|e| format!("预览失败: {}", e).into())
    }

    /// 显示预览并获取用户确认
    fn display_and_confirm_preview(&self, preview: &wechat_cleaner::core::cleaner::CleaningPreview, verbose: bool) -> AppResult<()> {
        // 显示预览结果
        let preview_text = if verbose {
            preview.display_details()
        } else {
            preview.display_summary()
        };
        println!("{}", preview_text);
        
        // 检查是否有文件需要清理
        if preview.estimated_files_count == 0 {
            println!("没有需要清理的文件");
            return Ok(());
        }
        
        // 获取用户确认
        let should_clean = self.ops.get_user_confirmation("预览结果")?;
        
        if should_clean {
            let cleaner_handler = CleanerHandler::new(self.ops);
            cleaner_handler.execute_from_preview(preview)?;
        } else {
            println!("清理已取消");
        }
        
        Ok(())
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

    /// 执行直接清理（不经过预览）
    fn execute(&self, mode: &str) -> AppResult<()> {
        let cleaning_mode = self.ops.parse_cleaning_mode(mode);
        
        let scan_result = match self.ops.load_scan_result()? {
            Some(result) => result,
            None => return Ok(()), // 没有扫描结果，已显示提示
        };
        
        if scan_result.duplicate_count == 0 {
            println!("未发现重复文件，无需清理");
            return Ok(());
        }
        
        self.execute_cleaning(&scan_result, cleaning_mode)?;
        Ok(())
    }

    /// 基于预览结果执行清理
    fn execute_from_preview(&self, preview: &wechat_cleaner::core::cleaner::CleaningPreview) -> AppResult<()> {
        let pb = self.ops.create_progress_bar("正在清理重复文件...")?;
        let mut cleaner = self.create_cleaner_with_progress(pb.clone())?;
        
        cleaner.clean_from_preview(preview)?;
        
        pb.finish();
        self.display_cleanup_results(&cleaner)?;
        self.cleanup_scan_result_file()?;
        
        Ok(())
    }

    /// 执行常规清理
    fn execute_cleaning(&self, scan_result: &ScanResult, mode: CleaningMode) -> AppResult<()> {
        let pb = self.ops.create_progress_bar("正在清理重复文件...")?;
        let mut cleaner = self.create_cleaner_with_progress(pb.clone())?;
        
        cleaner.clean_all_duplicates(scan_result, mode)?;
        
        pb.finish();
        self.display_cleanup_results(&cleaner)?;
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
    fn display_cleanup_results(&self, cleaner: &FileCleaner) -> AppResult<()> {
        println!("清理完成！");
        println!("总共删除 {} 个文件", cleaner.files_deleted);
        println!("释放空间 {:.2} MB", 
                 cleaner.freed_space as f64 / (1024.0 * 1024.0));
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
        let settings = self.ops.settings();
        
        println!("当前配置:");
        println!("  微信缓存路径: {:?}", settings.wechat.cache_path);
        println!("  缓存文件模式: {:?}", settings.wechat.cache_patterns);
        println!("  默认清理模式: {:?}", settings.cleaning.default_mode);
        println!("  保留原始文件: {}", settings.cleaning.preserve_originals);
        println!("  最小文件大小: {} 字节", settings.cleaning.min_file_size);
        
        Ok(())
    }
}
