//! 文件工具模块
//!
//! 提供文件操作、权限管理、文件信息收集和重复文件检测等核心功能。
//! 采用函数式编程风格和分层处理逻辑，支持并行计算以提升性能。
//!
//! # 主要功能
//! - 跨平台文件权限设置
//! - 文件元数据收集与处理
//! - 基于模式和哈希的重复文件检测
//! - 微信缓存目录自动发现
//!
//! # 性能优化
//! - 使用 rayon 进行并行计算
//! - 按大小预分组避免不必要的哈希计算
//! - 8KB-32KB 动态缓冲区优化文件读取
//! - 分层处理逻辑：大小 → 模式 → 哈希

use crate::errors::{Error, Result};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
// 无用导入：文件权限相关的import没有被使用
// #[cfg(unix)]
// use std::os::unix::fs::PermissionsExt;
// #[cfg(windows)]
// use std::os::windows::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

/// 设置文件权限（跨平台）
///
/// 根据不同操作系统平台设置文件权限，确保跨平台兼容性。
///
/// # 参数
/// * `path` - 要设置权限的文件路径
/// * `mode` - Unix 权限模式（Windows 下被忽略）
///
/// # 返回值
/// * `Result<()>` - 设置成功返回 Ok(())，失败返回相应错误
///
/// # 平台支持
/// - **Unix/Linux/macOS**: 使用标准的文件权限模式
/// - **Windows**: 尝试移除只读属性，忽略权限模式参数
/// - **其他平台**: 记录警告日志，不执行实际操作
// 无用代码：设置文件权限函数没有被使用
// fn set_file_permissions(path: &Path, mode: u32) -> Result<()> {
//     #[cfg(unix)]
//     {
//         let mut perms = fs::metadata(path)?.permissions();
//         perms.set_mode(mode);
//         fs::set_permissions(path, perms)?;
//     }
//
//     #[cfg(windows)]
//     {
//         // Windows 上尝试移除只读属性
//         let mut perms = fs::metadata(path)?.permissions();
//         if perms.readonly() {
//             perms.set_readonly(false);
//             fs::set_permissions(path, perms)?;
//         }
//         // Windows 下 mode 参数被忽略，因为 Windows 权限模型不同
//         let _ = mode; // 避免未使用变量警告
//     }
//
//     #[cfg(not(any(unix, windows)))]
//     {
//         // 其他平台的占位实现
//         log::warn!("文件权限设置在当前平台不被支持: {}", path.display());
//         let _ = mode; // 避免未使用变量警告
//     }
//
//     Ok(())
// }

/// 检测文件或目录是否为隐藏文件
///
/// 根据 Unix 约定，以点号 "." 开头的文件名被认为是隐藏文件。
/// 在文件扫描过程中用于过滤不需要处理的隐藏文件。
///
/// # 参数
/// * `entry` - walkdir 的目录项
///
/// # 返回值
/// * `bool` - 如果是隐藏文件返回 true，否则返回 false
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

/// 文件信息结构体
///
/// 封装文件的基本元数据信息，包括路径、大小和修改时间。
/// 支持序列化和反序列化，可用于结果的持久化存储。
///
/// # 字段
/// * `path` - 文件的绝对路径
/// * `size` - 文件大小（字节）
/// * `modified` - 文件最后修改时间（Unix 时间戳）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    path: PathBuf,
    size: u64,
    pub modified: u64,
}

impl FileInfo {
    /// 从文件路径创建 FileInfo 实例
    ///
    /// 读取文件的元数据信息并创建相应的 FileInfo 结构体。
    /// 会自动获取文件大小和最后修改时间。
    ///
    /// # 参数
    /// * `file` - 文件路径引用
    ///
    /// # 返回值
    /// * `Result<Self>` - 成功返回 FileInfo 实例，失败返回错误
    ///
    /// # 错误
    /// 当文件不存在或无法访问时返回 IO 错误。
    fn new(file: &Path) -> Result<Self> {
        if !file.is_file() {
            return Err(Error::FileProcessing("FileInfo: 请传入一个文件".to_string()))
        }
        let metadata = fs::metadata(file)?;
        let size = metadata.len();
        let modified = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        Ok(FileInfo {
            path: file.to_path_buf(),
            size,
            modified,
        })
    }

