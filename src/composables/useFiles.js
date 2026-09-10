import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/app'
import { useI18n } from '../utils/i18n'

const MD_EXT = /\.(md|markdown)$/i

/**
 * 文件操作：对话框打开 / 按路径打开 / 最近文件
 */
export function useFiles() {
  const store = useAppStore()
  const { t } = useI18n()

  async function refreshRecent() {
    store.recentFiles = await invoke('get_recent_files')
  }

  /** 打开文件对话框（默认定位上次打开的目录）；失败时明确提示，避免静默无反应 */
  async function openDialog() {
    try {
      const payload = await invoke('open_file_dialog', {
        lastDir: store.preferences?.lastDir ?? null,
      })
      if (payload) await loadFile(payload)
    } catch (e) {
      alert(`打开文件失败：${e}`)
    }
  }

  /** 按路径打开（命令行参数 / 最近文件 / 拖拽），仅支持 Markdown 文件 */
  async function openByPath(path) {
    if (!MD_EXT.test(path)) {
      alert(t('onlyMd'))
      return
    }
    const payload = await invoke('read_markdown_file', { path })
    await loadFile(payload)
  }

  /** 统一装载：更新当前文件 + 记录最近文件 + 切到大纲 Tab 定位阅读位置 */
  async function loadFile(payload) {
    store.currentFile = payload
    store.toc = []
    store.activeHeadingId = ''
    store.searchOpen = false
    store.sidebarActiveTab = 'outline'
    store.sidebarVisible = true

    await invoke('add_recent_file', {
      path: payload.path,
      name: payload.name,
      dir: payload.dir,
      size: payload.size,
      progressPercent: payload.progress?.progressPercent ?? 0,
    })
    // 记住上次打开目录与文件，供文件对话框定位、启动时恢复
    if (store.preferences) {
      store.preferences.lastDir = payload.dir
      store.preferences.lastOpenedPath = payload.path
      await invoke('save_preferences', { prefs: store.preferences })
    }
    await refreshRecent()
  }

  return { refreshRecent, openDialog, openByPath, loadFile }
}
