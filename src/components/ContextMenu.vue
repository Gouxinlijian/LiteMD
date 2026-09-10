<script setup>
import { contextMenu } from '../composables/useContextMenu'

function onAction(item) {
  contextMenu.hide()
  item.action?.()
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="contextMenu.visible"
      class="ctx-overlay"
      @click="contextMenu.hide()"
      @contextmenu.prevent="contextMenu.hide()"
    >
      <div class="ctx-menu" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }">
        <template v-for="(item, i) in contextMenu.items" :key="i">
          <div v-if="item.divider" class="ctx-divider"></div>
          <button v-else class="ctx-item" :class="{ danger: item.danger }" @click="onAction(item)">
            <span v-if="item.icon" class="ctx-icon">{{ item.icon }}</span>
            <span class="ctx-label">{{ item.label }}</span>
          </button>
        </template>
      </div>
    </div>
  </Teleport>
</template>
