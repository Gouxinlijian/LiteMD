import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/app'

const MIN_FONT = 12
const MAX_FONT = 28
const DEFAULT_FAMILY = "'Microsoft YaHei', 'Segoe UI', system-ui, sans-serif"

/**
 * 字号/字体族：通过 CSS 变量生效，变更持久化到设置
 */
export function useFont() {
  const store = useAppStore()

  function apply() {
    const size = store.preferences?.fontSize ?? 16
    document.documentElement.style.setProperty('--reader-font-size', `${size}px`)
  }

  /** 应用正文字体族 */
  function applyFamily() {
    const family = store.preferences?.fontFamily || DEFAULT_FAMILY
    document.documentElement.style.setProperty('--reader-font-family', family)
  }

  async function zoom(delta) {
    if (!store.preferences) return
    store.preferences.fontSize = Math.min(
      MAX_FONT,
      Math.max(MIN_FONT, store.preferences.fontSize + delta),
    )
    apply()
    await invoke('save_preferences', { prefs: store.preferences })
  }

  async function reset() {
    if (!store.preferences) return
    store.preferences.fontSize = 16
    apply()
    await invoke('save_preferences', { prefs: store.preferences })
  }

  return { apply, applyFamily, zoom, reset }
}
