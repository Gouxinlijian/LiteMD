// 收集打包产物到项目根目录 release/。
// Tauri 没有「输出目录」配置项，NSIS 安装包固定落在 src-tauri/target/release/bundle/nsis/，
// 所以构建完成后由本脚本搬到 release/（交付约定：安装包统一放 release/，每次替换旧包、不留历史版本）。
import { copyFileSync, existsSync, mkdirSync, readdirSync, rmSync, statSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const nsisDir = join(root, 'src-tauri', 'target', 'release', 'bundle', 'nsis')
const outDir = join(root, 'release')
/** 对外交付的固定文件名（与 LiteMD-android-arm64.apk 命名风格一致） */
const outName = 'LiteMD-win-x64-setup.exe'

if (!existsSync(nsisDir)) {
  console.error(`未找到 NSIS 产物目录：${nsisDir}\n请先执行 npm run build:win`)
  process.exit(1)
}

// 取最新的安装包：target 里可能残留上一次构建的旧版本
const installers = readdirSync(nsisDir)
  .filter((f) => f.toLowerCase().endsWith('.exe'))
  .map((f) => ({ name: f, mtime: statSync(join(nsisDir, f)).mtimeMs }))
  .sort((a, b) => b.mtime - a.mtime)

if (installers.length === 0) {
  console.error(`NSIS 目录下没有 .exe 安装包：${nsisDir}`)
  process.exit(1)
}

const src = join(nsisDir, installers[0].name)
mkdirSync(outDir, { recursive: true })

// 每次替换旧包：先清掉 release/ 里的 Windows 安装包（APK 等其他产物不动）
let replaced = 0
for (const f of readdirSync(outDir)) {
  if (f.toLowerCase().endsWith('.exe')) {
    rmSync(join(outDir, f))
    replaced++
  }
}

const dst = join(outDir, outName)
copyFileSync(src, dst)
const size = (statSync(dst).size / 1024 / 1024).toFixed(2)
console.log(`已收集 ${installers[0].name} → release/${outName}（${size} MB${replaced ? `，替换旧包 ${replaced} 个` : ''}）`)
