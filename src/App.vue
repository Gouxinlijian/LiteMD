<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from './stores/app'
import { useFiles } from './composables/useFiles'
import { useTheme } from './composables/useTheme'
import { useFont } from './composables/useFont'
import { progressController } from './composables/useProgress'
import { contextMenu } from './composables/useContextMenu'
import { useI18n } from './utils/i18n'
import { isAndroid } from './utils/platform'
import TopBar from './components/TopBar.vue'
import Sidebar from './components/Sidebar.vue'
import Reader from './components/Reader.vue'
import StatusBar from './components/StatusBar.vue'
import SearchBar from './components/SearchBar.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import ContextMenu from './components/ContextMenu.vue'
import MobileLayout from './components/mobile/MobileLayout.vue'

const store = useAppStore()
const { openDialog, openByPath, refreshRecent, loadFile } = useFiles()
const theme = useTheme()
const font = useFont()
const { t } = useI18n()

const isDragging = ref(false)
const showFsToolbar = ref(false)
const readerRef = ref(null)
let unlistenOpen = null
let unlistenDrop = null
let unlistenClose = null
let unlistenWifi = null
let incomingTimer = null
// 轮询报错去重：同一错误只弹一次，避免刷屏
let lastPollError = ''

onMounted(async () => {
  // 1. 加载设置并应用
  store.preferences = await invoke('load_preferences')
  store.sidebarVisible = store.preferences.sidebarVisible
  store.sidebarWidth = store.preferences.sidebarWidth
  store.sidebarActiveTab = store.preferences.sidebarActiveTab
  theme.apply()
  theme.watchSystem()
  font.apply()
  font.applyFamily()
  applyTypography()
  await refreshRecent()

  // 2. 桌面：命令行参数打开优先，否则恢复上次阅读文档
  //    安卓：恢复上次打开的文档（私有目录路径持久有效）
  if (!isAndroid) {
    const startup = await invoke('get_startup_file')
    if (startup) {
      await openByPath(startup)
    } else if (store.preferences.lastOpenedPath) {
      try {
        await openByPath(store.preferences.lastOpenedPath)
      } catch {
        /* 上次文件已不存在，忽略 */
      }
    }
  } else if (store.preferences?.lastOpenedPath) {
    try {
      await openByPath(store.preferences.lastOpenedPath)
    } catch {
      /* 上次文件已不存在，忽略 */
    }
  }

  // 3. 桌面：单实例复用窗口打开新文件
  if (!isAndroid) {
    unlistenOpen = await listen('open-file', async (e) => {
      try {
        await openByPath(e.payload)
      } catch {
        /* 文件可能已不存在，忽略 */
      }
    })
  }

  // 4/5. 桌面：拖拽打开 + 窗口关闭前强写进度（安卓无窗口事件，用 visibilitychange）
  if (!isAndroid) {
    const win = getCurrentWindow()
    unlistenDrop = await win.onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        isDragging.value = true
      } else if (event.payload.type === 'drop') {
        isDragging.value = false
        const path = event.payload.paths?.[0]
        if (path) openByPath(path)
      } else {
        isDragging.value = false
      }
    })
    unlistenClose = await win.onCloseRequested(async (event) => {
      event.preventDefault()
      await progressController.flush()
      await win.destroy()
    })
    // 6. 全屏时鼠标移近顶缘显示迷你工具栏
    window.addEventListener('mousemove', onMouseMove)
  } else {
    // 安卓：页面切后台时强写进度，防 OriginOS 杀进程丢进度
    window.addEventListener('visibilitychange', onVisibilityChange)

    // 3a. 接收「其他应用打开 / 分享」导入：Kotlin 已复制到私有目录，这里定时拉取结果
    incomingTimer = window.setInterval(async () => {
      try {
        const payload = await invoke('pull_incoming_file')
        if (payload) await loadFile(payload)
        lastPollError = ''
      } catch (e) {
        if (String(e) !== lastPollError) {
          lastPollError = String(e)
          alert(`接收文件失败：${e}`)
        }
      }
    }, 2000)

    // 3b. Wi-Fi 电脑传书：文件落盘后自动打开
    unlistenWifi = await listen('wifi-received', async (e) => {
      const path = e.payload?.path
      if (path) {
        try {
          await openByPath(path)
        } catch {
          /* 打开失败静默 */
        }
      }
    })
  }

  window.addEventListener('keydown', onKeydown)
  // 7. Ctrl+滚轮缩放字号
  window.addEventListener('wheel', onWheel, { passive: false })
  // 8. 统一右键菜单
  window.addEventListener('contextmenu', onContextMenu)
})

/** 安卓切后台/隐藏页面时立即落盘进度 */
function onVisibilityChange() {
  if (document.hidden) progressController.flush()
}

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', onMouseMove)
  window.removeEventListener('keydown', onKeydown)
  window.removeEventListener('wheel', onWheel)
  window.removeEventListener('contextmenu', onContextMenu)
  window.removeEventListener('visibilitychange', onVisibilityChange)
  unlistenOpen?.()
  unlistenDrop?.()
  unlistenClose?.()
  unlistenWifi?.()
  if (incomingTimer) clearInterval(incomingTimer)
})

