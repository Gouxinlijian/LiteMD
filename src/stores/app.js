import { defineStore } from 'pinia'

/**
 * 全局应用状态：当前文件、设置、UI 面板开关、目录树、最近文件
 */
export const useAppStore = defineStore('app', {
  state: () => ({
    preferences: null,         // 用户设置（后端加载）
    currentFile: null,         // 当前打开文件的 FilePayload
    toc: [],                   // 当前文件目录树
    activeHeadingId: '',       // 当前阅读到的章节锚点 id
    recentFiles: [],           // 最近文件列表
    sidebarVisible: true,      // 侧边栏是否显示
    sidebarWidth: 260,         // 侧边栏宽度
    sidebarActiveTab: 'outline', // outline | recent
    searchOpen: false,         // 搜索浮层
    searchPrefill: '',         // 右键"搜索选中内容"预填的关键字
    lightboxSrc: '',           // 图片放大：App 设置，Reader 监听显示
    settingsOpen: false,       // 设置面板
    isFullscreen: false,       // 全屏阅读模式
    mobileImmersive: false,    // 移动端沉浸阅读：隐藏上下栏，仅正文
    theme: 'light',            // 当前实际主题 light|dark
  }),

  getters: {
    progressPercent: (state) => state.currentFile?.progress?.progressPercent ?? 0,
    hasFile: (state) => !!state.currentFile,
  },
})
