#!/bin/bash

echo "=== Display Feature 功能测试 ==="
echo

echo "1. 测试 Core 包 - 启用 display"
cargo run --manifest-path core/Cargo.toml --example simple_import_test --features display
echo

echo "2. 测试 Core 包 - 禁用 display"
cargo run --manifest-path core/Cargo.toml --example simple_import_test --no-default-features
echo

echo "3. 测试 CLI 包 - 启用 display"
cargo run --manifest-path cli/Cargo.toml --example display_usage --features display
echo

echo "4. 测试 CLI 包 - 禁用 display"
cargo run --manifest-path cli/Cargo.toml --example display_usage --no-default-features
echo

echo "5. 测试整个 workspace 编译 - 默认 features"
cargo build
echo

echo "6. 测试整个 workspace 编译 - 无默认 features"
cargo build --no-default-features
echo

echo "7. 测试复杂示例"
cargo run --manifest-path core/Cargo.toml --example display_example --features display
echo

echo "=== 所有测试完成 ==="