//! ToolTree 的核心逻辑，和界面、Tauri 无关，可以单独测试：
//! - clips：剪贴板历史的存储（SQLite + 图片文件）
//! - hash：文本和文件的哈希

pub mod clips;
pub mod hash;

/// 当前时间，毫秒
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
