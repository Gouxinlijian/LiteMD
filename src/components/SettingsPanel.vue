<script setup>
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../stores/app'
import { useTheme } from '../composables/useTheme'
import { useFont } from '../composables/useFont'
import { useI18n } from '../utils/i18n'

const store = useAppStore()
const theme = useTheme()
const font = useFont()
const { t } = useI18n()

/** 字体族候选（自用软件，固定中文常用字体） */
const FONTS = {
  default: "'Microsoft YaHei', 'Segoe UI', system-ui, sans-serif",
  song: "'SimSun', 'Songti SC', serif",
  hei: "'SimHei', 'Heiti SC', sans-serif",
  kai: "'KaiTi', 'Kaiti SC', serif",
  mono: "'Cascadia Code', Consolas, 'Courier New', monospace",
}

async function save() {
  await invoke('save_preferences', { prefs: store.preferences })
}

function setLanguage(lang) {
  store.preferences.language = lang
  save()
}

function setTheme(value) {
  store.preferences.theme = value
  theme.apply()
  save()
}

async function changeFont(delta) {
  await font.zoom(delta)
}

function setLineHeight(value) {
  store.preferences.lineHeight = value
  document.documentElement.style.setProperty('--reader-line-height', String(value))
  save()
}

function setFontFamily(value) {
  store.preferences.fontFamily = value
  document.documentElement.style.setProperty('--reader-font-family', value)
  save()
}

/** 内容宽度：0 表示铺满窗口，不设上限 */
function setMaxWidth(value) {
  store.preferences.maxContentWidth = value
  document.documentElement.style.setProperty('--reader-max-width', value ? value + 'px' : 'none')
  save()
}

function toggleCodeHighlight() {
  store.preferences.codeHighlight = !store.preferences.codeHighlight
  save()
}
</script>

<template>
  <div class="settings-mask" @click.self="$emit('close')">
    <div class="settings-panel">
      <div class="settings-header">
        <span class="settings-title">{{ t('settings') }}</span>
        <button class="icon-btn" :title="t('close')" @click="$emit('close')">✕</button>
      </div>

      <div class="settings-body">
        <div class="setting-row">
          <span class="setting-label">{{ t('language') }}</span>
          <div class="setting-control">
            <select :value="store.preferences.language" @change="setLanguage($event.target.value)">
              <option value="zh-CN">中文</option>
              <option value="en-US">English</option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <span class="setting-label">{{ t('theme') }}</span>
          <div class="setting-control">
            <select :value="store.preferences.theme" @change="setTheme($event.target.value)">
              <option value="light">{{ t('themeLight') }}</option>
              <option value="dark">{{ t('themeDark') }}</option>
              <option value="auto">{{ t('themeAuto') }}</option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <span class="setting-label">{{ t('fontSize') }}</span>
          <div class="setting-control setting-inline">
            <button class="small-btn" @click="changeFont(-1)">−</button>
            <span class="setting-value">{{ store.preferences.fontSize }}px</span>
            <button class="small-btn" @click="changeFont(1)">+</button>
            <button class="small-btn" @click="font.reset()">{{ t('reset') }}</button>
          </div>
        </div>

        <div class="setting-row">
          <span class="setting-label">{{ t('fontFamily') }}</span>
          <div class="setting-control">
            <select :value="store.preferences.fontFamily" @change="setFontFamily($event.target.value)">
              <option :value="FONTS.default">默认（微软雅黑）</option>
              <option :value="FONTS.song">宋体</option>
              <option :value="FONTS.hei">黑体</option>
              <option :value="FONTS.kai">楷体</option>
              <option :value="FONTS.mono">等宽</option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <span class="setting-label">{{ t('lineHeight') }}</span>
          <div class="setting-control">
            <select :value="store.preferences.lineHeight" @change="setLineHeight(Number($event.target.value))">
              <option :value="1.5">1.5</option>
              <option :value="1.7">1.7</option>
              <option :value="1.8">1.8</option>
              <option :value="2.0">2.0</option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <span class="setting-label">{{ t('contentWidth') }}</span>
          <div class="setting-control">
            <select :value="store.preferences.maxContentWidth" @change="setMaxWidth(Number($event.target.value))">
              <option :value="720">720</option>
              <option :value="900">900</option>
              <option :value="1080">1080</option>
              <option :value="0">{{ t('contentWidthFull') }}</option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <span class="setting-label">{{ t('codeHighlight') }}</span>
          <div class="setting-control">
            <label class="switch">
              <input type="checkbox" :checked="store.preferences.codeHighlight" @change="toggleCodeHighlight" />
              <span class="switch-slider"></span>
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