    /// 从指定目录收集所有文件信息
    ///
    /// 递归遍历指定目录，收集所有文件的元数据信息。
    /// 使用并行处理优化性能，自动过滤隐藏文件。
    ///
    /// # 性能优化
    /// - 先收集所有文件路径（快速操作）
    /// - 使用并行处理进行元数据收集
    /// - 只在 debug 模式下记录详细错误日志
    ///
    /// # 参数
    /// * `path` - 要扫描的目录路径
    ///
    /// # 返回值
    /// * `Result<Vec<Self>>` - 成功返回文件信息列表，失败返回错误
    ///
    /// # 错误
    /// - `Error::CacheNotFound` - 目录不存在或无文件
    pub fn collect_from(path: &Path) -> Option<Vec<Self>> {
        // 先检查路径是否存在
        if !path.is_dir() {
            return None;
        }

        // 优化1: 首先收集所有文件路径（快速操作）
        let file_entries: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        if file_entries.is_empty() {
            return None;
        }

        // 优化2: 预分配容量并使用并行处理进行元数据收集
        let files: Vec<Self> = file_entries
            .into_par_iter()
            .filter_map(|entry| {
                // 优化3: 减少错误处理开销，只记录严重错误
                match FileInfo::new(entry.path()) {
                    Ok(info) => Some(info),
                    Err(_e) => {
                        // 只在 debug 模式下记录详细日志
                        #[cfg(debug_assertions)]
                        log::warn!("Failed to process {}: {}", entry.path().display(), _e);
                        None
                    }
                }
            })
            .collect();

        if files.is_empty() {
            None
        } else {
            Some(files)
        }
    }
}

/// 文件名称相关操作 trait
///
/// 为文件对象提供名称提取和模式匹配功能。
/// 支持基本文件名、基本名称和模式匹配名称提取。
pub trait Named {
    /// 获取完整文件名（包含扩展名）
    fn name(&self) -> Option<Cow<'_, str>>;

    /// 获取文件的基本名称（不包含扩展名）
    fn base_name(&self) -> Option<Cow<'_, str>>;

    /// 根据正则表达式提取模式化名称
    ///
    /// 优先使用正则表达式匹配，如果匹配失败则移除扩展名。
    /// 用于识别具有类似模式的文件（如序号后缀、时间戳等）。
    fn patterned_name(&self, regex: &Regex) -> Option<Cow<'_, str>>;
}

impl Named for FileInfo {
    fn name(&self) -> Option<Cow<'_, str>> {
        self.path.file_name().map(|n| n.to_string_lossy())
    }

    fn base_name(&self) -> Option<Cow<'_, str>> {
        self.path.file_stem().map(|n| n.to_string_lossy())
    }

    fn patterned_name(&self, regex: &Regex) -> Option<Cow<'_, str>> {
        let file_name = self.name()?;

        // 优先使用正则表达式匹配，提取匹配位置之前的部分作为基本名称
        if let Some(captures) = regex.captures(file_name.as_ref()) {
            if let Some(matched) = captures.get(0) {
                let base_name = &file_name.as_ref()[..matched.start()];
                if !base_name.is_empty() {
                    return Some(Cow::Owned(base_name.to_owned()));
                }
            }
        }

        // 如果正则匹配失败，则移除文件扩展名
        if let Some(dot_pos) = file_name.as_ref().rfind(".") {
            Some(Cow::Owned(file_name.as_ref()[..dot_pos].to_owned()))
        } else {
            Some(file_name)
        }
    }
}

/// 文件大小相关操作 trait
///
/// 为文件对象提供获取文件大小的能力。
/// 用于文件分组和排序操作。
pub trait HasSize {
    /// 获取文件大小（字节数）
    fn size(&self) -> u64;
}

impl HasSize for FileInfo {
    fn size(&self) -> u64 {
        self.size
    }
}

pub trait HasPath {
    fn path(&self) -> &PathBuf;
}

