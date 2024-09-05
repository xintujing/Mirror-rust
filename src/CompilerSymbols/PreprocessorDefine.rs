use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{Read, Write};

const DEFINE_FILE_PATH: &str = "build_config.txt"; // 模拟Unity中的定义符号存储

/// 在编译后添加定义符号
fn add_define_symbols() {
    // 模拟从配置文件加载当前定义符号
    let current_defines = load_current_defines();

    // 我们想要确保存在的滚动定义符号
    let mut defines: HashSet<String> = current_defines.into_iter().collect();

    defines.insert("MIRROR".to_string());
    defines.insert("MIRROR_81_OR_NEWER".to_string());
    defines.insert("MIRROR_82_OR_NEWER".to_string());
    defines.insert("MIRROR_83_OR_NEWER".to_string());
    defines.insert("MIRROR_84_OR_NEWER".to_string());
    defines.insert("MIRROR_85_OR_NEWER".to_string());
    defines.insert("MIRROR_86_OR_NEWER".to_string());
    defines.insert("MIRROR_89_OR_NEWER".to_string());
    defines.insert("MIRROR_90_OR_NEWER".to_string());

    let new_defines = defines.into_iter().collect::<Vec<_>>().join(";");

    // 只有当内容发生变化时才更新
    if new_defines != load_define_string_from_file() {
        save_defines_to_file(&new_defines);
    }
}

/// 从文件中加载当前定义（模拟Unity的PlayerSettings）
fn load_define_string_from_file() -> String {
    let mut file_content = String::new();
    if let Ok(mut file) = OpenOptions::new().read(true).open(DEFINE_FILE_PATH) {
        file.read_to_string(&mut file_content).unwrap_or_default();
    }
    file_content
}

/// 将当前的定义解析成字符串向量
fn load_current_defines() -> Vec<String> {
    let current_define_str = load_define_string_from_file();
    current_define_str.split(';').map(|s| s.to_string()).collect()
}

/// 将更新后的定义符号保存回文件
fn save_defines_to_file(defines: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(DEFINE_FILE_PATH)
    {
        file.write_all(defines.as_bytes()).unwrap();
    }
}