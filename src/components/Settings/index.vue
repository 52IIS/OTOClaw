<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  Save,
  Loader2,
  FolderOpen,
  FileCode,
  Trash2,
  AlertTriangle,
  X,
  RefreshCw,
  Download,
  Info,
} from 'lucide-vue-next'
import { useDialog } from '../../composables/useDialog'
import { api, type UpdateConfig, type VersionInfo, type OTOClawUpdateInfo, isTauri } from '../../lib/tauri'
import UpdateDialog from '../UpdateDialog.vue'

const { alert } = useDialog()

interface InstallResult {
  success: boolean
  message: string
  error?: string
}

const showUninstallConfirm = ref(false)
const uninstalling = ref(false)
const uninstallResult = ref<InstallResult | null>(null)

const updateConfig = ref<UpdateConfig>({
  mode: 'auto',
  check_on_startup: true,
  last_check_time: null,
  skipped_version: null,
})
const appVersion = ref<VersionInfo | null>(null)
const checkingUpdate = ref(false)
const updateInfo = ref<OTOClawUpdateInfo | null>(null)
const showUpdateDialog = ref(false)
const loadingConfig = ref(true)

onMounted(async () => {
  if (isTauri()) {
    await loadUpdateConfig()
    await loadAppVersion()
  } else {
    loadingConfig.value = false
  }
})

const loadUpdateConfig = async () => {
  try {
    const config = await api.getUpdateConfig()
    updateConfig.value = config
  } catch (e) {
    console.error('加载更新配置失败:', e)
  }
}

const loadAppVersion = async () => {
  try {
    const version = await api.getAppVersion()
    appVersion.value = version
  } catch (e) {
    console.error('加载版本信息失败:', e)
  } finally {
    loadingConfig.value = false
  }
}

const checkForUpdate = async () => {
  checkingUpdate.value = true
  updateInfo.value = null
  try {
    const info = await api.checkOTOClawUpdate()
    updateInfo.value = info
    if (info.update_available) {
      showUpdateDialog.value = true
    } else {
      await alert('当前已是最新版本', { title: '检查更新', variant: 'success' })
    }
  } catch (e) {
    console.error('检查更新失败:', e)
    await alert('检查更新失败，请稍后重试', { title: '检查更新', variant: 'error' })
  } finally {
    checkingUpdate.value = false
  }
}

const handleSkipVersion = async (version: string) => {
  try {
    await api.skipVersion(version)
  } catch (e) {
    console.error('跳过版本失败:', e)
  }
}

const handleSave = async () => {
  try {
    if (isTauri()) {
      await api.saveUpdateConfig(updateConfig.value)
    }
    await new Promise((resolve) => setTimeout(resolve, 500))
    await alert('设置已保存！', { title: '保存成功', variant: 'success' })
  } catch (e) {
    console.error('保存失败:', e)
    await alert('保存失败，请重试', { title: '保存失败', variant: 'error' })
  }
}

const openConfigDir = async () => {
  try {
    const { openPath } = await import('@tauri-apps/plugin-opener')
    const home = await api.getSystemInfo()
    await openPath(home.config_dir)
  } catch (e) {
    console.error('打开目录失败:', e)
  }
}

const handleUninstall = async () => {
  uninstalling.value = true
  uninstallResult.value = null
  try {
    const result = await invoke<InstallResult>('uninstall_openclaw')
    uninstallResult.value = result
    if (result.success) {
      setTimeout(() => {
        showUninstallConfirm.value = false
      }, 2000)
    }
  } catch (e) {
    uninstallResult.value = {
      success: false,
      message: '卸载过程中发生错误',
      error: String(e),
    }
  } finally {
    uninstalling.value = false
  }
}
</script>

