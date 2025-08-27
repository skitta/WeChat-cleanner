//! Display特性应用完整演示
//! 
//! 展示scanner、cleaner模块中使用display特性的效果

#[cfg(feature = "display")]
use wechat_cleaner::display::*;

#[cfg(feature = "display")]
use wechat_cleaner::{
    scanner::ScanResult,
    cleaner::{CleaningPreview, CleaningStats, PreviewGroup},
    file_utils::FileInfo,
};

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[cfg(feature = "display")]
fn create_mock_scan_result() -> ScanResult {
    let mut duplicate_files = HashMap::new();
    
    // 创建模拟的重复文件
    let file1 = create_mock_file_info("/path/to/duplicate1.jpg", 1024 * 512); // 512KB
    let file2 = create_mock_file_info("/path/to/duplicate2.jpg", 1024 * 512); // 512KB
    
    duplicate_files.insert("hash1".to_string(), vec![file1, file2]);
    
    ScanResult {
        total_files_count: 150,
        duplicate_count: 2,
        duplicate_files,
        scan_time: Duration::from_secs(3),
    }
}

#[cfg(feature = "display")]
fn create_mock_file_info(path: &str, size: u64) -> FileInfo {
    // 注意：这里我们直接构造FileInfo而不调用FileInfo::new
    // 因为路径可能不存在，这仅用于演示
    use serde_json;
    
    let json = format!(r#"{{
        "path": "{}",
        "size": {},
        "modified": 1640995200
    }}"#, path, size);
    
    serde_json::from_str(&json).unwrap()
}

#[cfg(feature = "display")]
fn create_mock_cleaning_preview() -> CleaningPreview {
    let file_to_keep = create_mock_file_info("/path/to/original.jpg", 1024 * 512);
    let files_to_delete = vec![
        create_mock_file_info("/path/to/duplicate1.jpg", 1024 * 512),
        create_mock_file_info("/path/to/duplicate2.jpg", 1024 * 512),
    ];
    
    let group = PreviewGroup {
        parent_path: PathBuf::from("/path/to"),
        file_to_keep,
        files_to_delete,
    };
    
    CleaningPreview::new(vec![group], 1024) // 1KB最小文件大小
}

#[cfg(feature = "display")]
fn create_mock_cleaning_stats() -> CleaningStats {
    let mut stats = CleaningStats::new();
    stats.add_deleted_file(1024 * 512); // 512KB
    stats.add_deleted_file(1024 * 256); // 256KB
    stats
}

#[cfg(feature = "display")]
fn main() {
    println!("=== Display特性应用完整演示 ===\n");
    
    // 1. 演示ScanResult的display功能
    println!("1. ScanResult 显示演示:");
    let scan_result = create_mock_scan_result();
    
    println!("摘要模式:");
    println!("{}", scan_result.display_summary());
    
    println!("\n详细模式:");
    println!("{}", scan_result.display_details());
    
    // 2. 演示CleaningPreview的display功能
    println!("\n\n2. CleaningPreview 显示演示:");
    let preview = create_mock_cleaning_preview();
    
    println!("摘要模式:");
    println!("{}", preview.display_summary());
    
    println!("\n详细模式:");
    println!("{}", preview.display_details());
    
    // 3. 演示CleaningStats的display功能
    println!("\n\n3. CleaningStats 显示演示:");
    let stats = create_mock_cleaning_stats();
    
    println!("摘要模式:");
    println!("{}", stats.display_summary());
    
    // 4. 演示FileInfo的显示功能（通过DisplayValue trait）
    println!("\n\n4. FileInfo 显示演示:");
    let file_info = create_mock_file_info("/path/to/example.jpg", 1024 * 1024 * 2); // 2MB
    
    println!("基本显示: {}", file_info.format_display());
    println!("摘要显示: {}", file_info.format_display_summary());
    println!("详细显示: {}", file_info.format_display_details());
    
    // 5. 演示Vec<T>的特殊显示模式
    println!("\n\n5. Vec<FileInfo> 显示演示:");
    let file_list = vec![
        create_mock_file_info("/path/to/file1.jpg", 1024 * 500),
        create_mock_file_info("/path/to/file2.png", 1024 * 750),
        create_mock_file_info("/path/to/file3.gif", 1024 * 300),
    ];
    
    println!("Vec摘要显示: {}", file_list.format_display_summary());
    println!("Vec详细显示:\n{}", file_list.format_display_details());
    
    println!("\n=== 演示完成 ===");
}

#[cfg(not(feature = "display"))]
fn main() {
    println!("Display功能未启用，请使用 --features display 来启用此功能");
    println!("命令: cargo run --example complete_display_demo --features display");
}