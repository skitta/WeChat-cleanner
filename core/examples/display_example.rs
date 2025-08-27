#[cfg(feature = "display")]
use wechat_cleaner::display;

#[cfg(feature = "display")]
use wechat_cleaner::display::*;

#[cfg(feature = "display")]
#[derive(Display)]
pub struct CleaningPreview {
    pub files_to_delete: Vec<PreviewGroup>,
    #[display(summary, name="预计清理空间")]
    pub estimated_freed_space: u64,
    #[display(details, name="文件数量")]
    pub estimated_files_count: usize,
}

#[cfg(feature = "display")]
#[derive(Display)]
pub struct PreviewGroup {
    #[display(summary, name="分组名称")]
    pub group_name: String,
    #[display(summary, name="文件大小")]
    pub total_size: u64,
    #[display(summary, name="文件列表")]
    pub file_paths: Vec<String>,
}

#[cfg(feature = "display")]
fn main() {
    let preview_group1 = PreviewGroup {
        group_name: "重复图片".to_string(),
        total_size: 1024 * 1024 * 10, // 10MB
        file_paths: vec![
            "/path/to/image1.jpg".to_string(),
            "/path/to/image2.jpg".to_string(),
        ],
    };

    let preview_group2 = PreviewGroup {
        group_name: "缓存文件".to_string(),
        total_size: 1024 * 1024 * 50, // 50MB
        file_paths: vec![
            "/path/to/cache1.tmp".to_string(),
            "/path/to/cache2.tmp".to_string(),
            "/path/to/cache3.tmp".to_string(),
        ],
    };

    let cleaning_preview = CleaningPreview {
        files_to_delete: vec![preview_group1, preview_group2],
        estimated_freed_space: 1024 * 1024 * 60, // 60MB
        estimated_files_count: 5,
    };

    println!("=== 清理预览摘要 ===");
    println!("{}", cleaning_preview.display_summary());
    
    println!("\n=== 清理预览详情 ===");
    println!("{}", cleaning_preview.display_details());
    
    println!("\n=== 分组详情 ===");
    for (i, group) in cleaning_preview.files_to_delete.iter().enumerate() {
        println!("分组 {}: {}", i + 1, group.display_details());
    }
}

#[cfg(not(feature = "display"))]
fn main() {
    println!("Display feature 未启用，请使用 --features display 来启用此功能");
}