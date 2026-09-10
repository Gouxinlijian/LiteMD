# LiteMD — 轻量级 Markdown 阅读器 设计文档

> 版本：v1.0  
> 日期：2026-08-14  
> 定位：一个**只做阅读**的轻量 Markdown 阅读器，不做编辑，不做笔记，不做知识管理。  
> 核心卖点：**记住阅读进度** + **目录大纲** + **中文界面** + **极致轻量**

---

## 一、需求背景

### 1.1 痛点

现有 Markdown 工具的问题：

| 工具 | 问题 |
|------|------|
| MarkText | 不记忆滚动位置，大文件重新打开跳到末尾（已知缺陷） |
| Obsidian | 太重，安装包 200MB+，自带大量不需要的功能（双链、图谱、插件市场） |
| Typora | 收费，且偏编辑器 |
| VS Code | 代码编辑器，阅读体验差，预览和大纲分屏割裂 |

### 1.2 目标用户

- 需要阅读大型 Markdown 文档（10万字+）的开发者/技术人员
- 核心诉求：**关掉再打开，还在上次读到的地方**
- 不需要编辑功能，纯阅读

### 1.3 设计原则

1. **只读优先**：不做编辑，把渲染和阅读体验做到极致
2. **轻量至上**：安装包 < 15MB，内存占用 < 80MB，启动时间 < 1秒
3. **进度为王**：阅读进度是第一优先级功能，精确到像素级滚动位置
4. **开箱即用**：零配置，打开就用，中文界面默认

---

## 二、技术选型

### 2.1 技术栈

| 层 | 技术 | 理由 |
|----|------|------|
| 桌面框架 | **Tauri 2.0** | Rust 后端 + 系统 WebView，安装包 ~10MB，内存占用极低（对比 Electron 100MB+） |
| 前端框架 | **Vue 3 + Vite** | 轻量、生态好、构建产物小 |
| Markdown 渲染 | **markdown-it** | 解析速度快，插件生态丰富，支持 GFM |
| 代码高亮 | **highlight.js**（按需加载语言包） | 体积可控，常用语言足够 |
| 数学公式 | **KaTeX**（可选） | 比 MathJax 轻量 10 倍，渲染快 |
| 本地存储 | **JSON 文件** | 无需数据库，简单可靠，人类可读 |
| UI 样式 | **手写 CSS**（不引 UI 框架） | 控制体积，阅读器界面简单不需要复杂组件 |

### 2.2 为什么选 Tauri 不选 Electron

| 维度 | Tauri | Electron |
|------|-------|----------|
| 安装包大小 | ~10MB | ~80-150MB |
| 内存占用 | ~40-60MB | ~150-300MB |
| 启动速度 | <1s | 2-5s |
| 后端语言 | Rust（安全、高性能） | Node.js |
| WebView | 系统原生（无捆绑） | 捆绑 Chromium |
| 适用场景 | 轻量工具 | 复杂桌面应用 |

**结论**：一个纯阅读器用 Electron 是杀鸡用牛刀，Tauri 是最佳选择。

### 2.3 系统要求

- **仅支持 Windows 10/11**（WebView2 已预装，无需额外运行时）
- .NET Framework 4.6.2+（WebView2 依赖，系统自带）
- 不支持 macOS / Linux（不考虑跨平台，减少复杂度）

---

## 三、功能设计

### 3.1 核心功能（MVP）

| 编号 | 功能 | 优先级 | 说明 |
|------|------|--------|------|
| F1 | Markdown 渲染 | P0 | 支持 GFM（表格、任务列表、删除线、代码块） |
| F2 | 阅读进度记忆 | P0 | 每个文件独立记录滚动位置，重新打开精确恢复 |
| F3 | 目录大纲 | P0 | 自动从标题（h1-h6）生成树形目录，点击跳转 |
| F4 | 中文界面 | P0 | 默认中文，可切换英文 |
| F5 | 打开文件 | P0 | 文件对话框 + 拖拽打开 + 命令行参数打开 |
| F6 | 最近文件列表 | P0 | 记录最近打开的 20 个文件，一键打开 |
| F7 | 明暗主题 | P1 | 跟随系统 / 手动切换 |
| F8 | 文内搜索 | P1 | Ctrl+F 搜索，高亮匹配项，上下跳转 |
| F9 | 字体缩放 | P1 | Ctrl+滚轮 / Ctrl +/- 调整字号 |
| F10 | 全屏阅读 | P2 | F11 全屏，隐藏所有 UI 只留正文 |

### 3.2 阅读进度记忆（核心功能详解）

