# LiteMD Android 版 — 设计文档

> 版本：v1.0
> 日期：2026-08-25
> 定位：桌面版 LiteMD（Tauri 2 + Vue 3 的轻量 Markdown 阅读器）的安卓版本
> 目标设备：vivo X200 Pro mini（Android 15 / OriginOS 5）
> 核心卖点延续：**记住阅读进度 + 目录大纲 + 中文界面 + 极致轻量**

---

## 一、背景与目标

### 1.1 背景

桌面版 LiteMD 已实现：Markdown 阅读、像素级进度记忆、目录大纲、最近文件、明暗主题、字体缩放、文内搜索、中文界面。核心逻辑（渲染、进度、大纲、i18n）全部在 Vue 前端，可整体复用。

本次目标：基于同一代码库，开发适配安卓（vivo X200 Pro mini）的移动端版本，复用现有前端能力，改造后端与交互以适配移动端。

### 1.2 设计原则

1. **最大化复用**：前端渲染/进度/大纲/主题/搜索逻辑零重写，只改 IPC 层与 UI 壳
2. **文件可靠**：三种打开方式都保证进度可记、文件可读（临时授权先复制进私有目录）
3. **移动端优先**：竖屏为主、横屏自适应；触控交互；小屏布局
4. **兼容优先**：针对 vivo / OriginOS / Android 15 已知坑逐一应对

---

## 二、技术方案

**选型：Tauri 2 移动端**（复用现有 Vue 前端 + Rust 后端适配安卓）

| 项 | 内容 |
|----|------|
| 前端 | 现有 Vue 3 + Vite（复用），仅调整 App/TopBar/Sidebar/StatusBar 布局与 IPC 调用 |
| 后端 | 现有 Rust（复用 storage/progress/recent/preferences/hash/path），改造 file.rs，隔离桌面专属命令 |
| 打包 | Tauri 2 Android target → APK（aarch64） |
| WebView | 系统 Android System WebView |

**构建环境（实施第一步，需安装）**：
- JDK 17+（本机现为 JDK 8）
  - **安装约束（涛哥明确要求）：免安装 zip 版（Temurin 17）解压到独立目录（如 `D:\Software\JDK\jdk_17`），不写注册表、不改任何系统环境变量**
  - 构建时在 `android/gradle.properties` 写 `org.gradle.java.home=<JDK17 路径>`，gradle 直接用该路径；**不设置 JAVA_HOME、不设置任何环境变量**，现有 JDK 8 的 JAVA_HOME / PATH 原封不动
- Android SDK + NDK（`%LOCALAPPDATA%\Android\Sdk`，未安装，需下载约 1-2GB）
- Rust android target：`aarch64-linux-android`（rustup target add）

---

## 三、功能范围

### 3.1 核心功能（本次实现，与桌面版对齐）

| 功能 | 说明 |
|------|------|
| Markdown 渲染 | GFM：表格、任务列表、删除线、代码块高亮、[TOC] |
| 阅读进度记忆 | 像素级恢复 + 哈希校验 + 防抖落盘 + 切后台强写 |
| 目录大纲 | 树形目录、点击跳转、当前章节高亮+自动展开 |
| 最近文件 | 记录最近打开，标星置顶 |
| 明暗主题 | 浅色/深色/跟随系统 |
| 字体缩放 | 阅读中缩放字号 |
| 文内搜索 | 高亮匹配、上下跳转 |
| 中文界面 | 复用现有 i18n，默认中文 |

### 3.2 文件来源（三种，均实现）

1. **系统文件选择器**（SAF 文档选择器，原地读 + 持久授权）
2. **注册 md 打开方式 + 分享接入**（文件管理器点开 / 微信/QQ/网盘分享，临时授权 → 复制进私有目录）
3. **内置文件浏览器**（SAF 目录树授权，浏览选中 md）

### 3.3 不做（延续桌面版定位）

编辑、云同步、多端进度共享（本地独立存储）、PDF 导出、Mermaid/KaTeX（桌面版也未启用）。

---

## 四、总体架构

### 4.1 复用与改造清单

**复用（不动）**：
- 前端：Reader.vue、Outline.vue、RecentFiles.vue、SearchBar.vue、SettingsPanel.vue、useMarkdown.js、useProgress.js、useTOC.js、useTheme.js、useFont.js、stores/app.js、i18n、样式（variables/global/reader/highlight）
- Rust：storage/{progress,recent,preferences}.rs、utils/{hash,path}.rs、命令 progress.rs/recent.rs/preferences.rs