/** Ctrl+滚轮：上下滚动缩放字号 */
function onWheel(e) {
  if (!e.ctrlKey) return
  e.preventDefault()
  font.zoom(e.deltaY < 0 ? 1 : -1)
}

// ---------- 统一右键菜单 ----------

/** 全局右键：输入框保留原生菜单，其余按点击目标构建菜单 */
function onContextMenu(e) {
  const tag = e.target?.tagName
  if (tag === 'INPUT' || tag === 'TEXTAREA') return
  e.preventDefault()

  const items = buildContextItems(e.target)
  if (!items?.length) return
  const x = Math.min(e.clientX, window.innerWidth - 200)
  const y = Math.min(e.clientY, window.innerHeight - items.length * 34 - 20)
  contextMenu.show(x, y, items)
}

function buildContextItems(target) {
  const recentItem = target.closest('.recent-item')
  if (recentItem) return buildRecentItems(recentItem)
  const outlineItem = target.closest('.outline-item')
  if (outlineItem) return buildOutlineItems(outlineItem)
  if (target.closest('.reader-content')) return buildReaderItems(target)
  return buildGlobalItems()
}

function buildRecentItems(el) {
  const path = el.getAttribute('title') || ''
  const file = store.recentFiles.find((f) => f.path === path)
  return [
    { label: t('openFile'), icon: '📂', action: () => openByPath(path) },
    { label: file?.starred ? t('unstar') : t('star'), icon: '★', action: () => toggleRecentStar(path, file?.starred) },
    { label: t('showInFolder'), icon: '📁', action: () => invoke('reveal_in_explorer', { path }) },
    { divider: true },
    { label: t('remove'), icon: '🗑', danger: true, action: () => removeRecent(path) },
  ]
}

function buildOutlineItems(el) {
  const id = el.getAttribute('data-id') || ''
  return [
    { label: t('jumpTo'), icon: '⤵', action: () => jumpToHeading(id) },
    { label: t('copy'), icon: '📋', action: () => copyText(el.textContent.trim()) },
  ]
}

function buildReaderItems(target) {
  if (target.tagName === 'IMG' && target.src) {
    return [
      { label: t('viewImage'), icon: '🖼', action: () => (store.lightboxSrc = target.src) },
      { label: t('copy'), icon: '📋', action: () => copyText(target.src) },
    ]
  }
  const link = target.closest('a[href^="http"]')
  if (link) {
    return [{ label: t('copyLink'), icon: '🔗', action: () => copyText(link.href) }]
  }
  return [
    { label: t('copy'), icon: '📋', action: () => copyText(window.getSelection()?.toString() || '') },
    { label: t('selectAll'), icon: '⊞', action: selectAllText },
    { label: t('searchSelection'), icon: '🔍', action: searchSelection },
    { divider: true },
    { label: t('top'), icon: '⬆', action: () => scrollToTop() },
    { label: t('bottom'), icon: '⬇', action: () => scrollToBottom() },
    { divider: true },
    { label: `${t('fontSize')} +`, icon: 'A', action: () => font.zoom(1) },
    { label: `${t('fontSize')} −`, icon: 'A', action: () => font.zoom(-1) },
    { divider: true },
    { label: t('settings'), icon: '⚙️', action: () => (store.settingsOpen = true) },
  ]
}

function buildGlobalItems() {
  return [
    { label: t('openFile'), icon: '📂', action: () => openDialog() },
    { label: t('theme'), icon: store.theme === 'dark' ? '☀️' : '🌙', action: () => theme.toggle() },
    { label: t('settings'), icon: '⚙️', action: () => (store.settingsOpen = true) },
  ]
}

async function toggleRecentStar(path, starred) {
  await invoke('set_recent_starred', { path, starred: !starred })
  await refreshRecent()
}

async function removeRecent(path) {
  await invoke('remove_recent_file', { path })
  await refreshRecent()
}

async function copyText(text) {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    /* 剪贴板不可用时静默 */
  }
}

function selectAllText() {
  const el = document.querySelector('.reader-content')
  if (!el) return
  const range = document.createRange()
  range.selectNodeContents(el)
  const sel = window.getSelection()
  sel.removeAllRanges()
  sel.addRange(range)
}

function searchSelection() {
  const sel = window.getSelection()?.toString().trim()
  if (!sel) return
  store.searchPrefill = sel
  store.searchOpen = true
}

function scrollToTop() {
  document.querySelector('.reader-scroll')?.scrollTo({ top: 0 })
}

function scrollToBottom() {
  const el = document.querySelector('.reader-scroll')
  if (el) el.scrollTo({ top: el.scrollHeight })
}