<template>
  <div class="overflow-y-auto pr-2 h-full scroll-container">
    <div class="space-y-6">
      <div class="p-6 rounded-2xl border bg-dark-700 border-dark-500">
        <div class="flex gap-3 items-center mb-6">
          <div class="flex justify-center items-center w-10 h-10 rounded-xl bg-blue-500/20">
            <RefreshCw :size="20" class="text-blue-400" />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-white">自动更新</h3>
            <p class="text-xs text-gray-500">管理 OTOClaw 应用更新</p>
          </div>
        </div>

        <div class="space-y-4">
          <div>
            <label class="block mb-3 text-sm text-gray-400">更新模式</label>
            <div class="space-y-2">
              <label class="flex gap-3 items-center p-4 rounded-lg cursor-pointer transition-colors bg-dark-600 hover:bg-dark-500" :class="{ 'ring-2 ring-claw-500': updateConfig.mode === 'auto' }">
                <input
                  type="radio"
                  v-model="updateConfig.mode"
                  value="auto"
                  class="w-4 h-4 text-claw-500 bg-dark-500 border-dark-400 focus:ring-claw-500"
                />
                <div>
                  <p class="text-sm font-medium text-white">自动更新（推荐）</p>
                  <p class="text-xs text-gray-500">检测到更新时自动下载并安装</p>
                </div>
              </label>
              <label class="flex gap-3 items-center p-4 rounded-lg cursor-pointer transition-colors bg-dark-600 hover:bg-dark-500" :class="{ 'ring-2 ring-claw-500': updateConfig.mode === 'prompt' }">
                <input
                  type="radio"
                  v-model="updateConfig.mode"
                  value="prompt"
                  class="w-4 h-4 text-claw-500 bg-dark-500 border-dark-400 focus:ring-claw-500"
                />
                <div>
                  <p class="text-sm font-medium text-white">提示更新</p>
                  <p class="text-xs text-gray-500">检测到更新时弹窗提示您选择</p>
                </div>
              </label>
            </div>
          </div>

          <div class="flex justify-between items-center p-4 rounded-lg bg-dark-600">
            <div>
              <p class="text-sm text-white">启动时检查更新</p>
              <p class="text-xs text-gray-500">每次启动应用时自动检查新版本</p>
            </div>
            <label class="inline-flex relative items-center cursor-pointer">
              <input type="checkbox" v-model="updateConfig.check_on_startup" class="sr-only peer" />
              <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
            </label>
          </div>

          <div class="p-4 rounded-lg bg-dark-600">
            <div class="flex justify-between items-center mb-3">
              <div class="flex gap-2 items-center">
                <Info :size="16" class="text-gray-400" />
                <span class="text-sm text-gray-400">当前版本</span>
              </div>
              <span class="text-sm font-medium text-white">{{ appVersion?.version || '-' }}</span>
            </div>
            <button
              @click="checkForUpdate"
              :disabled="checkingUpdate"
              class="flex gap-2 justify-center items-center px-4 py-2 w-full text-sm text-white rounded-lg transition-colors bg-claw-600 hover:bg-claw-500 disabled:opacity-50"
            >
              <Loader2 v-if="checkingUpdate" :size="16" class="animate-spin" />
              <RefreshCw v-else :size="16" />
              {{ checkingUpdate ? '检查中...' : '检查更新' }}
            </button>
          </div>

          <div v-if="updateInfo?.update_available" class="p-4 rounded-lg bg-green-900/30 border border-green-800">
            <div class="flex gap-2 items-center">
              <Download :size="16" class="text-green-400" />
              <span class="text-sm font-medium text-green-300">发现新版本 {{ updateInfo.latest_version }}，正在打开更新窗口...</span>
            </div>
          </div>
        </div>
      </div>

      <div class="p-6 rounded-2xl border bg-dark-700 border-dark-500">
        <div class="flex gap-3 items-center mb-6">
          <div class="flex justify-center items-center w-10 h-10 rounded-xl bg-purple-500/20">
            <FileCode :size="20" class="text-purple-400" />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-white">高级设置</h3>
            <p class="text-xs text-gray-500">配置文件和目录</p>
          </div>
        </div>

        <div class="space-y-3">
          <button
            @click="openConfigDir"
            class="flex gap-3 items-center p-4 w-full text-left rounded-lg transition-colors bg-dark-600 hover:bg-dark-500"
          >
            <FolderOpen :size="18" class="text-gray-400" />
            <div class="flex-1">
              <p class="text-sm text-white">打开配置目录</p>
              <p class="text-xs text-gray-500">~/.openclaw</p>
            </div>
          </button>
        </div>
      </div>

      <div class="p-6 rounded-2xl border bg-dark-700 border-red-900/30">
        <div class="flex gap-3 items-center mb-6">
          <div class="flex justify-center items-center w-10 h-10 rounded-xl bg-red-500/20">
            <AlertTriangle :size="20" class="text-red-400" />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-white">危险操作</h3>
            <p class="text-xs text-gray-500">以下操作不可撤销，请谨慎操作</p>
          </div>
        </div>

        <div class="space-y-3">
          <button
            @click="showUninstallConfirm = true"
            class="flex gap-3 items-center p-4 w-full text-left rounded-lg border transition-colors bg-red-950/30 hover:bg-red-900/40 border-red-900/30"
          >
            <Trash2 :size="18" class="text-red-400" />
            <div class="flex-1">
              <p class="text-sm text-red-300">卸载 OpenClaw</p>
              <p class="text-xs text-red-400/70">从系统中移除 OpenClaw CLI 工具</p>
            </div>
          </button>
        </div>
      </div>

      <Transition name="fade">
        <div v-if="showUninstallConfirm" class="flex fixed inset-0 z-50 justify-center items-center backdrop-blur-sm bg-black/60">
          <div class="p-6 mx-4 w-full max-w-md rounded-2xl border shadow-2xl bg-dark-700 border-dark-500">
            <div class="flex justify-between items-center mb-4">
              <div class="flex gap-3 items-center">
                <div class="flex justify-center items-center w-10 h-10 rounded-xl bg-red-500/20">
                  <AlertTriangle :size="20" class="text-red-400" />
                </div>
                <h3 class="text-lg font-semibold text-white">确认卸载</h3>
              </div>
              <button
                @click="showUninstallConfirm = false; uninstallResult = null"
                class="text-gray-400 transition-colors hover:text-white"
              >
                <X :size="20" />
              </button>
            </div>

            <div v-if="!uninstallResult">
              <p class="mb-4 text-gray-300">确定要卸载 OpenClaw 吗？此操作将：</p>
              <ul class="mb-6 space-y-2 text-sm text-gray-400">
                <li class="flex gap-2 items-center">
                  <span class="w-1.5 h-1.5 bg-red-400 rounded-full"></span>
                  停止正在运行的服务
                </li>
                <li class="flex gap-2 items-center">
                  <span class="w-1.5 h-1.5 bg-red-400 rounded-full"></span>
                  移除 OpenClaw CLI 工具
                </li>
                <li class="flex gap-2 items-center">
                  <span class="w-1.5 h-1.5 bg-yellow-400 rounded-full"></span>
                  配置文件将被保留在 ~/.openclaw
                </li>
              </ul>

              <div class="flex gap-3">
                <button
                  @click="showUninstallConfirm = false"
                  class="flex-1 px-4 py-2.5 text-white rounded-lg transition-colors bg-dark-600 hover:bg-dark-500"
                >
                  取消
                </button>
                <button
                  @click="handleUninstall"
                  :disabled="uninstalling"
                  class="flex flex-1 gap-2 justify-center items-center px-4 py-2.5 text-white bg-red-600 rounded-lg transition-colors hover:bg-red-500 disabled:opacity-50"
                >
                  <Loader2 v-if="uninstalling" :size="16" class="animate-spin" />
                  <Trash2 v-else :size="16" />
                  {{ uninstalling ? '卸载中...' : '确认卸载' }}
                </button>
              </div>
            </div>

            <div v-else :class="['p-4 rounded-lg', uninstallResult.success ? 'bg-green-900/30 border border-green-800' : 'bg-red-900/30 border border-red-800']">
              <p :class="['text-sm', uninstallResult.success ? 'text-green-300' : 'text-red-300']">
                {{ uninstallResult.message }}
              </p>
              <p v-if="uninstallResult.error" class="mt-2 font-mono text-xs text-red-400">
                {{ uninstallResult.error }}
              </p>
              <p v-if="uninstallResult.success" class="mt-3 text-xs text-gray-400">
                对话框将自动关闭...
              </p>
            </div>
          </div>
        </div>
      </Transition>

      <div class="flex justify-end">
        <button
          @click="handleSave"
          class="flex gap-2 items-center btn-primary"
        >
          <Save :size="16" />
          保存设置
        </button>
      </div>
    </div>

    <UpdateDialog
      v-if="showUpdateDialog"
      :update-info="updateInfo"
      @close="showUpdateDialog = false"
      @skip="handleSkipVersion"
    />
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
</style>
