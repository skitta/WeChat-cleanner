//! 显示函数模块
//!
//! 提供统一的显示输出功能，用于格式化显示扫描结果、清理结果和配置信息。

use core::config::settings::Settings;
use core::display::Display;

/// 通用显示函数，支持详细和摘要模式
pub fn display<T: Display>(item: &T, verbose: bool) {
    if verbose {
        println!("{}", item.display_details());
    } else {
        println!("{}", item.display_summary());
    }
}

/// 显示配置信息
pub fn display_config(settings: &Settings, verbose: bool) {
    println!("当前配置:");
    println!("  微信缓存路径: {:?}", settings.wechat.cache_path);
    println!("  默认清理模式: {:?}", settings.cleaning.mode);
    
    if verbose {
        println!("  缓存文件模式: {:?}", settings.wechat.cache_patterns);
    }
}

/// 显示错误信息
pub fn display_error(err: &dyn std::error::Error) {
    eprintln!("❌ {}", err);
    
    // 显示错误链
    let mut source = err.source();
    while let Some(err) = source {
        eprintln!("   └─ 原因: {}", err);
        source = err.source();
    }
}