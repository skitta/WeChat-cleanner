use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use lib::config::ConfigManager;
use lib::core::scanner::{FileScanner, ScanResult};
use lib::core::cleaner::FileCleaner;
use lib::core::progressor::Progress;

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
    /// 清理重复文件
    Clean {
        /// 清理模式: auto, smart
        #[arg(short, long, default_value = "smart")]
        mode: String,
        
        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,
    },
    /// 显示配置信息
    Config,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Scan { verbose }) => {
            scan_files(*verbose)?;
        }
        Some(Commands::Clean { mode, verbose }) => {
            clean_files(mode, *verbose)?;
        }
        Some(Commands::Config) => {
            show_config()?;
        }
        None => {
            // 默认显示帮助信息
            println!("微信缓存清理工具");
            println!("使用 'wechat-cleaner --help' 查看可用命令");
        }
    }

    Ok(())
}

fn scan_files(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    // 创建进度条
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")?
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message("正在扫描微信缓存文件...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    // 加载配置
    let config_manager = ConfigManager::new()?;
    let settings = config_manager.settings().clone();
    
    // 创建扫描器并设置进度回调
    let pb_clone = pb.clone();
    let mut scanner = FileScanner::new(settings)
        .with_progress_callback(move |progress: &Progress| {
            if progress.is_completed() {
                pb_clone.finish_with_message(progress.display(|_,_,f| -> String {format!("{}: 完成!", f)}));
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
    
    // 执行扫描
    let result = scanner.scan()?;
    let save_result_path = result.save(None)?;
    
    // 显示结果
    println!("总文件数: {}", result.total_files_count);
    println!("发现 {} 份重复文件", result.duplicate_count);
    println!("扫描耗时: {:?}", result.scan_time);
    if save_result_path.exists() {
        println!("扫描结果已保存到: {}", save_result_path.display());
    }
    
    if verbose {
        for (hash, files) in &result.duplicate_files {
            println!("\n重复文件组 (哈希: {}):", hash);
            for file in files {
                println!("  - {} (大小: {} 字节, 修改时间: {:?})", 
                         file.path.display(), file.size, file.modified);
            }
        }
    }
    
    Ok(())
}

fn clean_files(mode: &str, _verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let config_manager = ConfigManager::new()?;
    let mut settings = config_manager.settings().clone();
    
    // 设置清理模式
    settings.cleaning.default_mode = match mode.to_lowercase().as_str() {
        "auto" => lib::config::settings::CleaningMode::Auto,
        "smart" => lib::config::settings::CleaningMode::Smart,
        _ => {
            eprintln!("无效的清理模式: {}，使用默认的 smart 模式", mode);
            lib::config::settings::CleaningMode::Smart
        }
    };
    
    // 尝试从临时文件加载扫描结果
    let results = match ScanResult::load(None) {
        Ok(results) => results,
        Err(e) => {
            println!("{}", e);
            println!("请先执行扫描命令: wechat-cleaner scan");
            return Ok(());
        }
    };
    
    if results.duplicate_count == 0 {
        println!("未发现重复文件");
        results.delete(None)?;
        return Ok(());
    }
    
    // 创建进度条用于清理过程
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")?
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message("正在清理重复文件...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    let pb_clone = pb.clone();
    let mut cleaner = FileCleaner::new(settings.clone())
        .with_progress_callback(move |progress: &Progress| {
            if progress.is_completed() {
                pb_clone.set_message(format!("已清理完成"));
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
    
    // 执行清理
    cleaner.clean_all_duplicates(&results, settings.cleaning.default_mode)?;
    // 显示结果
    println!("清理完成！");
    println!("总共删除 {} 个文件", cleaner.files_deleted);
    println!("释放空间 {} MB", cleaner.freed_space / (1024 * 1024));
    results.delete(None)?;
    Ok(())
}

fn show_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = ConfigManager::new()?;
    let settings = config_manager.settings();
    
    println!("当前配置:");
    println!("  微信缓存路径: {:?}", settings.wechat.cache_path);
    println!("  缓存文件模式: {:?}", settings.wechat.cache_patterns);
    println!("  默认清理模式: {:?}", settings.cleaning.default_mode);
    println!("  保留原始文件: {}", settings.cleaning.preserve_originals);
    println!("  最小文件大小: {} 字节", settings.cleaning.min_file_size);
    
    Ok(())
}
