import MarkdownIt from 'markdown-it'
import taskLists from 'markdown-it-task-lists'
import hljs from 'highlight.js'

let renderer = null

/**
 * 创建 markdown-it 渲染器（单例，复用解析缓存）。
 * 支持 GFM（表格/任务列表/删除线）、代码高亮。
 * 注：markdown-it-footnote 与 markdown-it 15 不兼容，脚注不再单独支持。
 */
export function createMarkdownRenderer() {
  if (renderer) return renderer

  const md = new MarkdownIt({
    html: true,
    linkify: true,
    breaks: false,
    highlight(str, lang) {
      if (lang && hljs.getLanguage(lang)) {
        try {
          return hljs.highlight(str, { language: lang, ignoreIllegals: true }).value
        } catch {
          /* 高亮失败回退到转义 */
        }
      }
      return md.utils.escapeHtml(str)
    },
  })

  md.use(taskLists, { label: true, enabled: true })

  renderer = md
  return md
}
