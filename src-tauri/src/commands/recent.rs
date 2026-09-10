use crate::storage::recent::RecentFile;

/// 获取最近文件列表（按打开时间倒序）
#[tauri::command]
pub fn get_recent_files() -> Vec<RecentFile> {
    crate::storage::recent::load().files
}

/// 记录一次文件打开（去重置顶，保留最近 20 个）
#[tauri::command(rename_all = "camelCase")]
pub fn add_recent_file(
    path: String,
    name: String,
    dir: String,
    size: u64,
    progress_percent: f64,
) -> Result<(), String> {
    let mut store = crate::storage::recent::load();
    crate::storage::recent::push(
        &mut store,
        RecentFile {
            path,
            name,
            dir,
            size,
            last_opened_at: crate::storage::now_iso(),
            progress_percent,
            starred: false,
        },
    );
    crate::storage::recent::save(&store)
}

/// 从最近列表移除一条记录
#[tauri::command]
pub fn remove_recent_file(path: String) -> Result<(), String> {
    let mut store = crate::storage::recent::load();
    store.files.retain(|f| !f.path.eq_ignore_ascii_case(&path));
    crate::storage::recent::save(&store)
}

/// 设置标星状态（标星文件置顶展示）
#[tauri::command(rename_all = "camelCase")]
pub fn set_recent_starred(path: String, starred: bool) -> Result<(), String> {
    let mut store = crate::storage::recent::load();
    if let Some(f) = store.files.iter_mut().find(|f| f.path.eq_ignore_ascii_case(&path)) {
        f.starred = starred;
    }
    store.files.sort_by_key(|f| std::cmp::Reverse(f.starred));
    crate::storage::recent::save(&store)
}
