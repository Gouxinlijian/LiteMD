<script setup>
import { ref, watch, nextTick, onBeforeUnmount } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/app'
import { useFiles } from '../composables/useFiles'
import {
  renderMarkdown,
  renderMarkdownChunked,
  buildTocFromDom,
} from '../composables/useMarkdown'
import { createMarkdownRenderer } from '../utils/markdownConfig'
import { useTOC } from '../composables/useTOC'
import { progressController } from '../composables/useProgress'
import { useI18n } from '../utils/i18n'
import { isAndroid } from '../utils/platform'

const store = useAppStore()
const { openDialog } = useFiles()
const toc = useTOC()
const { t } = useI18n()

const scrollEl = ref(null)
const contentEl = ref(null)
const lightboxSrc = ref('')
const showLightbox = ref(false)
const rendering = ref(false)
const renderPercent = ref(0)

/** 大文件分片渲染阈值（字符数） */
const LARGE_THRESHOLD = 50000

// 渲染代次标记：快速切换文件时忽略过期回调，防止旧文件内容插入新容器
let renderGen = 0
// 进度恢复标志：提前定位与渲染收尾都可能触发恢复，保证只执行一次
let progressApplied = false
// 渲染结果缓存：按文件路径缓存已渲染 HTML 与目录，切回已读文件时免重新解析渲染
const renderCache = new Map()
const CACHE_MAX = 5

// ---------- 文件切换：渲染 → 目录 → 恢复进度 ----------
watch(
  () => store.currentFile,
  async (file) => {
    // 1. 停止旧文件进度监听并保存
    progressController.stop()
    showLightbox.value = false
    rendering.value = false
    progressApplied = false

    if (!file) {
      contentEl.value && (contentEl.value.innerHTML = '')
      store.toc = []
      return
    }

    // 2. 等待组件更新，确保阅读区 DOM 就绪（首次打开时 article 尚未挂载）
    const gen = ++renderGen
    await nextTick()
    if (!contentEl.value || gen !== renderGen) return

    // 3. 命中渲染缓存：直接恢复已渲染内容（含目录、代码按钮、已解析图片），免重新解析
    const cached = renderCache.get(file.path)
    if (cached && cached.hash === file.hash) {
      contentEl.value.innerHTML = cached.html
      store.toc = cached.toc
      decorateCodeBlocks(true)
      applyProgress()
      updateDisplayPercent(scrollEl.value)
      return
    }

    // 4. 渲染 Markdown（大文件分片，避免 UI 冻结）
    contentEl.value.innerHTML = ''
    store.toc = []
    try {
      if (file.content.length > LARGE_THRESHOLD) {
        rendering.value = true
        renderPercent.value = 0
        renderMarkdownChunked(
          file.content,
          (chunk, percent) => {
            if (gen !== renderGen) return
            contentEl.value.insertAdjacentHTML('beforeend', chunk)
            renderPercent.value = percent
            // 边渲染边构建目录：标题块一旦出现，大纲立即可用
            updateTocProgressively()
            // 内容已渲染到上次阅读位置附近则立即定位，无需等全文渲染完
            applyProgressEarly()
          },
          () => {
            if (gen !== renderGen) return
            rendering.value = false
            finalize(file)
          },
        )
      } else {
        contentEl.value.innerHTML = renderMarkdown(file.content)
        finalize(file)
      }
    } catch (e) {
      console.error('渲染 Markdown 失败', e)
      rendering.value = false
      contentEl.value.innerHTML = `<p class="render-error">⚠️ ${t('renderError')}：${e.message}</p>`
    }
  },
)

// 外部触发图片放大（如右键菜单「放大图片」）
watch(
  () => store.lightboxSrc,
  (src) => {
    if (src) {
      lightboxSrc.value = src
      showLightbox.value = true
    }
  },
)

// ---------- 进度恢复与滚动监听 ----------

