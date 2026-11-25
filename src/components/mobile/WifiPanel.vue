<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../utils/i18n'
import AppIcon from '../icons/AppIcon.vue'

const emit = defineEmits(['close'])
const { t } = useI18n()

const url = ref('')
const port = ref(0)
const token = ref('')
const error = ref('')
/** 复制结果反馈：'' | 'ok' | 'fail' */
const copyState = ref('')
const qrCanvas = ref(null)
let copyTimer = null

/** 模拟器网络是 NAT、跨网段也可能不通，统一给出 adb forward 兜底方案 */
const emuHint = computed(() => {
  if (!port.value) return ''
  return t('wifiEmuHint')
    .replaceAll('{port}', String(port.value))
    .replaceAll('{token}', token.value)
})

async function startServer() {
  try {
    // 启动服务并拼接带 token 的完整地址（扫码即授权）
    const info = await invoke('wifi_server_start')
    const lanIp = await invoke('wifi_lan_address')
    if (!lanIp) {
      error.value = t('wifiOff')
      return
    }
    port.value = info.port
    token.value = info.token
    url.value = `http://${lanIp}:${info.port}/?k=${info.token}`
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

/**
 * 写剪贴板：WebView 里 clipboard API 常常不可用（无权限或无焦点），
 * 失败后回退到 execCommand('copy')，两者都失败则提示用户长按手动复制。
 */
async function writeClipboard(text) {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text)
      return true
    }
  } catch {
    /* 继续尝试兼容方案 */
  }
  try {
    const ta = document.createElement('textarea')
    ta.value = text
    ta.setAttribute('readonly', '')
    ta.style.cssText = 'position:fixed;top:0;left:-9999px;opacity:0'
    document.body.appendChild(ta)
    ta.select()
    ta.setSelectionRange(0, text.length)
    const ok = document.execCommand('copy')
    document.body.removeChild(ta)
    return ok
  } catch {
    return false
  }
}

async function copyUrl() {
  if (!url.value) return
  const ok = await writeClipboard(url.value)
  copyState.value = ok ? 'ok' : 'fail'
  clearTimeout(copyTimer)
  copyTimer = setTimeout(() => (copyState.value = ''), ok ? 1400 : 3000)
}

async function close() {
  // 关闭面板同时停掉接收服务，省电
  invoke('wifi_server_stop').catch(() => {})
  emit('close')
}

onMounted(startServer)
onBeforeUnmount(() => clearTimeout(copyTimer))
</script>

<template>
  <div class="wifi-mask" @click.self="close">
    <div class="wifi-card">
      <div class="wifi-head">
        <span>{{ t('computerTransfer') }}</span>
        <button class="m-btn" @click="close"><AppIcon name="close" :size="16" /></button>
      </div>
      <p class="wifi-tip">{{ t('wifiTip') }}</p>

      <template v-if="error">
        <p class="wifi-error">{{ error }}</p>
      </template>
      <template v-else-if="url">
        <canvas ref="qrCanvas" class="qr-canvas"></canvas>
        <div class="wifi-url" @click="copyUrl">
          <span class="wifi-url-text">{{ url }}</span>
          <span class="wifi-copy" :class="copyState">
            {{ copyState === 'ok' ? t('copied') : copyState === 'fail' ? t('copyFailed') : t('copy') }}
          </span>
        </div>
        <p v-if="emuHint" class="wifi-emu">{{ emuHint }}</p>
      </template>
      <p v-else class="wifi-tip">…</p>

      <button class="wifi-close" @click="close">{{ t('stopAndClose') }}</button>
    </div>
  </div>
</template>
