<script setup>
import { ref, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import { useAppStore } from '../stores/app'
import { useSearch } from '../composables/useSearch'
import { useI18n } from '../utils/i18n'
import { isAndroid } from '../utils/platform'

const store = useAppStore()
const { query, matches, current, total, run, next, prev, close } = useSearch()
const { t } = useI18n()
const inputRef = ref(null)

/** 输入防抖：连续打字时不必每次按键都全量遍历文档 */
const INPUT_DEBOUNCE = 180
let inputTimer = null

const countText = computed(() => {
  if (!total.value) return '0/0'
  // 命中数超过高亮上限时，实际可跳转的只有已高亮的部分
  const truncated = matches.value.length < total.value ? '*' : ''
  return `${current.value + 1}/${total.value}${truncated}`
})

const countTitle = computed(() =>
  matches.value.length < total.value ? `${t('searchTruncated')} ${matches.value.length}` : '',
)

function getContainer() {
  return document.querySelector('.reader-content')
}

function cancelPending() {
  if (inputTimer) {
    clearTimeout(inputTimer)
    inputTimer = null
  }
}

function searchNow() {
  cancelPending()
  run(getContainer(), query.value)
}

// 打开时聚焦（如有预填关键字则直接搜索），关闭时清除高亮
watch(
  () => store.searchOpen,
  async (open) => {
    if (open) {
      await nextTick()
      inputRef.value?.focus()
      if (store.searchPrefill) {
        query.value = store.searchPrefill
        store.searchPrefill = ''
        searchNow()
      } else {
        inputRef.value?.select()
      }
    } else {
      cancelPending()
      close(getContainer())
    }
  },
)

function onInput() {
  cancelPending()
  inputTimer = setTimeout(searchNow, INPUT_DEBOUNCE)
}

function onKeydown(e) {
  if (e.key === 'Enter') {
    e.preventDefault()
    // 防抖未落地时先执行本次输入，直接定位到第一处
    if (inputTimer) {
      searchNow()
      return
    }
    e.shiftKey ? prev(getContainer()) : next(getContainer())
  } else if (e.key === 'Escape') {
    cancelPending()
    store.searchOpen = false
  }
}

onBeforeUnmount(cancelPending)
</script>

<template>
  <div class="search-bar" :class="{ mobile: isAndroid }">
    <input
      ref="inputRef"
      v-model="query"
      class="search-input"
      :placeholder="t('searchPlaceholder')"
      @input="onInput"
      @keydown="onKeydown"
    />
    <span class="search-count" :title="countTitle">{{ countText }}</span>
    <button class="icon-btn" :title="t('prevMatch')" @click="prev(getContainer())">▲</button>
    <button class="icon-btn" :title="t('nextMatch')" @click="next(getContainer())">▼</button>
    <button class="icon-btn" :title="t('close')" @click="store.searchOpen = false">✕</button>
  </div>
</template>