impl HasPath for FileInfo {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// 文件哈希相关操作 trait
///
/// 为文件对象提供计算哈希值的能力。
/// 使用 MD5 算法计算文件内容的哈希值，用于精确的重复文件检测。
pub trait Hashed {
    /// 计算文件的 MD5 哈希值
    ///
    /// # 返回值
    /// * `Option<String>` - 成功返回哈希值字符串，失败返回 None
    fn hash(&self) -> Option<String>;
}

impl Hashed for FileInfo {
    fn hash(&self) -> Option<String> {
        use md5::{Digest, Md5};
        use std::io::{BufReader, Read};

        // 使用 BufReader 优化 I/O 性能
        let file = fs::File::open(&self.path).ok()?;
        let mut reader = BufReader::with_capacity(65536, file); // 64KB 缓冲区
        let mut hasher = Md5::new();

        // 根据文件大小动态调整缓冲区大小
        let buffer_size = if self.size < 1024 * 1024 {
            8192 // 8KB - 适合小文件
        } else {
            32768 // 32KB - 适合大文件
        };

        let mut buffer = vec![0u8; buffer_size];

        // 流式读取文件内容并计算哈希
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => return None, // 读取失败
            }
        }

        let result = hasher.finalize();
        Some(format!("{:x}", result))
    }
}

/// 文件分组操作 trait
///
/// 为文件集合提供各种分组功能，支持按不同维度进行文件分类。
/// 采用函数式编程风格，支持链式调用和并行处理。
///
/// # 性能特性
/// - 按需使用并行处理以提升性能
/// - 支持各种自定义分组策略
/// - 内存优化的集合操作
pub trait FileGrouper: IntoIterator {
    /// 通用分组方法
    ///
    /// 根据提供的键提取函数对文件集合进行分组。
    /// 是所有其他分组方法的基础实现。
    ///
    /// # 参数
    /// * `key_extractor` - 键提取函数，从文件中提取分组键
    ///
    /// # 返回值
    /// * `HashMap<K, Vec<Self::Item>>` - 按键分组的文件集合
    fn group_by<F, K>(self, key_extractor: F) -> HashMap<K, Vec<Self::Item>>
    where
        Self: Sized,
        F: Fn(&Self::Item) -> Option<K>,
        K: Eq + Hash,
    {
        let mut map: HashMap<K, Vec<Self::Item>> = HashMap::new();
        for item in self.into_iter() {
            if let Some(key) = key_extractor(&item) {
                map.entry(key).or_insert_with(Vec::new).push(item);
            }
        }
        map
    }

    /// 按文件大小分组
    ///
    /// 将文件按照大小进行分组，相同大小的文件将被分在同一组。
    /// 这是性能优化的第一步，只有相同大小的文件才可能是重复文件。
    fn group_by_size(self) -> HashMap<u64, Vec<Self::Item>>
    where
        Self: Sized,
        Self::Item: HasSize,
    {
        self.group_by(|item| Some(item.size()))
    }

    /// 按文件名模式分组
    ///
    /// 根据正则表达式对文件名进行模式匹配分组。
    /// 用于识别具有类似命名模式的文件（如序号后缀、时间戳等）。
    fn group_by_pattern(self, regex: &Regex) -> HashMap<String, Vec<Self::Item>>
    where
        Self: Sized,
        Self::Item: Named,
    {
        self.group_by(|item| item.patterned_name(regex).map(|name| name.into_owned()))
    }

    /// 按文件哈希值分组（并行版本）
    ///
    /// 使用并行处理计算文件哈希值并进行分组。
    /// 这是最耗时的操作，但能提供最精确的重复文件检测。
    ///
    /// # 性能特性
    /// - 使用 rayon 进行并行哈希计算
    /// - 动态调整缓冲区大小以优化性能
    fn group_by_hash(self) -> HashMap<String, Vec<Self::Item>>
    where
        Self: Sized + Send,
        Self::Item: Hashed + Send,
    {
        let items: Vec<_> = self.into_iter().collect();

        // 并行计算所有文件的哈希值
        let hash_pairs: Vec<(String, Self::Item)> = items
            .into_par_iter()
            .filter_map(|item| item.hash().map(|hash| (hash, item)))
            .collect();

        // 按哈希值分组
        let mut map: HashMap<String, Vec<Self::Item>> = HashMap::new();
        for (hash, item) in hash_pairs {
            map.entry(hash).or_insert_with(Vec::new).push(item);
        }
        map
    }

