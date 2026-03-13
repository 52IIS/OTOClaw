<script setup lang="ts">
import { computed } from 'vue'
import {
  LayoutDashboard,
  Bot,
  MessageSquare,
  FlaskConical,
  ScrollText,
  Settings,
} from 'lucide-vue-next'
import type { PageType, ServiceStatus } from '../../vite-env.d'
import clsx from 'clsx'

interface Props {
  currentPage: PageType
  serviceStatus: ServiceStatus | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  navigate: [page: PageType]
}>()

const menuItems: { id: PageType; label: string; icon: any }[] = [
  { id: 'dashboard', label: '概览', icon: LayoutDashboard },
  { id: 'ai', label: 'AI 配置', icon: Bot },
  { id: 'channels', label: '消息渠道', icon: MessageSquare },
  { id: 'testing', label: '测试诊断', icon: FlaskConical },
  { id: 'logs', label: '应用日志', icon: ScrollText },
  { id: 'settings', label: '设置', icon: Settings },
]

const isRunning = computed(() => props.serviceStatus?.running ?? false)
</script>

<template>
  <aside class="w-64 bg-dark-800 border-r border-dark-600 flex flex-col">
    <div class="h-14 flex items-center px-6 titlebar-drag border-b border-dark-600">
      <div class="flex items-center gap-3 titlebar-no-drag">
        <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-claw-400 to-claw-600 flex items-center justify-center">
          <span class="text-lg">🦞</span>
        </div>
        <div>
          <h1 class="text-sm font-semibold text-white">OpenClaw</h1>
          <p class="text-xs text-gray-500">Manager</p>
        </div>
      </div>
    </div>

    <nav class="flex-1 py-4 px-3">
      <ul class="space-y-1">
        <li v-for="item in menuItems" :key="item.id">
          <button
            @click="emit('navigate', item.id)"
            :class="clsx(
              'w-full flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-medium transition-all relative',
              currentPage === item.id
                ? 'text-white bg-dark-600'
                : 'text-gray-400 hover:text-white hover:bg-dark-700'
            )"
          >
            <div
              v-if="currentPage === item.id"
              v-motion
              :initial="{ opacity: 0, x: -10 }"
              :enter="{ opacity: 1, x: 0 }"
              :duration="200"
              class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-claw-500 rounded-r-full"
            />
            <component
              :is="item.icon"
              :size="18"
              :class="currentPage === item.id ? 'text-claw-400' : ''"
            />
            <span>{{ item.label }}</span>
          </button>
        </li>
      </ul>
    </nav>

    <div class="p-4 border-t border-dark-600">
      <div class="px-4 py-3 bg-dark-700 rounded-lg">
        <div class="flex items-center gap-2 mb-2">
          <div :class="['status-dot', isRunning ? 'running' : 'stopped']" />
          <span class="text-xs text-gray-400">
            {{ isRunning ? '服务运行中' : '服务未启动' }}
          </span>
        </div>
        <p class="text-xs text-gray-500">端口: {{ serviceStatus?.port ?? 18789 }}</p>
      </div>
    </div>
  </aside>
</template>
