use crate::storage::bookmarks::{self, Bookmark};

/// 取某文件的书签列表（按时间倒序）
#[tauri::command]
pub fn get_bookmarks(path: String) -> Vec<Bookmark> {
    bookmarks::load().files.get(&path).cloned().unwrap_or_default()
}

/// 新增书签
#[tauri::command(rename_all = "camelCase")]
pub fn add_bookmark(
    path: String,
    label: String,
    heading_id: String,
    progress_percent: f64,
) -> Vec<Bookmark> {
    let mut store = bookmarks::load();
    bookmarks::push(
        &mut store,
        &path,
        Bookmark {
            label,
            heading_id,
            progress_percent,
            created_at: chrono::Utc::now().timestamp_millis(),
        },
    );
    bookmarks::save(&store).ok();
    store.files.get(&path).cloned().unwrap_or_default()
}

/// 删除指定书签
#[tauri::command(rename_all = "camelCase")]
pub fn remove_bookmark(path: String, created_at: i64) -> Vec<Bookmark> {
    let mut store = bookmarks::load();
    bookmarks::remove(&mut store, &path, created_at);
    bookmarks::save(&store).ok();
    store.files.get(&path).cloned().unwrap_or_default()
}
