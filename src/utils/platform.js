/** 平台判断：移动端（安卓）与桌面端共用同一套前端，按需切换交互与布局 */
export const isAndroid =
  typeof navigator !== 'undefined' && /Android/i.test(navigator.userAgent || '')
