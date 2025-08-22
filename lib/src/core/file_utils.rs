use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(windows)]
use std::os::windows::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::core::progressor::{Progress, ProgressTracker, ProgressReporter};

/// 设置文件权限（跨平台）
pub fn set_file_permissions(path: &Path, mode: u32) -> Result<()> {
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(path, perms)?;
    }
    
    #[cfg(windows)]
    {
        // Windows 上尝试移除只读属性
        let mut perms = fs::metadata(path)?.permissions();
        if perms.readonly() {
            perms.set_readonly(false);
            fs::set_permissions(path, perms)?;
        }
        // Windows 下 mode 参数被忽略，因为 Windows 权限模型不同
        let _ = mode; // 避免未使用变量警告
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        // 其他平台的占位实现
        log::warn!("文件权限设置在当前平台不被支持: {}", path.display());
        let _ = mode; // 避免未使用变量警告
    }
    
    Ok(())
}

/// 文件信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
    pub hash: Option<String>,
}

impl FileInfo {
    pub fn new(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        let modified = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        Ok(FileInfo {
            path: path.to_path_buf(),
            size,
            modified,
            hash: None,
        })
    }

    /// 计算文件的 MD5 哈希值
    fn hash(&mut self) -> Result<()> {
        use md5::{Digest, Md5};
        use std::io::Read;

        let mut file = fs::File::open(&self.path)?;
        let mut hasher = Md5::new();
        let mut buffer = [0; 8192]; // 8KB 缓冲区

        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        self.hash = Some(format!("{:x}", hasher.finalize()));

        Ok(())
    }
}

pub trait FileInfosExt {
    fn calculate_hashes(self) -> Self
    where
        Self: Sized,
    {
        self.calculate_hashes_with_callback(|_| {})
    }
    
    fn calculate_hashes_with_callback<F>(self, callback: F) -> Self
    where
        Self: Sized,
        F: Fn(&Progress) + Sync + Send + 'static;
    
    fn group_by_hash(self) -> HashMap<String, Vec<FileInfo>>
    where
        Self: Sized,
    {
        self.group_by_hash_with_callback(|_| {})
    }

    fn group_by_hash_with_callback<F>(self, callback: F) -> HashMap<String, Vec<FileInfo>>
    where
        Self: Sized,
        F: Fn(&Progress) + Sync + Send + 'static;
}

impl FileInfosExt for Vec<FileInfo> {
    fn calculate_hashes_with_callback<F>(mut self, callback: F) -> Self
    where 
    F: Fn(&Progress) + Sync + Send + 'static,
    {
        let total = self.len();
        let mut progress = ProgressTracker::new(Progress::new(), callback);
        progress.report_msg("开始计算文件哈希...");
        progress.report_progress(0, total);

        for file in &mut self {
            progress.update(|| {
                file.hash().ok();
            });
        }
        progress.report_complete();
        self
    }

    fn group_by_hash_with_callback<F>(self, callback: F) -> HashMap<String, Vec<FileInfo>>
    where
        F: Fn(&Progress) + Sync + Send + 'static,
    {
        let total = self.len();
        let mut progress = ProgressTracker::new(Progress::new(), callback);
        let mut groups: HashMap<String, Vec<FileInfo>> = HashMap::with_capacity(total / 2);  // 预分配空间

        progress.report_msg("开始分组文件...");
        progress.report_progress(0, total);

        self.into_iter().for_each(|file| {
            progress.update(|| {
                if let Some(ref hash) = file.hash {
                    groups.entry(hash.clone()).or_default().push(file);
                }
            });
        });

        progress.report_complete();
        
        groups
    }
}

/// 微信缓存目录解析器（跨平台）
pub struct WechatCacheResolver;

