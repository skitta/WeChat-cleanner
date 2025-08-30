//! 扫描操作处理器模块

use core::scanner::{FileScanner};
use core::progress::Progress;
use core::display::*;

use crate::{AppResult, operations::CliOperations};

/// 扫描操作处理器
pub struct ScanHandler<'a> {
    ops: &'a CliOperations,
}

impl<'a> ScanHandler<'a> {
    /// 创建新的扫描处理器
    pub fn new(ops: &'a CliOperations) -> Self {
        Self { ops }
    }

    pub fn execute(&self, verbose: bool) -> AppResult<()> {
        let mut scanner = FileScanner::new(self.ops.settings().clone());
        let progress = Progress::Bar(self.ops.create_progress_bar()?);
        if let Some(result) = scanner.scan_with_progress(&progress) {
            if verbose {
                println!("{}", result.display_details());
            } else {
                println!("{}", result.display_summary());
            }
            
            result.save()?;
        }
        
        Ok(())
    }
}