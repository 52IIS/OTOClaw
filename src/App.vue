<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Sidebar from './components/Layout/Sidebar.vue'
import Header from './components/Layout/Header.vue'
import Dashboard from './components/Dashboard/index.vue'
import AIConfig from './components/AIConfig/index.vue'
import Channels from './components/Channels/index.vue'
import Settings from './components/Settings/index.vue'
import Testing from './components/Testing/index.vue'
import Logs from './components/Logs/index.vue'
import { appLogger } from './lib/logger'
import { isTauri } from './lib/tauri'
import { Download, X, Loader2, CheckCircle, AlertCircle } from 'lucide-vue-next'
import type { PageType, EnvironmentStatus, ServiceStatus, UpdateInfo, UpdateResult } from './vite-env.d'

const currentPage = ref<PageType>('dashboard')
const isReady = ref<boolean | null>(null)
const envStatus = ref<EnvironmentStatus | null>(null)
const serviceStatus = ref<ServiceStatus | null>(null)

const updateInfo = ref<UpdateInfo | null>(null)
const showUpdateBanner = ref(false)
const updating = ref(false)
const updateResult = ref<UpdateResult | null>(null)

const checkEnvironment = async () => {
  if (!isTauri()) {
    appLogger.warn('不在 Tauri 环境中，跳过环境检查')
    isReady.value = true
    return
  }
  
  appLogger.info('开始检查系统环境...')
  try {
    const status = await invoke<EnvironmentStatus>('check_environment')
    appLogger.info('环境检查完成', status)
    envStatus.value = status
    isReady.value = true
  } catch (e) {
    appLogger.error('环境检查失败', e)
    isReady.value = true
  }
}

const checkUpdate = async () => {
  if (!isTauri()) return
  
  appLogger.info('检查 OpenClaw 更新...')
  try {
    const info = await invoke<UpdateInfo>('check_openclaw_update')
    appLogger.info('更新检查结果', info)
    updateInfo.value = info
    if (info.update_available) {
      showUpdateBanner.value = true
    }
  } catch (e) {
    appLogger.error('检查更新失败', e)
  }
}

const handleUpdate = async () => {
  updating.value = true
  updateResult.value = null
  try {
    const result = await invoke<UpdateResult>('update_openclaw')
    updateResult.value = result
    if (result.success) {
      await checkEnvironment()
      setTimeout(() => {
        showUpdateBanner.value = false
        updateResult.value = null
      }, 3000)
    }
  } catch (e) {
    updateResult.value = {
      success: false,
      message: '更新过程中发生错误',
      error: String(e),
    }
  } finally {
    updating.value = false
  }
}

const handleSetupComplete = () => {
  appLogger.info('安装向导完成')
  checkEnvironment()
}

const handleNavigate = (page: PageType) => {
  appLogger.action('页面切换', { from: currentPage.value, to: page })
  currentPage.value = page
}

let statusInterval: ReturnType<typeof setInterval> | null = null
let updateTimer: ReturnType<typeof setTimeout> | null = null

onMounted(() => {
  appLogger.info('🦞 App 组件已挂载')
  checkEnvironment()
  
  if (isTauri()) {
    updateTimer = setTimeout(() => {
      checkUpdate()
    }, 2000)
    
    const fetchServiceStatus = async () => {
      try {
        const status = await invoke<ServiceStatus>('get_service_status')
        serviceStatus.value = status
      } catch {
      }
    }
    fetchServiceStatus()
    statusInterval = setInterval(fetchServiceStatus, 3000)
  }
})

onUnmounted(() => {
  if (statusInterval) clearInterval(statusInterval)
  if (updateTimer) clearTimeout(updateTimer)
})

const currentComponent = computed(() => {
  const components: Record<PageType, any> = {
    dashboard: Dashboard,
    ai: AIConfig,
    channels: Channels,
    testing: Testing,
    logs: Logs,
    settings: Settings,
  }
  return components[currentPage.value]
})
</script>

<template>
  <div v-if="isReady === null" class="flex h-screen bg-dark-900 items-center justify-center">
    <div class="fixed inset-0 bg-gradient-radial pointer-events-none" />
    <div class="relative z-10 text-center">
      <div class="inline-flex items-center justify-center w-16 h-16 rounded-xl bg-gradient-to-br from-brand-500 to-purple-600 mb-4 animate-pulse">
        <span class="text-3xl">🦞</span>
      </div>
      <p class="text-dark-400">正在启动...</p>
    </div>
  </div>

  <div v-else class="flex h-screen bg-dark-900 overflow-hidden">
    <div class="fixed inset-0 bg-gradient-radial pointer-events-none" />
    
    <Transition name="slide-down">
      <div
        v-if="showUpdateBanner && updateInfo?.update_available"
        class="fixed top-0 left-0 right-0 z-50 bg-gradient-to-r from-claw-600 to-purple-600 shadow-lg"
      >
        <div class="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <CheckCircle v-if="updateResult?.success" :size="20" class="text-green-300" />
            <AlertCircle v-else-if="updateResult && !updateResult.success" :size="20" class="text-red-300" />
            <Download v-else :size="20" class="text-white" />
            <div>
              <template v-if="updateResult">
                <p :class="['text-sm font-medium', updateResult.success ? 'text-green-100' : 'text-red-100']">
                  {{ updateResult.message }}
                </p>
              </template>
              <template v-else>
                <p class="text-sm font-medium text-white">
                  发现新版本 OpenClaw {{ updateInfo.latest_version }}
                </p>
                <p class="text-xs text-white/70">
                  当前版本: {{ updateInfo.current_version }}
                </p>
              </template>
            </div>
          </div>
          
          <div class="flex items-center gap-2">
            <button
              v-if="!updateResult"
              @click="handleUpdate"
              :disabled="updating"
              class="px-4 py-1.5 bg-white/20 hover:bg-white/30 text-white text-sm font-medium rounded-lg transition-colors flex items-center gap-2 disabled:opacity-50"
            >
              <Loader2 v-if="updating" :size="14" class="animate-spin" />
              <Download v-else :size="14" />
              {{ updating ? '更新中...' : '立即更新' }}
            </button>
            <button
              @click="showUpdateBanner = false; updateResult = null"
              class="p-1.5 hover:bg-white/20 rounded-lg transition-colors text-white/70 hover:text-white"
            >
              <X :size="16" />
            </button>
          </div>
        </div>
      </div>
    </Transition>
    
    <Sidebar
      :current-page="currentPage"
      :service-status="serviceStatus"
      @navigate="handleNavigate"
    />
    
    <div class="flex-1 flex flex-col overflow-hidden">
      <Header :current-page="currentPage" />
      
      <main class="flex-1 overflow-hidden p-6">
        <Transition name="fade-slide" mode="out-in">
          <component
            :is="currentComponent"
            :key="currentPage"
            :env-status="envStatus"
            @setup-complete="handleSetupComplete"
            @environment-change="checkEnvironment"
          />
        </Transition>
      </main>
    </div>
  </div>
</template>

<style scoped>
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.3s ease;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-50px);
}

.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}
</style>