这是本应用的**核心差异化功能**，需要重点设计。

#### 3.2.1 存储策略

```json
// reading_progress.json
{
  "files": {
    "D:\\Data\\Project\\my-demo\\docs\\Java_2026.md": {
      "scrollTop": 18432,
      "progressPercent": 67.3,
      "lastReadAt": "2026-08-14T18:02:01+08:00",
      "readTimeMinutes": 142,
      "fileHash": "a1b2c3d4e5f6"
    },
    "D:\\Docs\\design.md": {
      "scrollTop": 0,
      "progressPercent": 0,
      "lastReadAt": "2026-08-13T10:30:00+08:00",
      "readTimeMinutes": 5,
      "fileHash": "f6e5d4c3b2a1"
    }
  }
}
```

#### 3.2.2 进度保存机制

```
┌─────────────────────────────────────────────────┐
│                  进度保存流程                      │
├─────────────────────────────────────────────────┤
│                                                   │
│  用户滚动                                          │
│    │                                              │
│    ▼                                              │
│  防抖计时器启动（500ms 延迟）                        │
│    │                                              │
│    ▼                                              │
│  500ms 内无新滚动？                                │
│    │                                              │
│    ├─ 是 → 记录当前 scrollTop + 滚动百分比          │
│    │         写入内存缓存                           │
│    │                                              │
│    └─ 否 → 重置计时器                              │
│                                                   │
│  定时器（每 3 秒）                                  │
│    │                                              │
│    ▼                                              │
│  内存缓存有变更？                                   │
│    ├─ 是 → 写入 reading_progress.json              │
│    └─ 否 → 跳过                                   │
│                                                   │
│  窗口关闭 / 失焦                                    │
│    │                                              │
│    ▼                                              │
│  立即写入 reading_progress.json（强制落盘）         │
│                                                   │
└─────────────────────────────────────────────────┘
```

**关键设计点**：
1. **防抖保存**：滚动时 500ms 内无新操作才记录，避免频繁 IO
2. **定时落盘**：每 3 秒将内存缓存写入磁盘，防止异常退出丢失
3. **关闭强写**：窗口关闭/失焦时立即强制写入，确保进度不丢
4. **文件哈希校验**：记录文件内容的 hash，如果文件被修改（hash 变化），进度按比例对齐而非精确恢复
5. **双维度记录**：同时记录 `scrollTop`（像素）和 `progressPercent`（百分比），恢复时优先用百分比定位（文件内容变化时更鲁棒）

#### 3.2.3 进度恢复机制

```javascript
// 伪代码：打开文件时恢复进度
async function restoreProgress(filePath, fileContent) {
  const progress = readProgress(filePath);
  
  if (!progress) {
    // 新文件，从顶部开始
    window.scrollTo(0);
    return;
  }
  
  const currentHash = md5(fileContent);
  if (currentHash === progress.fileHash) {
    // 文件未修改，精确恢复像素位置
    window.scrollTo(progress.scrollTop);
  } else {
    // 文件已修改，按百分比恢复
    const docHeight = document.documentElement.scrollHeight;
    window.scrollTo(docHeight * progress.progressPercent / 100);
  }
}
```

#### 3.2.4 进度可视化

- **底部状态栏**：显示当前阅读进度百分比 `67% ━━━━━━━━━━━━━━━━━━░░░░░░░░░░`
- **侧边栏目录**：当前阅读到的章节高亮显示（类似 GitBook 的阅读指示器）
- **进度条**：右侧细条显示整体进度，可点击跳转

### 3.3 目录大纲

#### 3.3.1 生成逻辑

```javascript
// 从 markdown-it 解析结果中提取标题
function generateTOC(tokens) {
  const toc = [];
  const stack = []; // 用于构建层级

  for (const token of tokens) {
    if (token.type === 'heading_open') {
      const level = parseInt(token.tag.substring(1)); // h1 -> 1
      const nextToken = tokens[tokens.indexOf(token) + 1];
      const text = nextToken?.content || '';
      const id = slugify(text); // 生成锚点 ID

      const node = { level, text, id, children: [] };

      // 构建树形结构
      while (stack.length > 0 && stack[stack.length - 1].level >= level) {
        stack.pop();
      }
      if (stack.length === 0) {
        toc.push(node);
      } else {
        stack[stack.length - 1].children.push(node);
      }
      stack.push(node);
    }
  }

  return toc;
}
```

#### 3.3.2 交互设计

