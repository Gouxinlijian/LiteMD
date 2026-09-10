use super::{load_json, save_json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单个文件的阅读进度
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEntry {
    /// 像素级滚动位置
    pub scroll_top: f64,
    /// 阅读进度百分比 0-100
    pub progress_percent: f64,
    /// 上次阅读时间 ISO8601
    pub last_read_at: String,
    /// 累计阅读时长（分钟）
    #[serde(default)]
    pub read_time_minutes: u64,
    /// 记录进度时的文件内容 MD5，用于判断文件是否被修改
    pub file_hash: String,
}

/// 全部文件的阅读进度，以绝对路径为 key
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProgressStore {
    #[serde(default)]
    pub files: HashMap<String, ProgressEntry>,
}

pub fn load() -> ProgressStore {
    load_json("reading_progress.json")
}

pub fn save(store: &ProgressStore) -> Result<(), String> {
    save_json("reading_progress.json", store)
}
