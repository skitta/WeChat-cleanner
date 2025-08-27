# 贡献指南

感谢您对 WeChat Cleaner 项目的关注！我们欢迎任何形式的贡献，无论是功能改进、bug 修复、文档完善还是使用反馈。

## 🎯 贡献方式

### 代码贡献

1. **Fork 项目**
   - 在 GitHub 上 fork 这个仓库
   - 克隆你的 fork 到本地

2. **设置开发环境**
   ```bash
   git clone https://github.com/[your-username]/wechat-cleaner.git
   cd wechat-cleaner
   cargo build
   ```

3. **创建特性分支**
   ```bash
   git checkout -b feature/your-feature-name
   ```

4. **进行开发**
   - 编写代码和测试
   - 确保代码符合项目规范
   - 运行测试确保所有功能正常

5. **提交更改**
   ```bash
   git add .
   git commit -m "Add: 描述你的更改"
   git push origin feature/your-feature-name
   ```

6. **创建 Pull Request**
   - 在 GitHub 上创建 PR
   - 填写详细的描述说明

### 问题反馈

如果你发现了 bug 或有改进建议：

1. 查看 [Issues](https://github.com/skitta/wechat-cleaner/issues) 确认问题未被报告
2. 创建新的 Issue，包含：
   - 清晰的问题描述
   - 复现步骤
   - 预期行为 vs 实际行为
   - 环境信息（OS、Rust 版本等）

### 文档贡献

文档改进同样重要：
- 修复错别字或语法错误
- 补充使用示例
- 完善 API 文档
- 翻译文档

## 📋 开发规范

### 代码风格

- 遵循 Rust 官方编码规范
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 进行静态检查

### 提交规范

提交信息应遵循以下格式：

```
类型: 简短描述

详细描述（可选）

相关 Issue: #123
```

**类型说明：**
- `Add`: 新增功能
- `Fix`: 修复 bug
- `Update`: 更新现有功能
- `Remove`: 移除功能
- `Refactor`: 重构代码
- `Docs`: 文档相关
- `Test`: 测试相关
- `Style`: 代码格式调整

### 测试要求

- 新功能必须包含相应的测试
- 确保所有现有测试通过
- 测试覆盖率应保持在合理水平

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test -p core

# 检查测试覆盖率
cargo tarpaulin --out Html
```

### 性能考虑

- 新功能不应显著降低性能
- 大的性能改进请提供基准测试
- 避免引入不必要的依赖

## 🔧 开发工具

### 推荐工具

- **IDE**: VS Code with rust-analyzer
- **格式化**: rustfmt
- **静态检查**: clippy
- **测试**: cargo test
- **文档**: cargo doc

### 有用的命令

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行所有检查
cargo fmt && cargo clippy && cargo test

# 生成文档
cargo doc --open

# 清理构建缓存
cargo clean
```

## 📖 项目结构

了解项目结构有助于更好地贡献：

```
wechat-cleaner/
├── core/              # 核心业务逻辑
│   ├── src/
│   │   ├── scanner.rs    # 文件扫描
│   │   ├── cleaner.rs    # 文件清理
│   │   ├── config/       # 配置管理
│   │   └── ...
├── cli/               # 命令行界面
├── display_core/      # 显示特性核心
├── display_derive/    # 显示特性宏
├── docs/             # 项目文档
└── tests/            # 集成测试
```

### 模块说明

- **core**: 包含所有核心业务逻辑
- **cli**: 命令行界面实现
- **display_***: 统一的显示特性系统
- **docs**: 详细的项目文档

## 🎨 设计理念

在贡献时请考虑项目的设计理念：

### 核心原则

1. **安全第一**: 避免误删用户数据
2. **性能优化**: 充分利用 Rust 的性能优势
3. **用户友好**: 提供清晰的界面和反馈
4. **模块化**: 保持代码的可维护性
5. **简洁性**: 避免过度设计和复杂性

### 架构模式

- **单一职责**: 每个模块专注特定功能
- **依赖注入**: 通过接口解耦模块
- **错误处理**: 统一的错误类型和处理
- **配置驱动**: 灵活的配置管理

## 🚀 发布流程

项目维护者负责版本发布：

1. **版本规划**: 确定新版本内容
2. **代码审核**: 审核所有 PR
3. **测试验证**: 全面测试新功能
4. **文档更新**: 更新相关文档
5. **版本发布**: 创建 release tag
6. **公告发布**: 发布更新说明

### 版本号规则

遵循 [Semantic Versioning](https://semver.org/):

- `MAJOR.MINOR.PATCH`
- MAJOR: 破坏性变更
- MINOR: 新增功能（向后兼容）
- PATCH: bug 修复（向后兼容）

## 🤝 社区准则

### 行为规范

- 尊重所有参与者
- 建设性地讨论技术问题
- 欢迎不同观点和建议
- 帮助新手贡献者

### 沟通方式

- 使用 GitHub Issues 讨论功能和 bug
- PR 中详细说明变更内容
- 及时回应 code review 意见

## 📝 许可证

通过贡献代码，您同意您的贡献将在 MIT 或 Apache-2.0 双许可证下分发。

## 🙏 致谢

感谢所有贡献者的努力！您的每一个贡献都让这个项目变得更好。

---

如果您有任何问题，请随时通过 Issues 或 PR 与我们联系。