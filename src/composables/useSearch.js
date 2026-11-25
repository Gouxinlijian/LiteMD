import { ref } from 'vue'

/** 单次高亮上限：命中过多时只高亮前 N 处，避免超长文档插入海量节点导致界面卡死 */
const MAX_HIGHLIGHT = 1000
/** 这些标签内的文本不参与搜索，避免命中样式/脚本内容 */
const SKIP_TAGS = new Set(['SCRIPT', 'STYLE', 'NOSCRIPT', 'TEXTAREA'])

/**
 * 文内搜索：遍历文本节点包 <mark>，支持上一个/下一个跳转
 *
 * 性能要点（旧实现会在长文档上卡死）：
 * 1. 先纯遍历收集命中位置，再统一改写 DOM，避免"边遍历边插入"造成节点反复遍历与嵌套；
 * 2. 每个文本节点只做一次 replaceChild（DocumentFragment 批量插入），
 *    而不是每处命中调一次 range.surroundContents 触发多次拆分与重排；
 * 3. 高亮数量封顶，命中再多也不会插入海量节点；
 * 4. 跳转用瞬时滚动，避免 smooth 在长文档里连续触发滚动回调。
 */
export function useSearch() {
  const query = ref('')
  const matches = ref([]) // 已高亮的 <mark>（最多 MAX_HIGHLIGHT 个）
  const current = ref(0)
  const total = ref(0) // 全文命中总数，可能大于已高亮数量

  let containerRef = null
  let currentEl = null

  /** 清除既有高亮，恢复原始文本 */
  function clear(container) {
    const root = container || containerRef
    if (root) {
      // 用子节点替换而非 textContent，嵌套 mark 也能逐层拆干净
      root.querySelectorAll('mark.search-hit').forEach((m) => {
        m.replaceWith(...m.childNodes)
      })
    }
    matches.value = []
    current.value = 0
    total.value = 0
    currentEl = null
  }

  /** 执行搜索并高亮匹配 */
  function run(container, text) {
    containerRef = container || containerRef
    clear(containerRef)
    query.value = text
    if (!containerRef || !text) return

    const qLen = text.length
    const lower = text.toLowerCase()
    const jobs = [] // [文本节点, [[start, end], ...]]
    let budget = MAX_HIGHLIGHT
    let totalCount = 0

    // 第一遍：只遍历、只计算位置，不修改 DOM
    const walker = document.createTreeWalker(containerRef, NodeFilter.SHOW_TEXT, {
      acceptNode(node) {
        if (!node.nodeValue) return NodeFilter.FILTER_REJECT
        const parent = node.parentElement
        if (!parent || SKIP_TAGS.has(parent.tagName)) return NodeFilter.FILTER_REJECT
        return NodeFilter.FILTER_ACCEPT
      },
    })

    while (walker.nextNode()) {
      const node = walker.currentNode
      const nodeLower = (node.nodeValue || '').toLowerCase()
      let positions = null
      let from = 0
      let idx
      while ((idx = nodeLower.indexOf(lower, from)) !== -1) {
        totalCount++
        if (budget > 0) {
          if (!positions) positions = []
          positions.push([idx, idx + qLen])
          budget--
        }
        from = idx + qLen
      }
      if (positions) jobs.push([node, positions])
    }

    total.value = totalCount
    if (!totalCount) return

    // 第二遍：每个文本节点一次性替换，节点数不变形、重排次数最少
    const marks = []
    for (const [node, positions] of jobs) {
      const parent = node.parentNode
      if (!parent) continue
      const source = node.nodeValue || ''
      const frag = document.createDocumentFragment()
      let cursor = 0
      for (const [start, end] of positions) {
        if (start > cursor) frag.appendChild(document.createTextNode(source.slice(cursor, start)))
        const mark = document.createElement('mark')
        mark.className = 'search-hit'
        mark.textContent = source.slice(start, end)
        frag.appendChild(mark)
        marks.push(mark)
        cursor = end
      }
      if (cursor < source.length) frag.appendChild(document.createTextNode(source.slice(cursor)))
      parent.replaceChild(frag, node)
    }

    matches.value = marks
    if (marks.length) goto(containerRef, 0)
  }

  /** 跳到第 n 个匹配 */
  function goto(container, n) {
    const len = matches.value.length
    if (!len) return
    const idx = ((n % len) + len) % len
    current.value = idx
    // 只改动前后两个节点的高亮类，避免每次遍历全部命中
    currentEl?.classList.remove('current')
    const el = matches.value[idx]
    currentEl = el
    el.classList.add('current')
    el.scrollIntoView({ block: 'center' })
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

  return { query, matches, current, total, run, next, prev, close, clear }
}
