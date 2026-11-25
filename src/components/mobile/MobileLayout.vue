<script setup>
import { ref } from 'vue'
import { useAppStore } from '../../stores/app'
import { useFiles } from '../../composables/useFiles'
import { useTheme } from '../../composables/useTheme'
import { useI18n } from '../../utils/i18n'
import RecentFiles from '../sidebar/RecentFiles.vue'
import Outline from '../sidebar/Outline.vue'
import BrowsePanel from './BrowsePanel.vue'
import WifiPanel from './WifiPanel.vue'
import AppIcon from '../icons/AppIcon.vue'

const store = useAppStore()
const { openDialog } = useFiles()
const theme = useTheme()
const { t } = useI18n()

/** 底部抽屉当前面板：'' | 'recent' | 'files' | 'outline' */
const panel = ref('')
/** 电脑传书弹层 */
const wifiOpen = ref(false)

function togglePanel(name) {
  panel.value = panel.value === name ? '' : name
}
</script>

<template>
  <div class="mobile-shell" :class="{ immersive: store.mobileImmersive }">
    <!-- 顶部栏：打开文件 + 文件名 + 搜索 + 主题 + 设置（沉浸时隐藏） -->
    <header class="m-topbar" v-show="!store.mobileImmersive">
      <button class="m-btn" :title="t('openFile')" @click="openDialog">
        <AppIcon name="folderOpen" :size="18" />
      </button>
      <span class="m-title" :title="store.currentFile?.path">
        {{ store.currentFile?.name || t('noFile') }}
      </span>
      <button class="m-btn" :title="t('search')" @click="store.searchOpen = true">
        <AppIcon name="search" :size="18" />
      </button>
      <button class="m-btn" :title="t('theme')" @click="theme.toggle()">
        <AppIcon :name="store.theme === 'dark' ? 'sun' : 'moon'" :size="18" />
      </button>
      <button class="m-btn" :title="t('settings')" @click="store.settingsOpen = true">
        <AppIcon name="settings" :size="18" />
      </button>
    </header>

    <!-- 正文区（插槽放 Reader） -->
    <slot />

    <!-- 底部浮动工具条（沉浸时隐藏） -->
    <nav class="m-dock" v-show="!store.mobileImmersive">
      <button class="m-dock-btn" :class="{ on: panel === 'recent' }" @click="togglePanel('recent')">
        <span class="m-dock-icon"><AppIcon name="clock" :size="17" /></span>
        <span class="m-dock-label">{{ t('recent') }}</span>
      </button>
      <button class="m-dock-btn" :class="{ on: panel === 'files' }" @click="togglePanel('files')">
        <span class="m-dock-icon"><AppIcon name="folder" :size="17" /></span>
        <span class="m-dock-label">{{ t('folder') }}</span>
      </button>
      <button class="m-dock-btn" :class="{ on: panel === 'outline' }" @click="togglePanel('outline')">
        <span class="m-dock-icon"><AppIcon name="list" :size="17" /></span>
        <span class="m-dock-label">{{ t('outline') }}</span>
      </button>
    </nav>

    <!-- 底部抽屉面板（最近 / 文件 / 大纲+书签） -->
    <transition name="m-panel">
      <div v-if="panel" class="m-panel-mask" @click="panel = ''">
        <div class="m-panel-sheet" @click.stop>
          <div class="m-panel-head">
            <span class="m-panel-title">
              {{ panel === 'recent' ? t('recent') : panel === 'files' ? t('folder') : t('outline') }}
            </span>
            <button class="m-btn" @click="panel = ''">
              <AppIcon name="close" :size="16" />
            </button>
          </div>
          <div class="m-panel-body">
            <RecentFiles v-if="panel === 'recent'" />
            <BrowsePanel v-else-if="panel === 'files'" @opened="panel = ''" @wifi="wifiOpen = true" />
            <Outline v-else />
          </div>
        </div>
      </div>
    </transition>

    <!-- 电脑传书 -->
    <WifiPanel v-if="wifiOpen" @close="wifiOpen = false" />
  </div>
</template>
