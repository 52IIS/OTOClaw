<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  Settings,
  X,
  Link,
  Key,
  Lock,
  CheckCircle,
  XCircle,
  Loader2,
  Copy,
  Terminal,
  Eye,
  EyeOff,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { useChatStore } from '../../stores/chatStore'

const store = useChatStore()

const localConfig = ref({
  url: '',
  token: '',
  password: '',
})

const connecting = ref(false)
const copied = ref(false)
const showFullToken = ref(false)

const connectionStatus = computed(() => {
  if (store.gatewayStatus.connected) {
    return { type: 'connected', label: '已连接', color: 'text-green-400' }
  }
  if (store.gatewayStatus.error) {
    return { type: 'error', label: '连接失败', color: 'text-red-400' }
  }
  return { type: 'disconnected', label: '未连接', color: 'text-gray-500' }
})

const viewTokenCommand = computed(() => {
  return `openclaw gateway run`
})

const runGatewayCommand = computed(() => {
  return `openclaw dashboard --no-open`
})

const hasExistingToken = computed(() => {
  return !!store.gatewayConfig.token
})

const handleConnect = async () => {
  connecting.value = true
  await store.saveGatewayConfig({
    url: localConfig.value.url,
    token: localConfig.value.token,
    password: localConfig.value.password,
  })
  const success = await store.connectGateway()
  connecting.value = false
  
  if (success && store.gatewayStatus.connected) {
    await store.loadSessions()
  }
}

const handleDisconnect = async () => {
  await store.disconnectGateway()
}

