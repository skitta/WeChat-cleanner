// lib/build.rs
use std::env;

fn main() {
    // 根据目标平台设置特定优化
    let target = env::var("TARGET").unwrap();
    
    if target.contains("darwin") {
        println!("cargo:rustc-link-arg=-framework");
        println!("cargo:rustc-link-arg=CoreFoundation");
    }
    
    // 启用 CPU 特定优化
    if cfg!(feature = "native-optimizations") {
        println!("cargo:rustc-env=RUSTFLAGS=-C target-cpu=native");
    }
    
    // 构建时间戳
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", 
             std::process::Command::new("date")
                 .arg("+%Y-%m-%d %H:%M:%S")
                 .output()
                 .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                 .unwrap_or_else(|_| "unknown".to_string()));
}