import { useAppStore } from '../stores/app'
import zhCN from '../locales/zh-CN.json'
import enUS from '../locales/en-US.json'

const messages = { 'zh-CN': zhCN, 'en-US': enUS }

/**
 * 极简 i18n：t(key) 读取当前语言包，缺失回退中文。
 * 通过访问响应式 store.preferences.language，语言切换可即时生效。
 */
export function useI18n() {
  const store = useAppStore()
  const t = (key) => {
    const lang = store.preferences?.language || 'zh-CN'
    return messages[lang]?.[key] ?? zhCN[key] ?? key
  }
  return { t }
}