- 目录树可折叠/展开
- 点击标题 → 平滑滚动到对应位置（带动画）
- 当前阅读位置对应的标题自动高亮 + 自动展开
- 目录面板宽度可拖拽调整

#### 3.3.3 当前章节追踪

```javascript
// 滚动时实时追踪当前阅读到哪个章节
function trackCurrentHeading() {
  const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
  const scrollY = window.scrollY + 100; // 偏移量

  let current = null;
  for (const heading of headings) {
    if (heading.offsetTop <= scrollY) {
      current = heading;
    } else {
      break;
    }
  }

  // 高亮侧边栏对应的目录项
  highlightTOCItem(current?.id);
}
```

### 3.4 Markdown 渲染

#### 3.4.1 支持的语法

| 语法 | 支持 | 说明 |
|------|------|------|
| 标题 h1-h6 | ✅ | 自动生成锚点 |
| 段落/换行 | ✅ | |
| 粗体/斜体/删除线 | ✅ | GFM |
| 链接/图片 | ✅ | 图片支持本地路径 + 点击放大 |
| 代码块 | ✅ | 语法高亮 + 复制按钮 |
| 行内代码 | ✅ | |
| 引用块 | ✅ | |
| 有序/无序列表 | ✅ | |
| 任务列表 | ✅ | GFM |
| 表格 | ✅ | GFM，支持宽列调整 |
| 分割线 | ✅ | |
| 数学公式 | ✅（可选） | KaTeX 渲染 |
| 脚注 | ✅ | |
| 目录 (TOC) | ✅ | 支持 `[TOC]` 语法自动生成 |
| Mermaid 图表 | ✅（可选） | 流程图/时序图 |

#### 3.4.2 渲染优化

- **虚拟滚动**：超大文件（10万字+）使用虚拟滚动，只渲染可视区域 ± 缓冲区的 DOM 节点
- **惰性图片加载**：`<img>` 使用 `loading="lazy"`
- **增量渲染**：先渲染前 1000 行让用户快速看到内容，后台继续渲染剩余部分
- **缓存解析结果**：同一文件未修改时直接使用缓存的 HTML

### 3.5 主题系统

```css
/* 浅色主题（默认） */
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f7f8fa;
  --bg-sidebar: #f0f1f3;
  --text-primary: #1a1a1a;
  --text-secondary: #666666;
  --text-heading: #111111;
  --accent-color: #4a90d9;
  --border-color: #e0e0e0;
  --code-bg: #f5f5f5;
  --blockquote-border: #4a90d9;
  --link-color: #4a90d9;
  --toc-active-bg: #e8f0fe;
}

/* 暗色主题 */
[data-theme="dark"] {
  --bg-primary: #1e1e1e;
  --bg-secondary: #252526;
  --bg-sidebar: #2d2d2d;
  --text-primary: #d4d4d4;
  --text-secondary: #999999;
  --text-heading: #ffffff;
  --accent-color: #569cd6;
  --border-color: #3e3e42;
  --code-bg: #2d2d2d;
  --blockquote-border: #569cd6;
  --link-color: #569cd6;
  --toc-active-bg: #2d4f6e;
}
```

### 3.6 文件关联

支持将应用设为 `.md` 文件的默认打开方式：
- 安装时写入 Windows 注册表（`.md` 关联到 LiteMD）
- 支持 `litemd.exe <file.md>` 命令行直接打开文件
- 设置面板提供「设为默认 .md 打开方式」按钮

---

## 四、界面设计

### 4.1 整体布局

```
┌──────────────────────────────────────────────────────────────┐
│  LiteMD                                  🔍  🌙  ⚙️  ─ □ ✕   │  ← 顶栏（可隐藏）
├────────────┬─────────────────────────────────────────────────┤
│            │                                                  │
│  📂 最近    │              Markdown 正文渲染区                  │
│  ────────  │                                                  │
│  📋 大纲    │     # Java 核心知识体系                            │
│  ────────  │                                                  │
│  > Java基础 │     ## 第一章 JVM                                 │
│    JVM     │                                                  │
│    GC      │     Java虚拟机（JVM）是运行Java字节码的...           │
│  > 并发编程  │                                                  │
│  > 集合框架  │     ### 1.1 内存区域                              │
│  > IO/NIO  │                                                  │
│            │     JVM内存区域分为：堆、栈、方法区...                │
│            │                                                  │
│            │     ```java                                       │
│            │     public class HelloWorld {                      │
│            │         public static void main(String[] args) {  │
│            │             System.out.println("Hello");          │
│            │         }                                         │
│            │     }                                             │
│            │     ```                                           │
│            │                                                  │
├────────────┴─────────────────────────────────────────────────┤
│  Java_2026.md  |  67% ━━━━━━━━━━━━━━━━━━░░░░░░  |  28万字  ⏱ 2h │  ← 底部状态栏
└──────────────────────────────────────────────────────────────┘
     ↑ 侧边栏                        ↑ 阅读区                ↑ 状态栏
   (可折叠)                       (主区域)             (进度+信息)
```

