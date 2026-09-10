import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/app'

const CYCLE = ['light', 'dark', 'auto']

/**
 * 主题管理：auto（跟随系统）/ light / dark
 * 通过 <html data-theme="dark"> 切换 CSS 变量
 */
export function useTheme() {
  const store = useAppStore()
  const media = window.matchMedia('(prefers-color-scheme: dark)')

  function apply() {
    const theme = store.preferences?.theme || 'auto'
    const dark = theme === 'dark' || (theme === 'auto' && media.matches)
    store.theme = dark ? 'dark' : 'light'
    document.documentElement.dataset.theme = store.theme
  }

  /** 循环切换 浅色 → 深色 → 跟随系统 */
  async function toggle() {
    const cur = store.preferences?.theme || 'auto'
    store.preferences.theme = CYCLE[(CYCLE.indexOf(cur) + 1) % CYCLE.length]
    apply()
    await invoke('save_preferences', { prefs: store.preferences })
  }

  /** 监听系统主题变化（仅 auto 模式生效） */
  function watchSystem() {
    media.addEventListener('change', apply)
  }

  return { apply, toggle, watchSystem }
}
