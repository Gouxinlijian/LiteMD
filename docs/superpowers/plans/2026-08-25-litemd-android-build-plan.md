# LiteMD Android Plan 1 — 构建链路打通 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在现有 LiteMD 项目上打通 Android 构建链路——装好环境、生成安卓工程、首次构建出 APK，并安装到 vivo X200 Pro mini 成功启动。

**Architecture:** 在现有 `src-tauri` 中 `tauri android init` 生成安卓工程；JDK 17 用免安装 zip 隔离安装，通过 `gradle.properties` 的 `org.gradle.java.home` 指定，不设置任何环境变量；Android SDK 通过 `local.properties` 的 `sdk.dir` 指定；首次 `tauri android build --apk` 产出 aarch64 APK。

**Tech Stack:** JDK 17（Temurin 免安装）、Android SDK + NDK 26、Rust `aarch64-linux-android`、Tauri 2 Android、gradle

## Global Constraints

- **JDK17 不得影响现有环境（涛哥明确要求）**：免安装 zip 解压到 `D:\Software\JDK\jdk_17`，不写注册表、不改系统环境变量；**不设置 JAVA_HOME、不设置任何环境变量**；现有 JDK 8 的 JAVA_HOME / PATH 原封不动
- 构建时 JDK 由 `src-tauri/gen/android/gradle.properties` 的 `org.gradle.java.home=D:/Software/JDK/jdk_17` 指定（正斜杠，避免转义）
- Android SDK 安装到 `%LOCALAPPDATA%\Android\Sdk`；SDK 路径由 `src-tauri/gen/android/local.properties` 的 `sdk.dir` 指定，不设 ANDROID_HOME 系统变量
- Rust android target：`aarch64-linux-android`（vivo X200 Pro mini 是 aarch64）
- 安卓包名：`com.litemd.android`（与 Windows 的 `com.litemd.reader` 并存不冲突）
- 现有 Windows 构建（NSIS）不受影响

---

### Task 1: 安装 JDK 17（免安装 zip，隔离）

**Files:**
- 无项目文件修改（仅系统目录 `D:\Software\JDK\jdk_17`）

**Interfaces:**
- Produces: `D:\Software\JDK\jdk_17\bin\java.exe`（版本 17），供 Task 5 gradle.properties 引用路径

- [ ] **Step 1: 下载 Temurin 17 Windows x64 zip**

```powershell
Invoke-WebRequest -Uri "https://api.adoptium.net/v3/binary/latest/17/ga/windows/x64/jdk/hotspot/normal/eclipse" -OutFile "$env:TEMP\litemd-jdk17.zip"
```

预期：下载完成（约 180MB），无报错。

- [ ] **Step 2: 解压到独立目录（不写注册表/环境变量）**

```powershell
Expand-Archive -Path "$env:TEMP\litemd-jdk17.zip" -DestinationPath "$env:TEMP\litemd-jdk17-x" -Force
$src = Get-ChildItem "$env:TEMP\litemd-jdk17-x" -Directory | Select-Object -First 1
Move-Item $src.FullName "D:\Software\JDK\jdk_17" -Force
```

- [ ] **Step 3: 验证版本且不动现有 JAVA_HOME**

```powershell
& "D:\Software\JDK\jdk_17\bin\java.exe" -version
$env:JAVA_HOME
```

预期：`java -version` 输出 `openjdk version "17.0.x"`；`$env:JAVA_HOME` 仍为 `D:\Software\JDK\jdk_8u311`（未被修改）。

- [ ] **Step 4: 记录完成**

任务交付：JDK17 就绪且现有环境零污染。记录到会话进度（本目录非 git 仓库，用日志/说明代替 commit）。

---

### Task 2: 安装 Android SDK + NDK

**Files:**
- 无项目文件修改（仅系统目录 `%LOCALAPPDATA%\Android\Sdk`）

**Interfaces:**
- Produces: `%LOCALAPPDATA%\Android\Sdk\cmdline-tools\latest\bin\sdkmanager.bat`；SDK 含 platform-tools、platforms;android-34、build-tools;34.0.0、ndk;26.1.10909125

- [ ] **Step 1: 下载 commandline-tools**

```powershell
$sdk = "$env:LOCALAPPDATA\Android\Sdk"
New-Item -ItemType Directory -Force $sdk | Out-Null
Invoke-WebRequest -Uri "https://dl.google.com/android/repository/commandlinetools-win-11076708_latest.zip" -OutFile "$env:TEMP\litemd-cmdtools.zip"
```

预期：下载完成（约 150MB）。

- [ ] **Step 2: 解压到标准位置（latest）**

