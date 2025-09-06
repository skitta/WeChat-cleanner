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
    
    fn format_display_summary(&self) -> String {
        self.display().to_string()
    }
    
    fn format_display_details(&self) -> String {
        self.display().to_string()
    }
}

// 为HashMap实现DisplayValue
impl<K: std::fmt::Debug, V: std::fmt::Debug> DisplayValue for std::collections::HashMap<K, V> {
    fn format_display(&self) -> String {
        match self.len() {
            0 => "无数据".to_string(),
            1 => "1 项".to_string(),
            n => format!("{} 项", n),
        }
    }
    
    fn format_display_summary(&self) -> String {
        match self.len() {
            0 => "无数据".to_string(),
            1 => "1 项".to_string(),
            n => format!("{} 项", n),
        }
    }
    
    fn format_display_details(&self) -> String {
        if self.is_empty() {
            "  (无内容)".to_string()
        } else {
            let entries: Vec<String> = self.iter()
                .enumerate()
                .map(|(i, (k, v))| {
                    // 格式化值的显示
                    let value_str = format!("{:#?}", v)
                        .lines()
                        .map(|line| format!("    {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    
                    format!("  [{}] 目录: {:?}\n{}", i + 1, k, value_str)
                })
                .collect();
            format!("\n{}", entries.join("\n\n"))
        }
    }
}

impl<T: std::fmt::Debug> DisplayValue for Vec<T> {
    fn format_display(&self) -> String {
        match self.len() {
            0 => "无数据".to_string(),
            1 => "1 项".to_string(),
            n => format!("{} 项", n),
        }
    }
    
    fn format_display_summary(&self) -> String {
        match self.len() {
            0 => "无数据".to_string(),
            1 => "1 项".to_string(),
            n => format!("{} 项", n),
        }
    }
    
    fn format_display_details(&self) -> String {
        if self.is_empty() {
            "  (无内容)".to_string()
        } else {
            let items: Vec<String> = self.iter()
                .enumerate()
                .map(|(i, item)| {
                    let item_str = format!("{:#?}", item)
                        .lines()
                        .map(|line| format!("    {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("  [{}]\n{}", i + 1, item_str)
                })
                .collect();
            format!("\n{}", items.join("\n"))
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