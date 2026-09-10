//! Wi-Fi 局域网传书（仅 Android）：手机起一个极简 HTTP 服务，
//! 电脑浏览器打开「IP:端口」页面拖拽上传 .md/.markdown 文件，
//! 落盘到私有导入目录后通过事件通知前端自动打开。仅用标准库实现，零新增依赖。

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};

const MAX_BODY_BYTES: usize = 200 * 1024 * 1024;
const BASE_PORT: u16 = 8766;

static STOP: AtomicBool = AtomicBool::new(false);
static TOKEN: OnceLock<String> = OnceLock::new();
static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// 运行中的服务（端口 + 接收线程），stop 时置空
struct Running {
    port: u16,
    /// 保留句柄便于后续排查；暂不 join，stop 通过 STOP 标志结束循环
    #[allow(dead_code)]
    handle: Option<JoinHandle<()>>,
}
static RUNNING: Mutex<Option<Running>> = Mutex::new(None);

/// 启动服务（已运行则直接返回现有信息）。返回 { port, token }
pub fn start(app: AppHandle) -> Result<serde_json::Value, String> {
    {
        let mut guard = RUNNING.lock().map_err(|e| e.to_string())?;
        if let Some(r) = guard.as_ref() {
            return Ok(serde_json::json!({ "port": r.port, "token": token() }));
        }
        STOP.store(false, Ordering::SeqCst);

        // 尝试绑定若干端口
        let mut listener = None;
        let mut bound_port = 0;
        for p in BASE_PORT..BASE_PORT + 10 {
            match TcpListener::bind(("0.0.0.0", p)) {
                Ok(l) => {
                    listener = Some(l);
                    bound_port = p;
                    break;
                }
                Err(_) => continue,
            }
        }
        let listener = listener.ok_or_else(|| "无法绑定局域网端口".to_string())?;

        let app_handle = Arc::new(app);
        let hnd = std::thread::spawn(move || accept_loop(listener, app_handle));

        *guard = Some(Running {
            port: bound_port,
            handle: Some(hnd),
        });
        return Ok(serde_json::json!({ "port": bound_port, "token": token() }));
    }
}

/// 停止服务（接收屏关闭时调用）
pub fn stop() {
    STOP.store(true, Ordering::SeqCst);
}

fn token() -> String {
    TOKEN
        .get_or_init(|| format!("{:06x}", rand_u32() & 0xffffff))
        .clone()
}

/// 简单随机数来源（自用软件无需密码学强度）
fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    nanos.rotate_left(13) ^ std::process::id()
}

fn accept_loop(listener: TcpListener, app: Arc<AppHandle>) {
    loop {
        if STOP.load(Ordering::SeqCst) {
            break;
        }
        match listener.accept() {
            Ok((stream, _addr)) => {
                let app2 = app.clone();
                std::thread::spawn(move || {
                    let _ = handle_conn(stream, app2);
                });
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(200)),
        }
    }
    // 退出前清空状态，允许重新启动
    if let Ok(mut g) = RUNNING.lock() {
        *g = None;
    }
}

