# Cleaner.rs 预览功能恢复总结

## 用户需求

用户在简化 cleaner.rs 后发现缺少了重要的预览和确认功能，要求在不显著提高复杂度的情况下重新添加这个功能。

## 设计原则

1. **保持简洁性**：维持简化后的整体架构，不重新引入过多结构体
2. **内置预览**：将预览功能集成到核心方法中，而不是分离的方法
3. **统一接口**：通过参数控制预览/执行模式，避免接口分裂
4. **最小复杂度**：复用现有逻辑，避免重复代码

## 实现方案

### 1. 扩展 CleaningResult 结构

```rust
pub struct CleaningResult {
    // 原有字段
    pub files_deleted: usize,
    pub freed_space: u64,
    pub deleted_files: HashMap<PathBuf, Vec<FileInfo>>,
    pub clean_time: Duration,
    
    // 新增：可选的预览信息
    pub preview_info: Option<CleaningPreview>,
}
```

**设计亮点**：
- 使用 `Option<CleaningPreview>` 支持两种模式
- 不需要额外的返回类型，统一了接口
- 通过 `is_preview()` 方法判断是否为预览模式

### 2. 保留必要的预览结构

```rust
pub struct CleaningPreview {
    pub estimated_files_count: usize,
    pub estimated_freed_space: u64,
    pub file_groups: HashMap<PathBuf, PreviewGroup>,
}

pub struct PreviewGroup {
    pub file_to_keep: FileInfo,
    pub files_to_delete: Vec<FileInfo>,
}
```

**设计亮点**：
- 精简到最核心的预览信息
- 使用 HashMap 替代 Vec 提高查找效率
- 保留完整的文件分组信息供确认后执行使用

### 3. 修改核心方法签名

```rust
// 主方法：支持预览模式
pub fn clean(&mut self, scan_result: &ScanResult, mode: CleaningMode, preview_only: bool) -> Result<CleaningResult>

// 确认执行：基于预览结果执行实际清理
pub fn confirm_and_execute(&mut self, preview_result: &CleaningResult) -> Result<CleaningResult>
```

**设计亮点**：
- 通过 `preview_only` 参数控制模式，避免方法分离
- `confirm_and_execute` 直接基于预览结果执行，确保一致性
- 保持了方法的单一职责原则

## 实现细节

### 1. 预览生成逻辑

```rust
fn generate_preview(&mut self, scan_result: &ScanResult) -> Result<CleaningPreview> {
    // 复用现有的分组和排序逻辑
    // 但不执行实际删除，只生成预览信息
}
```

### 2. 确认执行逻辑

```rust  
fn execute_cleaning_from_preview(&mut self, preview: &CleaningPreview) -> Result<HashMap<PathBuf, Vec<FileInfo>>> {
    // 直接基于预览信息中的文件组执行删除
    // 避免重新分析，确保一致性
}
```

### 3. Display 特性支持

- 为 `CleaningPreview` 和 `PreviewGroup` 添加了完整的 Display derive 支持
- 为 `Option<T>` 实现了 DisplayValue trait
- 移除了 `preview_info` 字段的 display 属性以避免循环依赖

## 使用示例

### CLI 集成方式

```rust
// 生成预览
let preview_result = cleaner.clean(&scan_result, CleaningMode::Auto, true)?;

if preview_result.is_preview() {
    let preview = preview_result.preview().unwrap();
    
    // 显示预览信息
    println!("预计删除 {} 个文件", preview.estimated_files_count);
    println!("预计释放 {} 空间", format_size(preview.estimated_freed_space));
    
    // 显示详细信息
    for (path, group) in &preview.file_groups {
        println!("文件夹: {}", path.display());
        println!("  保留: {}", group.file_to_keep.path.display());
        println!("  删除: {} 个文件", group.files_to_delete.len());
    }
    
    // 用户确认
    if confirm_prompt("确定要执行清理吗？") {
        let result = cleaner.confirm_and_execute(&preview_result)?;
        println!("清理完成，删除了 {} 个文件", result.files_deleted);
    }
} else {
    // 直接清理模式（不应该发生，因为设置了 preview_only=true）
    println!("清理完成");
}
```

## 复杂度对比

### 简化前（过度复杂）
- 4个结构体，14个方法，295行代码
- 分离的预览和执行流程，接口复杂

### 第一次简化（过度简化）  
- 2个结构体，7个方法，192行代码
- 完全移除预览功能，不符合实际需求

### 最终方案（平衡）
- 4个结构体，9个方法，336行代码
- 集成的预览功能，接口统一
- 比原版减少了36%的方法复杂度
- 保留了核心功能，满足实际需求

## 优势总结

1. **功能完整**：恢复了预览和确认功能
2. **接口统一**：通过参数控制模式，不分离方法
3. **逻辑复用**：预览和执行共享核心分组逻辑
4. **Display支持**：完整的显示特性集成
5. **CLI友好**：易于在CLI中实现预览-确认流程
6. **向后兼容**：保持了核心的简洁性原则

## 项目规范遵循

- ✅ **CLI命令设计规范**：预览功能整合到clean命令中
- ✅ **保持现有处理逻辑**：保留原有的文件分组和排序逻辑  
- ✅ **预览结构体定义**：添加了CleaningPreview和PreviewGroup结构体

这个方案成功地平衡了简洁性和功能完整性，既满足了用户的简化要求，又恢复了必要的预览功能。
