use super::{load_json, save_json};
use serde::{Deserialize, Serialize};

/// 最近打开的一个文件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub dir: String,
    pub size: u64,
    pub last_opened_at: String,
    pub progress_percent: f64,
    /// 标星：置顶展示并显示 ★ 标记，优先阅读
    #[serde(default)]
    pub starred: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RecentStore {
    #[serde(default)]
    pub files: Vec<RecentFile>,
}

/// 最近文件列表上限
const MAX_RECENT: usize = 20;

pub fn load() -> RecentStore {
    load_json("recent_files.json")
}

pub fn save(store: &RecentStore) -> Result<(), String> {
    save_json("recent_files.json", store)
}

/// 插入最近文件：同路径去重并置顶；保留已有标星状态；标星文件整体排在前面
pub fn push(store: &mut RecentStore, file: RecentFile) {
    let starred = store
        .files
        .iter()
        .find(|f| f.path.eq_ignore_ascii_case(&file.path))
        .map(|f| f.starred)
        .unwrap_or(false);
    let mut new_file = file;
    new_file.starred = starred;
    store.files.retain(|f| !f.path.eq_ignore_ascii_case(&new_file.path));
    store.files.insert(0, new_file);
    store.files.sort_by_key(|f| std::cmp::Reverse(f.starred));
    store.files.truncate(MAX_RECENT);
}
