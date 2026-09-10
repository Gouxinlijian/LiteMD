# LiteMD — 轻量级 Markdown 阅读器

只做阅读的轻量 Markdown 阅读器。核心卖点：**记住阅读进度** + **目录大纲** + **书签** + **中文界面** + **极致轻量**。

同一套 Vue 前端 + Tauri 2 Rust 后端，同时构建 Windows 桌面版与 Android 手机版，按平台切换布局与交互。

## 平台支持

| 平台 | 形态 | 交付产物 |
|------|------|----------|
| Windows 10/11 x64 | 桌面应用（WebView2） | `release/LiteMD-win-x64-setup.exe`（NSIS 安装包） |
| Android（arm64） | 手机 App | `release/LiteMD-android-arm64.apk` |

## 技术栈

| 层 | 技术 |
|----|------|
| 应用框架 | Tauri 2（Rust 后端 + 系统 WebView） |
| 前端 | Vue 3 + Vite + Pinia |
| Markdown | markdown-it（GFM + 任务列表；脚注插件与 markdown-it 15 不兼容，未使用） |
| 代码高亮 | highlight.js |
| 二维码 | qrcode（安卓「电脑传书」用，按需动态加载，不进主包） |

## 功能

### 两端通用

- **阅读进度记忆**：每个文件独立记录滚动位置，关闭再打开精确恢复；文件被外部修改后按百分比对齐
- **目录大纲**：自动从 h1-h6 生成树形目录，当前章节自动高亮 + 自动展开，点击平滑跳转
- **书签**：在大纲面板切到「书签」页签，按当前阅读位置（优先章节锚点，否则百分比）加书签，点击跳回
- **最近文件**：记录最近 20 个文件，支持**标星置顶**（★ 优先阅读）、**移除**、hover 快捷操作，一键打开
- **启动恢复**：打开应用自动回到上次阅读的文件和位置
- **明暗主题**：浅色 / 深色 / 跟随系统
- **文内搜索**：高亮匹配 + 上下跳转
- **中文界面**：默认中文，可切换英文
- **大文件优化**：5 万字以上分片渲染，打开不冻结，显示进度

### 桌面端专属

- **单实例**：已在运行时双击 .md 文件复用现有窗口打开
- **右键菜单**：正文复制/全选/搜索选中/跳顶跳底/字号调整/图片放大，最近项打开/标星/删除，大纲跳转/复制标题
- **打开文件**：文件对话框 / 拖拽 / 命令行参数 / 最近文件
- **阅读设置**：字号、字体族（雅黑 / 宋体 / 黑体 / 楷体 / 等宽）、行距、内容宽度（可铺满窗口）、代码高亮开关
- **全屏阅读**：F11 隐藏全部 UI，鼠标移顶缘唤出迷你工具栏
- **文档内目录**：正文写 `[TOC]` 自动插入目录树，锚点平滑跳转
- **打开对话框**：记住上次打开目录，自动定位
- **窗口状态记忆**：大小 / 位置 / 最大化状态自动恢复

### 安卓端专属

- **移动布局**：顶部栏（打开 / 搜索 / 主题 / 设置）+ 正文 + 底部工具条（最近 / 文件 / 大纲）
- **沉浸阅读**：点击正文空白隐藏上下栏，只看正文（滑动不会误触）
- **打开与分享导入**：文件管理器里打开 .md，或从其他应用分享 .md 到 LiteMD
- **文件浏览**：SAF 授权一个文件夹后，直接列出其中的 .md 打开（同目录图片一并拷贝，保证正文图片可显示）
- **电脑传书**：手机开启局域网 HTTP 服务并显示二维码，电脑浏览器扫码或输入网址即可上传 .md，上传完自动打开

## 快捷键（桌面端）

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+O` | 打开文件 |
| `Ctrl+F` | 文内搜索 |
| `Ctrl+J` | 切换侧边栏 |
| `Ctrl+K` / `Ctrl+R` | 大纲 / 最近 Tab |
| `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | 放大 / 缩小 / 重置字号 |
| `Ctrl+滚轮` | 缩放字号 |
| `Ctrl+,` | 打开设置 |
| `Ctrl+Tab` | 循环切换最近文件 |
| `F11` | 全屏切换 |
| `Esc` | 退出全屏 / 关闭搜索 |
| `Space` / `Shift+Space` | 向下 / 向上翻页 |

## 环境准备（新机器克隆后）

仓库只含源码与构建脚本；`node_modules/`、`target/`、`release/`、以及带本机路径的配置文件都不入库，需在本机生成。克隆后按下面补齐。

### 桌面端（Windows）

| 依赖 | 说明 |
|------|------|
| Node.js 18+ | 前端依赖与构建 |
| Rust 工具链 | 用 rustup 安装，默认 `x86_64-pc-windows-msvc` 目标 |
| MSVC C++ 生成工具 | Visual Studio 2022「使用 C++ 的桌面开发」工作负载（rustup 安装时会提示缺失） |
| WebView2 运行时 | Win11 自带；较老的 Win10 需单独安装（安装包也会自动下载） |

```bash
npm ci                # 按 package-lock.json 精确还原依赖
npm run tauri dev     # 验证能跑起来
npm run build:win     # 验证能打包 → release/LiteMD-win-x64-setup.exe
```

