//! 清理操作处理器模块
use core::{
    cleaner::FileCleaner,
    progress::Progress,
    display::*,
};

use crate::{
    AppResult,
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

    /// 执行
    pub fn execute(&self, mode: &str, force: bool) -> AppResult<()> {
        let file_cleaner = FileCleaner::new(&self.ops.settings().scanner)?;

        let preview = file_cleaner.preview()?;
        println!("{}", preview.display_details());
        
        let should_clean = if force {
            true
        } else {
            self.ops.get_user_confirmation("确认执行清理操作？")?
        };

        if should_clean {
            let mode = self.ops.parse_cleaning_mode(mode);
            let progress = Progress::Bar(self.ops.create_progress_bar()?);
            let clean_result = preview.clean_with_progress(mode, &progress).ok_or("没能清理任何文件")?;

            println!("{}", clean_result.display_summary());
            file_cleaner.delete_scan_result()?;
        } else {
            println!("清理已取消");
        }

        Ok(())
    }
}
