<script setup>
import { ref, watch, nextTick } from 'vue'
import { useAppStore } from '../stores/app'
import { useSearch } from '../composables/useSearch'
import { useI18n } from '../utils/i18n'

const store = useAppStore()
const { query, matches, current, run, next, prev, close } = useSearch()
const { t } = useI18n()
const inputRef = ref(null)

function getContainer() {
  return document.querySelector('.reader-content')
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
        run(getContainer(), query.value)
      } else {
        inputRef.value?.select()
      }
    } else {
      close(getContainer())
    }
  },
)

function onInput() {
  run(getContainer(), query.value)
}

function onKeydown(e) {
  if (e.key === 'Enter') {
    e.preventDefault()
    e.shiftKey ? prev(getContainer()) : next(getContainer())
  } else if (e.key === 'Escape') {
    store.searchOpen = false
  }
}
</script>

<template>
  <div class="search-bar">
    <input
      ref="inputRef"
      v-model="query"
      class="search-input"
      :placeholder="t('searchPlaceholder')"
      @input="onInput"
      @keydown="onKeydown"
    />
    <span class="search-count">{{ matches.length ? `${current + 1}/${matches.length}` : '0/0' }}</span>
    <button class="icon-btn" :title="t('prevMatch')" @click="prev(getContainer())">▲</button>
    <button class="icon-btn" :title="t('nextMatch')" @click="next(getContainer())">▼</button>
    <button class="icon-btn" :title="t('close')" @click="store.searchOpen = false">✕</button>
  </div>
</template>
