<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Sidebar from './components/Layout/Sidebar.vue'
import Header from './components/Layout/Header.vue'
import Dashboard from './components/Dashboard/index.vue'
import Chat from './components/Chat/index.vue'
import Agents from './components/Agents/index.vue'
import Skills from './components/Skills/index.vue'
import AIConfig from './components/AIConfig/index.vue'
import Channels from './components/Channels/index.vue'
import Sandbox from './components/Sandbox/index.vue'
import Settings from './components/Settings/index.vue'
import Testing from './components/Testing/index.vue'
import Logs from './components/Logs/index.vue'
import Dialog from './components/Dialog.vue'
import UpdateDialog from './components/UpdateDialog.vue'
import { useDialog } from './composables/useDialog'
import { appLogger } from './lib/logger'
import { isTauri, api, type OTOClawUpdateInfo, type UpdateConfig } from './lib/tauri'
import { Download, X, Loader2, CheckCircle, AlertCircle } from 'lucide-vue-next'
import type { PageType, EnvironmentStatus, ServiceStatus, UpdateInfo, UpdateResult } from './vite-env.d'

const { state: dialogState, handleConfirm, handleCancel } = useDialog()

const currentPage = ref<PageType>('dashboard')
const isReady = ref<boolean | null>(null)
const envStatus = ref<EnvironmentStatus | null>(null)
const serviceStatus = ref<ServiceStatus | null>(null)

const updateInfo = ref<UpdateInfo | null>(null)
const showUpdateBanner = ref(false)
const updating = ref(false)
const updateResult = ref<UpdateResult | null>(null)

const otoclawUpdateInfo = ref<OTOClawUpdateInfo | null>(null)
const showOtoClawUpdateDialog = ref(false)
const updateConfig = ref<UpdateConfig | null>(null)

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

const checkOTOClawUpdate = async () => {
  if (!isTauri()) return
  
  try {
    const config = await api.getUpdateConfig()
    updateConfig.value = config
    
    if (!config.check_on_startup) {
      appLogger.info('OTOClaw 启动时检查更新已禁用')
      return
    }
    
    if (config.skipped_version) {
      appLogger.info('已跳过版本', config.skipped_version)
    }
    
    appLogger.info('检查 OTOClaw 更新...')
    const info = await api.checkOTOClawUpdate()
    appLogger.info('OTOClaw 更新检查结果', info)
    
    if (info.update_available) {
      if (config.skipped_version === info.latest_version) {
        appLogger.info('用户已跳过此版本')
        return
      }
      
      otoclawUpdateInfo.value = info
      
      if (config.mode === 'auto') {
        appLogger.info('自动更新模式，开始下载更新')
        showOtoClawUpdateDialog.value = true
      } else {
        appLogger.info('提示更新模式，显示更新弹窗')
        showOtoClawUpdateDialog.value = true
      }
    }
  } catch (e) {
    appLogger.error('检查 OTOClaw 更新失败', e)
  }
}

const handleSkipVersion = async (version: string) => {
  try {
    await api.skipVersion(version)
    appLogger.info('已跳过版本', version)
  } catch (e) {
    appLogger.error('跳过版本失败', e)
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
let otoclawUpdateTimer: ReturnType<typeof setTimeout> | null = null

onMounted(() => {
  appLogger.info('🦞 App 组件已挂载')
  checkEnvironment()
  
  if (isTauri()) {
    updateTimer = setTimeout(() => {
      checkUpdate()
    }, 2000)
    
    otoclawUpdateTimer = setTimeout(() => {
      checkOTOClawUpdate()
    }, 5000)
    
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
  if (otoclawUpdateTimer) clearTimeout(otoclawUpdateTimer)
})

const currentComponent = computed(() => {
  const components: Record<PageType, any> = {
    dashboard: Dashboard,
    chat: Chat,
    agents: Agents,
    skills: Skills,
    ai: AIConfig,
    channels: Channels,
    sandbox: Sandbox,
    testing: Testing,
    logs: Logs,
    settings: Settings,
  }
  return components[currentPage.value]
})
</script>

<template>
  <div v-if="isReady === null" class="flex justify-center items-center h-screen bg-dark-900">
    <div class="fixed inset-0 pointer-events-none bg-gradient-radial" />
    <div class="relative z-10 text-center">
      <div class="inline-flex justify-center items-center mb-4 w-16 h-16 bg-gradient-to-br to-purple-600 rounded-xl animate-pulse from-brand-500">
        <span class="text-3xl">🦞</span>
      </div>
      <p class="text-dark-400">正在启动...</p>
    </div>
  </div>

  <div v-else class="flex overflow-hidden h-screen bg-dark-900">
    <div class="fixed inset-0 pointer-events-none bg-gradient-radial" />
    
    <Transition name="slide-down">
      <div
        v-if="showUpdateBanner && updateInfo?.update_available"
        class="fixed top-0 right-0 left-0 z-50 bg-gradient-to-r to-purple-600 shadow-lg from-claw-600"
      >
        <div class="flex justify-between items-center px-4 py-3 mx-auto max-w-4xl">
          <div class="flex gap-3 items-center">
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
          
          <div class="flex gap-2 items-center">
            <button
              v-if="!updateResult"
              @click="handleUpdate"
              :disabled="updating"
              class="flex gap-2 items-center px-4 py-1.5 text-sm font-medium text-white rounded-lg transition-colors bg-white/20 hover:bg-white/30 disabled:opacity-50"
            >
              <Loader2 v-if="updating" :size="14" class="animate-spin" />
              <Download v-else :size="14" />
              {{ updating ? '更新中...' : '立即更新' }}
            </button>
            <button
              @click="showUpdateBanner = false; updateResult = null"
              class="p-1.5 rounded-lg transition-colors hover:bg-white/20 text-white/70 hover:text-white"
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
    
    <div class="flex overflow-hidden flex-col flex-1">
      <Header :current-page="currentPage" />
      
      <main class="overflow-hidden flex-1 p-6">
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
    
    <Dialog
      :visible="dialogState.visible"
      :options="dialogState.options"
      @confirm="handleConfirm"
      @cancel="handleCancel"
    />
    
    <UpdateDialog
      v-if="showOtoClawUpdateDialog"
      :update-info="otoclawUpdateInfo"
      @close="showOtoClawUpdateDialog = false; otoclawUpdateInfo = null"
      @skip="handleSkipVersion"
    />
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
