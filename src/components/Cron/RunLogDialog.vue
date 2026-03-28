<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  X,
  Clock,
  CheckCircle,
  XCircle,
  Play,
  Loader2,
  History,
  Calendar,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { api, type CronRunLogEntry, type CronRunLogResult } from '../../lib/tauri'

const props = defineProps<{
  jobId: string
  jobName?: string
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const logs = ref<CronRunLogEntry[]>([])
const loading = ref(false)
const error = ref('')

const loadLogs = async () => {
  if (!props.jobId) {
    error.value = '任务ID不能为空'
    return
  }
  loading.value = true
  error.value = ''
  try {
    const result: CronRunLogResult = await api.getCronRunLogs(props.jobId, 50)
    logs.value = result.entries
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  if (props.visible) {
    loadLogs()
  }
})

const formatTime = (ts: number) => {
  return new Date(ts).toLocaleString('zh-CN')
}

const formatDuration = (ms?: number) => {
  if (!ms) return '-'
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(1)}s`
}

const getStatusIcon = (status?: string) => {
  switch (status) {
    case 'ok':
      return CheckCircle
    case 'error':
      return XCircle
    case 'running':
      return Loader2
    default:
      return Clock
  }
}

const getStatusClass = (status?: string) => {
  switch (status) {
    case 'ok':
      return 'text-green-400'
    case 'error':
      return 'text-red-400'
    case 'running':
      return 'text-blue-400'
    default:
      return 'text-gray-400'
  }
}

const getActionText = (action: string) => {
  switch (action) {
    case 'start':
      return '开始执行'
    case 'complete':
      return '执行完成'
    case 'skip':
      return '跳过执行'
    default:
      return action
  }
}

const handleClose = () => {
  emit('close')
}

const groupedLogs = computed(() => {
  const groups: Record<string, CronRunLogEntry[]> = {}
  logs.value.forEach((log) => {
    const date = new Date(log.ts).toLocaleDateString('zh-CN')
    if (!groups[date]) {
      groups[date] = []
    }
    groups[date].push(log)
  })
  return groups
})
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
        @click.self="handleClose"
      >
        <Transition
          enter-active-class="transition duration-300 ease-out"
          enter-from-class="opacity-0 scale-95 translate-y-4"
          enter-to-class="opacity-100 scale-100 translate-y-0"
          leave-active-class="transition duration-200 ease-in"
          leave-from-class="opacity-100 scale-100 translate-y-0"
          leave-to-class="opacity-0 scale-95 translate-y-4"
        >
          <div
            v-if="visible"
            class="bg-[#1a1a1a] border border-white/10 rounded-xl w-full max-w-3xl max-h-[80vh] flex flex-col shadow-2xl"
          >
            <!-- Header -->
            <div class="flex items-center justify-between p-4 border-b border-white/10">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-blue-500/20 flex items-center justify-center">
                  <History class="w-5 h-5 text-blue-400" />
                </div>
                <div>
                  <h3 class="text-lg font-semibold text-white">
                    {{ jobName ? `${jobName} - 执行日志` : '执行日志' }}
                  </h3>
                  <p class="text-sm text-gray-400">
                    共 {{ logs.length }} 条记录
                  </p>
                </div>
              </div>
              <button
                @click="handleClose"
                class="p-2 rounded-lg hover:bg-white/10 transition-colors"
              >
                <X class="w-5 h-5 text-gray-400" />
              </button>
            </div>

            <!-- Content -->
            <div class="flex-1 overflow-y-auto p-4">
              <div v-if="loading" class="flex items-center justify-center py-12">
                <Loader2 class="w-8 h-8 text-blue-400 animate-spin" />
              </div>

              <div v-else-if="error" class="text-center py-12">
                <XCircle class="w-12 h-12 text-red-400 mx-auto mb-4" />
                <p class="text-red-400">{{ error }}</p>
                <button
                  @click="loadLogs"
                  class="mt-4 px-4 py-2 bg-blue-500 hover:bg-blue-600 rounded-lg text-white text-sm transition-colors"
                >
                  重试
                </button>
              </div>

              <div v-else-if="logs.length === 0" class="text-center py-12">
                <History class="w-12 h-12 text-gray-500 mx-auto mb-4" />
                <p class="text-gray-400">暂无执行记录</p>
              </div>

              <div v-else class="space-y-6">
                <div v-for="[date, dayLogs] in Object.entries(groupedLogs)" :key="date">
                  <div class="flex items-center gap-2 mb-3">
                    <Calendar class="w-4 h-4 text-gray-500" />
                    <span class="text-sm font-medium text-gray-400">{{ date }}</span>
                  </div>
                  
                  <div class="space-y-2">
                    <div
                      v-for="log in dayLogs"
                      :key="log.ts + log.action"
                      :class="clsx(
                        'p-3 rounded-lg border transition-colors',
                        log.status === 'error'
                          ? 'bg-red-500/10 border-red-500/20'
                          : log.status === 'ok'
                          ? 'bg-green-500/10 border-green-500/20'
                          : 'bg-white/5 border-white/10'
                      )"
                    >
                      <div class="flex items-start gap-3">
                        <component
                          :is="getStatusIcon(log.status)"
                          :class="clsx('w-5 h-5 mt-0.5', getStatusClass(log.status))"
                        />
                        <div class="flex-1 min-w-0">
                          <div class="flex items-center gap-2 flex-wrap">
                            <span class="font-medium text-white">
                              {{ getActionText(log.action) }}
                            </span>
                            <span class="text-xs text-gray-500">
                              {{ formatTime(log.ts) }}
                            </span>
                          </div>
                          
                          <div v-if="log.summary" class="mt-1 text-sm text-gray-300">
                            {{ log.summary }}
                          </div>
                          
                          <div v-if="log.error" class="mt-1 text-sm text-red-400">
                            错误: {{ log.error }}
                          </div>
                          
                          <div class="mt-2 flex items-center gap-4 text-xs text-gray-500">
                            <span v-if="log.durationMs !== undefined && log.durationMs !== null">
                              耗时: {{ formatDuration(log.durationMs) }}
                            </span>
                            <span v-if="log.nextRunAtMs">
                              下次执行: {{ formatTime(log.nextRunAtMs) }}
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Footer -->
            <div class="p-4 border-t border-white/10 flex justify-end">
              <button
                @click="loadLogs"
                class="px-4 py-2 bg-white/10 hover:bg-white/20 rounded-lg text-white text-sm transition-colors flex items-center gap-2"
              >
                <Play class="w-4 h-4" />
                刷新
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>
