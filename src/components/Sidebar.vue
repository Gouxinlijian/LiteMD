<script setup>
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/app'
import { useI18n } from '../utils/i18n'
import RecentFiles from './sidebar/RecentFiles.vue'
import Outline from './sidebar/Outline.vue'

const store = useAppStore()
const { t } = useI18n()

// ---------- 宽度拖拽调整 ----------
let startX = 0
let startW = 0

function onResizeStart(e) {
  startX = e.clientX
  startW = store.sidebarWidth
  document.addEventListener('mousemove', onResizeMove)
  document.addEventListener('mouseup', onResizeEnd)
  e.preventDefault()
}

function onResizeMove(e) {
  store.sidebarWidth = Math.min(480, Math.max(180, startW + (e.clientX - startX)))
}

async function onResizeEnd() {
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeEnd)
  store.preferences.sidebarWidth = store.sidebarWidth
  await invoke('save_preferences', { prefs: store.preferences })
}

function switchTab(tab) {
  store.sidebarActiveTab = tab
}
</script>

<template>
  <aside class="sidebar" :style="{ width: store.sidebarWidth + 'px' }">
    <div class="sidebar-body">
      <Outline v-show="store.sidebarActiveTab === 'outline'" />
      <RecentFiles v-show="store.sidebarActiveTab === 'recent'" />
    </div>
    <div class="sidebar-tabs">
      <button class="tab-btn" :class="{ active: store.sidebarActiveTab === 'recent' }" @click="switchTab('recent')">
        📂 {{ t('recent') }}
      </button>
      <button class="tab-btn" :class="{ active: store.sidebarActiveTab === 'outline' }" @click="switchTab('outline')">
        📋 {{ t('outline') }}
      </button>
    </div>
    <div class="sidebar-resizer" @mousedown="onResizeStart"></div>
  </aside>
</template>
