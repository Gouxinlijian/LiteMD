<script setup>
import { onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../../stores/app'
import { useI18n } from '../../utils/i18n'

const store = useAppStore()
const { t } = useI18n()
const items = ref([])

/** 当前文件的阅读位置上下文：百分比 + 最近章节 */
function captureContext() {
  const sc = document.querySelector('.reader-scroll')
  if (!sc) return null
  const max = sc.scrollHeight - sc.clientHeight
  const percent = max > 0 ? (sc.scrollTop / max) * 100 : 0
  let headingId = ''
  let label = ''
  document.querySelectorAll('.reader-content h1, .reader-content h2, .reader-content h3, .reader-content h4, .reader-content h5, .reader-content h6').forEach((h) => {
    const top = h.getBoundingClientRect().top - sc.getBoundingClientRect().top + sc.scrollTop
    if (top <= sc.scrollTop + 90) {
      headingId = h.id || ''
      label = (h.textContent || '').trim().slice(0, 40)
    }
  })
  if (!label) label = `${Math.round(percent)}%`
  return { label, headingId, progressPercent: percent }
}

/** 加载当前文件的书签；切文件与面板打开时都会触发 */
async function reload() {
  const path = store.currentFile?.path
  if (!path) {
    items.value = []
    return
  }
  try {
    items.value = await invoke('get_bookmarks', { path })
    loadedPath.value = path
  } catch {
    /* 读取失败按空处理 */
  }
}

onMounted(reload)

async function addBookmark() {
  const path = store.currentFile?.path
  const ctx = captureContext()
  if (!path || !ctx) return
  items.value = await invoke('add_bookmark', {
    path,
    label: ctx.label,
    headingId: ctx.headingId,
    progressPercent: ctx.progressPercent,
  })
}

async function removeBookmark(bm) {
  const path = store.currentFile?.path
  if (!path) return
  items.value = await invoke('remove_bookmark', { path, createdAt: bm.createdAt })
}

function jumpTo(bm) {
  const sc = document.querySelector('.reader-scroll')
  if (!sc) return
  // 跳回书签位置：优先按章节锚点精确跳转，锚点失效退回百分比定位
  const heading = bm.headingId ? document.getElementById(bm.headingId) : null
  if (heading) {
    heading.scrollIntoView({ behavior: 'auto', block: 'start' })
    return
  }
  const max = sc.scrollHeight - sc.clientHeight
  sc.scrollTop = (max * bm.progressPercent) / 100
}

watch(() => store.currentFile?.path, () => {
  items.value = []
  reload()
})
</script>

<template>
  <div class="bookmark-list">
    <button class="bm-add" :disabled="!store.currentFile" @click="addBookmark">＋ {{ t('addBookmark') }}</button>
    <template v-if="items.length">
      <div v-for="bm in items" :key="bm.createdAt" class="bm-row" @click="jumpTo(bm)">
        <span class="bm-label">{{ bm.label }}</span>
        <span class="bm-pct">{{ Math.round(bm.progressPercent) }}%</span>
        <button class="bm-del" :title="t('remove')" @click.stop="removeBookmark(bm)">✕</button>
      </div>
    </template>
    <div v-else class="sidebar-empty">{{ t('noBookmarks') }}</div>
  </div>
</template>
