use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::core::progressor::{Progress, ProgressTracker, ProgressReporter};

/// 设置文件权限
pub fn set_file_permissions(path: &Path, mode: u32) -> Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(mode);
    fs::set_permissions(path, perms)?;
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

/// 微信缓存目录解析器
pub struct WechatCacheResolver;

impl WechatCacheResolver {
    /// 查找微信缓存目录
    pub fn find_wechat_dirs() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| Error::CacheNotFound)?;

        let base_path =
            home.join("Library/Containers/com.tencent.xinWeChat/Data/Documents/xwechat_files");

        if !base_path.exists() {
            return Err(Error::CacheNotFound);
        }

        for entry in fs::read_dir(&base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with("wxid_") {
                        let temp_path = path.join("msg/file");
                        if temp_path.exists() {
                            return Ok(temp_path);
                        }
                    }
                }
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
}
