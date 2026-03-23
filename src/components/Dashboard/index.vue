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
import { useDialog } from '../../composables/useDialog'

interface Props {
  envStatus: EnvironmentStatus | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  setupComplete: []
}>()

const dialog = useDialog()

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

const handleFix = async () => {
  if (!isTauri()) return
  
  const confirmed = await dialog.confirm(
    '将运行 openclaw doctor --fix 命令来自动修复配置问题。此操作会自动移除或更正无效的配置键值，是否继续？',
    {
      title: '确认修复',
      confirmText: '开始修复',
      cancelText: '取消',
    }
  )
  
  if (!confirmed) return
  
  actionLoading.value = true
  logs.value.push('[OTOClaw] 开始运行 openclaw doctor --fix...')
  
  try {
    const result = await invoke<{ success: boolean; message: string; details?: string }>('run_openclaw_fix')
    
    if (result.success) {
      logs.value.push(`[OTOClaw] ✓ ${result.message}`)
      if (result.details) {
        const detailLines = result.details.split('\n').filter(l => l.trim())
        detailLines.forEach(line => {
          logs.value.push(`  ${line}`)
        })
      }
      await dialog.alert(result.message, { 
        title: '修复完成',
        variant: 'success',
        details: result.details,
        detailsLabel: '修复详情'
      })
    } else {
      logs.value.push(`[OTOClaw] ✗ ${result.message}`)
      if (result.details) {
        const detailLines = result.details.split('\n').filter(l => l.trim())
        detailLines.forEach(line => {
          logs.value.push(`  ${line}`)
        })
      }
      await dialog.alert(result.message, { 
        title: '修复结果',
        variant: 'warning',
        details: result.details,
        detailsLabel: '修复日志'
      })
    }
    
    await fetchStatus()
  } catch (e) {
    const errorMsg = String(e)
    logs.value.push(`[OTOClaw] ✗ 修复失败: ${errorMsg}`)
    await dialog.alert(`修复失败: ${errorMsg}`, { 
      title: '错误',
      variant: 'error',
      details: errorMsg,
      detailsLabel: '错误详情'
    })
  } finally {
    actionLoading.value = false
  }
}

const handleDiagnose = async () => {
  if (!isTauri()) return
  
  actionLoading.value = true
  logs.value.push('[OTOClaw] 开始运行系统诊断...')
  
  try {
    const result = await invoke<{ results: Array<{ name: string; passed: boolean; message: string; suggestion?: string }> }>('run_doctor')
    
    logs.value.push('[OTOClaw] 诊断结果:')
    let hasIssues = false
    const detailsLines: string[] = []
    
    for (const item of result.results) {
      const icon = item.passed ? '✓' : '✗'
      const logLine = `${icon} ${item.name}: ${item.message}`
      logs.value.push(`  ${logLine}`)
      detailsLines.push(logLine)
      
      if (!item.passed) {
        hasIssues = true
        if (item.suggestion) {
          logs.value.push(`    建议: ${item.suggestion}`)
          detailsLines.push(`  建议: ${item.suggestion}`)
        }
      }
    }
    
    const passedCount = result.results.filter(r => r.passed).length
    const totalCount = result.results.length
    const detailsText = detailsLines.join('\n')
    
    if (hasIssues) {
      await dialog.alert(
        `诊断完成：${passedCount}/${totalCount} 项通过\n\n建议点击"修复"按钮自动修复问题`,
        { 
          title: '诊断结果',
          variant: 'warning',
          details: detailsText,
          detailsLabel: '诊断详情'
        }
      )
    } else {
      await dialog.alert(
        `诊断完成：所有 ${totalCount} 项检查均通过`,
        { 
          title: '诊断结果',
          variant: 'success',
          details: detailsText,
          detailsLabel: '诊断详情'
        }
      )
    }
  } catch (e) {
    const errorMsg = String(e)
    logs.value.push(`[OTOClaw] ✗ 诊断失败: ${errorMsg}`)
    await dialog.alert(`诊断失败: ${errorMsg}`, { 
      title: '错误',
      variant: 'error',
      details: errorMsg,
      detailsLabel: '错误详情'
    })
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
  <div class="overflow-y-auto pr-2 h-full scroll-container">
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
          @diagnose="handleDiagnose"
          @fix="handleFix"
        />
      </Transition>

      <Transition name="fade-slide">
        <div class="overflow-hidden rounded-2xl border bg-dark-700 border-dark-500">
          <div 
            class="flex justify-between items-center px-4 py-3 cursor-pointer bg-dark-600/50"
            @click="logsExpanded = !logsExpanded"
          >
            <div class="flex gap-2 items-center">
              <Terminal :size="16" class="text-gray-500" />
              <span class="text-sm font-medium text-white">实时日志</span>
              <span class="text-xs text-gray-500">
                ({{ logs.length }} 行)
              </span>
            </div>
            <div class="flex gap-3 items-center">
              <template v-if="logsExpanded">
                <label 
                  class="flex gap-2 items-center text-xs text-gray-400"
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
            class="overflow-y-auto p-4 h-64 font-mono text-xs leading-relaxed bg-dark-800"
          >
            <div v-if="logs.length === 0" class="flex justify-center items-center h-full text-gray-500">
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