```powershell
Expand-Archive -Path "$env:TEMP\litemd-cmdtools.zip" -DestinationPath "$env:TEMP\litemd-cmdtools-x" -Force
New-Item -ItemType Directory -Force "$sdk\cmdline-tools" | Out-Null
Move-Item "$env:TEMP\litemd-cmdtools-x\cmdline-tools" "$sdk\cmdline-tools\latest" -Force
```

- [ ] **Step 3: 用 sdkmanager 安装组件**

```powershell
$sm = "$sdk\cmdline-tools\latest\bin\sdkmanager.bat"
& $sm --sdk_root="$sdk" "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;26.1.10909125"
```

预期：各组件下载安装完成。若 NDK 版本与 Tauri 模板要求的 ndkVersion 不符，后续 Task 6 首次构建报错时补装对应版本。

- [ ] **Step 4: 接受所有许可**

```powershell
"y" * 30 | & $sm --sdk_root="$sdk" --licenses
```

预期：无未接受许可的报错。

- [ ] **Step 5: 验证 SDK 组件已装**

```powershell
Get-ChildItem "$sdk\platforms", "$sdk\build-tools", "$sdk\ndk" | Select-Object -ExpandProperty Name
```

预期：列出 android-34、34.0.0、26.1.10909125。

---

### Task 3: 添加 Rust Android 目标

**Files:**
- 无项目文件修改

**Interfaces:**
- Produces: rustc 支持 `aarch64-linux-android` 目标

- [ ] **Step 1: 添加目标**

```powershell
rustup target add aarch64-linux-android
```

- [ ] **Step 2: 验证**

```powershell
rustup target list --installed
```

预期：输出包含 `aarch64-linux-android`。

---

### Task 4: 生成安卓工程（tauri android init）

**Files:**
- Create: `src-tauri/gen/android/`（整目录，Tauri 生成）
- Modify: `src-tauri/Cargo.toml`（若 Tauri 自动添加 mobile feature 标记）

**Interfaces:**
- Produces: 安卓工程（gradle 结构、MainActivity、AndroidManifest.xml），供 Task 5 配置

- [ ] **Step 1: 运行 init**

```powershell
cd D:\Data\Project\LiteMD
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
npm run tauri android init
```

预期：生成 `src-tauri/gen/android/`，无报错。

- [ ] **Step 2: 验证工程结构生成**

```powershell
Get-ChildItem "src-tauri\gen\android" -Name
Get-ChildItem "src-tauri\gen\android\app\src\main\kotlin" -Recurse -Name
```

预期：存在 `app`、`build.gradle.kts`、`settings.gradle.kts`、`gradle.properties`、`MainActivity.kt` 等。

---

### Task 5: 配置构建参数（JDK 路径 / SDK 路径 / 包名 / 图标）

**Files:**
- Create: `src-tauri/gen/android/gradle.properties`（追加 `org.gradle.java.home`）
- Create: `src-tauri/gen/android/local.properties`（写 `sdk.dir`）
- Modify: `src-tauri/gen/android/app/build.gradle.kts`（applicationId 改为 `com.litemd.android`）
- Modify: `src-tauri/gen/android/app/src/main/AndroidManifest.xml`（声明 md 打开 Intent filter，供 Plan 2 使用）
- Create: `app-icon.png`（1024x1024 源图标，用于生成各尺寸）

**Interfaces:**
- Produces: 构建时 gradle 使用 JDK17 + SDK 路径；APK 包名为 `com.litemd.android`；具备 md 文件关联声明

- [ ] **Step 1: 设置 gradle JDK 路径（关键约束：不设 JAVA_HOME）**

`src-tauri/gen/android/gradle.properties` 末尾追加（正斜杠路径）：

```
org.gradle.java.home=D:/Software/JDK/jdk_17
```

验证：`Get-Content src-tauri/gen/android/gradle.properties` 末尾包含该行。

- [ ] **Step 2: 设置 SDK 路径（不设 ANDROID_HOME 系统变量）**

`src-tauri/gen/android/local.properties` 写入：

```
sdk.dir=C\:\\Users\\Administrator\\AppData\\Local\\Android\\Sdk
```

- [ ] **Step 3: 改安卓包名**

`src-tauri/gen/android/app/build.gradle.kts` 中 `applicationId`（默认取自 tauri.conf.json identifier `com.litemd.reader`）改为 `"com.litemd.android"`。同时把同文件里可能引用 identifier 的位置（如 `namespace`、manifest 生成处）保持一致。

- [ ] **Step 4: 声明 md 打开 Intent filter**

`src-tauri/gen/android/app/src/main/AndroidManifest.xml` 的 `<activity>` 内新增：

