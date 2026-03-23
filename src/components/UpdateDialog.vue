<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { X, Download, Loader2, AlertCircle } from 'lucide-vue-next'
import { api, type OTOClawUpdateInfo, type DownloadProgress } from '../lib/tauri'
import { appLogger } from '../lib/logger'

const props = defineProps<{
  updateInfo: OTOClawUpdateInfo | null
}>()

const emit = defineEmits<{
  close: []
  skip: [version: string]
}>()

const downloading = ref(false)
const installing = ref(false)
const progress = ref<DownloadProgress | null>(null)
const error = ref<string | null>(null)
const downloadedFilePath = ref<string | null>(null)

let unlisten: (() => void) | null = null

onMounted(async () => {
  unlisten = await listen<DownloadProgress>('update-download-progress', (event) => {
    progress.value = event.payload
    appLogger.info('下载进度', event.payload)
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
})

const formatFileSize = (bytes: number | null): string => {
  if (!bytes) return ''
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

const releaseNotesHtml = computed(() => {
  if (!props.updateInfo?.release_notes) return ''
  return props.updateInfo.release_notes
    .replace(/\n/g, '<br>')
    .replace(/#{1,6}\s(.*)/g, '<strong>$1</strong>')
})

const handleUpdate = async () => {
  if (!props.updateInfo?.download_url) {
    error.value = '无法获取下载链接'
    return
  }

  downloading.value = true
  error.value = null
  progress.value = null

  try {
    appLogger.info('开始下载更新', props.updateInfo.download_url)
    const filePath = await api.downloadUpdate(props.updateInfo.download_url)
    downloadedFilePath.value = filePath
    appLogger.info('下载完成', filePath)

    downloading.value = false
    installing.value = true

    const result = await api.installUpdate(filePath)
    if (result.success) {
      appLogger.info('安装完成', result)
    } else {
      error.value = result.error || result.message
    }
  } catch (e) {
    appLogger.error('更新失败', e)
    error.value = String(e)
  } finally {
    downloading.value = false
    installing.value = false
  }
}

const handleSkip = () => {
  if (props.updateInfo?.latest_version) {
    emit('skip', props.updateInfo.latest_version)
  }
  emit('close')
}

const handleClose = () => {
  if (downloading.value) {
    api.cancelUpdate()
  }
  emit('close')
}
</script>

<template>
  <Transition name="fade">
    <div
      v-if="updateInfo && updateInfo.update_available"
      class="flex fixed inset-0 z-50 justify-center items-center backdrop-blur-sm bg-black/60"
    >
      <div class="flex flex-col mx-4 w-full max-w-md rounded-2xl border shadow-2xl bg-dark-700 border-dark-500 max-h-[90vh]">
        <div class="flex shrink-0 justify-between items-center p-6 border-b border-dark-500">
          <div class="flex gap-3 items-center">
            <div class="flex justify-center items-center w-10 h-10 rounded-xl bg-claw-500/20">
              <Download :size="20" class="text-claw-400" />
            </div>
            <div>
              <h3 class="text-lg font-semibold text-white">发现新版本</h3>
              <p class="text-xs text-gray-500">OTOClaw 有更新可用</p>
            </div>
          </div>
          <button
            @click="handleClose"
            :disabled="downloading || installing"
            class="text-gray-400 transition-colors hover:text-white disabled:opacity-50"
          >
            <X :size="20" />
          </button>
        </div>

        <div class="flex-1 overflow-y-auto p-6">
          <div class="mb-6">
            <div class="flex justify-between items-center mb-4">
              <div>
                <p class="text-sm text-gray-400">当前版本</p>
                <p class="text-lg font-medium text-white">{{ updateInfo.current_version }}</p>
              </div>
              <div class="text-right">
                <p class="text-sm text-gray-400">最新版本</p>
                <p class="text-lg font-medium text-claw-400">{{ updateInfo.latest_version }}</p>
              </div>
            </div>

            <div v-if="updateInfo.file_size" class="py-2 text-xs text-center text-gray-500">
              更新大小: {{ formatFileSize(updateInfo.file_size) }}
            </div>
          </div>

          <div v-if="updateInfo.release_notes" class="p-4 mb-6 rounded-lg bg-dark-600">
            <h4 class="mb-2 text-sm font-medium text-white">更新内容</h4>
            <div
              class="text-sm text-gray-300 prose prose-invert prose-sm max-w-none"
              v-html="releaseNotesHtml"
            />
          </div>

          <div v-if="progress" class="mb-6">
            <div class="flex justify-between items-center mb-2">
              <span class="text-sm text-gray-400">下载进度</span>
              <span class="text-sm text-white">{{ progress.percentage.toFixed(1) }}%</span>
            </div>
            <div class="overflow-hidden h-2 rounded-full bg-dark-500">
              <div
                class="h-full transition-all duration-300 rounded-full bg-claw-500"
                :style="{ width: `${progress.percentage}%` }"
              />
            </div>
            <div class="flex justify-between items-center mt-2 text-xs text-gray-500">
              <span>{{ formatFileSize(progress.downloaded) }} / {{ formatFileSize(progress.total) }}</span>
              <span>{{ progress.speed }}</span>
            </div>
          </div>

          <div v-if="error" class="p-4 mb-6 rounded-lg bg-red-900/30 border border-red-800">
            <div class="flex gap-2 items-center mb-1">
              <AlertCircle :size="16" class="text-red-400" />
              <span class="text-sm font-medium text-red-300">更新失败</span>
            </div>
            <p class="text-sm text-red-400">{{ error }}</p>
          </div>

          <div v-if="installing" class="flex gap-3 items-center p-4 mb-6 rounded-lg bg-claw-500/10 border border-claw-500/30">
            <Loader2 :size="20" class="animate-spin text-claw-400" />
            <span class="text-sm text-claw-300">正在安装更新，请稍候...</span>
          </div>
        </div>

        <div class="shrink-0 p-6 border-t border-dark-500">
          <div v-if="!downloading && !installing" class="flex gap-3">
            <button
              @click="handleSkip"
              class="flex-1 px-4 py-2.5 text-white rounded-lg transition-colors bg-dark-600 hover:bg-dark-500"
            >
              跳过此版本
            </button>
            <button
              @click="handleClose"
              class="flex-1 px-4 py-2.5 text-white rounded-lg transition-colors bg-dark-600 hover:bg-dark-500"
            >
              稍后提醒
            </button>
            <button
              @click="handleUpdate"
              class="flex flex-1 gap-2 justify-center items-center px-4 py-2.5 text-white rounded-lg transition-colors bg-claw-600 hover:bg-claw-500"
            >
              <Download :size="16" />
              立即更新
            </button>
          </div>

          <div v-else-if="downloading" class="flex gap-3">
            <button
              @click="handleClose"
              class="flex-1 px-4 py-2.5 text-white rounded-lg transition-colors bg-dark-600 hover:bg-dark-500"
            >
              取消下载
            </button>
            <div class="flex flex-1 gap-2 justify-center items-center px-4 py-2.5 text-white rounded-lg bg-claw-600/50">
              <Loader2 :size="16" class="animate-spin" />
              下载中...
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
