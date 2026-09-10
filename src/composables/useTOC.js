import { useAppStore } from '../stores/app'

/**
 * 当前章节追踪：滚动时找出视口内最新的标题，更新目录高亮
 */
export function useTOC() {
  const store = useAppStore()

  function track(scrollEl) {
    const headings = scrollEl.querySelectorAll('h1, h2, h3, h4, h5, h6')
    const containerTop = scrollEl.getBoundingClientRect().top
    const threshold = scrollEl.scrollTop + 100
    let current = null

    for (const h of headings) {
      // 标题相对滚动容器的绝对位置
      const top = h.getBoundingClientRect().top - containerTop + scrollEl.scrollTop
      if (top <= threshold) current = h
      else break
    }
    store.activeHeadingId = current?.id || ''
  }

  return { track }
}
