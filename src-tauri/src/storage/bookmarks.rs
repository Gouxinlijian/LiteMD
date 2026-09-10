use super::{load_json, save_json};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 单个书签：按章节标题展示，按百分比定位（跨设备/布局变化都稳）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub label: String,
    /// 书签对应章节锚点；恢复时若页面仍存在该 id 则优先精确跳转
    #[serde(default)]
    pub heading_id: String,
    pub progress_percent: f64,
    /// 创建时间戳（毫秒），也作为唯一标识
    pub created_at: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BookmarkStore {
    #[serde(default)]
    pub files: BTreeMap<String, Vec<Bookmark>>,
}

/// 单文件书签上限
const MAX_PER_FILE: usize = 50;

pub fn load() -> BookmarkStore {
    load_json("bookmarks.json")
}

pub fn save(store: &BookmarkStore) -> Result<(), String> {
    save_json("bookmarks.json", store)
}

/// 插入书签：同一位置附近（±0.5%）视为重复覆盖，按创建时间倒序并截断
pub fn push(store: &mut BookmarkStore, path: &str, bookmark: Bookmark) {
    let list = store.files.entry(path.to_string()).or_default();
    list.retain(|b| (b.progress_percent - bookmark.progress_percent).abs() >= 0.5);
    list.push(bookmark);
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    list.truncate(MAX_PER_FILE);
}

/// 按创建时间移除单个书签
pub fn remove(store: &mut BookmarkStore, path: &str, created_at: i64) {
    if let Some(list) = store.files.get_mut(path) {
        list.retain(|b| b.created_at != created_at);
        if list.is_empty() {
            store.files.remove(path);
        }
    }
}
