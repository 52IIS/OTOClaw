<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { Trash2, RefreshCw, Download, Filter, Terminal } from 'lucide-vue-next'
import clsx from 'clsx'
import { logStore, type LogEntry } from '../../lib/logger'

type FilterLevel = 'all' | 'debug' | 'info' | 'warn' | 'error'

const LEVEL_COLORS: Record<string, string> = {
  debug: 'text-gray-400',
  info: 'text-green-400',
  warn: 'text-yellow-400',
  error: 'text-red-400',
}

const LEVEL_BG: Record<string, string> = {
  debug: 'bg-gray-500/10',
  info: 'bg-green-500/10',
  warn: 'bg-yellow-500/10',
  error: 'bg-red-500/10',
}

const MODULE_COLORS: Record<string, string> = {
  App: 'text-purple-400',
  Service: 'text-blue-400',
  Config: 'text-emerald-400',
  AI: 'text-pink-400',
  Channel: 'text-orange-400',
  Setup: 'text-cyan-400',
  Dashboard: 'text-lime-400',
  Testing: 'text-fuchsia-400',
  API: 'text-amber-400',
}

const logs = ref<LogEntry[]>([])
const filter = ref<FilterLevel>('all')
const moduleFilter = ref<string>('all')
const autoScroll = ref(true)
const logsEndRef = ref<HTMLDivElement | null>(null)
const logsContainerRef = ref<HTMLDivElement | null>(null)

let unsubscribe: (() => void) | null = null

const updateLogs = () => {
  logs.value = logStore.getAll()
}

onMounted(() => {
  updateLogs()
  unsubscribe = logStore.subscribe(updateLogs)
})

onUnmounted(() => {
  if (unsubscribe) unsubscribe()
})

watch([logs, autoScroll], () => {
  if (autoScroll.value && logsEndRef.value) {
    logsEndRef.value?.scrollIntoView({ behavior: 'smooth' })
  }
})

const filteredLogs = computed(() => {
  return logs.value.filter(log => {
    if (filter.value !== 'all' && log.level !== filter.value) return false
    if (moduleFilter.value !== 'all' && log.module !== moduleFilter.value) return false
    return true
  })
})

const modules = computed(() => [...new Set(logs.value.map(log => log.module))])

const handleClear = () => {
  logStore.clear()
}

const handleExport = () => {
  const content = filteredLogs.value.map(log => {
    const time = log.timestamp.toLocaleTimeString('zh-CN', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    })
    const args = log.args.length > 0 ? ' ' + JSON.stringify(log.args) : ''
    return `[${time}] [${log.level.toUpperCase()}] [${log.module}] ${log.message}${args}`
  }).join('\n')

  const blob = new Blob([content], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `otoclaw-logs-${new Date().toISOString().slice(0, 10)}.txt`
  a.click()
  URL.revokeObjectURL(url)
}

const formatTime = (date: Date) => {
  return date.toLocaleTimeString('zh-CN', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }) + '.' + String(date.getMilliseconds()).padStart(3, '0')
}

const formatArgs = (args: unknown[]): string => {
  if (args.length === 0) return ''
  try {
    return args.map(arg => {
      if (typeof arg === 'object') {
        return JSON.stringify(arg, null, 2)
      }
      return String(arg)
    }).join(' ')
  } catch {
    return '[无法序列化]'
  }
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div class="flex items-center gap-4 mb-4 flex-wrap">
      <div class="flex items-center gap-2">
        <Filter :size="14" class="text-gray-500" />
        <select
          v-model="filter"
          class="bg-dark-700 border border-dark-500 rounded-lg px-3 py-1.5 text-sm text-gray-300"
        >
          <option value="all">所有级别</option>
          <option value="debug">Debug</option>
          <option value="info">Info</option>
          <option value="warn">Warn</option>
          <option value="error">Error</option>
        </select>
      </div>

      <select
        v-model="moduleFilter"
        class="bg-dark-700 border border-dark-500 rounded-lg px-3 py-1.5 text-sm text-gray-300"
      >
        <option value="all">所有模块</option>
        <option v-for="mod in modules" :key="mod" :value="mod">{{ mod }}</option>
      </select>

      <div class="flex-1" />

      <div class="flex items-center gap-3 text-xs text-gray-500">
        <span>{{ filteredLogs.length }} / {{ logs.length }} 条</span>
        <span class="text-red-400">{{ logs.filter(l => l.level === 'error').length }} 错误</span>
        <span class="text-yellow-400">{{ logs.filter(l => l.level === 'warn').length }} 警告</span>
      </div>

      <div class="flex items-center gap-2">
        <label class="flex items-center gap-1 text-xs text-gray-400">
          <input
            type="checkbox"
            v-model="autoScroll"
            class="w-3 h-3 rounded"
          />
          自动滚动
        </label>
        <button
          @click="handleExport"
          class="icon-button text-gray-400 hover:text-white"
          title="导出日志"
        >
          <Download :size="16" />
        </button>
        <button
          @click="updateLogs"
          class="icon-button text-gray-400 hover:text-white"
          title="刷新"
        >
          <RefreshCw :size="16" />
        </button>
        <button
          @click="handleClear"
          class="icon-button text-gray-400 hover:text-red-400"
          title="清除日志"
        >
          <Trash2 :size="16" />
        </button>
      </div>
    </div>

    <div class="flex-1 bg-dark-800 rounded-xl border border-dark-600 overflow-hidden flex flex-col">
      <div class="flex items-center gap-2 px-4 py-2 bg-dark-700 border-b border-dark-600">
        <Terminal :size="14" class="text-gray-500" />
        <span class="text-xs text-gray-400 font-medium">应用日志</span>
      </div>

      <div ref="logsContainerRef" class="flex-1 overflow-y-auto p-2 font-mono text-xs">
        <div v-if="filteredLogs.length === 0" class="h-full flex items-center justify-center text-gray-500">
          <div class="text-center">
            <Terminal :size="32" class="mx-auto mb-2 opacity-50" />
            <p>暂无日志</p>
          </div>
        </div>
        <template v-else>
          <div
            v-for="log in filteredLogs"
            :key="log.id"
            :class="clsx('py-1.5 px-2 rounded mb-1', LEVEL_BG[log.level])"
          >
            <div class="flex items-start gap-2">
              <span class="text-gray-600 flex-shrink-0">
                {{ formatTime(log.timestamp) }}
              </span>
              <span :class="['px-1.5 py-0.5 rounded text-[10px] uppercase flex-shrink-0', LEVEL_COLORS[log.level]]">
                {{ log.level }}
              </span>
              <span :class="['flex-shrink-0', MODULE_COLORS[log.module] || 'text-gray-400']">
                [{{ log.module }}]
              </span>
              <span class="text-gray-300 break-all">
                {{ log.message }}
              </span>
            </div>
            <div v-if="log.args.length > 0" class="mt-1 ml-20 text-gray-500 break-all whitespace-pre-wrap">
              {{ formatArgs(log.args) }}
            </div>
          </div>
          <div ref="logsEndRef" />
        </template>
      </div>
    </div>
  </div>
</template>
