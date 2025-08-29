//! 配置操作处理器模块

use crate::{AppResult, operations::CliOperations, display::display_config};

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
        display_config(self.ops.settings(), true);
        Ok(())
    }
}
