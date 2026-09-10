import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Tauri 开发环境需要固定的 devServer 端口，且不自动打开浏览器
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    target: 'es2021',
  },
})