    fn group_by_parent(self) -> HashMap<PathBuf, Vec<Self::Item>>
    where 
        Self: Sized,
        Self::Item: HasPath,
    {
        self.into_iter().fold(HashMap::new(), |mut acc, file| {
            let parent = file.path().parent().unwrap_or("/".as_ref()).to_path_buf();
            acc.entry(parent).or_default().push(file);
            acc
        })
    }
}

/// 文件过滤与重复检测 trait
///
/// 扩展 FileGrouper，提供高级的重复文件检测功能。
/// 采用分层处理策略，结合模式匹配和哈希比较实现高效的重复文件识别。
///
/// # 处理策略
/// 1. 按模式分组，直接识别模式重复文件
/// 2. 对非模式重复文件按大小进一步过滤
/// 3. 最后使用哈希比较进行精确检测
pub trait FileFilter: FileGrouper {
    /// 按模式检测重复文件
    ///
    /// 采用分层检测策略，先按模式分组，再对非模式重复文件进行哈希检测。
    /// 这种方法能够高效地识别两种类型的重复文件：
    /// 1. 模式重复：具有相似命名模式的文件
    /// 2. 内容重复：文件内容完全相同的文件
    ///
    /// # 参数
    /// * `regex` - 用于模式匹配的正则表达式
    ///
    /// # 返回值
    /// * `HashMap<String, Vec<Self::Item>>` - 重复文件组，键为识别标识，值为重复文件列表
    ///
    /// # 性能优化
    /// - 先按模式分组，直接识别模式重复
    /// - 只对非模式重复文件进行耗时的哈希计算
    /// - 使用并行处理提升性能
    /// - 按大小预过滤减少不必要的计算
    fn duplicates_by_pattern(self, regex: &Regex) -> HashMap<String, Vec<Self::Item>>
    where
        Self: Sized,
        Self::Item: HasSize + Named + Hashed + Send + Clone,
    {
        // 第一步：按模式分组，分离模式重复和候选文件
        let (pattern_duplicates, size_candidates): (Vec<_>, Vec<_>) = self
            .group_by_pattern(regex)
            .into_par_iter()
            .partition(|(_, items)| items.len() > 1);

        // 初始化结果集合，先加入模式重复文件
        let mut duplicates: HashMap<String, Vec<Self::Item>> =
            pattern_duplicates.into_iter().collect();

        // 第二步：对非模式重复文件进行哈希检测
        if !size_candidates.is_empty() {
            // 收集所有候选文件
            let candidates: Vec<Self::Item> = size_candidates
                .into_par_iter()
                .flat_map(|(_, items)| items)
                .collect();

            // 按大小分组后再按哈希检测
            let hash_duplicate: HashMap<String, Vec<Self::Item>> = candidates
                .group_by_size()
                .into_par_iter()
                .filter(|(_, item)| item.len() > 1) // 只处理大小相同的文件组
                .flat_map(|(_, items)| items)
                .collect::<Vec<_>>()
                .group_by_hash()
                .into_iter()
                .filter(|(_, items)| items.len() > 1) // 只保留真正重复的文件组
                .collect();

            duplicates.extend(hash_duplicate);
        }

        duplicates
    }
}

pub trait FileProcessor {
    type ProcessResult;
    fn delete(&self) -> Result<Self::ProcessResult>;
}

impl FileProcessor for FileInfo {
    type ProcessResult = bool;
    fn delete(&self) -> Result<bool> {
        fs::remove_file(&self.path)
            .map(|_| {
                log::debug!("已删除: {}", self.path.display());
                true
            })
            .map_err(|e| {
                Error::FileProcessing(format!(
                    "删除失败: {} - {}", self.path.display(), e
                ))
            })
    }
}

impl FileProcessor for Vec<FileInfo> {
    type ProcessResult = Vec<FileInfo>;
    // TODO: 检验是否会因为错误中断
    fn delete(&self) -> Result<Vec<FileInfo>> {
        self.into_iter()
            .filter_map(|f| match f.delete() {
                Ok(true) => Some(Ok(f.to_owned())),
                Ok(false) => None,
                Err(e) => Some(Err(e))
            })
            .collect()
    }
}


