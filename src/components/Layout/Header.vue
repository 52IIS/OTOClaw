<script setup lang="ts">
import { ref, computed } from 'vue'
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
  chat: { title: '聊天', description: '与 AI 智能体对话' },
  agents: { title: '智能体管理', description: '创建和管理多个AI智能体' },
  skills: { title: '技能管理', description: '管理AI技能，扩展智能体能力' },
  ai: { title: 'AI 模型配置', description: '配置 AI 提供商和模型' },
  channels: { title: '消息渠道', description: '配置 Telegram、Discord、飞书等' },
  sandbox: { title: '沙箱管理', description: '配置和管理 Docker 沙箱环境' },
  cron: { title: '定时任务', description: '创建和管理定时执行的任务' },
  testing: { title: '测试诊断', description: '系统诊断与问题排查' },
  logs: { title: '应用日志', description: '查看 Manager 应用的控制台日志' },
  settings: { title: '设置', description: '身份配置与高级选项' },
}

const currentTitle = computed(() => pageTitles[props.currentPage])

const handleRefresh = () => {
  window.location.reload()
}

const handleOpenDashboard = async () => {
  opening.value = true
  try {
    const url = await invoke<string>('get_dashboard_url')
    console.log('打开 Dashboard URL:', url)
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
  <header class="flex justify-between items-center px-6 h-14 border-b backdrop-blur-sm bg-dark-800/50 border-dark-600 titlebar-drag">
    <div class="titlebar-no-drag">
      <h2 class="text-lg font-semibold text-white">{{ currentTitle.title }}</h2>
      <p class="text-xs text-gray-500">{{ currentTitle.description }}</p>
    </div>

    <div class="flex gap-2 items-center titlebar-no-drag">
      <button
        @click="handleRefresh"
        class="text-gray-400 icon-button hover:text-white"
        title="刷新"
      >
        <RefreshCw :size="16" />
      </button>
      <button
        @click="handleOpenDashboard"
        :disabled="opening"
        class="flex gap-2 items-center px-3 py-1.5 text-sm text-gray-300 rounded-lg transition-colors bg-dark-600 hover:bg-dark-500 hover:text-white disabled:opacity-50"
        title="打开 Web Dashboard"
      >
        <Loader2 v-if="opening" :size="14" class="animate-spin" />
        <ExternalLink v-else :size="14" />
        <span>Dashboard</span>
      </button>
    </div>
  </header>
</template>
