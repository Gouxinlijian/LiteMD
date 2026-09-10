<script setup>
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../utils/i18n'

const emit = defineEmits(['close'])
const { t } = useI18n()

const url = ref('')
const copied = ref(false)
const error = ref('')
const qrCanvas = ref(null)

async function startServer() {
  try {
    // 启动服务并拼接带 token 的完整地址（扫码即授权）
    const info = await invoke('wifi_server_start')
    const ip = await invoke('wifi_lan_address')
    if (!ip) {
      error.value = t('wifiOff')
      return
    }
    url.value = `http://${ip}:${info.port}/?k=${info.token}`
    drawQr(url.value)
  } catch (e) {
    error.value = String(e)
  }
}

/** 二维码按需动态加载，不进主包 */
async function drawQr(text) {
  try {
    const mod = await import('qrcode')
    const QRCode = mod.default || mod
    QRCode.toCanvas(qrCanvas.value, text, { width: 200, margin: 1 })
  } catch {
    /* 二维码绘制失败时仍显示文字地址 */
  }
}

async function copyUrl() {
  if (!url.value) return
  try {
    await navigator.clipboard.writeText(url.value)
    copied.value = true
    setTimeout(() => (copied.value = false), 1200)
  } catch {
    /* 剪贴板不可用时静默 */
  }
}

async function close() {
  // 关闭面板同时停掉接收服务，省电
  invoke('wifi_server_stop').catch(() => {})
  emit('close')
}

onMounted(startServer)
</script>

<template>
  <div class="wifi-mask" @click.self="close">
    <div class="wifi-card">
      <div class="wifi-head">
        <span>{{ t('computerTransfer') }}</span>
        <button class="m-btn" @click="close">✕</button>
      </div>
      <p class="wifi-tip">{{ t('wifiTip') }}</p>

      <template v-if="error">
        <p class="wifi-error">{{ error }}</p>
      </template>
      <template v-else-if="url">
        <canvas ref="qrCanvas" class="qr-canvas"></canvas>
        <div class="wifi-url" @click="copyUrl">{{ url }}<span class="wifi-copy">{{ copied ? t('copied') : t('copy') }}</span></div>
      </template>
      <p v-else class="wifi-tip">…</p>

      <button class="wifi-close" @click="close">{{ t('stopAndClose') }}</button>
    </div>
  </div>
</template>
