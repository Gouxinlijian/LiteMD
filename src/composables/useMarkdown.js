import { createMarkdownRenderer } from '../utils/markdownConfig'
import { slugify } from '../utils/slugify'

/** [TOC] 占位符（渲染后由 insertToc 替换为实际目录） */
const TOC_PLACEHOLDER = '<div data-toc></div>'

/** 渲染前处理：[TOC] 单独行替换为占位 div */
function preprocess(content) {
  return String(content || '').replace(/^\s*\[TOC\]\s*$/gm, TOC_PLACEHOLDER)
}

/**
 * 渲染 Markdown 为 HTML（同步，适合中小文件）
 */
export function renderMarkdown(content) {
  return createMarkdownRenderer().render(preprocess(content))
}

/**
 * 大文件分片渲染：按顶层块切分，每帧渲染若干块插入 DOM，
 * 避免一次性渲染超大文档时 UI 冻结。onChunk(chunkHtml, percent) 逐批回调。
 * 注意：requestAnimationFrame 在窗口失焦/后台时可能长时间不触发，
 * 因此用 setTimeout 兜底保证渲染持续推进。
 */
export function renderMarkdownChunked(content, onChunk, onDone) {
  const md = createMarkdownRenderer()
  const tokens = md.parse(preprocess(content), {})
  const chunks = splitTopLevel(tokens)
  const BATCH = 4 // 每帧渲染的块数
  let i = 0

  function renderNext() {
    const end = Math.min(i + BATCH, chunks.length)
    for (; i < end; i++) {
      onChunk(md.renderer.render(chunks[i], md.options, {}), Math.round((i / chunks.length) * 100))
    }
    if (i < chunks.length) schedule()
    else onDone()
  }

  function schedule() {
    let fired = false
    const timer = setTimeout(() => {
      if (!fired) {
        fired = true
        renderNext()
      }
    }, 100)
    requestAnimationFrame(() => {
      if (!fired) {
        fired = true
        clearTimeout(timer)
        renderNext()
      }
    })
  }

  schedule()
}

/**
 * 把 tokens 按顶层块切分。顶层块以 _open/_close 配对为准；
 * fence/code/hr/html 等自包含块单独成块，保证每块渲染完整。
 */
function splitTopLevel(tokens) {
  const SELF_CONTAINED = new Set(['fence', 'code_block', 'hr', 'html_block'])
  const chunks = []
  let current = []
  let depth = 0

  for (const tok of tokens) {
    // 自包含顶层块：单独成块
    if (tok.level === 0 && SELF_CONTAINED.has(tok.type)) {
      if (current.length) {
        chunks.push(current)
        current = []
      }
      chunks.push([tok])
      continue
    }
    current.push(tok)
    if (tok.type.endsWith('_open')) depth++
    else if (tok.type.endsWith('_close')) {
      depth--
      if (depth === 0) {
        chunks.push(current)
        current = []
      }
    }
  }
  if (current.length) chunks.push(current)
  return chunks
}

/**
 * 渲染完成后从 DOM 提取标题、分配唯一锚点 id、构建目录树。
 * 通过 DOM 遍历（而非 token 解析）保证 id 与页面元素 100% 一致。
 */
export function buildTocFromDom(container) {
  const headings = container.querySelectorAll('h1, h2, h3, h4, h5, h6')
  const root = []
  const stack = []
  const used = new Set()

  for (const el of headings) {
    const level = Number(el.tagName[1])
    const text = (el.textContent || '').trim() || el.textContent || ''
    const base = slugify(text)

    // 去重：同名标题追加序号
    let id = base
    let n = 2
    while (used.has(id)) id = `${base}-${n++}`
    used.add(id)
    el.id = id

    // 构建树形结构：同一层级弹栈
    const node = { level, text, id, children: [] }
    while (stack.length && stack[stack.length - 1].level >= level) stack.pop()
    if (!stack.length) root.push(node)
    else stack[stack.length - 1].children.push(node)
    stack.push(node)
  }

  return root
}