### 4.2 侧边栏设计

侧边栏有两个 Tab 页，底部切换：

```
┌────────────────┐
│ 📂 最近  📋 大纲 │  ← Tab 切换
├────────────────┤
│                │
│  📄 Java_2026   │  ← 最近文件列表
│     2小时前 67% │
│                │
│  📄 design.md   │
│     昨天 100%   │
│                │
│  📄 README.md   │
│     3天前 12%   │
│                │
│  📄 api.md      │
│     上周 45%   │
│                │
│   ...          │
│                │
├────────────────┤
│  📂 最近  📋 大纲 │  ← 底部固定 Tab 栏
└────────────────┘
```

大纲 Tab：

```
┌────────────────┐
│ 📂 最近  📋 大纲 │
├────────────────┤
│                │
│  📁 Java核心    │  ← h1
│  ├─ JVM        │  ← h2
│  │  ├─ 内存区域 │  ← h3（当前阅读位置高亮）
│  │  ├─ GC算法  │
│  │  └─ 类加载  │
│  ├─ 并发编程    │
│  │  ├─ 线程基础 │
│  │  ├─ 锁机制  │
│  │  └─ 线程池  │
│  └─ 集合框架    │
│                │
├────────────────┤
│  📂 最近  📋 大纲 │
└────────────────┘
```

### 4.3 全屏阅读模式

按 `F11` 进入全屏，隐藏顶栏、侧边栏、状态栏，只保留正文。鼠标移到顶部边缘显示迷你工具栏：

```
                    ┌─────────────────────────┐
                    │  📋  🔍  🌙  ⚙️  ⛶退出 │
                    └─────────────────────────┘
─────────────────────────────────────────────────
                     # Java 核心知识体系

                     ## 第一章 JVM

                     Java虚拟机（JVM）是运行...
─────────────────────────────────────────────────
```

### 4.4 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+O` | 打开文件 |
| `Ctrl+S` | （无操作，只读模式） |
| `Ctrl+F` | 文内搜索 |
| `Ctrl+J` | 切换侧边栏 |
| `Ctrl+K` | 切换到大纲 Tab |
| `Ctrl+R` | 切换最近文件 Tab |
| `Ctrl+=` | 放大字体 |
| `Ctrl+-` | 缩小字体 |
| `Ctrl+0` | 重置字体 |
| `Ctrl+,` | 打开设置 |
| `F11` | 全屏切换 |
| `Esc` | 退出全屏 / 关闭搜索 |
| `Ctrl+Tab` | 切换最近文件 |
| `Space` | 向下翻页 |
| `Shift+Space` | 向上翻页 |

---

## 五、数据模型

### 5.1 配置文件

```
%APPDATA%/LiteMD/
├── preferences.json        # 用户设置
├── reading_progress.json   # 阅读进度
├── recent_files.json       # 最近文件列表
└── window_state.json       # 窗口状态
```

### 5.2 preferences.json

```json
{
  "language": "zh-CN",
  "theme": "auto",
  "fontSize": 16,
  "fontFamily": "'Microsoft YaHei', 'Segoe UI', system-ui, sans-serif",
  "sidebarVisible": true,
  "sidebarWidth": 260,
  "sidebarActiveTab": "outline",
  "tabBarVisible": false,
  "autoSaveProgress": true,
  "renderMermaid": true,
  "renderMath": true,
  "codeHighlight": true,
  "lineHeight": 1.8,
  "maxContentWidth": 900
}
```

### 5.3 reading_progress.json

（见 3.2.1 节）

### 5.4 recent_files.json

```json
{
  "files": [
    {
      "path": "D:\\Data\\Project\\my-demo\\docs\\Java_2026.md",
      "name": "Java_2026.md",
      "dir": "D:\\Data\\Project\\my-demo\\docs",
      "size": 286000,
      "lastOpenedAt": "2026-08-14T18:02:01+08:00",
      "progressPercent": 67.3
    }
  ]
}
```

### 5.5 window_state.json

```json
{
  "width": 1200,
  "height": 800,
  "x": 100,
  "y": 100,
  "isMaximized": false,
  "isFullScreen": false
}
```

