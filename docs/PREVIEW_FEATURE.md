# 预览清理功能使用指南

## 功能概述

新增的预览功能让用户能够在实际删除文件前查看将要被删除的文件列表，提供了更安全的清理体验。

## 代码重构亮点

### 🎨 设计改进

采用了参考 [file_utils.rs](file:///Users/skittachen/Documents/Codes/Github/wechat-cleaner/lib/src/core/file_utils.rs) 的设计模式，对 CLI 代码进行了全面重构：

1. **模块化结构体**：
   - `CliOperations`：核心操作类，统一管理配置和通用功能
   - `ScanHandler`：扫描操作处理器
   - `PreviewHandler`：预览操作处理器
   - `CleanerHandler`：清理操作处理器
   - `ConfigHandler`：配置操作处理器

2. **单一职责原则**：每个处理器只负责一类操作
3. **抽象封装**：公共操作抽取到基础类中
4. **类型安全**：使用统一的 `AppResult<T>` 类型
5. **详细文档**：每个模块和方法都有完整的文档说明

### 🔧 技术优化

- **消除重复代码**：原本 500+ 行的重复逻辑缩减为结构化的模块
- **统一进度显示**：抽取了通用的进度条创建逻辑
- **错误处理统一**：使用一致的错误处理模式
- **内存优化**：减少不必要的克隆和分配

## 使用流程

### 1. 扫描重复文件

首先需要扫描重复文件：

```bash
cargo run --bin wechat-cleaner scan
```

### 2. 预览清理操作

查看将要被删除的文件：

```bash
# 简单预览（推荐）
cargo run --bin wechat-cleaner preview

# 详细预览（显示完整文件列表）
cargo run --bin wechat-cleaner preview --verbose
```

### 3. 确认并执行清理

预览后如果确认无误，在提示时输入 `y` 确认清理，或输入 `n` 取消。

## 预览输出示例

### 简单预览模式

```
预览清理结果:
  - 预计删除 15 个文件
  - 预计释放空间 125.67 MB
  - 涉及 8 个文件夹

是否继续清理? (y/n)
```

### 详细预览模式 (`--verbose`)

```
预览清理结果:
  - 预计删除 15 个文件
  - 预计释放空间 125.67 MB
  - 涉及 8 个文件夹

详细文件列表:

[1] 文件夹: /Users/user/Library/Containers/com.tencent.xinWeChat/Data/Library/Application Support/com.tencent.xinWeChat/2.0b4.0.9/cache/images
  保留: IMG_001.jpg (2.5 MB)
  删除:
    - IMG_001(1).jpg (2.5 MB)
    - IMG_001(2).jpg (2.5 MB)

[2] 文件夹: /Users/user/Library/Containers/com.tencent.xinWeChat/Data/Library/Application Support/com.tencent.xinWeChat/2.0b4.0.9/cache/videos
  保留: VIDEO_123.mp4 (15.2 MB)
  删除:
    - VIDEO_123(1).mp4 (15.2 MB)
    - VIDEO_123(2).mp4 (15.2 MB)

是否继续清理? (y/n)
```

## 功能特点

### 🔍 安全预览
- 在删除前显示完整的文件列表
- 清楚标示保留和删除的文件
- 提供统计信息帮助决策

### 📊 详细统计
- 显示预计删除的文件数量
- 计算预计释放的存储空间
- 按文件夹分组显示

### 🎛️ 灵活控制
- 支持简单和详细两种预览模式
- 用户确认后才执行实际删除
- 可随时取消清理操作

### 📈 进度追踪
- 实时显示清理进度
- 详细的日志记录
- 完成后的统计报告

## 清理策略

当前支持的清理模式：

- **Auto 模式**：在每个文件夹内，保留修改时间最早的文件，删除其他重复文件

## 安全机制

1. **预览确认**：必须用户确认后才执行删除
2. **文件大小检查**：跳过小于配置阈值的文件
3. **权限处理**：自动处理只读文件权限
4. **错误恢复**：单个文件删除失败不影响整体流程
5. **详细日志**：记录所有操作和错误信息

## 配置选项

可以通过配置文件调整：

- `min_file_size`：最小文件大小阈值（默认 1KB）
- `default_mode`：默认清理模式
- `scan_result_save_path`：扫描结果保存路径

查看当前配置：

```bash
cargo run --bin wechat-cleaner config
```

## 注意事项

1. 确保在执行清理前备份重要文件
2. 预览功能基于文件的修改时间进行决策
3. 建议先在小范围内测试清理效果
4. 清理完成后扫描结果文件会被自动删除

## 故障排除

### 找不到扫描结果
```
请先执行扫描命令: wechat-cleaner scan
```
**解决方案**：先运行 `scan` 命令生成扫描结果

### 没有重复文件
```
未发现重复文件，无需清理
```
**说明**：当前没有检测到重复文件，无需进行清理操作

### 权限问题
如果遇到文件权限问题，程序会自动尝试修改文件权限后重试删除。
