<script setup>
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useAppStore } from '../stores/app'
import { useFiles } from '../composables/useFiles'
import { useTheme } from '../composables/useTheme'
import { useI18n } from '../utils/i18n'

const store = useAppStore()
const { openDialog } = useFiles()
const theme = useTheme()
const { t } = useI18n()
const win = getCurrentWindow()
const isMaximized = ref(false)

win.onResized(async () => {
  isMaximized.value = await win.isMaximized()
})

function minimize() {
  win.minimize()
}
function toggleMaximize() {
  win.toggleMaximize()
}
function close() {
  win.close()
}
</script>

<template>
  <header class="topbar" data-tauri-drag-region @dblclick="toggleMaximize">
    <div class="topbar-left" data-tauri-drag-region>
      <button class="icon-btn" :title="`${t('openFile')} (Ctrl+O)`" @dblclick.stop @click="openDialog">📂</button>
      <span class="filename" :title="store.currentFile?.path">{{ store.currentFile?.name || t('noFile') }}</span>
    </div>
    <div class="topbar-right">
      <button class="icon-btn" :title="t('search')" @dblclick.stop @click="store.searchOpen = true">🔍</button>
      <button class="icon-btn" :title="t('theme')" @dblclick.stop @click="theme.toggle()">
        {{ store.theme === 'dark' ? '☀️' : '🌙' }}
      </button>
      <button class="icon-btn" :title="t('settings')" @dblclick.stop @click="$emit('open-settings')">⚙️</button>
      <span class="win-divider"></span>
      <button class="win-btn" :title="t('minimize')" @dblclick.stop @click="minimize">─</button>
      <button class="win-btn" :title="t('maximize')" @dblclick.stop @click="toggleMaximize">
        {{ isMaximized ? '❐' : '☐' }}
      </button>
      <button class="win-btn win-close" :title="t('close')" @dblclick.stop @click="close">✕</button>
    </div>
  </header>
</template>