---

## 六、架构设计

### 6.1 整体架构

```
┌─────────────────────────────────────────────────┐
│                  Tauri 应用                       │
│                                                   │
│  ┌─────────────────────────────────────────────┐ │
│  │              Rust 后端 (src-tauri)            │ │
│  │                                               │ │
│  │  ┌──────────┐  ┌──────────┐  ┌───────────┐  │ │
│  │  │ 文件操作  │  │ 进度存储  │  │ 窗口管理   │  │ │
│  │  │ 命令     │  │ 命令     │  │           │  │ │
│  │  └──────────┘  └──────────┘  └───────────┘  │ │
│  │                                               │ │
│  │  ┌──────────┐  ┌──────────┐  ┌───────────┐  │ │
│  │  │ 文件监听  │  │ 系统托盘  │  │ 自动更新   │  │ │
│  │  │ (notify) │  │          │  │ (updater) │  │ │
│  │  └──────────┘  └──────────┘  └───────────┘  │ │
│  └─────────────────────────────────────────────┘ │
│                       │ IPC                      │
│  ┌─────────────────────────────────────────────┐ │
│  │           Vue 3 前端 (src)                    │ │
│  │                                               │ │
│  │  ┌──────────┐  ┌──────────┐  ┌───────────┐  │ │
│  │  │ 顶栏组件  │  │ 侧边栏    │  │ 阅读区     │  │ │
│  │  │ TopBar   │  │ Sidebar  │  │ Reader    │  │ │
│  │  └──────────┘  └──────────┘  └───────────┘  │ │
│  │                                               │ │
│  │  ┌──────────┐  ┌──────────┐  ┌───────────┐  │ │
│  │  │ 状态栏    │  │ 搜索组件  │  │ 设置面板   │  │ │
│  │  │ StatusBar│  │ Search   │  │ Settings  │  │ │
│  │  └──────────┘  └──────────┘  └───────────┘  │ │
│  │                                               │ │
│  │  ┌─────────────────────────────────────────┐ │ │
│  │  │            核心逻辑层                     │ │ │
│  │  │  ┌─────────┐ ┌─────────┐ ┌───────────┐ │ │ │
│  │  │  │Markdown │ │ 进度管理 │ │ TOC生成    │ │ │ │
│  │  │  │Renderer │ │Progress │ │ TOCEngine │ │ │ │
│  │  │  └─────────┘ └─────────┘ └───────────┘ │ │ │
│  │  └─────────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### 6.2 前端模块划分

```
src/
├── main.js                      # Vue 应用入口
├── App.vue                      # 根组件
├── components/
│   ├── TopBar.vue               # 顶栏：文件名、搜索、主题、设置
│   ├── Sidebar.vue              # 侧边栏容器（Tab 切换）
│   ├── sidebar/
│   │   ├── RecentFiles.vue      # 最近文件列表
│   │   └── Outline.vue          # 目录大纲树
│   ├── Reader.vue               # Markdown 渲染区
│   ├── StatusBar.vue            # 底部状态栏（进度、字数、时间）
│   ├── SearchBar.vue            # 文内搜索浮层
│   └── SettingsPanel.vue        # 设置面板
├── composables/
│   ├── useMarkdown.js           # Markdown 解析与渲染
│   ├── useProgress.js           # 阅读进度管理（核心）
│   ├── useTOC.js                # 目录大纲生成与追踪
│   ├── useTheme.js              # 主题切换
│   ├── useFiles.js              # 文件操作（打开、最近列表）
│   └── useSearch.js             # 文内搜索
├── stores/
│   └── app.js                   # Pinia 全局状态
├── utils/
│   ├── markdownConfig.js        # markdown-it 配置
│   ├── slugify.js               # 标题转锚点
│   └── format.js                # 格式化工具
├── styles/
│   ├── variables.css            # CSS 变量（主题色）
│   ├── global.css               # 全局样式
│   ├── reader.css               # 阅读区样式
│   └── highlight.css            # 代码高亮主题
└── locales/
    ├── zh-CN.json               # 中文语言包
    └── en-US.json               # 英文语言包
