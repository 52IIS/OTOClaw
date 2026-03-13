<script setup lang="ts">
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-shell'
import { invoke } from '@tauri-apps/api/core'
import { RefreshCw, ExternalLink, Loader2 } from 'lucide-vue-next'
import type { PageType } from '../../vite-env.d'

interface Props {
  currentPage: PageType
}

const props = defineProps<Props>()

const opening = ref(false)

const pageTitles: Record<PageType, { title: string; description: string }> = {
  dashboard: { title: '概览', description: '服务状态、日志与快捷操作' },
  ai: { title: 'AI 模型配置', description: '配置 AI 提供商和模型' },
  channels: { title: '消息渠道', description: '配置 Telegram、Discord、飞书等' },
  testing: { title: '测试诊断', description: '系统诊断与问题排查' },
  logs: { title: '应用日志', description: '查看 Manager 应用的控制台日志' },
  settings: { title: '设置', description: '身份配置与高级选项' },
}

const currentTitle = pageTitles[props.currentPage]

const handleRefresh = () => {
  window.location.reload()
}

const handleOpenDashboard = async () => {
  opening.value = true
  try {
    const url = await invoke<string>('get_dashboard_url')
    await open(url)
  } catch (e) {
    console.error('打开 Dashboard 失败:', e)
    window.open('http://localhost:18789', '_blank')
  } finally {
    opening.value = false
  }
}
</script>

<template>
  <header class="h-14 bg-dark-800/50 border-b border-dark-600 flex items-center justify-between px-6 titlebar-drag backdrop-blur-sm">
    <div class="titlebar-no-drag">
      <h2 class="text-lg font-semibold text-white">{{ currentTitle.title }}</h2>
      <p class="text-xs text-gray-500">{{ currentTitle.description }}</p>
    </div>

    <div class="flex items-center gap-2 titlebar-no-drag">
      <button
        @click="handleRefresh"
        class="icon-button text-gray-400 hover:text-white"
        title="刷新"
      >
        <RefreshCw :size="16" />
      </button>
      <button
        @click="handleOpenDashboard"
        :disabled="opening"
        class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-dark-600 hover:bg-dark-500 text-sm text-gray-300 hover:text-white transition-colors disabled:opacity-50"
        title="打开 Web Dashboard"
      >
        <Loader2 v-if="opening" :size="14" class="animate-spin" />
        <ExternalLink v-else :size="14" />
        <span>Dashboard</span>
      </button>
    </div>
  </header>
</template>