```xml
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="content" android:mimeType="text/markdown" />
    <data android:scheme="content" android:mimeType="text/x-markdown" />
    <data android:scheme="file" android:mimeType="text/markdown" />
    <data android:scheme="file" android:mimeType="text/x-markdown" />
</intent-filter>
```

（Plan 2 才实现真正的 Intent 接收与复制；此处先声明，避免改一次 manifest 重装一次。）

- [ ] **Step 5: 生成安卓图标**

```powershell
cd D:\Data\Project\LiteMD
npm run tauri icon app-icon.png
```

预期：`src-tauri/icons/` 下生成 android 各 mipmap 图标。若 `app-icon.png` 不存在，从现有 `src-tauri/icons/icon.png`（若存在）或设计一份 1024x1024 PNG 作为源。

---

### Task 6: 首次构建 APK

**Files:**
- Build output: `src-tauri/target/android/apk/`（或 `src-tauri/gen/android/app/build/outputs/apk/`，随版本而定）

**Interfaces:**
- Produces: 可安装的 aarch64 APK

- [ ] **Step 1: 首次构建（较慢，预计 10-30 分钟：下载 gradle + 依赖 + 编译 Rust + 打包）**

```powershell
cd D:\Data\Project\LiteMD
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
npm run tauri android build -- --apk
```

预期：构建成功，无 Java/NDK/gradle 配置错误。

- [ ] **Step 2: 若首次构建报错，按序排查**

1. **gradle 找不到 JDK** → 确认 `org.gradle.java.home` 路径正确、`D:\Software\JDK\jdk_17` 里确实是 JDK17（含 bin/java.exe）
2. **NDK 版本不匹配** → 报错信息里会写 `ndkVersion` 要求，用 sdkmanager 补装对应版本再 build
3. **SDK 找不到** → 确认 `local.properties` 的 `sdk.dir` 路径正确
4. **Rust android target 缺失** → `rustup target list --installed` 确认

- [ ] **Step 3: 定位 APK 产物**

```powershell
Get-ChildItem "src-tauri\target\android\apk" -Filter *.apk -Recurse -ErrorAction SilentlyContinue
Get-ChildItem "src-tauri\gen\android\app\build\outputs\apk" -Filter *.apk -Recurse -ErrorAction SilentlyContinue
```

预期：找到 `litemd.apk` 或类似产物（aarch64 debug）。

---

### Task 7: 安装到 vivo X200 Pro mini 并验证启动

**Files:**
- 无项目文件修改

**Interfaces:**
- Produces: 真机可启动的 LiteMD 安卓版（MVP 壳，桌面 UI 布局，功能验证留到 Plan 2+）

- [ ] **Step 1: 选择安装方式**

方式 A（USB 调试，推荐开发期）：vivo 开启「开发者选项 → USB 调试」并连电脑，`adb devices` 能看到设备后 `adb install -r <apk>`。
方式 B（传文件安装）：把 APK 传到手机（微信/数据线/U盘），在手机上点击安装（允许未知来源）。

- [ ] **Step 2: 启动验证**

手机上打开 LiteMD，预期：应用正常启动，显示 LiteMD 界面（此阶段仍是桌面布局，后续 Plan 3 改移动端 UI）。能启动即 Plan 1 通过。

- [ ] **Step 3: 验证包名与共存**

设置 → 应用信息：包名显示 `com.litemd.android`，与 Windows 版 `com.litemd.reader` 无冲突。

---

## Plan 1 验收

| 编号 | 验收项 | 标准 |
|------|--------|------|
| B1 | JDK17 隔离 | `D:\Software\JDK\jdk_17\bin\java.exe -version` = 17；系统 JAVA_HOME 仍为 JDK8 |
| B2 | 环境变量零污染 | 构建前后 `$env:JAVA_HOME`、`$env:ANDROID_HOME`、PATH 均无新增持久变量 |
| B3 | APK 产出 | aarch64 APK 生成成功 |
| B4 | 手机启动 | vivo X200 Pro mini 安装并启动 LiteMD 成功 |
| B5 | 包名正确 | `com.litemd.android` |
| B6 | Windows 不受影响 | 现有 NSIS 打包脚本未改动（后续如需重打 Windows 包仍可） |

## 风险与卡点预案

| 风险 | 预案 |
|------|------|
| 首次构建网络下载慢/失败 | 分步重试；gradle 依赖走官方 maven，必要时配置镜像 |
| NDK 版本不匹配 | 按 gradle 报错提示补装对应 NDK 版本 |
| vivo USB 调试未开启 | 改用传文件安装（方式 B） |
| tauri android init 需要确认交互 | CLI 一般无交互；若提示，按提示回车 |
