//! 简化的进度报告系统
//!
//! 提供统一的进度报告接口，直接支持 indicatif::ProgressBar

/// 统一的进度报告器
pub enum Progress {
    /// 无进度显示
    None,
    /// CLI 进度条（需要 indicatif feature）
    #[cfg(feature = "cli")]
    Bar(indicatif::ProgressBar),
}

impl Progress {
    /// 创建无进度显示的实例
    pub fn none() -> Self {
        Progress::None
    }

    /// 创建 CLI 进度条实例
    #[cfg(feature = "cli")]
    pub fn bar(bar: indicatif::ProgressBar) -> Self {
        Progress::Bar(bar)
    }

    /// 更新进度
    pub fn update(&self, current: usize, total: usize, message: &str) {
        match self {
            Progress::None => { println!("{message}") },
            #[cfg(feature = "cli")]
            Progress::Bar(bar) => {
                if total > 0 {
                    bar.set_length(total as u64);
                    bar.set_position(current as u64);
                }
                bar.set_message(message.to_string());
            },
        }
    }

    /// 设置消息
    pub fn set_message(&self, message: &str) {
        match self {
            Progress::None => { println!("{message}") },
            #[cfg(feature = "cli")]
            Progress::Bar(bar) => {
                bar.set_message(message.to_string());
            },
        }
    }

    /// 完成进度
    pub fn finish(&self, message: &str) {
        match self {
            Progress::None => { println!("{message}") },
            #[cfg(feature = "cli")]
            Progress::Bar(bar) => {
                bar.finish_with_message(message.to_string());
            },
        }
    }

    /// 增量更新进度
    pub fn increment(&self, message: &str) {
        match self {
            Progress::None => {},
            #[cfg(feature = "cli")]
            Progress::Bar(bar) => {
                bar.inc(1);
                bar.set_message(message.to_string());
            },
        }
    }
}