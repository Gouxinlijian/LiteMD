pub mod bookmarks;
pub mod preferences;
pub mod progress;
pub mod recent;
pub mod window;

use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

/// 数据目录（进程初始化时设置）：桌面 %APPDATA%\LiteMD；安卓 App 私有目录
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// 初始化数据目录（移动端在 setup 时调用，指向 App 私有可写目录）
pub fn init_data_dir(path: PathBuf) {
    let _ = DATA_DIR.set(path);
}

/// 数据目录
fn data_dir() -> PathBuf {
    DATA_DIR.get().cloned().unwrap_or_else(|| {
        let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        PathBuf::from(base).join("LiteMD")
    })
}

/// 读取 JSON 文件；文件不存在或解析失败时返回默认值（自用软件，配置损坏不阻塞启动）
fn load_json<T: Default + DeserializeOwned>(filename: &str) -> T {
    let path = data_dir().join(filename);
    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

/// 写入 JSON 文件（确保目录存在）
fn save_json<T: Serialize>(filename: &str, value: &T) -> Result<(), String> {
    let dir = data_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(dir.join(filename), content).map_err(|e| e.to_string())
}

/// 当前时间 ISO8601（含时区偏移），如 2026-08-14T18:02:01+08:00
pub fn now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}
