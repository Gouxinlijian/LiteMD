<script setup>
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../../stores/app'
import { useFiles } from '../../composables/useFiles'
import { formatRelativeTime } from '../../utils/format'
import { useI18n } from '../../utils/i18n'

const store = useAppStore()
const { openByPath, refreshRecent } = useFiles()
const { t } = useI18n()

/** 切换标星（置顶 + ★ 标记） */
async function toggleStar(file) {
  await invoke('set_recent_starred', { path: file.path, starred: !file.starred })
  await refreshRecent()
}

/** 从最近列表移除 */
async function removeFile(file) {
  await invoke('remove_recent_file', { path: file.path })
  await refreshRecent()
}
</script>

<template>
  <div class="recent-list">
    <div
      v-for="file in store.recentFiles"
      :key="file.path"
      class="recent-item"
      :class="{ active: file.path === store.currentFile?.path, starred: file.starred }"
      :title="file.path"
      @click="openByPath(file.path)"
    >
      <div class="recent-name">
        <span v-if="file.starred" class="star-mark">★</span>
        <span class="recent-title">{{ file.name }}</span>
      </div>
      <div class="recent-meta">
        <span>{{ formatRelativeTime(file.lastOpenedAt) }}</span>
        <span v-if="file.progressPercent > 0" class="recent-progress">
          {{ Math.round(file.progressPercent) }}%
        </span>
        <span class="recent-actions" @click.stop>
          <button
            class="recent-btn"
            :class="{ on: file.starred }"
            :title="file.starred ? t('unstar') : t('star')"
            @click="toggleStar(file)"
          >★</button>
          <button class="recent-btn del" :title="t('remove')" @click="removeFile(file)">✕</button>
        </span>
      </div>
    </div>
    <div v-if="!store.recentFiles.length" class="sidebar-empty">{{ t('noRecent') }}</div>
  </div>
</template>
