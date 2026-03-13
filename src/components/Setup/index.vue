<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  CheckCircle2,
  Loader2,
  Download,
  ArrowRight,
  RefreshCw,
  ExternalLink,
  Cpu,
  Package
} from 'lucide-vue-next'
import type { EnvironmentStatus, InstallResult } from '../../vite-env'

interface Props {
  embedded?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  embedded: false
})

const emit = defineEmits<{
  complete: []
}>()

const envStatus = ref<EnvironmentStatus | null>(null)
const checking = ref(true)
const installing = ref<'nodejs' | 'openclaw' | null>(null)
const error = ref<string | null>(null)
const step = ref<'check' | 'install' | 'complete'>('check')

const checkEnvironment = async () => {
  checking.value = true
  error.value = null
  try {
    const status = await invoke<EnvironmentStatus>('check_environment')
    envStatus.value = status

    if (status.ready) {
      step.value = 'complete'
      setTimeout(() => emit('complete'), 1500)
    } else {
      step.value = 'install'
    }
  } catch (e) {
    error.value = `检查环境失败: ${e}`
  } finally {
    checking.value = false
  }
}

onMounted(() => {
  checkEnvironment()
})

const handleInstallNodejs = async () => {
  installing.value = 'nodejs'
  error.value = null

  try {
    const result = await invoke<InstallResult>('install_nodejs')

    if (result.success) {
      await checkEnvironment()
    } else if (result.message.includes('重启')) {
      error.value = 'Node.js 安装完成，请重启应用以使环境变量生效'
    } else {
      await invoke<string>('open_install_terminal', { installType: 'nodejs' })
      error.value = '已打开安装终端，请在终端中完成安装后点击"重新检查"'
    }
  } catch (e) {
    try {
      await invoke<string>('open_install_terminal', { installType: 'nodejs' })
      error.value = '已打开安装终端，请在终端中完成安装后点击"重新检查"'
    } catch (termErr) {
      error.value = `安装失败: ${e}。${termErr}`
    }
  } finally {
    installing.value = null
  }
}

const handleInstallOpenclaw = async () => {
  installing.value = 'openclaw'
  error.value = null

  try {
    const result = await invoke<InstallResult>('install_openclaw')

    if (result.success) {
      await invoke<InstallResult>('init_openclaw_config')
      await checkEnvironment()
    } else {
      await invoke<string>('open_install_terminal', { installType: 'openclaw' })
      error.value = '已打开安装终端，请在终端中完成安装后点击"重新检查"'
    }
  } catch (e) {
    try {
      await invoke<string>('open_install_terminal', { installType: 'openclaw' })
      error.value = '已打开安装终端，请在终端中完成安装后点击"重新检查"'
    } catch (termErr) {
      error.value = `安装失败: ${e}。${termErr}`
    }
  } finally {
    installing.value = null
  }
}

const getOsName = (os: string) => {
  switch (os) {
    case 'windows': return 'Windows'
    case 'macos': return 'macOS'
    case 'linux': return 'Linux'
    default: return os
  }
}

const nodeJsStatusClass = computed(() => {
  if (!envStatus.value) return 'bg-red-500/20 text-red-400'
  return envStatus.value.node_installed && envStatus.value.node_version_ok
    ? 'bg-green-500/20 text-green-400'
    : 'bg-red-500/20 text-red-400'
})

const openclawStatusClass = computed(() => {
  if (!envStatus.value) return 'bg-red-500/20 text-red-400'
  return envStatus.value.openclaw_installed
    ? 'bg-green-500/20 text-green-400'
    : 'bg-red-500/20 text-red-400'
})
</script>

