<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import {
  X,
  Clock,
  CheckCircle,
  XCircle,
  Loader2,
  History,
  Calendar,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
  FileText,
  HardDrive,
} from 'lucide-vue-next'
import clsx from 'clsx'
import {
  api,
  type CronRunLogEntry,
  type CronRunLogFile,
} from '../../lib/tauri'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const files = ref<CronRunLogFile[]>([])
const currentFileIndex = ref(0)
const entries = ref<CronRunLogEntry[]>([])
const loading = ref(false)
const loadingFiles = ref(false)
const error = ref('')

const currentFile = computed(() => {
  if (files.value.length === 0) return null
  return files.value[currentFileIndex.value]
})

const loadFiles = async () => {
  loadingFiles.value = true
  error.value = ''
  try {
    const result = await api.getCronRunLogFiles()
    files.value = result.files
    if (result.files.length > 0) {
      currentFileIndex.value = 0
      await loadFileContent(result.files[0].name)
    } else {
      entries.value = []
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    loadingFiles.value = false
  }
}

const loadFileContent = async (fileName: string) => {
  loading.value = true
  error.value = ''
  try {
    const result = await api.getCronRunLogFileContent(fileName)
    entries.value = result.entries
  } catch (e) {
    error.value = String(e)
    entries.value = []
  } finally {
    loading.value = false
  }
}

const handlePrevFile = () => {
  if (currentFileIndex.value > 0) {
    currentFileIndex.value--
    if (currentFile.value) {
      loadFileContent(currentFile.value.name)
    }
  }
}

const handleNextFile = () => {
  if (currentFileIndex.value < files.value.length - 1) {
    currentFileIndex.value++
    if (currentFile.value) {
      loadFileContent(currentFile.value.name)
    }
  }
}

const handleRefresh = () => {
  loadFiles()
}

const handleClose = () => {
  emit('close')
}

const formatTime = (ts: number) => {
  return new Date(ts).toLocaleString('zh-CN')
}

const formatSize = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

const formatDuration = (ms?: number) => {
  if (!ms) return '-'
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(1)}s`
}

const getStatusIcon = (status?: string) => {
  switch (status) {
    case 'ok':
    case 'finished':
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
    case 'finished':
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
    case 'finished':
      return '执行完成'
    case 'complete':
      return '执行完成'
    case 'skip':
      return '跳过执行'
    default:
      return action
  }
}

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      loadFiles()
    }
  }
)

onMounted(() => {
  if (props.visible) {
    loadFiles()
  }
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
            class="bg-[#1a1a1a] border border-white/10 rounded-xl w-full max-w-4xl max-h-[85vh] flex flex-col shadow-2xl"
          >
            <!-- Header -->
            <div class="flex items-center justify-between p-4 border-b border-white/10">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-purple-500/20 flex items-center justify-center">
                  <FileText class="w-5 h-5 text-purple-400" />
                </div>
                <div>
                  <h3 class="text-lg font-semibold text-white">
                    执行日志文件浏览器
                  </h3>
                  <p class="text-sm text-gray-400">
                    ~/.openclaw/cron/runs 目录
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

            <!-- File Navigation -->
            <div class="flex items-center justify-between p-4 border-b border-white/10 bg-white/5">
              <div class="flex items-center gap-2">
                <button
                  @click="handleRefresh"
                  :disabled="loadingFiles"
                  class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/10 hover:bg-white/20 text-white text-sm transition-colors disabled:opacity-50"
                >
                  <RefreshCw :class="['w-4 h-4', { 'animate-spin': loadingFiles }]" />
                  刷新
                </button>
              </div>

              <div v-if="files.length > 0" class="flex items-center gap-4">
                <button
                  @click="handlePrevFile"
                  :disabled="currentFileIndex === 0"
                  class="p-1.5 rounded-lg hover:bg-white/10 transition-colors disabled:opacity-30"
                >
                  <ChevronLeft class="w-5 h-5 text-white" />
                </button>
                
                <div class="text-center">
                  <div class="text-sm font-medium text-white">
                    {{ currentFile?.name }}
                  </div>
                  <div class="text-xs text-gray-400">
                    {{ currentFileIndex + 1 }} / {{ files.length }} 个文件
                  </div>
                </div>

                <button
                  @click="handleNextFile"
                  :disabled="currentFileIndex === files.length - 1"
                  class="p-1.5 rounded-lg hover:bg-white/10 transition-colors disabled:opacity-30"
                >
                  <ChevronRight class="w-5 h-5 text-white" />
                </button>
              </div>

              <div v-else class="text-sm text-gray-400">
                暂无日志文件
              </div>
            </div>

            <!-- File Info -->
            <div v-if="currentFile" class="flex items-center gap-6 px-4 py-2 border-b border-white/10 bg-white/[0.02] text-xs text-gray-400">
              <div class="flex items-center gap-1.5">
                <HardDrive class="w-3.5 h-3.5" />
                <span>{{ formatSize(currentFile.size) }}</span>
              </div>
              <div class="flex items-center gap-1.5">
                <History class="w-3.5 h-3.5" />
                <span>{{ currentFile.entryCount }} 条记录</span>
              </div>
              <div class="flex items-center gap-1.5">
                <Calendar class="w-3.5 h-3.5" />
                <span>修改于 {{ formatTime(currentFile.modifiedMs) }}</span>
              </div>
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
                  @click="handleRefresh"
                  class="mt-4 px-4 py-2 bg-blue-500 hover:bg-blue-600 rounded-lg text-white text-sm transition-colors"
                >
                  重试
                </button>
              </div>

              <div v-else-if="entries.length === 0" class="text-center py-12">
                <History class="w-12 h-12 text-gray-500 mx-auto mb-4" />
                <p class="text-gray-400">暂无执行记录</p>
              </div>

              <div v-else class="space-y-3">
                <div
                  v-for="entry in entries"
                  :key="entry.ts + entry.action"
                  :class="clsx(
                    'p-3 rounded-lg border transition-colors',
                    entry.status === 'error'
                      ? 'bg-red-500/10 border-red-500/20'
                      : entry.status === 'ok' || entry.status === 'finished'
                      ? 'bg-green-500/10 border-green-500/20'
                      : 'bg-white/5 border-white/10'
                  )"
                >
                  <div class="flex items-start gap-3">
                    <component
                      :is="getStatusIcon(entry.status)"
                      :class="clsx('w-5 h-5 mt-0.5 flex-shrink-0', getStatusClass(entry.status))"
                    />
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2 flex-wrap">
                        <span class="font-medium text-white">
                          {{ getActionText(entry.action) }}
                        </span>
                        <span class="text-xs text-gray-500">
                          {{ formatTime(entry.ts) }}
                        </span>
                      </div>
                      
                      <div v-if="entry.summary" class="mt-2 text-sm text-gray-300 whitespace-pre-wrap break-words">
                        {{ entry.summary }}
                      </div>
                      
                      <div v-if="entry.error" class="mt-2 text-sm text-red-400 break-words">
                        错误: {{ entry.error }}
                      </div>
                      
                      <div class="mt-2 flex items-center gap-4 text-xs text-gray-500 flex-wrap">
                        <span v-if="entry.durationMs !== undefined && entry.durationMs !== null">
                          耗时: {{ formatDuration(entry.durationMs) }}
                        </span>
                        <span v-if="entry.model">
                          模型: {{ entry.model }}
                        </span>
                        <span v-if="entry.provider">
                          提供商: {{ entry.provider }}
                        </span>
                        <span v-if="entry.sessionId">
                          会话: {{ entry.sessionId.slice(0, 8) }}...
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Footer -->
            <div class="p-4 border-t border-white/10 flex justify-between items-center">
              <div class="text-xs text-gray-500">
                共 {{ files.length }} 个日志文件，当前显示 {{ entries.length }} 条记录
              </div>
              <button
                @click="handleClose"
                class="px-4 py-2 bg-white/10 hover:bg-white/20 rounded-lg text-white text-sm transition-colors"
              >
                关闭
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>
