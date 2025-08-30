//! 配置操作处理器模块

use crate::{AppResult, operations::CliOperations};

/// 配置操作处理器
pub struct ConfigHandler<'a> {
    ops: &'a CliOperations,
}

impl<'a> ConfigHandler<'a> {
    /// 创建新的配置处理器
    pub fn new(ops: &'a CliOperations) -> Self {
        Self { ops }
    }

    /// 显示当前配置
    pub fn execute(&self) -> AppResult<()> {
        println!("当前配置:");
        println!("  微信缓存路径: {:?}", self.ops.settings().wechat.cache_path);
        println!("  默认清理模式: {:?}", self.ops.settings().cleaner.mode);
        println!("  缓存文件模式: {:?}", self.ops.settings().wechat.cache_patterns);
        Ok(())
    }
}