```

### 6.3 Rust 后端模块

```
src-tauri/
├── src/
│   ├── main.rs                  # Tauri 应用入口
│   ├── commands/
│   │   ├── file.rs              # 文件操作命令（打开、读取、监听变化）
│   │   ├── progress.rs          # 进度存储命令（读取、保存）
│   │   └── window.rs            # 窗口管理命令
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── preferences.rs       # 配置文件读写
│   │   ├── progress.rs          # 进度文件读写
│   │   └── recent.rs            # 最近文件读写
│   └── utils/
│       ├── hash.rs              # 文件哈希计算
│       └── path.rs              # 路径处理
├── Cargo.toml
└── tauri.conf.json              # Tauri 配置
```

### 6.4 关键流程：打开文件 → 渲染 → 恢复进度

```
用户打开文件（对话框/拖拽/命令行/最近文件）
        │
        ▼
┌─ Rust 后端 ──────────────────────────────────┐
│  1. 读取文件内容                               │
│  2. 计算文件 MD5 哈希                          │
│  3. 读取该文件的阅读进度（reading_progress.json）│
│  4. 将 { content, hash, progress } 返回前端    │
└──────────────────────────────────────────────┘
        │ IPC
        ▼
┌─ Vue 前端 ───────────────────────────────────┐
│  5. markdown-it 解析 content → HTML           │
│  6. 从 HTML 提取标题 → 生成 TOC 树             │
│  7. 渲染 HTML 到阅读区                         │
│  8. 等待 DOM 渲染完成（nextTick）              │
│                                               │
│  9. 恢复进度：                                 │
│     ├─ hash 匹配 → scrollTo(progress.scrollTop)│
│     └─ hash 不匹配 → scrollTo(百分比位置)      │
│                                               │
│  10. 启动滚动监听（防抖 500ms）                 │
│  11. 启动定时保存（每 3 秒）                    │
│  12. 高亮当前章节（TOC 追踪）                   │
└──────────────────────────────────────────────┘
```

---

## 七、关键技术方案

### 7.1 大文件虚拟滚动

对于 10 万字以上的文件，一次性渲染全部 DOM 会导致卡顿。方案：

```javascript
// useVirtualScroll.js
// 策略：将 markdown 渲染为块（block），每块是一个顶层元素
// 只渲染可视区域 + 上下各 5 块缓冲区

const BUFFER_BLOCKS = 5;
const BLOCK_HEIGHT_ESTIMATE = 80; // 预估每块高度

function calculateVisibleBlocks(scrollTop, viewportHeight, blocks) {
  const startIdx = Math.max(0, 
    findBlockIndex(scrollTop, blocks) - BUFFER_BLOCKS);
  const endIdx = Math.min(blocks.length - 1, 
    findBlockIndex(scrollTop + viewportHeight, blocks) + BUFFER_BLOCKS);
  
  return { startIdx, endIdx, blocks: blocks.slice(startIdx, endIdx + 1) };
}

// 用占位 div 撑出总高度，保持滚动条正确
// <div style="height: totalHeight">
//   <div style="transform: translateY(offset)">
//     {visibleBlocks}
//   </div>
// </div>
```

**注意**：虚拟滚动与进度恢复需要协调——进度按百分比计算而非像素。

### 7.2 滚动防抖与进度保存

```javascript
// useProgress.js
import { debounce } from 'lodash-es';

let progressCache = { scrollTop: 0, percent: 0, dirty: false };
let saveTimer = null;

// 防抖记录：滚动停止 500ms 后记录到内存
const onScroll = debounce(() => {
  const scrollTop = window.scrollY;
  const docHeight = document.documentElement.scrollHeight - window.innerHeight;
  const percent = docHeight > 0 ? (scrollTop / docHeight) * 100 : 0;
  
  progressCache = { scrollTop, percent, dirty: true };
}, 500);

// 定时落盘：每 3 秒检查缓存是否脏，是则写入磁盘
saveTimer = setInterval(async () => {
  if (!progressCache.dirty) return;
  
  await invoke('save_progress', {
    filePath: currentFilePath,
    scrollTop: progressCache.scrollTop,
    percent: progressCache.percent,
    timestamp: new Date().toISOString()
  });
  progressCache.dirty = false;
}, 3000);

// 窗口关闭/失焦时强制落盘
window.addEventListener('beforeunload', () => {
  if (progressCache.dirty) {
    // 同步写入（使用 sendBeacon 或 Tauri 同步命令）
    invoke('save_progress_sync', { ... });
  }
});

window.addEventListener('blur', () => {
  if (progressCache.dirty) {
    invoke('save_progress', { ... });
    progressCache.dirty = false;
  }
});
```

### 7.3 文件变化监听

当用户在其他编辑器修改了文件，LiteMD 应能检测并提示：

```rust
// src-tauri/src/commands/file.rs
use notify::{Watcher, RecursiveMode, Event};
use std::sync::mpsc::channel;

