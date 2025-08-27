//! Display 功能模块
//! 
//! 提供结构体的显示功能，包括摘要和详细信息显示。
//! 只在启用 `display` feature 时可用。

// 重新导出 display_core 和 display_derive 包中的所有内容
pub use display_core::*;
pub use display_derive::Display;
