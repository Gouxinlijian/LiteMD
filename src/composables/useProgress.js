import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/app'

/**
 * 阅读进度记忆（核心功能）：
 * 滚动防抖 500ms 记录内存 → 每 3 秒落盘 → 关闭/失焦强制保存；
 * 恢复时文件未修改按像素精确恢复，已修改按百分比对齐。
 * 以模块单例导出，Reader（监听）与 App（窗口关闭兜底）共享同一实例。
 * 注意：进度归属在 start() 时绑定到当时打开的文件，切换文件时 stop() 的兜底保存
 * 仍会写回旧文件，避免误写入新文件。
 */
let scrollTimer = null
let saveTimer = null
let dirty = false
let sessionStart = 0
let boundFile = null // start 时绑定的文件，进度始终归属该文件
let cached = { scrollTop: 0, percent: 0 }
let scrollTarget = null // 已绑定滚动的容器
let scrollHandler = null // 已绑定的滚动回调，stop 时需要解绑

function appStore() {
  return useAppStore()
}

/** 记录当前滚动位置到内存缓存 */
function capture(scrollEl) {
  const docHeight = scrollEl.scrollHeight - scrollEl.clientHeight
  const percent = docHeight > 0 ? (scrollEl.scrollTop / docHeight) * 100 : 0
  cached = { scrollTop: scrollEl.scrollTop, percent }
  dirty = true
}

/** 保存进度到磁盘 */
async function save() {
  if (!boundFile || !dirty) return
  const readTimeMinutes =
    Math.round((Date.now() - sessionStart) / 60000) +
    (boundFile.progress?.readTimeMinutes ?? 0)
  try {
    await invoke('save_progress', {
      path: boundFile.path,
      scrollTop: cached.scrollTop,
      progressPercent: cached.percent,
      readTimeMinutes,
      fileHash: boundFile.hash,
    })
    dirty = false
  } catch (e) {
    console.error('保存进度失败', e)
  }
}

/** 恢复滚动位置（文件未修改精确恢复，已修改按百分比） */
function restore(scrollEl) {
  const p = appStore().currentFile?.progress
  if (!p || (!p.progressPercent && !p.scrollTop)) return

  if (p.fileHash === appStore().currentFile.hash) {
    scrollEl.scrollTop = p.scrollTop || 0
  } else {
    const docHeight = scrollEl.scrollHeight - scrollEl.clientHeight
    scrollEl.scrollTop = docHeight * ((p.progressPercent || 0) / 100)
  }
}

/** 解绑上一次的滚动监听：否则每次切换文件都会多留一个回调，滚动时成倍叠加 */
function stopListening() {
  if (scrollTarget && scrollHandler) scrollTarget.removeEventListener('scroll', scrollHandler)
  scrollTarget = null
  scrollHandler = null
}

/** 启动滚动监听与定时保存；onScroll 用于外部同步章节高亮等 UI 状态 */
function start(scrollEl, onScroll) {
  boundFile = appStore().currentFile
  sessionStart = Date.now()

  stopListening()
  scrollTarget = scrollEl
  scrollHandler = () => {
    onScroll?.(scrollEl)
    clearTimeout(scrollTimer)
    scrollTimer = setTimeout(() => capture(scrollEl), 500)
  }
  scrollEl.addEventListener('scroll', scrollHandler)

  clearInterval(saveTimer)
  saveTimer = setInterval(save, 3000)
  window.addEventListener('blur', flush)
  window.addEventListener('beforeunload', flush)
}

/** 立即保存（返回 Promise，供关闭窗口前 await） */
function flush() {
  return dirty ? save() : Promise.resolve()
}

/** 停止监听并保存（切换文件时调用） */
function stop() {
  stopListening()
  clearInterval(saveTimer)
  saveTimer = null
  clearTimeout(scrollTimer)
  flush()
}

export const progressController = { start, stop, restore, flush, save }
