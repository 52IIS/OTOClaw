<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Monitor, Package, Folder, CheckCircle, XCircle } from 'lucide-vue-next'
import { api, SystemInfo as SystemInfoType, isTauri } from '../../lib/tauri'

const info = ref<SystemInfoType | null>(null)
const loading = ref(true)

const getOSLabel = (os: string) => {
  switch (os) {
    case 'macos':
      return 'macOS'
    case 'windows':
      return 'Windows'
    case 'linux':
      return 'Linux'
    default:
      return os
  }
}

onMounted(async () => {
  if (!isTauri()) {
    loading.value = false
    return
  }
  try {
    const result = await api.getSystemInfo()
    info.value = result
  } catch {
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="bg-dark-700 rounded-2xl p-6 border border-dark-500">
    <h3 class="text-lg font-semibold text-white mb-4">系统信息</h3>

    <div v-if="loading" class="animate-pulse space-y-3">
      <div class="h-4 bg-dark-500 rounded w-1/2"></div>
      <div class="h-4 bg-dark-500 rounded w-2/3"></div>
      <div class="h-4 bg-dark-500 rounded w-1/3"></div>
    </div>

    <div v-else class="space-y-4">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-dark-500 flex items-center justify-center">
          <Monitor :size="16" class="text-gray-400" />
        </div>
        <div class="flex-1">
          <p class="text-xs text-gray-500">操作系统</p>
          <p class="text-sm text-white">
            {{ info ? `${getOSLabel(info.os)} ${info.os_version}` : '--' }}
            <span class="text-gray-500">({{ info?.arch }})</span>
          </p>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-dark-500 flex items-center justify-center">
          <CheckCircle v-if="info?.openclaw_installed" :size="16" class="text-green-400" />
          <XCircle v-else :size="16" class="text-red-400" />
        </div>
        <div class="flex-1">
          <p class="text-xs text-gray-500">OpenClaw</p>
          <p class="text-sm text-white">
            {{ info?.openclaw_installed ? (info.openclaw_version || '已安装') : '未安装' }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-dark-500 flex items-center justify-center">
          <Package :size="16" class="text-green-500" />
        </div>
        <div class="flex-1">
          <p class="text-xs text-gray-500">Node.js</p>
          <p class="text-sm text-white">{{ info?.node_version || '--' }}</p>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-dark-500 flex items-center justify-center">
          <Folder :size="16" class="text-amber-400" />
        </div>
        <div class="flex-1">
          <p class="text-xs text-gray-500">配置目录</p>
          <p class="text-sm text-white font-mono text-xs truncate">
            {{ info?.config_dir || '--' }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
