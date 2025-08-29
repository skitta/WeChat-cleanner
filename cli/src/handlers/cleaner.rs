//! 清理操作处理器模块
use core::{cleaner::FileCleaner, progress::Progress};

use crate::{
    AppResult,
    display::display,
    operations::CliOperations,
};

/// 清理操作处理器
pub struct CleanerHandler<'a> {
    ops: &'a CliOperations,
}

impl<'a> CleanerHandler<'a> {
    /// 创建新的清理处理器
    pub fn new(ops: &'a CliOperations) -> Self {
        Self { ops }
    }

    /// 执行带预览的清理操作（总是显示预览）
    pub fn execute(&self, mode: &str, force: bool) -> AppResult<()> {
        let cleaning_settings = self.ops.settings().cleaning.clone();

        let file_cleaner = FileCleaner::new(cleaning_settings)?;

        display(&file_cleaner.preview, true);
        
        let should_clean = if force {
            true
        } else {
            self.ops.get_user_confirmation("确认执行清理操作？")?
        };

        if should_clean {
            let mode = self.ops.parse_cleaning_mode(mode);
            let progress = Progress::Bar(self.ops.create_progress_bar()?);
            let clean_result = file_cleaner.clean_with_progress(mode, &progress).ok_or("")?;
            display(&clean_result, false);
        } else {
            println!("清理已取消");
        }

        Ok(())
    }
}