**改造**：
- Rust `commands/file.rs`：桌面版（对话框/命令行/explorer 定位）→ 安卓版（复制导入 / 读取私有文件）
- Rust `lib.rs`：移除 single-instance、窗口状态恢复/保存（桌面专属，cfg 隔离）
- **新增自定义 Tauri Android 插件（Kotlin）**：目录树选择（ACTION_OPEN_DOCUMENT_TREE）、目录列举（DocumentFile）、分享/打开接收（onNewIntent 处理 ACTION_VIEW / ACTION_SEND）、ContentResolver 取文件名、复制到私有目录
- 前端 App.vue：移除拖拽/全屏窗口逻辑，改移动端布局与手势
- 前端 TopBar.vue / Sidebar.vue / StatusBar.vue：改移动端 UI 壳
- 前端 useFiles.js：IPC 调用适配安卓命令

### 4.2 项目结构

```
LiteMD/
├── src-tauri/
│   ├── android/              # Tauri android 工程（gradle，生成）
│   ├── src/
│   │   ├── commands/
│   │   │   ├── file.rs       # 改造：安卓 SAF/Intent/树浏览
│   │   │   ├── progress.rs   # 复用
│   │   │   ├── recent.rs     # 复用
│   │   │   ├── preferences.rs# 复用
│   │   │   └── window.rs     # 桌面专用（#[cfg(desktop)] 隔离）
│   │   ├── storage/          # 复用（data_dir 按平台解析）
│   │   └── utils/            # 复用
│   └── tauri.conf.json       # 平台相关配置
├── src/                      # Vue 前端（复用 + 移动适配壳）
│   ├── components/           # Reader 等复用
│   ├── mobile/               # 移动端专用壳组件（新增）
│   └── ...
└── package.json
```

### 4.3 桌面与安卓共存策略

- 同一 `src-tauri`，Windows 构建走 NSIS，Android 构建走 gradle
- 桌面专属代码用 `#[cfg(target_os = "windows")]` 或 `desktop` feature 隔离，避免编译冲突
- 前端用环境判断（Tauri mobile API 或 `isAndroid`）切换交互模式

---

## 五、文件与进度数据设计

### 5.1 三种打开方式落地

