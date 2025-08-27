/// Display trait for structures that can show summary and detailed information
pub trait Display {
    fn display_summary(&self) -> String;
    fn display_details(&self) -> String;
    fn display(&self, verbose: bool) -> String {
        if verbose {
            self.display_details()
        } else {
            self.display_summary()
        }
    }
}

/// Helper trait for formatting display values
pub trait DisplayValue {
    fn format_display(&self) -> String;
    fn format_display_summary(&self) -> String {
        self.format_display()
    }
    fn format_display_details(&self) -> String {
        self.format_display()
    }
}

impl DisplayValue for u64 {
    fn format_display(&self) -> String {
        format_size(*self)
    }
}

impl DisplayValue for usize {
    fn format_display(&self) -> String {
        self.to_string()
    }
}

impl DisplayValue for String {
    fn format_display(&self) -> String {
        self.clone()
    }
}

impl DisplayValue for &str {
    fn format_display(&self) -> String {
        self.to_string()
    }
}

// 为Duration实现DisplayValue
impl DisplayValue for std::time::Duration {
    fn format_display(&self) -> String {
        format!("{:.2?}", self)
    }
}

// 为PathBuf实现DisplayValue
impl DisplayValue for std::path::PathBuf {
    fn format_display(&self) -> String {
        self.display().to_string()
    }
}

// 为HashMap实现DisplayValue
impl<K: std::fmt::Debug, V: std::fmt::Debug> DisplayValue for std::collections::HashMap<K, V> {
    fn format_display(&self) -> String {
        format!("{} entries", self.len())
    }
    
    fn format_display_summary(&self) -> String {
        format!("{} entries", self.len())
    }
    
    fn format_display_details(&self) -> String {
        if self.is_empty() {
            "{}".to_string()
        } else {
            let entries: Vec<String> = self.iter()
                .map(|(k, v)| format!("  {:?}: {:?}", k, v))
                .collect();
            format!("{{
{}
}}", entries.join(",\n"))
        }
    }
}

impl<T: std::fmt::Debug> DisplayValue for Vec<T> {
    fn format_display(&self) -> String {
        format!("{} items", self.len())
    }
    
    fn format_display_summary(&self) -> String {
        format!("{} items", self.len())
    }
    
    fn format_display_details(&self) -> String {
        if self.is_empty() {
            "[]".to_string()
        } else {
            format!("[\n{}\n]", self.iter()
                .map(|item| format!("  {:?}", item))
                .collect::<Vec<_>>()
                .join(",\n"))
        }
    }
}

// 为Option实现DisplayValue
impl<T: DisplayValue> DisplayValue for Option<T> {
    fn format_display(&self) -> String {
        match self {
            Some(value) => value.format_display(),
            None => "None".to_string(),
        }
    }
    
    fn format_display_summary(&self) -> String {
        match self {
            Some(value) => value.format_display_summary(),
            None => "None".to_string(),
        }
    }
    
    fn format_display_details(&self) -> String {
        match self {
            Some(value) => value.format_display_details(),
            None => "None".to_string(),
        }
    }
}

/// Format size in bytes to human readable format
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }
    
    #[test]
    fn test_vec_display_modes() {
        let test_vec: Vec<String> = vec![
            "file1.txt".to_string(),
            "file2.jpg".to_string(),
            "file3.log".to_string(),
        ];
        
        // 测试摘要模式 - 应该显示数组长度
        let summary = test_vec.format_display_summary();
        assert_eq!(summary, "3 items");
        
        // 测试详细模式 - 应该显示所有内容
        let details = test_vec.format_display_details();
        assert_eq!(details, "[\n  \"file1.txt\",\n  \"file2.jpg\",\n  \"file3.log\"\n]");
        
        // 测试空数组
        let empty_vec: Vec<String> = vec![];
        assert_eq!(empty_vec.format_display_summary(), "0 items");
        assert_eq!(empty_vec.format_display_details(), "[]");
    }
}