<template>
  <div v-if="props.embedded" class="p-6 bg-gradient-to-br rounded-2xl border from-yellow-500/10 to-orange-500/10 border-yellow-500/30">
    <div class="flex gap-4 items-start mb-4">
      <div class="flex flex-shrink-0 justify-center items-center w-12 h-12 bg-gradient-to-br from-yellow-500 to-orange-500 rounded-xl">
        <span class="text-2xl">⚠️</span>
      </div>
      <div>
        <h2 class="mb-1 text-lg font-bold text-white">环境配置</h2>
        <p class="text-sm text-dark-400">检测到缺少必要的依赖，请完成以下安装</p>
      </div>
    </div>

    <Transition name="fade" mode="out-in">
      <div v-if="checking" key="checking" class="py-6 text-center">
        <Loader2 class="mx-auto mb-3 w-10 h-10 animate-spin text-brand-500" />
        <p class="text-dark-300">正在检测系统环境...</p>
      </div>

      <div v-else-if="step === 'install' && envStatus" key="install" class="space-y-4">
        <div class="flex justify-between items-center">
          <div class="flex gap-3 items-center">
            <div :class="['p-2 rounded-lg', nodeJsStatusClass]">
              <Cpu class="w-5 h-5" />
            </div>
            <div>
              <p class="font-medium text-white">Node.js</p>
              <p class="text-sm text-dark-400">
                {{ envStatus.node_version
                  ? `${envStatus.node_version} ${envStatus.node_version_ok ? '✓' : '(需要 v22+)'}`
                  : '未安装' }}
              </p>
            </div>
          </div>

          <CheckCircle2 v-if="envStatus.node_installed && envStatus.node_version_ok" class="w-6 h-6 text-green-400" />
          <button
            v-else
            @click="handleInstallNodejs"
            :disabled="installing !== null"
            class="flex gap-2 items-center px-4 py-2 text-sm btn-primary"
          >
            <Loader2 v-if="installing === 'nodejs'" class="w-4 h-4 animate-spin" />
            <Download v-else class="w-4 h-4" />
            {{ installing === 'nodejs' ? '安装中...' : '安装' }}
          </button>
        </div>

        <div class="flex justify-between items-center">
          <div class="flex gap-3 items-center">
            <div :class="['p-2 rounded-lg', openclawStatusClass]">
              <Package class="w-5 h-5" />
            </div>
            <div>
              <p class="font-medium text-white">OpenClaw</p>
              <p class="text-sm text-dark-400">
                {{ envStatus.openclaw_version || '未安装' }}
              </p>
            </div>
          </div>

          <CheckCircle2 v-if="envStatus.openclaw_installed" class="w-6 h-6 text-green-400" />
          <button
            v-else
            @click="handleInstallOpenclaw"
            :disabled="installing !== null || !envStatus.node_version_ok"
            :class="[
              'btn-primary text-sm px-4 py-2 flex items-center gap-2',
              !envStatus.node_version_ok ? 'opacity-50 cursor-not-allowed' : ''
            ]"
            :title="!envStatus.node_version_ok ? '请先安装 Node.js' : ''"
          >
            <Loader2 v-if="installing === 'openclaw'" class="w-4 h-4 animate-spin" />
            <Download v-else class="w-4 h-4" />
            {{ installing === 'openclaw' ? '安装中...' : '安装' }}
          </button>
        </div>

        <Transition name="fade">
          <div v-if="error" class="p-3 rounded-lg border bg-yellow-500/10 border-yellow-500/30">
            <p class="text-sm text-yellow-400">{{ error }}</p>
          </div>
        </Transition>

        <div class="flex gap-3 pt-4 border-t border-dark-700/50">
          <button
            @click="checkEnvironment"
            :disabled="checking || installing !== null"
            class="flex flex-1 gap-2 justify-center items-center py-2.5 btn-secondary"
          >
            <RefreshCw :class="['w-4 h-4', checking ? 'animate-spin' : '']" />
            重新检查
          </button>

          <button
            v-if="envStatus.ready"
            @click="emit('complete')"
            class="flex flex-1 gap-2 justify-center items-center py-2.5 btn-primary"
          >
            开始使用
            <ArrowRight class="w-4 h-4" />
          </button>
        </div>

        <div class="pt-1 text-center">
          <a
            href="https://nodejs.org/en/download"
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex gap-1 items-center text-sm transition-colors text-dark-400 hover:text-brand-400"
          >
            手动下载 Node.js
            <ExternalLink class="w-3 h-3" />
          </a>
        </div>
      </div>

      <div v-else-if="step === 'complete'" key="complete" class="py-6 text-center">
        <CheckCircle2 class="mx-auto mb-3 w-12 h-12 text-green-400" />
        <h3 class="mb-1 text-lg font-bold text-white">环境就绪！</h3>
        <p class="text-sm text-dark-400">
          Node.js 和 OpenClaw 已正确安装
        </p>
      </div>
    </Transition>
  </div>

  <div v-else class="flex justify-center items-center p-8 min-h-screen bg-dark-900">
    <div class="fixed inset-0 pointer-events-none bg-gradient-radial" />
    <div class="overflow-hidden absolute inset-0 pointer-events-none">
      <div class="absolute -top-40 -right-40 w-80 h-80 rounded-full blur-3xl bg-brand-500/10" />
      <div class="absolute -bottom-40 -left-40 w-80 h-80 rounded-full blur-3xl bg-purple-500/10" />
    </div>

    <div class="relative z-10 w-full max-w-lg">
      <div class="mb-8 text-center">
        <div class="inline-flex justify-center items-center mb-4 w-20 h-20 bg-gradient-to-br to-purple-600 rounded-2xl shadow-lg from-brand-500 shadow-brand-500/25">
          <span class="text-4xl">🦞</span>
        </div>
        <h1 class="mb-2 text-2xl font-bold text-white">OTOClaw</h1>
        <p class="text-dark-400">环境检测与安装向导</p>
      </div>

      <div class="p-6 rounded-2xl shadow-xl glass-card">
        <Transition name="fade" mode="out-in">
          <div v-if="checking" key="checking" class="py-6 text-center">
            <Loader2 class="mx-auto mb-3 w-10 h-10 animate-spin text-brand-500" />
            <p class="text-dark-300">正在检测系统环境...</p>
          </div>

          <div v-else-if="step === 'install' && envStatus" key="install" class="space-y-4">
            <div class="flex justify-between items-center pb-4 text-sm border-b text-dark-400 border-dark-700">
              <span>操作系统</span>
              <span class="text-dark-200">{{ getOsName(envStatus.os) }}</span>
            </div>

            <div class="flex justify-between items-center">
              <div class="flex gap-3 items-center">
                <div :class="['p-2 rounded-lg', nodeJsStatusClass]">
                  <Cpu class="w-5 h-5" />
                </div>
                <div>
                  <p class="font-medium text-white">Node.js</p>
                  <p class="text-sm text-dark-400">
                    {{ envStatus.node_version
                      ? `${envStatus.node_version} ${envStatus.node_version_ok ? '✓' : '(需要 v22+)'}`
                      : '未安装' }}
                  </p>
                </div>
              </div>

              <CheckCircle2 v-if="envStatus.node_installed && envStatus.node_version_ok" class="w-6 h-6 text-green-400" />
              <button
                v-else
                @click="handleInstallNodejs"
                :disabled="installing !== null"
                class="flex gap-2 items-center px-4 py-2 text-sm btn-primary"
              >
                <Loader2 v-if="installing === 'nodejs'" class="w-4 h-4 animate-spin" />
                <Download v-else class="w-4 h-4" />
                {{ installing === 'nodejs' ? '安装中...' : '安装' }}
              </button>
            </div>

            <div class="flex justify-between items-center">
              <div class="flex gap-3 items-center">
                <div :class="['p-2 rounded-lg', openclawStatusClass]">
                  <Package class="w-5 h-5" />
                </div>
                <div>
                  <p class="font-medium text-white">OpenClaw</p>
                  <p class="text-sm text-dark-400">
                    {{ envStatus.openclaw_version || '未安装' }}
                  </p>
                </div>
              </div>

              <CheckCircle2 v-if="envStatus.openclaw_installed" class="w-6 h-6 text-green-400" />
              <button
                v-else
                @click="handleInstallOpenclaw"
                :disabled="installing !== null || !envStatus.node_version_ok"
                :class="[
                  'btn-primary text-sm px-4 py-2 flex items-center gap-2',
                  !envStatus.node_version_ok ? 'opacity-50 cursor-not-allowed' : ''
                ]"
                :title="!envStatus.node_version_ok ? '请先安装 Node.js' : ''"
              >
                <Loader2 v-if="installing === 'openclaw'" class="w-4 h-4 animate-spin" />
                <Download v-else class="w-4 h-4" />
                {{ installing === 'openclaw' ? '安装中...' : '安装' }}
              </button>
            </div>

            <Transition name="fade">
              <div v-if="error" class="p-3 rounded-lg border bg-yellow-500/10 border-yellow-500/30">
                <p class="text-sm text-yellow-400">{{ error }}</p>
              </div>
            </Transition>

            <div class="flex gap-3 pt-4 border-t border-dark-700/50">
              <button
                @click="checkEnvironment"
                :disabled="checking || installing !== null"
                class="flex flex-1 gap-2 justify-center items-center py-2.5 btn-secondary"
              >
                <RefreshCw :class="['w-4 h-4', checking ? 'animate-spin' : '']" />
                重新检查
              </button>

              <button
                v-if="envStatus.ready"
                @click="emit('complete')"
                class="flex flex-1 gap-2 justify-center items-center py-2.5 btn-primary"
              >
                开始使用
                <ArrowRight class="w-4 h-4" />
              </button>
            </div>

            <div class="pt-1 text-center">
              <a
                href="https://nodejs.org/en/download"
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex gap-1 items-center text-sm transition-colors text-dark-400 hover:text-brand-400"
              >
                手动下载 Node.js
                <ExternalLink class="w-3 h-3" />
              </a>
            </div>
          </div>

          <div v-else-if="step === 'complete'" key="complete" class="py-6 text-center">
            <CheckCircle2 class="mx-auto mb-3 w-12 h-12 text-green-400" />
            <h3 class="mb-1 text-lg font-bold text-white">环境就绪！</h3>
            <p class="text-sm text-dark-400">
              Node.js 和 OpenClaw 已正确安装
            </p>
          </div>
        </Transition>
      </div>

      <p class="mt-6 text-xs text-center text-dark-500">
        OTOClaw v1.0.0
      </p>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