> 关键前提：Android 的 `content://` 授权**进程重启即失效**（[tauri#14587](https://github.com/tauri-apps/tauri/issues/14587)），原地读不可靠。**统一策略：所有打开的文件先复制进 App 私有目录（`imported/`），再从私有路径读取**。

| 方式 | 系统入口 | 处理 |
|------|----------|------|
| 系统文件选择器 | tauri-plugin-dialog `pick_file`（安卓已支持 SAF，返回 content uri） | 复制进私有目录 |
| 注册打开 / 分享 | 自定义 Kotlin 插件：`onNewIntent` 处理 `ACTION_VIEW`（文件管理器点开 md）+ `ACTION_SEND`（分享） | 复制进私有目录 |
| 内置文件浏览器 | 自定义 Kotlin 插件：`ACTION_OPEN_DOCUMENT_TREE` 选目录 + DocumentFile 列出 md | 打开时**连同兄弟图片**复制进私有目录 |

> 目录选择 / 目录列举官方 dialog、fs 插件均未实现（[#933](https://github.com/tauri-apps/plugins-workspace/issues/933)、[#14587](https://github.com/tauri-apps/tauri/issues/14587)），需自定义 Tauri Android 插件（Kotlin）。

### 5.2 进度记忆 key 规则

- **统一规则：所有文件复制到私有目录（`imported/`），进度 key 用私有目录路径**（稳定、跨重启可靠）
- 复制时按文件名去重：同名文件覆盖或追加序号，避免 key 冲突
- 进度存储机制（防抖 500ms → 3 秒落盘 → 关闭/切后台强写 → 哈希校验）**完全复用**，落盘目录用 Tauri app_data_dir（跨平台统一）
- **切后台强写**：移动端新增 onPause/onStop 时 flush，防 OriginOS 杀进程丢进度

### 5.3 图片处理策略

- 桌面版 `文件目录 + 相对路径` 加载同目录图片，SAF 下无路径概念
- **策略**：
  - **内置文件浏览器**（有目录授权上下文）→ 打开 md 时**连带复制同目录图片**进私有目录，本地图片完整支持 ✅
  - **SAF 单文件 / 分享** → 无目录上下文，仅显示远程/外链图片（`http(s)`/`data:`），本地相对图片跳过 ⚠️
- 图片复制去重：按文件名跳过已存在的
- 该限制写入验收说明，作为已知边界

### 5.4 存储位置

- 进度/最近/偏好 JSON → Android app 私有存储目录（`app_data_dir`）
- 复制导入的 md + 图片 → app 私有文件目录（files 子目录，如 `imported/`）

---

## 六、移动端界面适配

### 6.1 布局（竖屏为主）

- **阅读区**：正文全屏沉浸，宽度自适应，触控滚动；安全区 padding（状态栏/挖孔/手势条）
- **顶栏**：一行——☰ 菜单 + 文件名 + 🔍 搜索 + ⚙️ 设置
- **底部浮动工具条**：📂 最近 / 📋 大纲 / ⚙️ 设置；点开弹半屏抽屉面板（取代桌面左侧栏）
- **进度条**：悬浮正文底部，显示百分比，可点击跳转
- **图片放大**：保留点击放大（Lightbox）
- **横屏**：布局自适应，抽屉不遮挡正文；正文更宽

### 6.2 手势

- 点击大纲章节 → 跳转正文（保留）
- 图片点击 → 放大（保留）
- 左右滑动 → 切换最近文件（可选，P2）
- 下拉/返回键 → 关闭抽屉面板

### 6.3 文件打开流程

- 空状态/菜单「打开文件」→ 系统文件选择器
- 文件管理器点开 md → Intent → LiteMD 直接进入阅读
- 分享 md → LiteMD 接收 → 复制后进入阅读
- 内置浏览器入口 → 目录树授权 → 列表选 md

---

## 七、兼容性应对（vivo X200 Pro mini / Android 15 / OriginOS）

| 可能的问题 | 应对方案 |
|-----------|----------|
| OriginOS 后台杀进程 → 进度丢 | 滚动防抖落盘（复用）+ onPause/onStop **立即强写** |
| Android 15 edge-to-edge 强制 | 状态栏/导航栏透明处理，正文安全区 padding |
| 系统深色模式 | 跟随系统，明暗主题都测试 |
| 厂商 WebView 差异 | 只用 markdown-it / highlight.js 标准 API，不依赖新浏览器特性 |
| 分屏 / 小窗 | onPause / onResume 保存与恢复进度 |
| 首次用 LiteMD 打开 .md | Manifest Intent filter + 授权引导流程 |
| 从微信/QQ/网盘分享打开 | 临时授权处理，复制进私有目录保证稳定 |
| 系统字体/显示缩放 | 正文与 UI 用相对单位，自适应 |
| Android 权限拒绝 | SAF 选择器本身无需存储权限；目录树授权需引导 |

---

## 八、交付与验收

### 8.1 交付物

- Android APK（aarch64，签名 debug 版先行，后续可上正式签名）
- 支持两种安装：USB adb 安装 / 传文件到手机手动安装（允许未知来源）

### 8.2 验收清单

| 编号 | 验收项 | 标准 |
|------|--------|------|
| A1 | 系统选择器打开 md | 选文件后正常渲染 |
| A2 | 文件管理器点开 md | Intent 打开 LiteMD 并渲染 |
| A3 | 分享 md（微信/QQ/网盘） | 接收并渲染，进度可记 |
| A4 | 内置文件浏览器 | 目录树授权后可浏览并打开 |
| A5 | 进度记忆 | 杀进程/切后台/重开，恢复到上次位置 |
| A6 | 目录大纲 | 树形目录、点击跳转、当前章节高亮 |
| A7 | 最近文件 | 记录、标星、打开 |
| A8 | 主题 | 浅色/深色/跟随系统 |
| A9 | 搜索 | 高亮匹配、上下跳转 |
| A10 | 横竖屏 | 旋转布局正常，进度不丢 |
| A11 | 大文件 | 10 万字打开 < 2 秒，滚动流畅（复用分片渲染） |

### 8.3 已知边界（MVP）

- SAF 原地读文件的**同目录图片**不显示（仅复制方式支持）
- 同一文件多入口打开进度分笔记录

---

## 九、实施计划（拆分为 4 个独立 Plan，每个都可独立交付）

**Plan 1 — 构建链路打通**：环境（JDK17 免安装 + Android SDK/NDK + Rust target）→ `tauri android init` → 配置（gradle.properties 指定 JDK17、包名 com.litemd.android、图标）→ 首次 `tauri android build` 出 APK → 安装到 vivo 启动成功
**Plan 2 — 文件访问能力**：自定义 Kotlin 插件（目录树选择/列举、分享接收、复制导入）→ Rust file.rs 适配 + storage data_dir 统一 → 三种打开方式 + 进度记忆可用
**Plan 3 — 移动端 UI**：App/TopBar/Sidebar/StatusBar 移动端壳、底部浮动工具条、抽屉面板、安全区、横竖屏
**Plan 4 — 真机验收与兼容**：vivo X200 Pro mini 逐项验收清单（11 项）+ 兼容性应对（深色/分屏/杀进程/WebView）+ 已知边界确认

---

## 十、风险与开放问题

| 风险 | 等级 | 应对 |
|------|------|------|
| 首次安卓构建环境问题（网络/依赖） | 中 | 分步安装，逐步验证，卡点询问 |
| Tauri 移动端 API 与桌面差异（图片协议、WebView 行为） | 中 | 提前验证 asset protocol 安卓行为；图片按 5.3 策略降级 |
| OriginOS 后台限制导致进度丢失 | 中 | 强写机制 + 真机验证 |
| 厂商 WebView 渲染差异 | 低 | 标准 API 渲染，真机验证 |