/** 恢复进度 + 启动滚动监听；幂等，提前定位与渲染收尾都可能触发 */
function applyProgress() {
  if (progressApplied) return
  progressApplied = true
  progressController.restore(scrollEl.value)
  progressController.start(scrollEl.value, onScroll)
  // 立即同步当前章节，驱动大纲高亮与跟随定位（不等 scroll 事件）
  toc.track(scrollEl.value)
}

/**
 * 分片渲染中提前恢复进度：文件未修改时按绝对像素定位，只要已插入内容高度
 * 达到目标滚动位置即可定位（后续块追加在文档底部，不影响已定位位置）；
 * 已修改文件按百分比恢复依赖完整 scrollHeight，仍在渲染完成后恢复。
 */
function applyProgressEarly() {
  const p = store.currentFile?.progress
  if (progressApplied || !p || !p.scrollTop) return
  if (p.fileHash !== store.currentFile.hash) return
  const sc = scrollEl.value
  if (!sc) return
  if (sc.scrollHeight - sc.clientHeight >= p.scrollTop) applyProgress()
}

// ---------- 渲染完成后的统一收尾 ----------

async function finalize(file) {
  try {
    // 4. 构建目录树 + 锚点，解析本地图片，装饰代码块，插入 [TOC]
    await nextTick()
    store.toc = buildTocFromDom(contentEl.value)
    resolveImages(file.dir)
    decorateCodeBlocks()
    insertToc(store.toc)
    // 5. 恢复进度并启动监听（滚动时同步章节高亮与显示百分比）
    applyProgress()
    // 6. 渲染完成后按完整高度重算一次进度，修正提前定位时被钳制的百分比
    updateDisplayPercent(scrollEl.value)
    // 7. 缓存渲染结果，切回本文件时免重新渲染
    cacheRender(file)
  } catch (e) {
    console.error('渲染收尾失败', e)
    rendering.value = false
  }
}

/** 缓存渲染结果（路径为键、内容哈希校验）；超过上限时淘汰最早加入的 */
function cacheRender(file) {
  if (renderCache.size >= CACHE_MAX) {
    const oldest = renderCache.keys().next().value
    renderCache.delete(oldest)
  }
  renderCache.set(file.path, {
    hash: file.hash,
    html: contentEl.value.innerHTML,
    toc: store.toc,
  })
}

/** 相对路径图片基于文件目录解析为可加载 URL */
function resolveImages(dir) {
  contentEl.value.querySelectorAll('img').forEach((img) => {
    if (img.dataset.resolved) return
    img.dataset.resolved = '1'
    const src = img.getAttribute('src')
    if (!src || /^(https?:|data:)/.test(src) || !dir) return
    const rel = decodeURIComponent(src).replace(/\\/g, '/').replace(/^\.\//, '')
    img.src = convertFileSrc(`${dir.replace(/\\/g, '/')}/${rel}`)
    img.loading = 'lazy'
  })
}

/** 为代码块添加复制按钮 */
function decorateCodeBlocks(force = false) {
  contentEl.value.querySelectorAll('pre > code').forEach((code) => {
    const pre = code.parentElement
    if (!force && pre.dataset.decorated) return
    pre.dataset.decorated = '1'
    // 恢复缓存时旧按钮无事件监听，删除后重新绑定
    pre.querySelector('.copy-btn')?.remove()
    const btn = document.createElement('button')
    btn.className = 'copy-btn'
    btn.textContent = t('copy')
    btn.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(code.textContent)
        btn.textContent = t('copied')
        setTimeout(() => (btn.textContent = t('copy')), 1200)
      } catch {
        /* 剪贴板不可用时静默 */
      }
    })
    pre.appendChild(btn)
  })
}

/** 分片渲染过程中逐步构建目录：标题一旦出现大纲即可用 */
function updateTocProgressively() {
  if (!contentEl.value) return
  const built = buildTocFromDom(contentEl.value)
  if (built.length >= store.toc.length) {
    store.toc = built
    insertToc(built)
  }
}

