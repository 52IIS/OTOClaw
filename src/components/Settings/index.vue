<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  User,
  Shield,
  Save,
  Loader2,
  FolderOpen,
  FileCode,
  Trash2,
  AlertTriangle,
  X,
} from 'lucide-vue-next'
import { useDialog } from '../../composables/useDialog'

const { alert } = useDialog()

interface InstallResult {
  success: boolean
  message: string
  error?: string
}

const identity = ref({
  botName: 'Clawd',
  userName: '主人',
  timezone: 'Asia/Shanghai',
})
const saving = ref(false)
const showUninstallConfirm = ref(false)
const uninstalling = ref(false)
const uninstallResult = ref<InstallResult | null>(null)

const handleSave = async () => {
  saving.value = true
  try {
    await new Promise((resolve) => setTimeout(resolve, 500))
    await alert('设置已保存！', { title: '保存成功', variant: 'success' })
  } catch (e) {
    console.error('保存失败:', e)
  } finally {
    saving.value = false
  }
}

const openConfigDir = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-shell')
    const home = await invoke<{ config_dir: string }>('get_system_info')
    await open(home.config_dir)
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
    <div class="space-y-6 max-w-2xl">
      <div class="p-6 rounded-2xl border bg-dark-700 border-dark-500">
        <div class="flex gap-3 items-center mb-6">
          <div class="flex justify-center items-center w-10 h-10 rounded-xl bg-claw-500/20">
            <User :size="20" class="text-claw-400" />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-white">身份配置</h3>
            <p class="text-xs text-gray-500">设置 AI 助手的名称和称呼</p>
          </div>
        </div>

        <div class="space-y-4">
          <div>
            <label class="block mb-2 text-sm text-gray-400">AI 助手名称</label>
            <input
              type="text"
              v-model="identity.botName"
              placeholder="Clawd"
              class="input-base"
            />
          </div>

          <div>
            <label class="block mb-2 text-sm text-gray-400">你的称呼</label>
            <input
              type="text"
              v-model="identity.userName"
              placeholder="主人"
              class="input-base"
            />
          </div>

          <div>
            <label class="block mb-2 text-sm text-gray-400">时区</label>
            <select v-model="identity.timezone" class="input-base">
              <option value="Asia/Shanghai">Asia/Shanghai (北京时间)</option>
              <option value="Asia/Hong_Kong">Asia/Hong_Kong (香港时间)</option>
              <option value="Asia/Tokyo">Asia/Tokyo (东京时间)</option>
              <option value="America/New_York">America/New_York (纽约时间)</option>
              <option value="America/Los_Angeles">America/Los_Angeles (洛杉矶时间)</option>
              <option value="Europe/London">Europe/London (伦敦时间)</option>
              <option value="UTC">UTC</option>
            </select>
          </div>
        </div>
      </div>

      <div class="p-6 rounded-2xl border bg-dark-700 border-dark-500">
        <div class="flex gap-3 items-center mb-6">
          <div class="flex justify-center items-center w-10 h-10 rounded-xl bg-amber-500/20">
            <Shield :size="20" class="text-amber-400" />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-white">安全设置</h3>
            <p class="text-xs text-gray-500">权限和访问控制</p>
          </div>
        </div>

        <div class="space-y-4">
          <div class="flex justify-between items-center p-4 rounded-lg bg-dark-600">
            <div>
              <p class="text-sm text-white">启用白名单</p>
              <p class="text-xs text-gray-500">只允许白名单用户访问</p>
            </div>
            <label class="inline-flex relative items-center cursor-pointer">
              <input type="checkbox" class="sr-only peer" />
              <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
            </label>
          </div>

          <div class="flex justify-between items-center p-4 rounded-lg bg-dark-600">
            <div>
              <p class="text-sm text-white">文件访问权限</p>
              <p class="text-xs text-gray-500">允许 AI 读写本地文件</p>
            </div>
            <label class="inline-flex relative items-center cursor-pointer">
              <input type="checkbox" class="sr-only peer" />
              <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
            </label>
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
          :disabled="saving"
          class="flex gap-2 items-center btn-primary"
        >
          <Loader2 v-if="saving" :size="16" class="animate-spin" />
          <Save v-else :size="16" />
          保存设置
        </button>
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
</style>
