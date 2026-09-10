<script setup>
import { useAppStore } from '../../stores/app'

const props = defineProps({
  node: { type: Object, required: true },
  collapsed: { type: Set, required: true },
})
const store = useAppStore()

function toggleCaret() {
  const has = props.collapsed.has(props.node.id)
  has ? props.collapsed.delete(props.node.id) : props.collapsed.add(props.node.id)
}

/** 点击跳转到对应章节 */
function jump() {
  const el = document.getElementById(props.node.id)
  if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' })
}
</script>

<template>
  <div class="outline-node">
    <div
      class="outline-item"
      :data-id="node.id"
      :class="{ active: store.activeHeadingId === node.id }"
      :style="{ paddingLeft: (node.level - 1) * 14 + 8 + 'px' }"
      @click="jump"
    >
      <span v-if="node.children?.length" class="outline-caret" @click.stop="toggleCaret">
        {{ collapsed.has(node.id) ? '▸' : '▾' }}
      </span>
      <span v-else class="outline-caret outline-caret-empty"></span>
      <span class="outline-text">{{ node.text }}</span>
    </div>
    <div v-if="node.children?.length && !collapsed.has(node.id)" class="outline-children">
      <OutlineNode v-for="child in node.children" :key="child.id" :node="child" :collapsed="collapsed" />
    </div>
  </div>
</template>
