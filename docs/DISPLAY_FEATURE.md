# Display Feature 功能说明

Display功能现在已经作为`core`包的一个可选feature实现。这样可以让用户根据需要选择是否启用显示功能，减少不必要的依赖。

## 功能概述

Display feature 提供了强大的结构体显示功能，包括：
- 自动生成摘要和详细信息显示方法
- 支持字段级别的显示控制
- 内置类型的格式化支持（如文件大小、向量等）
- 简化的导入和使用方式

## 启用方式

### 1. 在Cargo.toml中启用（推荐）

```toml
[dependencies]
wechat_cleaner = { path = "core", features = ["display"] }
```

### 2. 通过命令行启用

```bash
# 编译时启用
cargo build --features display

# 运行时启用
cargo run --features display

# 测试时启用
cargo test --features display
```

### 3. 作为默认feature

display功能已经被设置为默认启用的feature，如果要禁用它：

```bash
# 禁用所有默认features
cargo build --no-default-features

# 或在Cargo.toml中指定
[dependencies]
wechat_cleaner = { path = "core", default-features = false }
```

## 使用示例

### 基本使用

```rust
// 当启用display feature时
#[cfg(feature = "display")]
use wechat_cleaner::display::*;

#[cfg(feature = "display")]
#[derive(Display)]
struct FileInfo {
    #[display(summary, name="文件名")]
    pub name: String,
    
    #[display(summary, name="大小")]
    pub size: u64,
    
    #[display(details, name="路径")]
    pub path: String,
    
    #[display(summary, details)]
    pub tags: Vec<String>,
}

#[cfg(feature = "display")]
fn main() {
    let file = FileInfo {
        name: "example.txt".to_string(),
        size: 1024 * 1024, // 1MB
        path: "/home/user/example.txt".to_string(),
        tags: vec!["文档".to_string(), "重要".to_string()],
    };
    
    println!("摘要: {}", file.display_summary());
    println!("详情: {}", file.display_details());
}

#[cfg(not(feature = "display"))]
fn main() {
    println!("Display功能未启用");
}
```

### 条件编译

当display feature未启用时，相关代码会被条件编译排除，确保不会有编译错误：

```rust
// 只有在启用display feature时才编译这个模块
#[cfg(feature = "display")]
pub mod display_utils;

// 只有在启用display feature时才导入
#[cfg(feature = "display")]
use wechat_cleaner::display::*;

// 条件实现
#[cfg(feature = "display")]
impl SomeStruct {
    pub fn show_info(&self) {
        println!("{}", self.display_summary());
    }
}

#[cfg(not(feature = "display"))]
impl SomeStruct {
    pub fn show_info(&self) {
        println!("Display功能未启用");
    }
}
```

## Feature配置详情

### Core包中的配置

在`core/Cargo.toml`中：

```toml
[dependencies]
# Display功能相关依赖（可选）
display_core = { path = "../display_core", optional = true }
display_derive = { path = "../display_derive", optional = true }

[features]
default = ["display"]
display = ["dep:display_core", "dep:display_derive"]
```

### 在其他包中使用

如果要在其他包中使用core的display功能：

```toml
[dependencies]
core = { path = "../core", package = "wechat_cleaner", features = ["display"] }

# 或者使用默认features（已包含display）
core = { path = "../core", package = "wechat_cleaner" }
```

由于display是默认feature，所以大多数情况下不需要显式指定。

## 编译测试

### 启用display功能
```bash
# 使用默认features（包含display）
cargo check

# 显式启用display
cargo check --features display

# 运行示例
cargo run --example simple_import_test --features display
```

### 禁用display功能
```bash
# 禁用所有默认features
cargo check --no-default-features

# 运行示例（会显示feature未启用的消息）
cargo run --example simple_import_test --no-default-features
```

## 架构说明

Display功能由三个独立的包组成：

1. **display_core**: 包含核心trait定义和基础类型实现
2. **display_derive**: 包含`#[derive(Display)]`过程宏
3. **core/display**: 重新导出模块，提供统一的导入接口

这种设计的优势：
- **模块化**: 功能分离清晰
- **可选依赖**: 不需要时不会增加编译时间和二进制大小
- **向后兼容**: API保持一致
- **易于维护**: 每个包职责单一

## 注意事项

1. **条件编译**: 使用display功能的代码必须用`#[cfg(feature = "display")]`包装
2. **导入方式**: 推荐使用`use wechat_cleaner::display::*;`一行导入所有内容
3. **性能影响**: 当不启用display feature时，相关代码完全不会编译，无性能影响
4. **依赖管理**: display_core和display_derive只在启用feature时才会被编译和链接

## 示例项目

查看以下示例了解详细用法：
- `core/examples/simple_import_test.rs` - 基本使用示例
- `core/examples/display_example.rs` - 复杂结构体示例

运行示例：
```bash
# 启用display功能
cargo run --example simple_import_test --features display

# 禁用display功能
cargo run --example simple_import_test --no-default-features
```

## 跨包使用

### 在其他workspace成员中使用

如果要在CLI包等其他workspace成员中使用core的display功能：

1. 在目标包的`Cargo.toml`中添加依赖和features：

```toml
[dependencies]
core = { path = "../core", version = "0.1.0", package = "core" }

[features]
default = ["display"]
display = ["core/display"]
```

2. 在代码中正确导入：

```rust
// 必须导入模块和具体内容
#[cfg(feature = "display")]
use core::display;  // 导入模块

#[cfg(feature = "display")]
use core::display::*;  // 导入所有内容

#[cfg(feature = "display")]
use core::Display;  // 导入过程宏

// 使用
#[cfg(feature = "display")]
#[derive(Display)]
struct MyStruct {
    #[display(summary)]
    field: String,
}
```

### 条件编译最佳实践

```rust
// 结构体定义
#[cfg(feature = "display")]
#[derive(Display)]
struct MyStruct {
    #[display(summary, name="字段")]
    field: String,
}

// 功能实现
#[cfg(feature = "display")]
impl MyStruct {
    pub fn show_info(&self) {
        println!("{}", self.display_summary());
    }
}

#[cfg(not(feature = "display"))]
impl MyStruct {
    pub fn show_info(&self) {
        println!("Display功能未启用");
    }
}
```