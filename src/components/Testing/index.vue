<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  CheckCircle,
  XCircle,
  Play,
  Loader2,
  Stethoscope,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { testingLogger } from '../../lib/logger'

interface DiagnosticResult {
  name: string
  passed: boolean
  message: string
  suggestion: string | null
}

const diagnosticResults = ref<DiagnosticResult[]>([])
const loading = ref(false)

const runDiagnostics = async () => {
  testingLogger.action('运行系统诊断')
  testingLogger.info('开始系统诊断...')
  loading.value = true
  diagnosticResults.value = []
  try {
    const results = await invoke<DiagnosticResult[]>('run_doctor')
    testingLogger.info(`诊断完成，共 ${results.length} 项检查`)
    const passed = results.filter(r => r.passed).length
    testingLogger.state('诊断结果', { total: results.length, passed, failed: results.length - passed })
    diagnosticResults.value = results
  } catch (e) {
    testingLogger.error('诊断执行失败', e)
    diagnosticResults.value = [{
      name: '诊断执行',
      passed: false,
      message: String(e),
      suggestion: '请检查 OpenClaw 是否正确安装',
    }]
  } finally {
    loading.value = false
  }
}

const passedCount = computed(() => diagnosticResults.value.filter(r => r.passed).length)
const failedCount = computed(() => diagnosticResults.value.filter(r => !r.passed).length)
</script>

<template>
  <div class="h-full overflow-y-auto scroll-container pr-2">
    <div class="space-y-6">
      <div class="bg-dark-700 rounded-2xl p-6 border border-dark-500">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
              <Stethoscope :size="20" class="text-purple-400" />
            </div>
            <div>
              <h3 class="text-lg font-semibold text-white">系统诊断</h3>
              <p class="text-xs text-gray-500">检查 OpenClaw 安装和配置状态</p>
            </div>
          </div>
          <button
            @click="runDiagnostics"
            :disabled="loading"
            class="btn-primary flex items-center gap-2"
          >
            <Loader2 v-if="loading" :size="16" class="animate-spin" />
            <Play v-else :size="16" />
            运行诊断
          </button>
        </div>

        <Transition name="fade">
          <div v-if="diagnosticResults.length > 0" class="flex gap-4 mb-4 p-3 bg-dark-600 rounded-lg">
            <div class="flex items-center gap-2">
              <CheckCircle :size="16" class="text-green-400" />
              <span class="text-sm text-green-400">{{ passedCount }} 项通过</span>
            </div>
            <div v-if="failedCount > 0" class="flex items-center gap-2">
              <XCircle :size="16" class="text-red-400" />
              <span class="text-sm text-red-400">{{ failedCount }} 项失败</span>
            </div>
          </div>
        </Transition>

        <Transition name="fade-slide" mode="out-in">
          <div v-if="diagnosticResults.length > 0" class="space-y-2">
            <div
              v-for="(result, index) in diagnosticResults"
              :key="index"
              :class="clsx(
                'flex items-start gap-3 p-3 rounded-lg',
                result.passed ? 'bg-green-500/10' : 'bg-red-500/10'
              )"
            >
              <CheckCircle v-if="result.passed" :size="18" class="text-green-400 mt-0.5 flex-shrink-0" />
              <XCircle v-else :size="18" class="text-red-400 mt-0.5 flex-shrink-0" />
              <div class="flex-1 min-w-0">
                <p :class="['text-sm font-medium', result.passed ? 'text-green-400' : 'text-red-400']">
                  {{ result.name }}
                </p>
                <p class="text-xs text-gray-400 mt-1 whitespace-pre-wrap break-words">{{ result.message }}</p>
                <p v-if="result.suggestion" class="text-xs text-amber-400 mt-1">
                  💡 {{ result.suggestion }}
                </p>
              </div>
            </div>
          </div>
        </Transition>

        <div v-if="diagnosticResults.length === 0 && !loading" class="text-center py-8 text-gray-500">
          <Stethoscope :size="48" class="mx-auto mb-3 opacity-30" />
          <p>点击"运行诊断"按钮开始检查系统状态</p>
        </div>
      </div>

      <div class="bg-dark-700/50 rounded-xl p-4 border border-dark-500">
        <h4 class="text-sm font-medium text-gray-400 mb-2">诊断说明</h4>
        <ul class="text-sm text-gray-500 space-y-1">
          <li>• 系统诊断会检查 Node.js、OpenClaw 安装、配置文件等状态</li>
          <li>• AI 连接测试请前往 <span class="text-claw-400">AI 配置</span> 页面进行</li>
          <li>• 渠道测试请前往 <span class="text-claw-400">消息渠道</span> 页面进行</li>
        </ul>
      </div>
    </div>
  </div>
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

.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