首次 `npm run build:win` 需全量编译 Rust 依赖，10 分钟以上属正常。

> 用 `npm ci` 而非 `npm install`：仓库已锁定 `package-lock.json`，`ci` 严格按锁定版本还原，避免新机器装到更新的工具链导致构建行为不一致。
> 若终端提示 `cargo` 找不到命令，把 `%USERPROFILE%\.cargo\bin` 加入 PATH（重开终端一般即可生效）。

### 安卓端（可选，只做桌面可跳过）

Tauri 生成的安卓工程已入库（`src-tauri/gen/android/`，含手写的 `FileOpenerPlugin.kt` 等），但下列本机环境不在仓库内，需在新机器补齐：

| 依赖 | 做法 |
|------|------|
| JDK 17 | 安装后，在**用户级** `%USERPROFILE%\.gradle\gradle.properties` 写一行 `org.gradle.java.home=<JDK17 路径>`（用正斜杠）。仓库内不写死该路径，以免换机器构建失败 |
| Android SDK | 需包含 platform-tools、`platforms;android-36`、build-tools、NDK 26.x |
| SDK 路径 | 在 `src-tauri/gen/android/local.properties` 写 `sdk.dir=<SDK 路径>`（该文件含本机路径，按惯例不入库，需手动创建）；或改设 `ANDROID_HOME` 环境变量 |
| Rust 安卓目标 | `rustup target add aarch64-linux-android` |
| Windows 开发者模式 | 需要开启：Tauri 会把编译出的 .so 以符号链接方式放进 `jniLibs` |

```bash
npm run tauri android build -- --apk --split-per-abi
```

必须走 `npm run tauri android build`（Tauri CLI 每次会重新生成 `tauri.settings.gradle` 等带本机路径的文件），不要直接用 `gradlew` 构建。

## 开发

```bash
npm run tauri dev     # 桌面端开发运行
npm run dev           # 仅跑前端（浏览器里调 UI，不含 Rust 能力）
```

## 打包与交付

**交付约定**：所有安装包统一放项目根目录 `release/`，每次生成新包替换旧包，不保留历史版本。

| 平台 | 命令 / 步骤 | 输出 |
|------|-------------|------|
| Windows | `npm run build:win` | `release/LiteMD-win-x64-setup.exe` |
| Android | 见下方手动步骤 | `release/LiteMD-android-arm64.apk` |

Windows 打包后安装包本会落在 `src-tauri/target/release/bundle/nsis/`，Tauri 没有「输出目录」配置项，因此由 `scripts/collect_release.mjs` 在构建结束后把最新安装包搬到 `release/` 并重命名、清掉同名旧包。单独跑 `npm run tauri build` 不会搬运，需再手工执行 `node scripts/collect_release.mjs`。

Android 打包（只交付 arm64，universal 包太大不交付）：

```bash
npm run tauri android build -- --apk --split-per-abi
# 产物：src-tauri/gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-unsigned.apk
# 用 apksigner 签名（keystore 见下）后输出为 release/LiteMD-android-arm64.apk
```

安卓构建环境为隔离安装（JDK 17 免安装版 + Android SDK 走 `local.properties`），不改系统环境变量；细节见 `docs/superpowers/plans/` 下的构建计划。

安装包行为：首次安装可选安装位置；再次安装自动沿用上次位置覆盖升级，阅读进度等记录保留。

## 项目结构

```
LiteMD/
├─ src/                        Vue 3 前端（桌面 / 安卓共用）
│  ├─ components/
│  │  ├─ mobile/               移动端布局：底部工具条、文件浏览、电脑传书
│  │  ├─ sidebar/              侧边栏：大纲 / 书签 / 最近文件
│  │  └─ …                     Reader、SettingsPanel、TopBar 等桌面组件
│  ├─ composables/             阅读进度、搜索、字体、Markdown 渲染等逻辑
│  ├─ stores/                  Pinia 应用状态
│  ├─ locales/                 zh-CN / en-US 文案
│  └─ utils/                   i18n、平台判断、格式化
├─ src-tauri/
│  ├─ src/                     Rust 后端：commands / storage / wifi_server
│  ├─ capabilities/            Tauri 权限声明
│  ├─ gen/android/             安卓工程（含自定义 FileOpenerPlugin，勿随意删除/重建）
│  └─ tauri.conf.json          窗口、打包、NSIS 配置
├─ scripts/collect_release.mjs 打包产物收集到 release/
├─ release/                    交付目录：安装包 / APK
└─ docs/                       设计文档、demo、历史构建计划与规格
```

## 数据存储

桌面端保存在 `%APPDATA%\LiteMD\`，安卓端保存在应用私有数据目录，文件名一致：

- `preferences.json` — 用户设置
- `reading_progress.json` — 阅读进度
- `recent_files.json` — 最近文件
- `window_state.json` — 窗口状态（仅桌面端）
- 书签按文件存储（`storage/bookmarks.rs`）

## 文档

- `LiteMD-设计文档.md` — 整体设计文档
- `docs/superpowers/specs/`、`docs/superpowers/plans/` — 安卓版设计与构建计划
