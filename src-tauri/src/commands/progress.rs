use crate::storage::now_iso;

/// 保存某文件的阅读进度。前端每 3 秒调用一次（防抖由前端控制），文件哈希一并记录
#[tauri::command(rename_all = "camelCase")]
pub fn save_progress(
    path: String,
    scroll_top: f64,
    progress_percent: f64,
    read_time_minutes: u64,
    file_hash: String,
) -> Result<(), String> {
    // 1. 同步最近列表中的进度，保证状态栏与最近列表两处显示一致
    let mut recent = crate::storage::recent::load();
    if let Some(f) = recent.files.iter_mut().find(|f| f.path.eq_ignore_ascii_case(&path)) {
        f.progress_percent = progress_percent;
    }
    crate::storage::recent::save(&recent)?;
    // 2. 更新该文件阅读进度
    let mut store = crate::storage::progress::load();
    let entry = store.files.entry(path).or_default();
    entry.scroll_top = scroll_top;
    entry.progress_percent = progress_percent;
    entry.read_time_minutes = read_time_minutes;
    entry.file_hash = file_hash;
    entry.last_read_at = now_iso();
    // 3. 落盘
    crate::storage::progress::save(&store)
}