// Trait 实现
/// FileFilter trait 为 FileInfo 的实现
impl FileFilter for Vec<FileInfo> {}

/// FileGrouper trait 的通用实现
/// 为所有 Vec<T> 类型提供分组功能
impl<T> FileGrouper for Vec<T> {}

// 为FileInfo实现DisplayValue
#[cfg(feature = "display")]
impl crate::display::DisplayValue for FileInfo {
    fn format_display(&self) -> String {
        format!("{} ({})", 
                self.path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                crate::display::format_size(self.size))
    }
    
    fn format_display_summary(&self) -> String {
        self.format_display()
    }
    
    fn format_display_details(&self) -> String {
        format!("{} ({})",
                self.path.display(),
                crate::display::format_size(self.size))
    }
}

/// 微信缓存目录解析器（跨平台）
///
/// 提供跨平台的微信缓存目录自动发现功能。
/// 支持 macOS、Windows 和 Linux 平台的微信安装目录检测。
///
/// # 支持的平台
/// - **macOS**: `~/Library/Containers/com.tencent.xinWeChat/Data/Documents/xwechat_files`
/// - **Windows**: `%APPDATA%/Tencent/WeChat/All Users`, `%APPDATA%/WeChat Files`
/// - **Linux**: Wine 环境下的微信路径
///
/// # 缓存目录结构
/// - macOS: `msg/file` 子目录
/// - Windows: `FileStorage` 子目录
pub struct WechatCacheResolver;

impl WechatCacheResolver {
    /// 查找微信缓存目录（跨平台）
    ///
    /// 自动检测当前操作系统并查找相应的微信缓存目录。
    /// 会逐个尝试不同的可能路径，直到找到有效的缓存目录。
    ///
    /// # 返回值
    /// * `Option<PathBuf>` - 成功返回缓存目录路径，失败返回`None`
    pub fn find_wechat_dirs() -> Option<PathBuf> {
        let home = dirs::home_dir()?;

        // 尝试不同平台的微信路径
        let search_paths = Self::get_platform_paths(&home);

        for base_path in search_paths {
            return Self::scan_wechat_directory(&base_path);
        }
        None
    }

    /// 获取平台特定的微信路径
    ///
    /// 根据不同操作系统平台返回相应的微信安装目录列表。
    /// 使用条件编译确保只返回当前平台相关的路径。
    ///
    /// # 参数
    /// * `home` - 用户主目录路径
    ///
    /// # 返回值
    /// * `Vec<PathBuf>` - 可能的微信安装目录列表
    fn get_platform_paths(home: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "macos")]
        {
            // macOS 微信路径
            paths.push(
                home.join("Library/Containers/com.tencent.xinWeChat/Data/Documents/xwechat_files"),
            );
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
            paths.push(
                home.join(".wine/drive_c/users")
                    .join(std::env::var("USER").unwrap_or_default())
                    .join("Application Data/Tencent/WeChat"),
            );
            paths.push(home.join(".local/share/applications/WeChat"));
            paths.push(home.join("Documents/WeChat Files"));
        }

        paths
    }

    /// 扫描微信目录结构
    ///
    /// 在指定的基本路径中查找微信缓存目录。
    /// 会递归扫描目录结构，查找以 "wxid_" 开头或包含 "WeChat" 的用户目录。
    ///
    /// # 参数
    /// * `base_path` - 要扫描的基本路径
    ///
    /// # 返回值
    /// * `Option<PathBuf>` - 成功返回缓存目录路径，失败返回`None`
    ///
    /// # 扫描策略
    /// 1. 优先在用户目录中查找缓存子目录
    /// 2. 如果未找到，则在基本路径中直接查找
    fn scan_wechat_directory(base_path: &Path) -> Option<PathBuf> {
        if !base_path.exists() {
            return None;
        }

        // 尝试查找常见的缓存目录结构
        let cache_subdirs = [
            "msg/file",    // macOS 微信文件目录
            "FileStorage", // Windows 微信文件目录
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
                                    return Some(cache_path);
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
                return Some(cache_path);
            }
        }

        None
    }
}
