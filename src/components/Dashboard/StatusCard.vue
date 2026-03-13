<script setup lang="ts">
import { computed } from 'vue'
import { Activity, Cpu, HardDrive, Clock } from 'lucide-vue-next'
import type { ServiceStatus } from '../../vite-env.d'

interface Props {
  status: ServiceStatus | null
  loading: boolean
}

const props = defineProps<Props>()

const formatUptime = (seconds: number | null) => {
  if (!seconds) return '--'
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

const statusText = computed(() => {
  if (props.loading) return '检测中...'
  return props.status?.running ? '运行中' : '已停止'
})

const statusClass = computed(() => {
  if (props.loading) return 'text-yellow-400'
  return props.status?.running ? 'text-green-400' : 'text-red-400'
})

const dotClass = computed(() => {
  if (props.loading) return 'warning'
  return props.status?.running ? 'running' : 'stopped'
})
</script>

<template>
  <div class="bg-dark-700 rounded-2xl p-6 border border-dark-500">
    <div class="flex items-center justify-between mb-6">
      <h3 class="text-lg font-semibold text-white">服务状态</h3>
      <div class="flex items-center gap-2">
        <div :class="['status-dot', dotClass]" />
        <span :class="['text-sm font-medium', statusClass]">
          {{ statusText }}
        </span>
      </div>
    </div>

    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div class="bg-dark-600 rounded-xl p-4">
        <div class="flex items-center gap-2 mb-2">
          <Activity :size="16" class="text-accent-cyan" />
          <span class="text-xs text-gray-400">端口</span>
        </div>
        <p class="text-xl font-semibold text-white">
          {{ status?.port || 18789 }}
        </p>
      </div>

      <div class="bg-dark-600 rounded-xl p-4">
        <div class="flex items-center gap-2 mb-2">
          <Cpu :size="16" class="text-accent-purple" />
          <span class="text-xs text-gray-400">进程 ID</span>
        </div>
        <p class="text-xl font-semibold text-white">
          {{ status?.pid || '--' }}
        </p>
      </div>

      <div class="bg-dark-600 rounded-xl p-4">
        <div class="flex items-center gap-2 mb-2">
          <HardDrive :size="16" class="text-accent-green" />
          <span class="text-xs text-gray-400">内存</span>
        </div>
        <p class="text-xl font-semibold text-white">
          {{ status?.memory_mb ? `${status.memory_mb.toFixed(1)} MB` : '--' }}
        </p>
      </div>

      <div class="bg-dark-600 rounded-xl p-4">
        <div class="flex items-center gap-2 mb-2">
          <Clock :size="16" class="text-accent-amber" />
          <span class="text-xs text-gray-400">运行时间</span>
        </div>
        <p class="text-xl font-semibold text-white">
          {{ formatUptime(status?.uptime_seconds || null) }}
        </p>
      </div>
    </div>
  </div>
</template>
