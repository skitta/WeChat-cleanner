//! 展示改进后的显示效果示例

use core::display::*;
use std::collections::HashMap;

fn main() {
    println!("=== 微信清理工具 - 显示效果演示 ===\n");

    // 1. 基本数字显示
    let file_count: usize = 15;
    let total_size: u64 = 1024 * 1024 * 50; // 50MB
    
    println!("📊 基本信息显示：");
    println!("文件数量: {}", file_count.format_display());
    println!("总大小: {}", total_size.format_display());
    println!();

    // 2. 集合类型显示 - 空集合
    let empty_map: HashMap<String, String> = HashMap::new();
    let empty_vec: Vec<String> = Vec::new();
    
    println!("📋 空集合显示：");
    println!("空的配置: {}", empty_map.format_display());
    println!("空的文件列表: {}", empty_vec.format_display());
    println!();

    // 3. 集合类型显示 - 有内容
    let mut file_groups: HashMap<String, Vec<String>> = HashMap::new();
    file_groups.insert("照片".to_string(), vec!["IMG_001.jpg".to_string(), "IMG_002.jpg".to_string()]);
    file_groups.insert("文档".to_string(), vec!["report.pdf".to_string()]);
    
    println!("📁 文件分组摘要：");
    println!("文件分组: {}", file_groups.format_display_summary());
    println!();

    println!("📋 文件分组详情：");
    println!("文件分组: {}", file_groups.format_display_details());
    println!();

    // 4. Option 类型显示
    let some_config = Some("开启智能清理".to_string());
    let no_config: Option<String> = None;
    
    println!("⚙️  配置选项显示：");
    println!("清理模式: {}", some_config.format_display());
    println!("自动备份: {}", no_config.format_display());
    println!();

    println!("✨ 显示效果演示完成！");
}