const handleCopyViewTokenCommand = async () => {
  try {
    await navigator.clipboard.writeText(viewTokenCommand.value)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (e) {
    console.error('复制失败', e)
  }
}
const handleCopyRunGatewayCommand = async () => {
  try {
    await navigator.clipboard.writeText(runGatewayCommand.value)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (e) {
    console.error('复制失败', e)
  }
}


const toggleTokenVisibility = () => {
  showFullToken.value = !showFullToken.value
}

onMounted(async () => {
  await store.loadGatewayConfig()
  localConfig.value = {
    url: store.gatewayConfig.url,
    token: store.gatewayConfig.token || '',
    password: store.gatewayConfig.password || '',
  }
})
</script>

<template>
  <div class="relative">
    <button
      @click="store.toggleGatewayConfig"
      :class="clsx(
        'flex gap-2 items-center px-3 py-2 rounded-lg border transition-colors',
        store.gatewayConfigVisible
          ? 'bg-claw-500/20 border-claw-500/50 text-claw-400'
          : 'bg-dark-700 border-dark-500 text-gray-400 hover:border-dark-400'
      )"
    >
      <Settings :size="16" />
      <span class="text-sm">Gateway配置</span>
      <div
        :class="clsx(
          'w-2 h-2 rounded-full',
          store.gatewayStatus.connected ? 'bg-green-400' : 'bg-gray-500'
        )"
      />
    </button>

    <Transition name="slide-down">
      <div
        v-if="store.gatewayConfigVisible"
        class="absolute right-0 top-full z-30 mt-2 w-80 rounded-xl border shadow-xl bg-dark-700 border-dark-500"
      >
        <div class="flex justify-between items-center p-4 border-b border-dark-600">
          <h3 class="font-medium text-white">Gateway 连接配置</h3>
          <button
            @click="store.toggleGatewayConfig"
            class="p-1 rounded transition-colors hover:bg-dark-600"
          >
            <X :size="16" class="text-gray-500" />
          </button>
        </div>

        <div class="p-4 space-y-4">
          <div class="flex gap-2 items-center">
            <div
              :class="clsx(
                'flex gap-2 items-center px-2 py-1 rounded-full text-xs',
                store.gatewayStatus.connected
                  ? 'bg-green-500/20 text-green-400'
                  : 'bg-dark-600 text-gray-400'
              )"
            >
              <CheckCircle v-if="store.gatewayStatus.connected" :size="12" />
              <XCircle v-else :size="12" />
              {{ connectionStatus.label }}
            </div>
          </div>

          <div class="space-y-3">
            <div>
              <label class="block mb-1 text-xs text-gray-500">WebSocket 地址</label>
              <div class="relative">
                <Link :size="14" class="absolute left-3 top-1/2 text-gray-500 -translate-y-1/2" />
                <input
                  v-model="localConfig.url"
                  type="text"
                  placeholder="ws://localhost:18789"
                  class="py-2 pr-3 pl-9 w-full text-sm text-white rounded-lg border bg-dark-600 border-dark-500 focus:border-claw-500 focus:outline-none"
                />
              </div>
            </div>

            <div>
              <div class="flex justify-between items-center mb-1">
                <label class="text-xs text-gray-500">Token</label>
                <span v-if="hasExistingToken" class="text-xs text-claw-400">已配置</span>
              </div>
              <div class="relative">
                <Key :size="14" class="absolute left-3 top-1/2 text-gray-500 -translate-y-1/2" />
                <input
                  v-model="localConfig.token"
                  :type="showFullToken ? 'text' : 'password'"
                  :placeholder="hasExistingToken ? '••••••••' : '输入 Token'"
                  class="py-2 pr-16 pl-9 w-full text-sm text-white rounded-lg border bg-dark-600 border-dark-500 focus:border-claw-500 focus:outline-none"
                />
                <button
                  @click="toggleTokenVisibility"
                  class="absolute right-2 top-1/2 p-1 rounded transition-colors -translate-y-1/2 hover:bg-dark-500"
                  :title="showFullToken ? '隐藏 Token' : '显示 Token'"
                >
                  <EyeOff v-if="showFullToken" :size="14" class="text-gray-500" />
                  <Eye v-else :size="14" class="text-gray-500" />
                </button>
              </div>
              <p v-if="hasExistingToken && !localConfig.token" class="mt-1 text-xs text-gray-600">
                留空保持现有 Token 不变
              </p>
            </div>

            <div>
              <label class="block mb-1 text-xs text-gray-500">密码</label>
              <div class="relative">
                <Lock :size="14" class="absolute left-3 top-1/2 text-gray-500 -translate-y-1/2" />
                <input
                  v-model="localConfig.password"
                  type="password"
                  placeholder="输入密码（可选）"
                  class="py-2 pr-3 pl-9 w-full text-sm text-white rounded-lg border bg-dark-600 border-dark-500 focus:border-claw-500 focus:outline-none"
                />
              </div>
            </div>
          </div>

          <div v-if="!store.gatewayStatus.connected && store.gatewayStatus.error" class="p-3 rounded-lg border bg-red-500/10 border-red-500/30">
            <p class="text-xs text-red-400">{{ store.gatewayStatus.error }}</p>
          </div>

          <div v-if="!store.gatewayStatus.connected" class="p-3 rounded-lg bg-dark-600">
            <div class="flex gap-2 items-center mb-2">
              <Terminal :size="14" class="text-gray-500" />
              <span class="text-xs text-gray-400">1、在主机上启动网关：</span>
            </div>
            <div class="flex gap-2 items-center">
              <code class="flex-1 p-2 font-mono text-xs text-gray-400 rounded bg-dark-700">
                {{ viewTokenCommand }}
              </code>
              <button
                @click="handleCopyViewTokenCommand"
                class="p-2 rounded transition-colors hover:bg-dark-500"
              >
                <Copy v-if="!copied" :size="14" class="text-gray-500" />
                <CheckCircle v-else :size="14" class="text-green-400" />
              </button>
            </div>
          </div>
          <div v-if="!store.gatewayStatus.connected" class="p-3 rounded-lg bg-dark-600">
            <div class="flex gap-2 items-center mb-2">
              <Terminal :size="14" class="text-gray-500" />
              <span class="text-xs text-gray-400">2、获取带令牌的仪表盘 URL：</span>
            </div>
            <div class="flex gap-2 items-center">
              <code class="flex-1 p-2 font-mono text-xs text-gray-400 rounded bg-dark-700">
                {{ runGatewayCommand }}
              </code>
              <button
                @click="handleCopyRunGatewayCommand"
                class="p-2 rounded transition-colors hover:bg-dark-500"
              >
                <Copy v-if="!copied" :size="14" class="text-gray-500" />
                <CheckCircle v-else :size="14" class="text-green-400" />
              </button>
            </div>
          </div>

          <div class="flex gap-2">
            <button
              v-if="!store.gatewayStatus.connected"
              @click="handleConnect"
              :disabled="connecting || !localConfig.url"
              class="flex flex-1 gap-2 justify-center items-center py-2 text-sm text-white rounded-lg transition-colors bg-claw-500 hover:bg-claw-600 disabled:opacity-50"
            >
              <Loader2 v-if="connecting" :size="14" class="animate-spin" />
              <Link v-else :size="14" />
              {{ connecting ? '连接中...' : '连接' }}
            </button>
            <button
              v-else
              @click="handleDisconnect"
              class="flex flex-1 gap-2 justify-center items-center py-2 text-sm text-white rounded-lg transition-colors bg-dark-500 hover:bg-dark-400"
            >
              <X :size="14" />
              断开连接
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.2s ease;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