/** 把文档内 [TOC] 占位替换（或更新）为实际目录树 */
function insertToc(tocTree) {
  const container =
    contentEl.value.querySelector('.toc') || contentEl.value.querySelector('[data-toc]')
  if (!container || !tocTree.length) return
  const escape = createMarkdownRenderer().utils.escapeHtml
  const renderList = (nodes) =>
    `<ul>${nodes
      .map(
        (n) =>
          `<li><a href="#${n.id}">${escape(n.text)}</a>${n.children?.length ? renderList(n.children) : ''}</li>`,
      )
      .join('')}</ul>`
  container.outerHTML = `<div class="toc"><div class="toc-title">${t('toc')}</div>${renderList(tocTree)}</div>`
}

/** 同步状态栏与最近列表的实时进度百分比 */
function updateDisplayPercent(el) {
  if (!store.currentFile) return
  const docHeight = el.scrollHeight - el.clientHeight
  const percent = docHeight > 0 ? (el.scrollTop / docHeight) * 100 : 0
  store.currentFile.progress.progressPercent = percent
  // 同步最近列表的进度显示，避免与状态栏不一致
  const recent = store.recentFiles.find((f) => f.path === store.currentFile.path)
  if (recent) recent.progressPercent = percent
}

/** 滚动回调：兜底重建目录 + 章节追踪 + 进度显示 */
function onScroll(el) {
  // 兜底：若目录未构建成功（如分片渲染曾被中断），滑动时重建
  if (!store.toc.length && contentEl.value) {
    const built = buildTocFromDom(contentEl.value)
    if (built.length) {
      store.toc = built
      insertToc(built)
    }
  }
  toc.track(el)
  updateDisplayPercent(el)
}

// ---------- 交互 ----------

/** 锚点平滑跳转 + 图片放大 */
function onContentClick(e) {
  const anchor = e.target.closest('a[href^="#"]')
  if (anchor) {
    e.preventDefault()
    const id = decodeURIComponent(anchor.getAttribute('href').slice(1))
    const el = document.getElementById(id)
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' })
    return
  }
  if (e.target.tagName === 'IMG' && e.target.src) {
    e.preventDefault()
    lightboxSrc.value = e.target.src
    showLightbox.value = true
    return
  }
  // 移动端：点击正文空白切换沉浸全屏阅读（浏览器滑动会取消 click，所以滑动不触发）
  if (isAndroid) store.mobileImmersive = !store.mobileImmersive
}

/** 空格翻页 */
function scrollByPage(dir) {
  if (!scrollEl.value) return
  scrollEl.value.scrollBy({ top: dir * scrollEl.value.clientHeight * 0.9, behavior: 'auto' })
}

onBeforeUnmount(() => progressController.stop())

defineExpose({ scrollByPage, scrollEl })
</script>

<template>
  <main class="reader">
    <div v-if="store.hasFile" ref="scrollEl" class="reader-scroll">
      <article ref="contentEl" class="reader-content" @click="onContentClick" />
      <div v-if="rendering" class="rendering-overlay">
        <div class="spinner"></div>
        <div class="rendering-text">{{ t('rendering') }}</div>
        <div class="rendering-bar">
          <div class="rendering-fill" :style="{ width: renderPercent + '%' }"></div>
        </div>
      </div>
    </div>
    <div v-else class="reader-empty">
      <div class="empty-logo">LiteMD</div>
      <p class="empty-text">{{ t('emptyHint') }}</p>
      <button class="empty-btn" @click="openDialog">{{ t('openFile') }}</button>
    </div>

    <!-- 图片放大 -->
    <transition name="fade">
      <div v-if="showLightbox" class="lightbox" @click="showLightbox = false">
        <img :src="lightboxSrc" alt="" />
      </div>
    </transition>
  </main>
</template>
