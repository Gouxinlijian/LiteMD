<script setup>
import { nextTick, ref, watch } from 'vue'
import { useAppStore } from '../../stores/app'
import { useI18n } from '../../utils/i18n'
import OutlineNode from './OutlineNode.vue'
import BookmarkList from './BookmarkList.vue'

const store = useAppStore()
const { t } = useI18n()
const collapsed = ref(new Set())
const outlineEl = ref(null)
/** 面板内页签：outline | bookmarks */
const tab = ref('outline')

// 当前章节变化：展开其所有祖先节点，并把大纲滚动到该章节
watch(
  () => store.activeHeadingId,
  (id) => {
    if (!id) return
    expandAncestors(store.toc, id)
    if (tab.value === 'outline') nextTick(() => scrollActiveIntoView(id))
  },
)

/** 大纲滚动到当前章节（仅当节点不可见时滚动，避免打扰用户手动滚动） */
function scrollActiveIntoView(id) {
  const el = outlineEl.value?.querySelector(`.outline-item[data-id="${CSS.escape(id)}"]`)
  el?.scrollIntoView({ block: 'nearest', behavior: 'auto' })
}

function expandAncestors(nodes, targetId) {
  for (const node of nodes) {
    if (node.id === targetId) return true
    if (expandAncestors(node.children || [], targetId)) {
      collapsed.value.delete(node.id)
      return true
    }
  }
  return false
}
</script>

<template>
  <div class="outline-wrap">
    <div class="m-tabs">
      <button class="m-tab" :class="{ on: tab === 'outline' }" @click="tab = 'outline'">{{ t('outline') }}</button>
      <button class="m-tab" :class="{ on: tab === 'bookmarks' }" @click="tab = 'bookmarks'">{{ t('bookmarks') }}</button>
    </div>

    <div v-if="tab === 'outline'" ref="outlineEl" class="outline-tree">
      <OutlineNode v-for="node in store.toc" :key="node.id" :node="node" :collapsed="collapsed" />
      <div v-if="!store.toc.length" class="sidebar-empty">{{ t('noOutline') }}</div>
    </div>
    <BookmarkList v-else class="outline-bookmarks" />
  </div>
</template>