function jumpToHeading(id) {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

// ---------- 排版应用 ----------

/** 应用行距与内容宽度（0 = 铺满窗口） */
function applyTypography() {
  const p = store.preferences
  document.documentElement.style.setProperty('--reader-line-height', String(p.lineHeight))
  document.documentElement.style.setProperty('--reader-max-width', p.maxContentWidth ? p.maxContentWidth + 'px' : 'none')
}

// ---------- 全屏 ----------
async function toggleFullscreen() {
  if (isAndroid) return
  const win = getCurrentWindow()
  if (store.isFullscreen) {
    store.isFullscreen = false
    store.sidebarVisible = true
    await win.setFullscreen(false)
  } else {
    store.sidebarVisible = false
    store.isFullscreen = true
    await win.setFullscreen(true)
  }
}

function onMouseMove(e) {
  if (!store.isFullscreen) return
  showFsToolbar.value = e.clientY < 8
}

// ---------- 快捷键 ----------
function onKeydown(e) {
  const tag = e.target?.tagName
  if (tag === 'INPUT' || tag === 'TEXTAREA') return

  const ctrl = e.ctrlKey || e.metaKey
  const key = e.key.toLowerCase()

  switch (true) {
    case e.key === 'Escape':
      if (store.isFullscreen) toggleFullscreen()
      else if (store.searchOpen) store.searchOpen = false
      else if (store.settingsOpen) store.settingsOpen = false
      return
    case e.key === 'F11':
      e.preventDefault()
      toggleFullscreen()
      return
    case ctrl && key === 'o':
      e.preventDefault()
      openDialog()
      return
    case ctrl && key === 'f':
      e.preventDefault()
      store.searchOpen = true
      return
    case ctrl && key === 'j':
      e.preventDefault()
      store.sidebarVisible = !store.sidebarVisible
      return
    case ctrl && key === 'k':
      e.preventDefault()
      store.sidebarActiveTab = 'outline'
      store.sidebarVisible = true
      return
    case ctrl && key === 'r':
      e.preventDefault()
      store.sidebarActiveTab = 'recent'
      store.sidebarVisible = true
      return
    case ctrl && (key === '=' || key === '+'):
      e.preventDefault()
      font.zoom(1)
      return
    case ctrl && key === '-':
      e.preventDefault()
      font.zoom(-1)
      return
    case ctrl && key === '0':
      e.preventDefault()
      font.reset()
      return
    case ctrl && key === ',':
      e.preventDefault()
      store.settingsOpen = true
      return
    case ctrl && e.key === 'Tab':
      e.preventDefault()
      openNextRecent()
      return
    case e.key === ' ':
      e.preventDefault()
      readerRef.value?.scrollByPage(e.shiftKey ? -1 : 1)
      return
  }
}

/** 依次打开最近文件列表中的下一个文件 */
async function openNextRecent() {
  const files = store.recentFiles
  if (!files.length) return
  const cur = store.currentFile?.path
  const idx = files.findIndex((f) => f.path === cur)
  await openByPath(files[(idx + 1) % files.length].path)
}
</script>

<template>
  <!-- 安卓：移动端布局（全屏正文 + 底部工具条 + 抽屉） -->
  <MobileLayout v-if="isAndroid">
    <Reader ref="readerRef" />
  </MobileLayout>

  <!-- 桌面：原有布局 -->
  <div v-else class="app" :class="{ 'is-fullscreen': store.isFullscreen }">
    <!-- 全屏迷你工具栏 -->
    <transition name="fs-toolbar">
      <div v-if="store.isFullscreen && showFsToolbar" class="fs-toolbar" @mouseleave="showFsToolbar = false">
        <button class="fs-btn" :title="t('outline')" @click="store.sidebarActiveTab = 'outline'; store.sidebarVisible = true">📋</button>
        <button class="fs-btn" :title="t('search')" @click="store.searchOpen = true">🔍</button>
        <button class="fs-btn" :title="t('theme')" @click="theme.toggle()">{{ store.theme === 'dark' ? '☀️' : '🌙' }}</button>
        <button class="fs-btn" :title="t('settings')" @click="store.settingsOpen = true">⚙️</button>
        <button class="fs-btn" :title="t('exitFullscreen')" @click="toggleFullscreen">⛶</button>
      </div>
    </transition>

    <TopBar v-show="!store.isFullscreen" @open-settings="store.settingsOpen = true" />

    <div class="app-body">
      <Sidebar v-show="store.sidebarVisible" />
      <Reader ref="readerRef" />
    </div>

    <StatusBar v-show="!store.isFullscreen" />

    <!-- 拖拽遮罩 -->
    <div v-if="isDragging" class="drop-overlay">
      <div class="drop-hint">📂 {{ t('dropHint') }}</div>
    </div>
  </div>

  <SearchBar v-show="store.searchOpen" />
  <SettingsPanel v-if="store.settingsOpen" @close="store.settingsOpen = false" />
  <ContextMenu />
</template>
