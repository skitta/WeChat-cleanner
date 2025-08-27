#[cfg(feature = "display")]
use wechat_cleaner::display;

#[cfg(feature = "display")]
use wechat_cleaner::display::*;

#[cfg(feature = "display")]
#[derive(Display)]
struct SimpleTest {
    #[display(summary, name="测试字段")]
    value: u64,
}

#[cfg(feature = "display")]
fn main() {
    let test = SimpleTest { value: 1024 };
    
    println!("简化导入测试:");
    println!("摘要: {}", test.display_summary());
    println!("详情: {}", test.display_details());
    
    // 测试 DisplayValue trait
    let vec_test: Vec<String> = vec!["item1".to_string(), "item2".to_string()];
    println!("Vec 摘要: {}", vec_test.format_display_summary());
    println!("Vec 详情: {}", vec_test.format_display_details());
    
    // 测试 format_size 函数
    println!("大小格式化: {}", format_size(1024 * 1024));
}

#[cfg(not(feature = "display"))]
fn main() {
    println!("Display feature 未启用，请使用 --features display 来启用此功能");
}