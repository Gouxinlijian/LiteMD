import { reactive } from 'vue'

/**
 * 全局右键菜单状态（模块单例，App 与 ContextMenu 共享）。
 * items: [{ label, icon, action, danger?, divider? }]
 */
export const contextMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  items: [],
  show(x, y, items) {
    this.x = x
    this.y = y
    this.items = items
    this.visible = true
  },
  hide() {
    this.visible = false
  },
})
