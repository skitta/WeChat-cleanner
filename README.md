# WeChat Cleaner

[![Rust](https://img.shields.io/badge/rust-1.80%2B-brightgreen.svg)](https://www.rust-lang.org)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/skitta/wechat-cleaner)

一个用 Rust 开发的微信缓存清理工具，帮助用户快速识别和清理重复的微信缓存文件，释放本地磁盘空间。

## ✨ 特性

- 🚀 **高性能扫描**：基于并行计算，快速扫描大量缓存文件
- 🔍 **智能识别**：基于文件内容哈希检测重复文件
- 👀 **预览功能**：清理前显示详细预览，确保操作安全
- 🛡️ **安全清理**：交互式确认机制，避免误删重要文件
- 📊 **详细统计**：提供清理前后的详细统计信息
- ⚙️ **配置灵活**：支持自定义清理规则和路径配置
- 🎨 **友好界面**：清晰的命令行界面和实时进度显示

## 🛠️ 安装

### 前置要求

- Rust 1.80 或更高版本
- macOS 或 Linux 系统（暂不支持 Windows）

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/skitta/wechat-cleaner.git
cd wechat-cleaner

# 构建项目
cargo build --release

# 运行
./target/release/wechat-cleaner --help
```

### 开发构建

```bash
# 快速开发构建
cargo build

# 运行（开发模式）
cargo run --bin wechat-cleaner -- --help
```

## 🚀 快速开始

### 基本使用流程

1. **扫描重复文件**
   ```bash
   wechat-cleaner scan
   ```

2. **预览并清理**
   ```bash
   wechat-cleaner clean
   ```

3. **查看配置**
   ```bash
   wechat-cleaner config
   ```

### 完整使用示例

```bash
# 1. 扫描微信缓存中的重复文件
$ wechat-cleaner scan
⠋ 正在扫描微信缓存文件...
总文件数: 1,234
重复文件数: 89
扫描耗时: 2.34s
扫描结果已保存到: /tmp/scan_result.json

# 2. 清理重复文件（包含预览和确认）
$ wechat-cleaner clean --verbose
⠋ 正在生成预览...
预计删除文件数: 67
预计释放空间: 45.2 MB
文件分组详情:
  文件夹: /Users/xxx/Library/Caches/WeChat/...
    保留文件: image_001.jpg
    删除文件列表: 3个文件

是否继续? (y/n): y

⠋ 正在清理重复文件...
删除文件数: 67
释放空间: 45.2 MB
清理耗时: 1.23s

# 3. 查看当前配置
$ wechat-cleaner config
当前配置:
  微信缓存路径: Some("/Users/xxx/Library/Caches/WeChat")
  默认清理模式: Auto
  最小文件大小: 1024 字节
```

## 📚 命令参考

### scan - 扫描重复文件

```bash
wechat-cleaner scan [OPTIONS]

选项:
  -v, --verbose    显示详细扫描信息
  -h, --help       显示帮助信息
```

### clean - 清理重复文件

```bash
wechat-cleaner clean [OPTIONS]

选项:
  -m, --mode <MODE>    清理模式: auto [默认: auto]
  -v, --verbose        显示详细清理信息
  -h, --help          显示帮助信息
```

### config - 显示配置信息

```bash
wechat-cleaner config

显示当前的配置设置，包括微信缓存路径、清理规则等。
```

## ⚙️ 配置

项目使用 `default.toml` 配置文件，支持自定义以下设置：

```toml
[wechat]
# 微信缓存目录路径（自动检测）
cache_path = "/Users/username/Library/Caches/WeChat"
# 缓存文件匹配模式
cache_patterns = ".*\\.(jpg|jpeg|png|gif|bmp|webp|mp4|mov|avi|mkv|log|tmp)$"

[cleaning]
# 默认清理模式
default_mode = "Auto"
# 最小文件大小阈值（字节）
min_file_size = 1024
# 是否保留原始文件
preserve_originals = false
```

## 🏗️ 项目架构

```
wechat-cleaner/
├── core/              # 核心业务逻辑
│   ├── scanner.rs     # 文件扫描模块
│   ├── cleaner.rs     # 文件清理模块
│   ├── config/        # 配置管理
│   └── ...
├── cli/               # 命令行界面
├── display_core/      # 显示特性核心库
├── display_derive/    # 显示特性宏库
└── docs/             # 项目文档
```

### 核心模块

- **Scanner**: 递归扫描目录，基于哈希检测重复文件
- **Cleaner**: 提供预览和安全清理功能
- **Config**: 灵活的配置管理系统
- **Display**: 统一的显示特性系统

### 设计原则

- 🎯 **单一职责**: 每个模块专注特定功能
- 🔄 **模块化**: 清晰的模块边界和接口
- 🛡️ **安全第一**: 预览确认机制防止误操作
- ⚡ **性能优化**: 并行计算和智能缓存

## 🧪 开发指南

### 开发环境设置

```bash
# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/skitta/wechat-cleaner.git
cd wechat-cleaner

# 安装依赖并构建
cargo build
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test -p core

# 运行基准测试
cargo bench
```

### 代码检查

```bash
# 代码格式化
cargo fmt

# 静态检查
cargo clippy

# 检查所有包
cargo check --workspace
```

### 构建配置

项目提供多种构建配置：

```bash
# 开发构建（快速编译）
cargo build

# 发布构建（优化性能）
cargo build --release

# 生产构建（最大优化）
cargo build --profile dist
```

## 📖 文档

详细文档位于 [docs/](./docs/) 目录：

- [项目整合总结](./docs/project-integration-summary.md)
- [显示特性集成指南](./docs/DISPLAY_FEATURE.md)
- [清理功能重构文档](./docs/CLEANER_REFACTOR.md)
- [CLI 修复总结](./docs/cli-fix-summary.md)

## 🤝 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. **Fork** 本仓库
2. **创建** 特性分支 (`git checkout -b feature/amazing-feature`)
3. **提交** 更改 (`git commit -m 'Add some amazing feature'`)
4. **推送** 分支 (`git push origin feature/amazing-feature`)
5. **创建** Pull Request

### 贡献规范

- 遵循 Rust 官方编码规范
- 确保所有测试通过
- 添加适当的文档和注释
- 保持提交信息清晰明确

## 🐛 问题反馈

如果你发现了 bug 或有功能建议，请在 [Issues](https://github.com/skitta/wechat-cleaner/issues) 页面提交。

提交问题时请包含：
- 操作系统版本
- Rust 版本
- 错误信息或日志
- 复现步骤

## 📄 许可证

本项目采用 MIT 或 Apache-2.0 双许可证。详见：

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

## 🙏 致谢

感谢以下开源项目的支持：

- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [clap](https://github.com/clap-rs/clap) - 命令行参数解析
- [rayon](https://github.com/rayon-rs/rayon) - 并行计算框架
- [serde](https://github.com/serde-rs/serde) - 序列化框架

## 📊 项目状态

- ✅ 核心扫描功能
- ✅ 安全清理机制
- ✅ 预览确认功能
- ✅ 配置管理系统
- ✅ 显示特性系统
- ✅ 完整的 CLI 界面
- 🔄 图形界面（计划中）
- 🔄 Windows 支持（计划中）

---

<div align="center">

**[⬆ 回到顶部](#wechat-cleaner)**

Made with ❤️ by [skitta](https://github.com/skitta)

</div>