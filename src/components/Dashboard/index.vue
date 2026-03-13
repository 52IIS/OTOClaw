<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import StatusCard from './StatusCard.vue'
import QuickActions from './QuickActions.vue'
import SystemInfo from './SystemInfo.vue'
import Setup from '../Setup/index.vue'
import { api, ServiceStatus, isTauri } from '../../lib/tauri'
import { Terminal, RefreshCw, ChevronDown, ChevronUp } from 'lucide-vue-next'
import clsx from 'clsx'
import type { EnvironmentStatus } from '../../vite-env.d'

interface Props {
  envStatus: EnvironmentStatus | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  setupComplete: []
}>()

const status = ref<ServiceStatus | null>(null)
const loading = ref(true)
const actionLoading = ref(false)
const logs = ref<string[]>([])
const logsExpanded = ref(true)
const autoRefreshLogs = ref(true)
const logsContainerRef = ref<HTMLDivElement | null>(null)

const fetchStatus = async () => {
  if (!isTauri()) {
    loading.value = false
    return
  }
  try {
    const result = await api.getServiceStatus()
    status.value = result
  } catch {
  } finally {
    loading.value = false
  }
}

const fetchLogs = async () => {
  if (!isTauri()) return
  try {
    const result = await invoke<string[]>('get_logs', { lines: 50 })
    logs.value = result
  } catch {
  }
}

let statusInterval: ReturnType<typeof setInterval> | null = null
let logsInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  fetchStatus()
  fetchLogs()
  if (!isTauri()) return
  
  statusInterval = setInterval(fetchStatus, 3000)
  if (autoRefreshLogs.value) {
    logsInterval = setInterval(fetchLogs, 2000)
  }
})

watch(autoRefreshLogs, (newVal) => {
  if (logsInterval) clearInterval(logsInterval)
  if (newVal && isTauri()) {
    logsInterval = setInterval(fetchLogs, 2000)
  }
})

watch([logs, logsExpanded], () => {
  if (logsExpanded.value && logsContainerRef.value) {
    logsContainerRef.value.scrollTop = logsContainerRef.value.scrollHeight
  }
})

onUnmounted(() => {
  if (statusInterval) clearInterval(statusInterval)
  if (logsInterval) clearInterval(logsInterval)
})

const handleStart = async () => {
  if (!isTauri()) return
  actionLoading.value = true
  try {
    await api.startService()
    await fetchStatus()
    await fetchLogs()
  } catch (e) {
    console.error('启动失败:', e)
  } finally {
    actionLoading.value = false
  }
}

const handleStop = async () => {
  if (!isTauri()) return
  actionLoading.value = true
  try {
    await api.stopService()
    await fetchStatus()
    await fetchLogs()
  } catch (e) {
    console.error('停止失败:', e)
  } finally {
    actionLoading.value = false
  }
}

const handleRestart = async () => {
  if (!isTauri()) return
  actionLoading.value = true
  try {
    await api.restartService()
    await fetchStatus()
    await fetchLogs()
  } catch (e) {
    console.error('重启失败:', e)
  } finally {
    actionLoading.value = false
  }
}

const getLogLineClass = (line: string) => {
  if (line.includes('error') || line.includes('Error') || line.includes('ERROR')) {
    return 'text-red-400'
  }
  if (line.includes('warn') || line.includes('Warn') || line.includes('WARN')) {
    return 'text-yellow-400'
  }
  if (line.includes('info') || line.includes('Info') || line.includes('INFO')) {
    return 'text-green-400'
  }
  return 'text-gray-400'
}

const needsSetup = computed(() => props.envStatus && !props.envStatus.ready)
</script>

<template>
  <div class="h-full overflow-y-auto scroll-container pr-2">
    <div class="space-y-6">
      <Transition name="fade-slide">
        <Setup v-if="needsSetup" embedded @complete="emit('setupComplete')" />
      </Transition>

      <Transition name="fade-slide">
        <StatusCard :status="status" :loading="loading" />
      </Transition>

      <Transition name="fade-slide">
        <QuickActions
          :status="status"
          :loading="actionLoading"
          @start="handleStart"
          @stop="handleStop"
          @restart="handleRestart"
        />
      </Transition>

      <Transition name="fade-slide">
        <div class="bg-dark-700 rounded-2xl border border-dark-500 overflow-hidden">
          <div 
            class="flex items-center justify-between px-4 py-3 bg-dark-600/50 cursor-pointer"
            @click="logsExpanded = !logsExpanded"
          >
            <div class="flex items-center gap-2">
              <Terminal :size="16" class="text-gray-500" />
              <span class="text-sm font-medium text-white">实时日志</span>
              <span class="text-xs text-gray-500">
                ({{ logs.length }} 行)
              </span>
            </div>
            <div class="flex items-center gap-3">
              <template v-if="logsExpanded">
                <label 
                  class="flex items-center gap-2 text-xs text-gray-400"
                  @click.stop
                >
                  <input
                    type="checkbox"
                    v-model="autoRefreshLogs"
                    class="w-3 h-3 rounded border-dark-500 bg-dark-600 text-claw-500"
                  />
                  自动刷新
                </label>
                <button
                  @click.stop="fetchLogs"
                  class="text-gray-500 hover:text-white"
                  title="刷新日志"
                >
                  <RefreshCw :size="14" />
                </button>
              </template>
              <ChevronUp v-if="logsExpanded" :size="16" class="text-gray-500" />
              <ChevronDown v-else :size="16" class="text-gray-500" />
            </div>
          </div>

          <div
            v-show="logsExpanded"
            ref="logsContainerRef"
            class="h-64 overflow-y-auto p-4 font-mono text-xs leading-relaxed bg-dark-800"
          >
            <div v-if="logs.length === 0" class="h-full flex items-center justify-center text-gray-500">
              <p>暂无日志，请先启动服务</p>
            </div>
            <template v-else>
              <div
                v-for="(line, index) in logs"
                :key="index"
                :class="clsx('py-0.5 whitespace-pre-wrap break-all', getLogLineClass(line))"
              >
                {{ line }}
              </div>
            </template>
          </div>
        </div>
      </Transition>

      <Transition name="fade-slide">
        <SystemInfo />
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}
</style>