#[tauri::command]
pub async fn watch_file(path: String, window: Window) -> Result<(), String> {
    let (tx, rx) = channel();
    let mut watcher = notify::watcher(tx, std::time::Duration::from_secs(2))
        .map_err(|e| e.to_string())?;
    
    watcher.watch(&path, RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;
    
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = rx.recv() {
            if let EventKind::Modify(_) = event.kind {
                window.emit("file-changed", &path).ok();
            }
        }
    });
    
    Ok(())
}
```

前端监听：

```javascript
import { listen } from '@tauri-apps/api/event';

listen('file-changed', (event) => {
  // 显示提示：「文件已被修改，是否重新加载？」
  showReloadToast(event.payload);
});
```

### 7.4 拖拽打开文件

```vue
<!-- App.vue -->
<template>
  <div 
    @dragover.prevent="isDragging = true"
    @dragleave.prevent="isDragging = false"
    @drop.prevent="handleDrop"
  >
    <div v-if="isDragging" class="drop-overlay">
      <div class="drop-hint">📂 松手打开文件</div>
    </div>
    <!-- 正常内容 -->
  </div>
</template>

<script setup>
const isDragging = ref(false);

async function handleDrop(e) {
  isDragging.value = false;
  const files = e.dataTransfer.files;
  if (files.length > 0) {
    const file = files[0];
    if (file.name.endsWith('.md') || file.name.endsWith('.markdown')) {
      await openFile(file.path);
    } else {
      showToast('只支持 .md 文件');
    }
  }
}
</script>
```

---

## 八、开发计划

### Phase 1：MVP（核心可用）

**目标**：能打开 md 文件，能看目录，能记进度。

| 任务 | 技术点 | 预估 |
|------|--------|------|
| Tauri 项目初始化 | Tauri 2.0 + Vue 3 + Vite 脚手架 | 0.5天 |
| Rust 文件操作命令 | 打开文件对话框、读取文件内容 | 0.5天 |
| Markdown 渲染 | markdown-it 配置、GFM、代码高亮 | 1天 |
| 阅读区样式 | 排版、字体、间距、代码块样式 | 0.5天 |
| 目录大纲生成 | 标题提取、树形构建、锚点跳转 | 1天 |
| 阅读进度记忆 | 防抖记录、定时落盘、恢复滚动 | 1天 |
| 侧边栏布局 | 最近文件 + 大纲 Tab 切换 | 0.5天 |
| 顶栏 + 状态栏 | 文件名、进度条、字数 | 0.5天 |
| 中文界面 | 语言包、界面文案 | 0.5天 |
| 拖拽打开 | 拖拽事件处理 | 0.5天 |

**MVP 合计：约 6.5 天**

### Phase 2：体验优化

| 任务 | 说明 |
|------|------|
| 明暗主题切换 | CSS 变量 + 跟随系统 |
| 文内搜索 | Ctrl+F 高亮 + 上下跳转 |
| 字体缩放 | Ctrl+滚轮 / Ctrl +/- |
| 全屏阅读 | F11 隐藏 UI |
| 当前章节追踪 | 滚动时高亮 TOC 对应项 |
| 文件变化监听 | 外部修改提示重新加载 |
| 大文件虚拟滚动 | 10万字+文件性能优化 |
| 窗口状态记忆 | 大小、位置、最大化状态 |

### Phase 3：完善

| 任务 | 说明 |
|------|------|
| 系统文件关联 | 双击 .md 用 LiteMD 打开 |
| 命令行参数 | `litemd file.md` 直接打开 |
| Mermaid 图表 | 流程图/时序图渲染 |
| 数学公式 | KaTeX 集成 |
| 自动更新 | Tauri updater |
| 系统托盘 | 最小化到托盘 |
| 导出 PDF | 打印样式优化 |
| 书签功能 | 标记重点位置，快速跳回 |

---

## 九、项目结构总览

```
LiteMD/
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── file.rs
│   │   │   ├── progress.rs
│   │   │   └── window.rs
│   │   ├── storage/
│   │   │   ├── mod.rs
│   │   │   ├── preferences.rs
│   │   │   ├── progress.rs
│   │   │   └── recent.rs
│   │   └── utils/
│   │       ├── hash.rs
│   │       └── path.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # Vue 前端
│   ├── main.js
│   ├── App.vue
│   ├── components/
│   │   ├── TopBar.vue
│   │   ├── Sidebar.vue
│   │   ├── sidebar/
│   │   │   ├── RecentFiles.vue
│   │   │   └── Outline.vue
│   │   ├── Reader.vue
│   │   ├── StatusBar.vue
│   │   ├── SearchBar.vue
│   │   └── SettingsPanel.vue
│   ├── composables/
│   │   ├── useMarkdown.js
│   │   ├── useProgress.js
│   │   ├── useTOC.js
│   │   ├── useTheme.js
│   │   ├── useFiles.js
│   │   └── useSearch.js
│   ├── stores/
│   │   └── app.js
│   ├── utils/
│   │   ├── markdownConfig.js
│   │   ├── slugify.js
│   │   └── format.js
│   ├── styles/
│   │   ├── variables.css
│   │   ├── global.css
│   │   ├── reader.css
│   │   └── highlight.css
│   └── locales/
│       ├── zh-CN.json
│       └── en-US.json
├── package.json
├── vite.config.js
└── README.md
```

---

## 十、验收标准

| 编号 | 验收项 | 标准 |
|------|--------|------|
| A1 | 打开 md 文件 | 支持对话框、拖拽、命令行三种方式 |
| A2 | 渲染正确 | GFM 语法全部正确渲染，代码高亮正常 |
| A3 | 目录大纲 | 自动生成树形目录，点击跳转，当前章节高亮 |
| A4 | 阅读进度 | 关闭文件再打开，精确恢复到上次滚动位置（误差 < 50px） |
| A5 | 文件修改后进度 | 文件被外部修改后，按百分比恢复（不跳到错误位置） |
| A6 | 中文界面 | 所有 UI 文案为中文，无遗漏 |
| A7 | 性能 | 10万字文件打开 < 2秒，滚动无明显卡顿 |
| A8 | 内存 | 正常使用内存 < 80MB |
| A9 | 安装包 | Windows 安装包（NSIS/MSI）< 15MB |
| A10 | 主题 | 明暗主题切换正常，跟随系统模式有效 |

---

## 附录 A：依赖清单

### Rust (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2.0", features = ["dialog-all", "fs-all", "shell-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
notify = "6.0"          # 文件变化监听
md5 = "0.7"             # 文件哈希
chrono = "0.4"          # 时间处理
```