impl WechatCacheResolver {
    /// 查找微信缓存目录（跨平台）
    pub fn find_wechat_dirs() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| Error::CacheNotFound)?;

        // 尝试不同平台的微信路径
        let search_paths = Self::get_platform_paths(&home);
        
        for base_path in search_paths {
            if let Ok(cache_dir) = Self::scan_wechat_directory(&base_path) {
                return Ok(cache_dir);
            }
        }

        Err(Error::CacheNotFound)
    }
    
    /// 获取平台特定的微信路径
    fn get_platform_paths(home: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        #[cfg(target_os = "macos")]
        {
            // macOS 微信路径
            paths.push(home.join("Library/Containers/com.tencent.xinWeChat/Data/Documents/xwechat_files"));
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows 微信路径
            if let Some(appdata) = std::env::var_os("APPDATA") {
                let appdata_path = PathBuf::from(appdata);
                paths.push(appdata_path.join("Tencent/WeChat/All Users"));
                paths.push(appdata_path.join("WeChat Files"));
            }
            
            if let Some(documents) = dirs::document_dir() {
                paths.push(documents.join("WeChat Files"));
                paths.push(documents.join("Tencent Files"));
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux 微信路径（通过 Wine 或原生 Linux 版本）
            paths.push(home.join(".wine/drive_c/users").join(std::env::var("USER").unwrap_or_default()).join("Application Data/Tencent/WeChat"));
            paths.push(home.join(".local/share/applications/WeChat"));
            paths.push(home.join("Documents/WeChat Files"));
        }
        
        paths
    }
    
    /// 扫描微信目录结构
    fn scan_wechat_directory(base_path: &Path) -> Result<PathBuf> {
        if !base_path.exists() {
            return Err(Error::CacheNotFound);
        }

        // 尝试查找常见的缓存目录结构
        let cache_subdirs = [
            "msg/file",           // macOS 微信文件目录
            "FileStorage",        // Windows 微信文件目录
        ];
        
        // 递归扫描目录
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 检查是否为微信用户目录（以 wxid_ 开头或包含微信特征）
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        if dir_name.starts_with("wxid_") || dir_name.contains("WeChat") {
                            // 在用户目录中查找缓存子目录
                            for subdir in &cache_subdirs {
                                let cache_path = path.join(subdir);
                                if cache_path.exists() {
                                    return Ok(cache_path);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 如果没有找到用户目录，尝试直接在基本路径中查找缓存目录
        for subdir in &cache_subdirs {
            let cache_path = base_path.join(subdir);
            if cache_path.exists() {
                return Ok(cache_path);
            }
        }
        
        Err(Error::CacheNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_calculate_file_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"Hello, world!").unwrap();

        let mut fileinfo = FileInfo::new(&file_path)
            .unwrap();
        fileinfo.hash().unwrap();
        assert_eq!(fileinfo.hash.unwrap(), "6cd3556deb0da54bca060b4c39479839");
    }

    #[test]
    fn test_get_file_info() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"Hello, world!").unwrap();

        let info = FileInfo::new(&file_path).unwrap();

        assert_eq!(info.path, file_path);
        assert_eq!(info.size, 13);
    }

    #[test]
    fn test_find_wechat_dirs() {
        let dirs = WechatCacheResolver::find_wechat_dirs().unwrap();
        println!("{:?}", dirs);
        assert!(dirs.exists());
    }
    
    #[test]
    fn test_cross_platform_file_permissions() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_permissions.txt");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"Permission test").unwrap();
        drop(file);
        
        // 测试文件权限设置（在所有平台上都应该成功）
        let result = set_file_permissions(&file_path, 0o644);
        assert!(result.is_ok(), "File permission setting should not fail on any platform");
    }
    
    #[test]
    fn test_get_platform_paths() {
        let home = dirs::home_dir().unwrap();
        let paths = WechatCacheResolver::get_platform_paths(&home);
        
        // 应该至少返回一些路径
        assert!(!paths.is_empty(), "Should return at least some search paths");
        
        // 所有路径都应该是绝对路径
        for path in &paths {
            assert!(path.is_absolute(), "All paths should be absolute: {:?}", path);
        }
        
        println!("Platform search paths: {:#?}", paths);
    }
    
    #[test]
    fn test_scan_wechat_directory() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        
        // 创建模拟的微信目录结构
        let wxid_dir = base_path.join("wxid_test123");
        let cache_dir = wxid_dir.join("msg/file");
        fs::create_dir_all(&cache_dir).unwrap();
        
        // 测试扫描功能
        let result = WechatCacheResolver::scan_wechat_directory(base_path);
        assert!(result.is_ok(), "Should find the created cache directory");
        assert_eq!(result.unwrap(), cache_dir);
    }
}