/// 处理单个连接：GET 返回上传页；POST /upload 收文件
fn handle_conn(mut stream: TcpStream, app: Arc<AppHandle>) -> Result<(), String> {
    // 读请求头（上限 16KB）
    let mut head = Vec::with_capacity(1024);
    let mut byte = [0u8; 1];
    while head.len() < 16 * 1024 {
        match stream.read(&mut byte) {
            Ok(0) => break,
            Ok(_) => {
                head.push(byte[0]);
                if head.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    let head_str = String::from_utf8_lossy(&head).to_string();
    let mut lines = head_str.lines();
    let req_line = lines.next().unwrap_or("").to_string();
    let mut parts = req_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("").to_string();

    let mut headers: HashMap<String, usize> = HashMap::new();
    for l in lines {
        if let Some((k, v)) = l.split_once(':') {
            if k.eq_ignore_ascii_case("content-length") {
                headers.insert("content-length".to_string(), v.trim().parse().unwrap_or(0));
            }
        }
    }

    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.clone(), String::new()),
    };
    let qs = parse_query(&query);

    // 鉴权：所有请求必须带正确 token
    if qs.get("k").map(|v| v.as_str()) != Some(token().as_str()) {
        return write_resp(&mut stream, 403, "text/plain", b"Forbidden");
    }

    match method.as_str() {
        "GET" => {
            let html = PAGE_HTML.replace("__TOKEN__", &token());
            write_resp(&mut stream, 200, "text/html; charset=utf-8", html.as_bytes())
        }
        "POST" if path == "/upload" => {
            let len = headers.get("content-length").copied().unwrap_or(0);
            if len > MAX_BODY_BYTES {
                let body = serde_json::json!({ "ok": false, "error": "文件过大" }).to_string();
                return write_resp(&mut stream, 413, "application/json", body.as_bytes());
            }
            save_upload(stream, app, len, qs.get("name").cloned().unwrap_or_default())
        }
        _ => write_resp(&mut stream, 404, "text/plain", b"Not Found"),
    }
}

/// 流式接收 body 写入私有目录，成功后向前端发 wifi-received 事件
fn save_upload(
    mut stream: TcpStream,
    app: Arc<AppHandle>,
    len: usize,
    raw_name: String,
) -> Result<(), String> {
    let name = sanitize_name(&percent_decode(&raw_name));
    let lower = name.to_lowercase();
    if !(lower.ends_with(".md") || lower.ends_with(".markdown")) {
        let body = serde_json::json!({ "ok": false, "error": "仅支持 .md / .markdown" }).to_string();
        return write_resp(&mut stream, 400, "application/json", body.as_bytes());
    }

    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("imported");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = unique_file(&dir, &name);

    // 写入加锁，避免同名并发判断竞态
    let saved_path = {
        let _g = WRITE_LOCK.lock().map_err(|e| e.to_string())?;
        let mut file = File::create(&dest).map_err(|e| e.to_string())?;
        let mut remaining = len;
        let mut buf = [0u8; 64 * 1024];
        while remaining > 0 {
            let want = remaining.min(buf.len());
            match stream.read(&mut buf[..want]) {
                Ok(0) => break,
                Ok(n) => {
                    file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
                    remaining -= n;
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        let recv_len = len - remaining;
        drop(file);
        if remaining > 0 {
            let _ = std::fs::remove_file(&dest);
            let body = serde_json::json!({ "ok": false, "error": "传输不完整" }).to_string();
            return write_resp(&mut stream, 400, "application/json", body.as_bytes());
        }
        recv_len
    };

    let final_name = dest.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or(name);
    let out = serde_json::json!({
        "ok": true,
        "path": dest.to_string_lossy(),
        "name": final_name,
        "size": saved_path,
    });

    // 通知前端打开
    let _ = app.emit(
        "wifi-received",
        serde_json::json!({ "path": dest.to_string_lossy(), "name": final_name }),
    );

    write_resp(&mut stream, 200, "application/json", out.to_string().as_bytes())
}

fn write_resp(stream: &mut TcpStream, code: u16, ctype: &str, body: &[u8]) -> Result<(), String> {
    let reason = match code {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        413 => "Payload Too Large",
        _ => "Error",
    };
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        code,
        reason,
        ctype,
        body.len()
    );
    stream.write_all(head.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(body).map_err(|e| e.to_string())?;
    stream.flush().ok();
    Ok(())
}

/// 解析 query 为键值表（值为百分号解码后的字符串）
fn parse_query(q: &str) -> HashMap<String, String> {
    q.split('&')
        .filter_map(|kv| kv.split_once('='))
        .map(|(k, v)| (k.to_string(), percent_decode(v)))
        .collect()
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                // 十六进制两位解码
                let hi = hex_val(Some(bytes[i + 1]));
                let lo = hex_val(Some(bytes[i + 2]));
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push(h * 16 + l);
                    i += 3;
                    continue;
                } else {
                    out.push(b'%');
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: Option<u8>) -> Option<u8> {
    match b? {
        c @ b'0'..=b'9' => Some(c - b'0'),
        c @ b'a'..=b'f' => Some(c - b'a' + 10),
        c @ b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// 文件名安全化：去路径分隔符与父目录引用，保留中文名
fn sanitize_name(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect::<String>()
        .replace("..", "_")
        .trim()
        .to_string()
}

/// 同名文件追加序号
fn unique_file(dir: &PathBuf, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let dot = name.rfind('.');
    let (base, ext) = match dot {
        Some(d) => (&name[..d], &name[d..]),
        None => (name, ""),
    };
    for i in 1..u32::MAX {
        let p = dir.join(format!("{}_{}{}", base, i, ext));
        if !p.exists() {
            return p;
        }
    }
    dir.join(format!("{}.md", rand_u32()))
}

/// 电脑端上传页（极简、深色自适应）
const PAGE_HTML: &str = r#"<!doctype html><html lang="zh-CN"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>LiteMD · 传文档到手机</title>
<style>
body{font-family:system-ui,-apple-system,'Segoe UI','Microsoft YaHei',sans-serif;margin:0;min-height:100vh;display:flex;flex-direction:column;align-items:center;justify-content:center;background:#f7f8fa;color:#1a1a1a;transition:.2s}
@media(prefers-color-scheme:dark){body{background:#1e1e1e;color:#d4d4d4}}
h1{margin:0 0 4px;font-size:22px}.sub{color:#888;font-size:14px;margin-bottom:28px}
#drop{width:min(560px,86vw);padding:48px 20px;border:2px dashed #b9c6d8;border-radius:14px;text-align:center;cursor:pointer;font-size:15px;line-height:1.7;transition:.15s}
#drop:hover,#drop.over{border-color:#4a90d9;background:rgba(74,144,217,.08)}
#drop input{display:none}
#log{list-style:none;padding:0;width:min(560px,86vw);max-height:40vh;overflow:auto}
#log li{display:flex;justify-content:space-between;gap:12px;padding:9px 12px;border-radius:8px;background:rgba(127,127,127,.08);margin-top:8px;font-size:13px;word-break:break-all}
.ok{color:#18a058;font-weight:600}.err{color:#e04a4a;font-weight:600}
</style></head><body>
<h1>LiteMD</h1><div class="sub">把 .md 文档传送到手机</div>
<div id="drop">把 .md / .markdown 文件拖到这里<br>或点击选择（可多选）<input type="file" id="fi" multiple accept=".md,.markdown,text/markdown,text/x-markdown"></div>
<ul id="log"></ul>
<script>
const TOKEN="__TOKEN__";
const box=document.getElementById('drop'),fi=document.getElementById('fi'),log=document.getElementById('log');
box.addEventListener('click',()=>fi.click());
['dragover','dragenter'].forEach(ev=>box.addEventListener(ev,e=>{e.preventDefault();box.classList.add('over')}));
['dragleave','drop'].forEach(ev=>box.addEventListener(ev,e=>{e.preventDefault();box.classList.remove('over')}));
box.addEventListener('drop',e=>send([...e.dataTransfer.files]));
fi.addEventListener('change',()=>send([...fi.files]));
function add(n,cls,msg){const li=document.createElement('li');li.innerHTML=`<span>${n}</span><span class="${cls}">${msg}</span>`;log.prepend(li)}
async function send(files){for(const f of files){if(!/\.(md|markdown)$/i.test(f.name)){add(f.name,'err','跳过：不是 .md');continue}
 try{const r=await fetch(`/upload?k=${TOKEN}&name=${encodeURIComponent(f.name)}`,{method:'POST',body:f});
 const j=await r.json();j.ok?add(f.name,'ok','已发送 ✓ 手机上自动打开'):add(f.name,'err',j.error||'失败')}catch(e){add(f.name,'err',String(e))}}}
</script></body></html>"#;
