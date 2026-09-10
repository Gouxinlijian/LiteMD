//! Android 文件能力桥接：注册 Kotlin 的 FileOpenerPlugin 并提供通用调用入口。
//! 桌面端为空壳插件；所有移动文件操作（选择器/分享接收/文件夹浏览/Wi-Fi 辅助）
//! 都通过 run_mobile_plugin 调到 Kotlin。

use serde_json::Value;
use tauri::plugin::{Builder, PluginHandle};
use tauri::{AppHandle, Manager, Runtime};

/// 持有的 Android 插件句柄
pub struct FileOpener<R: Runtime>(pub PluginHandle<R>);

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    Builder::new("file-opener")
        .setup(|app, api| {
            // register_android_plugin 仅在 Android 编译可用；桌面端跳过
            #[cfg(target_os = "android")]
            {
                let handle = api.register_android_plugin("com.litemd.reader", "FileOpenerPlugin")?;
                app.manage(FileOpener(handle));
            }
            #[allow(unused_variables)]
            {
                let _ = app;
                let _ = api;
            }
            Ok(())
        })
        .build()
}

/// 调用 Kotlin 端 @Command 方法并取回 JSON 结果
#[cfg(target_os = "android")]
pub fn call(app: &AppHandle, method: &str, payload: Value) -> Result<Value, String> {
    let opener = app
        .try_state::<FileOpener<tauri::Wry>>()
        .ok_or_else(|| "安卓文件插件未初始化".to_string())?;
    opener
        .0
        .run_mobile_plugin::<Value>(method, payload)
        .map_err(|e| e.to_string())
}

/// 桌面占位：相关命令均带安卓 cfg，不会实际调用
#[cfg(not(target_os = "android"))]
pub fn call(_app: &AppHandle, _method: &str, _payload: Value) -> Result<Value, String> {
    Err("仅安卓支持此能力".to_string())
}
