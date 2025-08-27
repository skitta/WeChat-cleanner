//! CLI包中使用display功能的示例
//! 
//! 这个示例展示了如何在CLI包中使用core包的display功能

use core::display;
use core::display::*;
use core::Display;

#[derive(Display)]
struct CleaningResult {
    #[display(summary, name="清理文件数量")]
    pub files_cleaned: usize,
    
    #[display(summary, name="释放空间")]
    pub space_freed: u64,
    
    #[display(details, name="清理路径")]
    pub cleaned_paths: Vec<String>,
    
    #[display(details, name="耗时")]
    pub duration_ms: u64,
}

fn main() {
    println!("=== CLI包中使用Display功能 ===\n");
    
    let result = CleaningResult {
        files_cleaned: 150,
        space_freed: 1024 * 1024 * 512, // 512MB
        cleaned_paths: vec![
            "/Users/user/Library/WeChat/cache1.tmp".to_string(),
            "/Users/user/Library/WeChat/cache2.tmp".to_string(),
            "/Users/user/Library/WeChat/images/duplicate1.jpg".to_string(),
            "/Users/user/Library/WeChat/images/duplicate2.jpg".to_string(),
        ],
        duration_ms: 5420,
    };
    
    println!("清理结果摘要:");
    println!("{}", result.display_summary());
    
    println!("\n清理结果详情:");
    println!("{}", result.display_details());
    
    // 测试个别字段的格式化
    println!("\n独立格式化测试:");
    println!("空间格式化: {}", format_size(result.space_freed));
    
    let paths = &result.cleaned_paths;
    println!("路径摘要: {}", paths.format_display_summary());
    println!("路径详情: {}", paths.format_display_details());
}