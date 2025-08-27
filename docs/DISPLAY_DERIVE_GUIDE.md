# Display Derive 系统架构指南

现在 Display 功能被组织在专门的包中，提供了更清晰的架构和更好的可维护性！

## 架构概述

- **display_core**: 包含核心 trait 定义和实现
- **display_derive**: 包含 `#[derive(Display)]` 过程宏
- **core/display**: 重新导出上述两个包的所有内容

## 基本用法

```rust
// 简化导入，一行代码导入所有内容
use wechat_cleaner::display::*;

#[derive(Display)]
pub struct MyStruct {
    #[display(summary, name="自定义名称")]
    pub field1: u64,
    #[display(details)]
    pub field2: String,
    #[display(summary, details)]
    pub field3: Vec<String>,
    pub hidden_field: String,  // 没有 #[display] 属性，不会显示
}
```

## 包结构说明

### display_core 包
- 定义 `Display` 和 `DisplayValue` trait
- 实现基础类型的 `DisplayValue`（u64, usize, String, Vec<T> 等）
- 提供 `format_size` 等辅助函数
- 包含相关测试

### display_derive 包  
- 提供 `#[derive(Display)]` 过程宏
- 解析 `#[display(...)]` 属性
- 生成相应的 trait 实现代码

### core/display 模块
- 重新导出 `display_core` 的所有内容
- 重新导出 `display_derive::Display` 宏
- 为用户提供统一的导入接口

## 优势

1. **模块化**: 相关功能集中在专门的包中
2. **清晰分离**: trait 定义和宏实现分离
3. **易于测试**: 每个包都有自己的测试
4. **向后兼容**: 通过重新导出保持 API 不变
5. **避免冗余**: 消除了重复的代码
6. **简化导入**: 一行代码导入所有必要的内容

## 简化导入

现在您只需要一行代码就可以导入所有需要的内容：

```rust
use wechat_cleaner::display::*;
```

这一行代码将导入：
- `Display` 过程宏（用于 `#[derive(Display)]`）
- `Display` trait（提供 `display_summary()` 和 `display_details()` 方法）
- `DisplayValue` trait（用于自定义类型的显示格式）
- `format_size()` 函数（用于格式化文件大小）

### 旧的复杂导入方式（不再需要）

```rust
// 不再需要这样复杂的导入
use wechat_cleaner::Display;                    // 导入过程宏
use wechat_cleaner::display::Display as DisplayTrait;  // 导入 trait
```

### 新的简化导入方式

```rust
// 一行代码就够了！
use wechat_cleaner::display::*;
```

## 属性说明

- `#[display(summary)]`: 字段只在摘要中显示
- `#[display(details)]`: 字段只在详细信息中显示  
- `#[display(summary, details)]`: 字段在摘要和详细信息中都显示
- `#[display(name="自定义名称")]`: 使用自定义显示名称
- 没有 `#[display]` 属性的字段不会显示

## Vec<T> 特殊处理

- **摘要模式**: 显示数组长度（如 "3 items"）
- **详细模式**: 显示完整内容（如多行格式的数组内容）

## 示例

```rust
let my_struct = MyStruct {
    field1: 1024 * 1024,  // 1MB
    field2: "详细信息".to_string(),
    field3: vec!["file1.txt".to_string(), "file2.txt".to_string()],
    hidden_field: "不会显示".to_string(),
};

// 摘要显示
println!("{}", my_struct.display_summary());
// 输出:
// 自定义名称: 1.00 MB
// field3: 2 items

// 详细显示  
println!("{}", my_struct.display_details());
// 输出:
// 自定义名称: 1.00 MB
// field2: 详细信息
// field3: [
//   "file1.txt",
//   "file2.txt"
// ]
```

## 为什么这样设计？

之前的设计存在问题：
1. **core/src/display.rs** 和 **display_derive/src/lib.rs** 功能重复
2. 过程宏包不能导出非宏的内容（Rust 限制）
3. 测试和功能分散在不同文件中

新设计解决了这些问题：
1. **功能分离**: trait 定义在 display_core，宏在 display_derive
2. **统一接口**: core/display 重新导出所有内容
3. **清晰架构**: 每个包职责单一，易于维护

使用起来完全一样，但架构更清晰、更易维护！