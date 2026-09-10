import { ref } from 'vue'

/**
 * 文内搜索：遍历文本节点包 <mark>，支持上一个/下一个跳转
 */
export function useSearch() {
  const query = ref('')
  const matches = ref([])
  const current = ref(0)

  /** 清除既有高亮，恢复原始文本 */
  function clear(container) {
    if (!container) return
    container.querySelectorAll('mark.search-hit').forEach((m) => {
      m.replaceWith(m.textContent)
    })
    matches.value = []
    current.value = 0
  }

  /** 执行搜索并高亮所有匹配 */
  function run(container, text) {
    clear(container)
    query.value = text
    if (!text) return

    const lower = text.toLowerCase()
    const marks = []

    // 遍历正文文本节点
    const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT)
    while (walker.nextNode()) {
      const node = walker.currentNode
      const nodeText = node.nodeValue || ''
      const nodeLower = nodeText.toLowerCase()

      // 从后往前分割，避免索引起点漂移
      const positions = []
      let from = 0
      let idx
      while ((idx = nodeLower.indexOf(lower, from)) !== -1) {
        positions.push([idx, idx + text.length])
        from = idx + text.length
      }

      for (let i = positions.length - 1; i >= 0; i--) {
        const [start, end] = positions[i]
        const range = document.createRange()
        range.setStart(node, start)
        range.setEnd(node, end)
        const mark = document.createElement('mark')
        mark.className = 'search-hit'
        range.surroundContents(mark)
        marks.push(mark)
      }
    }

    matches.value = marks
    if (marks.length) goto(container, 0)
  }

  /** 跳到第 n 个匹配 */
  function goto(container, n) {
    const len = matches.value.length
    if (!len) return
    current.value = ((n % len) + len) % len
    const el = matches.value[current.value]
    // 逐个清除 current 高亮类
    matches.value.forEach((m) => m.classList.remove('current'))
    el.classList.add('current')
    el.scrollIntoView({ block: 'center', behavior: 'smooth' })
  }

  function next(container) {
    goto(container, current.value + 1)
  }

  function prev(container) {
    goto(container, current.value - 1)
  }

  function close(container) {
    clear(container)
    query.value = ''
  }

  return { query, matches, current, run, next, prev, close, clear }
}
