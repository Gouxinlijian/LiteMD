<script setup>
import { useAppStore } from '../stores/app'
import { formatWordCount, formatReadTime } from '../utils/format'

const store = useAppStore()

const percent = () => Math.round(store.progressPercent)

/** 点击进度条跳转到对应位置 */
function onTrackClick(e) {
  const el = document.querySelector('.reader-scroll')
  if (!el) return
  const rect = e.currentTarget.getBoundingClientRect()
  const ratio = Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width))
  const docHeight = el.scrollHeight - el.clientHeight
  el.scrollTop = docHeight * ratio
}
</script>

<template>
  <footer class="statusbar">
    <span class="status-file" :title="store.currentFile?.path">{{ store.currentFile?.name || ' ' }}</span>
    <div class="status-progress" @click="onTrackClick">
      <span class="status-percent">{{ percent() }}%</span>
      <div class="progress-track">
        <div class="progress-fill" :style="{ width: percent() + '%' }"></div>
      </div>
    </div>
    <span v-if="store.currentFile" class="status-meta">
      {{ formatWordCount(store.currentFile.content) }} 字 · {{ formatReadTime(store.currentFile.progress.readTimeMinutes) }}
    </span>
  </footer>
</template>
