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

mod display;
mod operations;
mod handlers;

use clap::{Parser, Subcommand};
use operations::CliOperations;
use handlers::{ScanHandler, CleanerHandler, ConfigHandler};
use display::display_error;

/// 应用错误类型
type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

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
    /// 清理重复文件（总是显示预览并要求确认）
    Clean {
        /// 清理模式: auto
        #[arg(short, long, default_value = "auto")]
        mode: String,
        
        /// 跳过确认，直接清理
        #[arg(short, long)]
        force: bool,
    },
    /// 显示配置信息
    Config,
}

fn main() {
    if let Err(err) = run() {
        display_error(err.as_ref());
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let cli = Cli::parse();
    let ops = CliOperations::new()?;

    match &cli.command {
        Some(Commands::Scan { verbose }) => {
            let handler = ScanHandler::new(&ops);
            handler.execute(*verbose)
        }
        Some(Commands::Clean { mode, force }) => {
            let handler = CleanerHandler::new(&ops);
            handler.execute(mode, *force)
        }
        Some(Commands::Config) => {
            let handler = ConfigHandler::new(&ops);
            handler.execute()
        }
        None => {
            println!("微信缓存清理工具");
            println!("使用 'wechat-cleaner --help' 查看可用命令");
            Ok(())
        }
    }
}
