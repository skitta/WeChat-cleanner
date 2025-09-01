//! CleaningPreview Display 特性集成测试
//!
//! 测试 CleaningPreview 结构体的 Display 特性派生功能，
//! 包括 summary 和 details 模式的显示效果。

#[cfg(feature = "display")]
mod display_tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    use wechat_cleaner::cleaner::{CleaningPreview, PreviewGroup};
    use wechat_cleaner::display::Display;
    use wechat_cleaner::file_utils::FileInfo;

    /// 创建测试用的临时文件和对应的 FileInfo
    fn create_test_file_info_with_tempdir(temp_dir: &TempDir, name: &str, content: &[u8]) -> FileInfo {
        let file_path = temp_dir.path().join(name);
        
        // 创建父目录（如果需要）
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        
        // 写入文件内容
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content).unwrap();
        file.flush().unwrap();
        
        // 使用 JSON 反序列化创建 FileInfo（因为构造函数是私有的）
        // 这是一个权宜之计，实际项目中可能需要提供公共的测试工具函数
        let json_str = format!(
            r#"{{"path": "{}", "size": {}, "modified": 1640995200}}"#,
            file_path.display(),
            content.len()
        );
        serde_json::from_str::<FileInfo>(&json_str).unwrap()
    }

    /// 创建测试用的 CleaningPreview
    fn create_test_cleaning_preview() -> (CleaningPreview, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut file_groups = HashMap::new();
        
        // 创建第一个预览组
        let content1 = b"test image content";
        let file_to_keep1 = create_test_file_info_with_tempdir(&temp_dir, "dir1/photo1.jpg", content1);
        let files_to_delete1 = vec![
            create_test_file_info_with_tempdir(&temp_dir, "dir1/photo1_copy.jpg", content1),
            create_test_file_info_with_tempdir(&temp_dir, "dir1/photo1_duplicate.jpg", content1),
        ];
        
        let group1 = PreviewGroup {
            file_to_keep: file_to_keep1,
            files_to_delete: files_to_delete1,
        };
        
        file_groups.insert(PathBuf::from(temp_dir.path().join("dir1")), group1);
        
        // 创建第二个预览组
        let content2 = b"test document content with more data to make it larger";
        let file_to_keep2 = create_test_file_info_with_tempdir(&temp_dir, "dir2/document.pdf", content2);
        let files_to_delete2 = vec![
            create_test_file_info_with_tempdir(&temp_dir, "dir2/document_backup.pdf", content2),
        ];
        
        let group2 = PreviewGroup {
            file_to_keep: file_to_keep2,
            files_to_delete: files_to_delete2,
        };
        
        file_groups.insert(PathBuf::from(temp_dir.path().join("dir2")), group2);
        
        let preview = CleaningPreview {
            estimated_files_count: 3,
            estimated_freed_space: content1.len() as u64 * 2 + content2.len() as u64,
            file_groups,
        };
        
        (preview, temp_dir)
    }

    #[test]
    fn test_cleaning_preview_display_summary() {
        let (preview, _temp_dir) = create_test_cleaning_preview();
        let summary = preview.display_summary();
        
        println!("Actual Summary Output:");
        println!("{}", summary);
        
        // 验证 summary 包含预期的字段
        assert!(summary.contains("预计删除文件数: 3"), "Expected '预计删除文件数: 3' in: {}", summary);
        // 由于文件大小在测试中是动态计算的，我们只验证格式
        assert!(summary.contains("预计释放空间:"), "Expected '预计释放空间:' in: {}", summary);
        
        // 验证不包含详细信息（文件分组详情不应该在 summary 中显示）
        assert!(!summary.contains("dir1"));
        assert!(!summary.contains("dir2"));
        
        println!("Summary 显示效果:");
        println!("{}", summary);
    }

    #[test]
    fn test_cleaning_preview_display_details() {
        let (preview, _temp_dir) = create_test_cleaning_preview();
        let details = preview.display_details();
        
        println!("Actual Details Output:");
        println!("{}", details);
        
        // 验证 details 包含所有字段
        assert!(details.contains("预计删除文件数: 3"), "Expected '预计删除文件数: 3' in: {}", details);
        assert!(details.contains("预计释放空间:"), "Expected '预计释放空间:' in: {}", details);
        // 在 details 模式下，HashMap 会显示详细的结构信息，而不是简单的 "2 entries"
        assert!(details.contains("文件分组详情:"), "Expected to contain '文件分组详情:' in: {}", details);
        assert!(details.contains("PreviewGroup"), "Expected 'PreviewGroup' in: {}", details);
        
        println!("Details 显示效果:");
        println!("{}", details);
    }

    #[test]
    fn test_cleaning_preview_display_with_verbose_false() {
        let (preview, _temp_dir) = create_test_cleaning_preview();
        let output = preview.display(false);
        
        // verbose=false 应该显示 summary
        let summary = preview.display_summary();
        assert_eq!(output, summary);
    }

    #[test]
    fn test_cleaning_preview_display_with_verbose_true() {
        let (preview, _temp_dir) = create_test_cleaning_preview();
        let output = preview.display(true);
        
        // verbose=true 应该显示 details
        let details = preview.display_details();
        assert_eq!(output, details);
    }

    #[test]
    fn test_preview_group_display_summary() {
        let temp_dir = TempDir::new().unwrap();
        let content = b"test file content";
        
        let file_to_keep = create_test_file_info_with_tempdir(&temp_dir, "original.jpg", content);
        let files_to_delete = vec![
            create_test_file_info_with_tempdir(&temp_dir, "copy1.jpg", content),
            create_test_file_info_with_tempdir(&temp_dir, "copy2.jpg", content),
        ];
        
        let group = PreviewGroup {
            file_to_keep,
            files_to_delete,
        };
        
        let summary = group.display_summary();
        
        // 验证包含删除文件列表的数量
        assert!(summary.contains("删除文件列表: 2 items"));
        
        println!("PreviewGroup Summary:");
        println!("{}", summary);
    }

    #[test]
    fn test_preview_group_display_details() {
        let temp_dir = TempDir::new().unwrap();
        let content = b"test file content";
        
        let file_to_keep = create_test_file_info_with_tempdir(&temp_dir, "original.jpg", content);
        let files_to_delete = vec![
            create_test_file_info_with_tempdir(&temp_dir, "copy1.jpg", content),
        ];
        
        let group = PreviewGroup {
            file_to_keep,
            files_to_delete,
        };
        
        let details = group.display_details();
        
        // 验证包含具体的文件信息
        assert!(details.contains("保留文件:"));
        assert!(details.contains("删除文件列表:"));
        
        println!("PreviewGroup Details:");
        println!("{}", details);
    }

    #[test]
    fn test_empty_cleaning_preview() {
        let empty_preview = CleaningPreview {
            estimated_files_count: 0,
            estimated_freed_space: 0,
            file_groups: HashMap::new(),
        };
        
        let summary = empty_preview.display_summary();
        let details = empty_preview.display_details();
        
        println!("Empty Preview Summary:");
        println!("{}", summary);
        println!("Empty Preview Details:");
        println!("{}", details);
        
        // 验证空预览的显示
        assert!(summary.contains("预计删除文件数: 0"));
        assert!(summary.contains("预计释放空间: 0 B"));
        
        // 在 details 模式下，空 HashMap 会显示为 "{}"
        assert!(details.contains("文件分组详情:"), "Expected to contain '文件分组详情:' in: {}", details);
    }

    #[test]
    fn test_display_trait_object() {
        let (preview, _temp_dir) = create_test_cleaning_preview();
        
        // 测试作为 trait object 使用
        let display_obj: &dyn Display = &preview;
        let summary = display_obj.display_summary();
        let details = display_obj.display_details();
        
        assert!(summary.contains("预计删除文件数"));
        assert!(details.contains("文件分组详情"));
        
        println!("Trait Object 测试通过");
    }
}

#[cfg(not(feature = "display"))]
mod non_display_tests {
    #[test]
    fn test_display_feature_disabled() {
        // 当 display 特性被禁用时，确保代码仍然可以编译
        // 但是 Display trait 不会被实现
        println!("Display 特性已禁用，跳过 Display 相关测试");
    }
}