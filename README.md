# LiteMD — 轻量级 Markdown 阅读器

一个只做「读」的 Markdown 阅读器。不做编辑、不联网、不塞编辑器生态，只把本地 `.md` 文件读得舒服：**记住每个文件读到哪**、**自动生成目录大纲**、**可以加书签**、**大文件不卡**。

同一套 Vue 前端 + Tauri 2（Rust）后端，同时构建 Windows 桌面版和 Android 手机版，按平台自动切换布局与交互。Windows 安装包约 1.4 MB，安卓安装包约 7 MB。

## 下载安装

安装包都发布在 GitHub Releases，点击即可直接下载：

| 平台 | 直接下载 | 适用说明 |
|------|----------|----------|
| **Windows 10/11 x64** | [LiteMD-win-x64-setup.exe](https://github.com/Gouxinlijian/LiteMD/releases/latest/download/LiteMD-win-x64-setup.exe) | NSIS 安装包，双击安装。首次安装可选安装位置，之后升级自动沿用 |
| **Android（arm64）** | [LiteMD-android-arm64.apk](https://github.com/Gouxinlijian/LiteMD/releases/latest/download/LiteMD-android-arm64.apk) | 适用于绝大多数近几年的安卓手机；安装时需允许「安装未知来源应用」 |

也可以打开 [Releases 页面](https://github.com/Gouxinlijian/LiteMD/releases) 手动挑选版本下载。

补充说明：

- **Windows** 依赖系统自带的 WebView2 运行时。Win11 均自带；较老的 Win10 若缺失，安装包会自动下载安装。
- **Android** 只提供 arm64 包（体积约为通用包的三分之一）。设备架构不确定时，先装，提示「应用未安装/不兼容」即说明不是 arm64。
- 覆盖安装**不会**清除阅读进度、书签和最近文件，可以放心升级。

## 功能亮点

### 两端通用

- **阅读进度记忆**：每个文件独立记录滚动位置，关掉再打开精确回到原处；文件被外部改过则按百分比对齐
- **目录大纲**：自动从 `h1`-`h6` 生成树形目录，当前章节自动高亮并展开，点击平滑跳转
- **书签**：按当前阅读位置（优先章节锚点，否则百分比）打点，随时跳回
- **最近文件**：记录最近 20 个文件，可**标星置顶**、移除、一键打开
- **启动即续读**：打开应用自动回到上次阅读的文件和位置
- **文内搜索**：全文高亮命中，上下翻跳转，超长文档也不卡
- **明暗主题**：浅色 / 深色 / 跟随系统
- **中文界面**：默认中文，可切换英文
- **大文件优化**：5 万字以上分片渲染，打开不冻结并显示进度

### 桌面端专属

- **单实例**：已在运行时双击别的 `.md` 文件，直接在现有窗口打开，不重复起进程
- **右键菜单**：正文支持复制 / 全选 / 搜索选中 / 跳顶跳底 / 字号调整 / 图片放大；最近文件支持标星、移除；大纲支持跳转、复制标题
- **多种打开方式**：文件对话框、拖拽文件、命令行参数、最近列表
- **阅读设置**：字号、字体族（雅黑 / 宋体 / 黑体 / 楷体 / 等宽）、行距、内容宽度（可铺满窗口）、代码高亮开关
- **全屏阅读**：`F11` 隐藏所有 UI，鼠标移到屏幕顶缘唤出迷你工具栏
- **文档内目录**：正文写 `[TOC]` 自动插入目录树
- **窗口状态记忆**：大小、位置、最大化状态自动恢复

### 安卓端专属

- **移动布局**：顶栏（打开 / 搜索 / 主题 / 设置）+ 正文 + 底部工具条（最近 / 文件 / 大纲）+ 底部抽屉面板
- **沉浸阅读**：点正文空白处隐藏上下栏，只看文字（滑动不会误触）
- **打开与分享导入**：可从文件管理器直接打开 `.md`，也可从其他应用「分享」到 LiteMD
- **文件浏览**：授权一个文件夹后，直接列出其中的 `.md` 打开；同目录图片会一并拷贝，正文图片正常显示
- **电脑传书**：手机开启局域网服务并显示二维码 / 网址，电脑浏览器打开后把 `.md` 拖进去，手机上自动打开该文档

## 快速上手

1. 安装后打开，点「打开文件」选一个 `.md` 文件（桌面端也可直接拖进来，或在资源管理器里双击关联打开）。
2. 左侧（安卓为底部「大纲」）会列出标题，点标题跳章节。
3. 看到哪关掉就行，下次打开自动回到这一行。
4. 想标记某处，在「大纲 / 书签」面板切到书签页签，点「加书签」。

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

## 常见问题

**支持哪些 Markdown 语法？**
GFM 常用语法：标题、列表、表格、任务列表、删除线、引用、代码块（highlight.js 高亮）、图片、链接。脚注插件与 markdown-it 15 不兼容，未启用；不打算支持数学公式和 Mermaid，那属于「渲染一切」的方向，与轻量阅读冲突。

**文档里的图片为什么显示不出来？**
相对路径图片是按 `.md` 所在目录解析的，文档和图片要保持相对位置。安卓从文件管理器打开单个文件时，会把同目录图片一并拷进应用私有目录，因此移动文件后需重新打开一次。

**进度会不会串到别的文件？**
不会。进度按文件路径独立存储，标星、书签同理。

**安卓上电脑传书连不上？**
手机和电脑要连同一个 Wi-Fi。若在安卓模拟器里调试，模拟器是 NAT 网络，电脑无法直连，需要在电脑上做端口转发：`adb forward tcp:8766 tcp:8766`，然后用 `http://127.0.0.1:8766/?k=<弹层里显示的6位口令>` 打开。

## 开发

### 环境准备（新机器克隆后）

仓库只含源码与构建脚本，`node_modules/`、`target/`、`release/` 以及带本机路径的配置都不入库。

**桌面端（Windows）**

| 依赖 | 说明 |
|------|------|
| Node.js 18+ | 前端依赖与构建 |
| Rust 工具链 | rustup 安装，默认 `x86_64-pc-windows-msvc` 目标 |
| MSVC C++ 生成工具 | Visual Studio 2022「使用 C++ 的桌面开发」工作负载 |
| WebView2 运行时 | Win11 自带；较老 Win10 需单独安装 |

```bash
npm ci                # 按 package-lock.json 精确还原依赖，不要用 npm install
npm run tauri dev     # 桌面端开发运行
npm run build:win     # 打包 → release/LiteMD-win-x64-setup.exe
```

首次打包需全量编译 Rust 依赖，10 分钟以上属正常。终端提示 `cargo` 找不到命令时，把 `%USERPROFILE%\.cargo\bin` 加进 PATH。

**安卓端（可选）**

安卓工程已入库（`src-tauri/gen/android/`，含手写的 `FileOpenerPlugin.kt`，不要随意删除重建），但本机环境需自备：

| 依赖 | 做法 |
|------|------|
| JDK 17 | 在用户级 `%USERPROFILE%\.gradle\gradle.properties` 写 `org.gradle.java.home=<JDK17 路径>`（正斜杠）。仓库内不写死，避免换机器构建失败 |
| Android SDK | 需含 platform-tools、`platforms;android-36`、build-tools、NDK 26.x |
| SDK 路径 | 在 `src-tauri/gen/android/local.properties` 写 `sdk.dir=<SDK 路径>`，或设 `ANDROID_HOME` |
| Rust 安卓目标 | `rustup target add aarch64-linux-android` |
| Windows 开发者模式 | 必须开启：Tauri 用符号链接把 `.so` 放进 `jniLibs` |

```bash
npm run tauri android build -- --apk --split-per-abi --target aarch64
```

必须走 `npm run tauri android build`（CLI 每次会重新生成带本机路径的 `tauri.settings.gradle`），不要直接用 `gradlew`。

### 打包与交付

交付约定：安装包统一放根目录 `release/`，每次生成新包替换旧包，不保留历史版本。

| 平台 | 命令 | 输出 |
|------|------|------|
| Windows | `npm run build:win` | `release/LiteMD-win-x64-setup.exe` |
| Android | 见下方步骤 | `release/LiteMD-android-arm64.apk` |

Windows 安装包默认落在 `src-tauri/target/release/bundle/nsis/`，由 `scripts/collect_release.mjs` 在构建后搬到 `release/` 并改名、清旧包。单独跑 `npm run tauri build` 不会搬运，需再执行 `node scripts/collect_release.mjs`。

Android 产物是未签名 APK，需自行签名后改名：

```bash
# 产物：src-tauri/gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-unsigned.apk
apksigner sign --ks <keystore> --out release/LiteMD-android-arm64.apk <unsigned.apk>
```

> 发布到 GitHub Releases 时，资产文件名必须与上表完全一致（`LiteMD-win-x64-setup.exe`、`LiteMD-android-arm64.apk`），README 里的 `releases/latest/download/...` 直链才有效。

体积相关：Rust 侧开启了 `lto`、`codegen-units=1`、`opt-level="z"`、`strip`、`panic="abort"`；安卓 release 额外开启 `isMinifyEnabled` + `isShrinkResources`。改这些配置前先量一遍产物大小，别凭感觉调。

## 技术栈

| 层 | 技术 |
|----|------|
| 应用框架 | Tauri 2（Rust 后端 + 系统 WebView） |
| 前端 | Vue 3 + Vite + Pinia |
| Markdown | markdown-it（GFM + 任务列表） |
| 代码高亮 | highlight.js |
| 二维码 | qrcode（安卓「电脑传书」用，按需动态加载，不进主包） |

## 项目结构

```
LiteMD/
├─ src/                        Vue 3 前端（桌面 / 安卓共用）
│  ├─ components/
│  │  ├─ mobile/               移动端布局：底部工具条、文件浏览、电脑传书
│  │  ├─ icons/                统一线性 SVG 图标
│  │  ├─ sidebar/              侧边栏：大纲 / 书签 / 最近文件
│  │  └─ …                     Reader、SettingsPanel、TopBar 等桌面组件
│  ├─ composables/             阅读进度、搜索、字体、Markdown 渲染等逻辑
│  ├─ stores/                  Pinia 应用状态
│  ├─ locales/                 zh-CN / en-US 文案
│  └─ utils/                   i18n、平台判断、格式化
├─ src-tauri/
│  ├─ src/                     Rust 后端：commands / storage / wifi_server
│  ├─ capabilities/            Tauri 权限声明
│  ├─ gen/android/             安卓工程（含自定义 FileOpenerPlugin）
│  └─ tauri.conf.json          窗口、打包、NSIS 配置
├─ scripts/collect_release.mjs 打包产物收集到 release/
├─ release/                    交付目录：安装包 / APK
└─ docs/                       设计文档、历史构建计划与规格
```

## 数据存储

桌面端在 `%APPDATA%\LiteMD\`，安卓端在应用私有数据目录，文件名一致：

- `preferences.json` — 用户设置
- `reading_progress.json` — 阅读进度
- `recent_files.json` — 最近文件
- `window_state.json` — 窗口状态（仅桌面端）
- 书签按文件存储（`storage/bookmarks.rs`）

## 文档

- `LiteMD-设计文档.md` — 整体设计文档
- `docs/superpowers/specs/`、`docs/superpowers/plans/` — 安卓版设计与构建计划