### tauri.conf.json（Windows 专属配置）

```json
{
  "build": {
    "distDir": "../dist",
    "devPath": "http://localhost:5173"
  },
  "package": {
    "productName": "LiteMD",
    "version": "1.0.0"
  },
  "tauri": {
    "bundle": {
      "active": true,
      "targets": ["nsis", "msi"],
      "identifier": "com.litemd.app",
      "icon": ["icons/icon.ico"],
      "windows": {
        "webviewInstallMode": {
          "type": "downloadBootstrapper"
        }
      }
    },
    "windows": [
      {
        "title": "LiteMD",
        "width": 1200,
        "height": 800,
        "minWidth": 600,
        "minHeight": 400,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```

> **关键配置说明**：
> - `"targets": ["nsis", "msi"]` — 只生成 Windows 安装包（NSIS 安装器 + MSI 安装包），不打包 macOS/Linux
> - `"webviewInstallMode"` — 如果用户系统没有 WebView2，安装时自动下载安装
> - 图标只需 `.ico` 格式（Windows 专用）

### Node (package.json)

```json
{
  "dependencies": {
    "vue": "^3.4.0",
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "markdown-it": "^14.0.0",
    "markdown-it-anchor": "^9.0.0",
    "markdown-it-task-lists": "^2.0.0",
    "markdown-it-footnote": "^4.0.0",
    "highlight.js": "^11.9.0",
    "katex": "^0.16.0",
    "pinia": "^2.1.0",
    "lodash-es": "^4.17.21"
  },
  "devDependencies": {
    "vite": "^5.0.0",
    "@vitejs/plugin-vue": "^5.0.0",
    "@tauri-apps/cli": "^2.0.0"
  }
}
```

---

## 附录 B：与 MarkText 对比

| 维度 | MarkText | LiteMD |
|------|----------|--------|
| 定位 | 编辑器 | **纯阅读器** |
| 安装包 | ~85MB | **~10MB** |
| 内存 | ~200MB | **~50MB** |
| 阅读进度 | ❌ 不记忆 | **✅ 精确记忆** |
| 目录大纲 | ✅ 有 | ✅ 有 |
| 中文 | ⚠️ 需配置 | **✅ 默认中文** |
| 启动速度 | 2-3秒 | **<1秒** |
| 编辑功能 | ✅ 有 | ❌ 不做 |
| 插件系统 | ❌ 无 | ❌ 不做 |
| 复杂度 | 高 | **极简** |
