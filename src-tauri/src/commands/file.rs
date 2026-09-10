use crate::storage::progress::ProgressEntry;
use serde::Serialize;

/// 打开文件后返回给前端的完整数据：内容 + 哈希 + 该文件的历史进度
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePayload {
    pub path: String,
    pub name: String,
    pub dir: String,
    pub content: String,
    pub size: u64,
    pub hash: String,
    pub progress: ProgressEntry,
}

/// 按路径读取文件内容，并附带该文件的阅读进度（用于恢复）
fn build_payload(path: &str) -> Result<FilePayload, String> {
    // 1. 读取文件内容
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取文件失败：{e}"))?;
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    // 2. 计算内容哈希，供前端判断文件是否被修改
    let hash = crate::utils::hash::md5_hex(&content);
    // 3. 取该文件历史进度（无记录则为空进度）
    let store = crate::storage::progress::load();
    let progress = store.files.get(path).cloned().unwrap_or_default();

    Ok(FilePayload {
        path: path.to_string(),
        name: crate::utils::path::file_name(path),
        dir: crate::utils::path::parent_dir(path),
        content,
        size,
        hash,
        progress,
    })
}

/// 打开文件：桌面弹系统对话框；安卓走 SAF 选择并复制到私有目录后读取（content:// 授权会过期）
#[tauri::command(rename_all = "camelCase")]
pub fn open_file_dialog(
    app: tauri::AppHandle,
    last_dir: Option<String>,
) -> Result<Option<FilePayload>, String> {
    #[cfg(target_os = "android")]
    {
        let _ = last_dir; // 安卓无"上次打开目录"概念
        let json = crate::file_opener::call(&app, "openAndImport", serde_json::json!({}))?;
        // 用户取消（cancelled）或空结果 → Ok(None)，前端静默
        if json.get("cancelled").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Ok(None);
        }
        let path = match json.get("path").and_then(|v| v.as_str()) {
            Some(p) if !p.is_empty() => p,
            _ => return Ok(None),
        };
        return build_payload(path).map(Some);
    }

    #[cfg(not(target_os = "android"))]
    {
        use tauri_plugin_dialog::DialogExt;

        let mut builder = app.dialog().file().add_filter("Markdown", &["md", "markdown"]);
        if let Some(dir) = last_dir {
            if std::path::Path::new(&dir).is_dir() {
                builder = builder.set_directory(dir);
            }
        }
        let picked = builder.blocking_pick_file();

        match picked {
            Some(path) => {
                let path = path.into_path().map_err(|e| e.to_string())?;
                build_payload(&path.to_string_lossy()).map(Some)
            }
            None => Ok(None),
        }
    }
}

/// 按路径读取文件（命令行参数 / 最近文件 / 拖拽 / 打开命令）
#[tauri::command]
pub fn read_markdown_file(path: String) -> Result<FilePayload, String> {
    build_payload(&path)
}

/// 启动时命令行参数传入的待打开文件（litemd.exe file.md）。
/// 用命令主动拉取而非事件推送：setup 阶段 emit 会早于前端监听导致事件丢失。
#[tauri::command]
pub fn get_startup_file() -> Option<String> {
    std::env::args()
        .nth(1)
        .filter(|arg| std::path::Path::new(arg).is_file())
}

/// 打开文件所在目录并选中该文件（explorer /select，仅 Windows）
#[cfg(windows)]
#[tauri::command]
pub fn reveal_in_explorer(path: String) -> Result<(), String> {
    // 必须用 ShellExecuteW 把参数原样传给 explorer：std::process::Command 会对含空格的
    // 参数二次加引号并转义，explorer 解析错误会跳到默认目录（如 D:\Documents）。
    // 实测原样传入 /select,"路径" 才能正确打开目标文件夹。
    let params = to_wide(&format!(r#"/select,"{}""#, path));
    let explorer = to_wide("explorer.exe");
    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(), // hwnd：无父窗口
            std::ptr::null(),     // operation：使用默认 open
            explorer.as_ptr(),
            params.as_ptr(),
            std::ptr::null(), // 工作目录：继承当前
            1,                // SW_SHOWNORMAL
        )
    };
    // ShellExecute 返回值 ≤32 表示失败
    if (result as isize) > 32 {
        Ok(())
    } else {
        Err(format!("打开文件夹失败，错误码：{}", result as isize))
    }
}

/// 转 UTF-16 空终止字符串，供 ShellExecuteW 使用（仅 Windows）
#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// ---------- 安卓专属：分享接收 / 文件夹浏览 / Wi-Fi 传书 ----------

/// 【安卓】前端启动与定时轮询：取外部「打开/分享」导入的文件
#[cfg(target_os = "android")]
#[tauri::command]
pub fn pull_incoming_file(app: tauri::AppHandle) -> Result<Option<FilePayload>, String> {
    let v = crate::file_opener::call(&app, "takeIncoming", serde_json::json!({}))?;
    let path = v.get("path").and_then(|x| x.as_str()).unwrap_or("");
    if path.is_empty() {
        return Ok(None);
    }
    build_payload(path).map(Some)
}

/// 【安卓】选择浏览文件夹；返回 { granted, list, cancelled } 原样透传给前端判断
#[cfg(target_os = "android")]
#[tauri::command]
pub fn choose_browse_folder(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    crate::file_opener::call(&app, "pickFolder", serde_json::json!({}))
}

/// 【安卓】列出已授权文件夹中的 md 文件，返回 { granted, list }
#[cfg(target_os = "android")]
#[tauri::command]
pub fn list_browse_files(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    crate::file_opener::call(&app, "listFolder", serde_json::json!({}))
}

/// 【安卓】打开文件夹中的 md（同目录图片一并拷贝）并读取
#[cfg(target_os = "android")]
#[tauri::command(rename_all = "camelCase")]
pub fn open_browse_file(app: tauri::AppHandle, doc_uri: String) -> Result<FilePayload, String> {
    let v = crate::file_opener::call(
        &app,
        "openFolderFile",
        serde_json::json!({ "docUri": doc_uri }),
    )?;
    let path = v
        .get("path")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "未获取到文件路径".to_string())?;
    build_payload(path)
}

/// 【安卓】Wi-Fi 传书用的局域网 IPv4 地址
#[cfg(target_os = "android")]
#[tauri::command]
pub fn wifi_lan_address(app: tauri::AppHandle) -> Result<String, String> {
    let v = crate::file_opener::call(&app, "wifiIp", serde_json::json!({}))?;
    Ok(v.get("ip").and_then(|x| x.as_str()).unwrap_or("").to_string())
}

/// 【安卓】启动 Wi-Fi 接收服务，返回 { port, token }
#[cfg(target_os = "android")]
#[tauri::command]
pub fn wifi_server_start(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    crate::wifi_server::start(app)
}

/// 【安卓】停止 Wi-Fi 接收服务
#[cfg(target_os = "android")]
#[tauri::command]
pub fn wifi_server_stop() {
    crate::wifi_server::stop()
}
