<script setup>
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useFiles } from '../../composables/useFiles'
import { useI18n } from '../../utils/i18n'
import AppIcon from '../icons/AppIcon.vue'

const emit = defineEmits(['opened', 'wifi'])
const { loadFile } = useFiles()
const { t } = useI18n()

const files = ref([])
const hasFolder = ref(false)
const loading = ref(false)
const openingUri = ref('')

/** 应用后端返回的列表结构 { granted, list, cancelled? } */
function applyResult(res) {
  if (!res || res.cancelled) return
  hasFolder.value = !!res.granted
  files.value = Array.isArray(res.list) ? res.list : []
}

/** 加载已授权文件夹中的 md 列表 */
async function reload() {
  loading.value = true
  try {
    applyResult(await invoke('list_browse_files'))
  } catch {
    // 尚未授权过文件夹
    files.value = []
    hasFolder.value = false
  }
  loading.value = false
}

/** SAF 目录树授权选择（用户取消则保持原状，不提示） */
async function chooseFolder() {
  loading.value = true
  try {
    applyResult(await invoke('choose_browse_folder'))
  } catch (e) {
    alert(`选择文件夹失败：${e}`)
  }
  loading.value = false
}

/** 打开所选文件（后端连同图片拷贝到私有目录） */
async function openFile(f) {
  if (openingUri.value) return
  openingUri.value = f.uri
  try {
    const payload = await invoke('open_browse_file', { docUri: f.uri })
    await loadFile(payload)
    emit('opened')
  } catch (e) {
    alert(`打开失败：${e}`)
  }
  openingUri.value = ''
}

function fmtSize(n) {
  if (!n) return ''
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`
  return `${(n / 1024 / 1024).toFixed(1)} MB`
}

onMounted(reload)
</script>

<template>
  <div class="browse-panel">
    <div class="bp-toolbar">
      <button class="bp-tool" :disabled="loading" @click="chooseFolder">
        <AppIcon name="folder" :size="16" /><span>{{ t('chooseFolder') }}</span>
      </button>
      <button class="bp-tool wifi" :disabled="loading" @click="emit('wifi')">
        <AppIcon name="wifi" :size="16" /><span>{{ t('computerTransfer') }}</span>
      </button>
    </div>

    <template v-if="hasFolder">
      <div v-if="loading" class="sidebar-empty">…</div>
      <template v-else-if="files.length">
        <div v-for="f in files" :key="f.uri" class="bp-file" @click="openFile(f)">
          <span class="bp-name">{{ f.name }}</span>
          <span class="bp-size">{{ fmtSize(f.size) }}</span>
        </div>
      </template>
      <div v-else class="sidebar-empty">{{ t('browseEmpty') }}</div>
    </template>
    <div v-else class="sidebar-empty">{{ t('browseEmpty') }}</div>
  </div>
</template>
