<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  ChevronLeft,
  ChevronRight,
  MessageSquare,
  Plus,
  Trash2,
  Clock,
  RefreshCw,
  Loader2,
  WifiOff,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { useChatStore } from '../../stores/chatStore'
import type { ChatSession } from './types'

const store = useChatStore()

const isRefreshing = ref(false)
const refreshError = ref<string | null>(null)

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (days === 0) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  } else if (days === 1) {
    return '昨天'
  } else if (days < 7) {
    return `${days}天前`
  } else {
    return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
  }
}

const handleSelectSession = (session: ChatSession) => {
  store.loadMessages(session.key)
}

const handleNewSession = () => {
  store.createSession()
}

const handleDeleteSession = (e: Event, sessionKey: string) => {
  e.stopPropagation()
  store.deleteSession(sessionKey)
}

const handleRefresh = async () => {
  if (isRefreshing.value) return
  
  isRefreshing.value = true
  refreshError.value = null
  
  try {
    if (!store.gatewayStatus.connected) {
      const connected = await store.connectGateway()
      if (!connected) {
        refreshError.value = '连接失败，请检查Gateway配置'
        return
      }
    }
    
    await store.loadSessions()
  } catch (e) {
    refreshError.value = `刷新失败: ${e}`
  } finally {
    isRefreshing.value = false
    setTimeout(() => {
      refreshError.value = null
    }, 3000)
  }
}

const groupedSessions = computed(() => {
  const today: ChatSession[] = []
  const yesterday: ChatSession[] = []
  const thisWeek: ChatSession[] = []
  const older: ChatSession[] = []

  const now = Date.now()
  const oneDayMs = 24 * 60 * 60 * 1000

  store.sessions.forEach(session => {
    const diff = now - session.updatedAt
    if (diff < oneDayMs) {
      today.push(session)
    } else if (diff < 2 * oneDayMs) {
      yesterday.push(session)
    } else if (diff < 7 * oneDayMs) {
      thisWeek.push(session)
    } else {
      older.push(session)
    }
  })

  return [
    { label: '今天', sessions: today },
    { label: '昨天', sessions: yesterday },
    { label: '本周', sessions: thisWeek },
    { label: '更早', sessions: older },
  ].filter(g => g.sessions.length > 0)
})
</script>

<template>
  <div
    :class="clsx(
      'flex flex-col h-full bg-dark-800 border-r border-dark-600 transition-all duration-300',
      store.historyPanelCollapsed ? 'w-0 overflow-hidden border-r-0' : 'w-72'
    )"
  >
    <div v-if="!store.historyPanelCollapsed" class="flex flex-col h-full">
      <div class="flex justify-between items-center p-4 border-b border-dark-600">
        <h2 class="text-sm font-medium text-white">历史会话</h2>
        <div class="flex gap-1 items-center">
          <button
            @click="handleRefresh"
            :disabled="isRefreshing"
            :title="store.gatewayStatus.connected ? '刷新会话列表' : '连接Gateway并刷新'"
            :class="clsx(
              'flex justify-center items-center w-7 h-7 rounded-lg transition-all duration-200',
              isRefreshing
                ? 'bg-dark-600 cursor-not-allowed'
                : 'hover:bg-dark-600 active:scale-95',
              !store.gatewayStatus.connected && 'text-yellow-500'
            )"
          >
            <Loader2
              v-if="isRefreshing"
              :size="14"
              class="animate-spin text-claw-400"
            />
            <WifiOff
              v-else-if="!store.gatewayStatus.connected"
              :size="14"
            />
            <RefreshCw
              v-else
              :size="14"
              class="text-gray-400"
            />
          </button>
          <button
            @click="handleNewSession"
            class="flex gap-1 items-center px-2 py-1 text-xs text-claw-400 rounded transition-colors hover:bg-dark-600"
          >
            <Plus :size="14" />
            新会话
          </button>
        </div>
      </div>

      <Transition name="error-slide">
        <div
          v-if="refreshError"
          class="mx-4 mt-2 px-3 py-2 text-xs text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg"
        >
          {{ refreshError }}
        </div>
      </Transition>

      <div class="overflow-y-auto flex-1 py-2 scroll-container">
        <div v-if="store.sessions.length === 0" class="flex flex-col justify-center items-center h-full px-4 text-center">
          <MessageSquare :size="32" class="mb-2 text-gray-600" />
          <p class="text-sm text-gray-500">暂无会话记录</p>
          <p class="mt-1 text-xs text-gray-600">点击"新会话"开始聊天</p>
        </div>

        <div v-else class="space-y-4">
          <div v-for="group in groupedSessions" :key="group.label">
            <div class="px-4 py-2 text-xs font-medium text-gray-500">
              {{ group.label }}
            </div>
            <div class="space-y-1 px-2">
              <div
                v-for="session in group.sessions"
                :key="session.key"
                @click="handleSelectSession(session)"
                :class="clsx(
                  'group flex gap-3 items-center p-2 rounded-lg cursor-pointer transition-colors',
                  store.currentSessionKey === session.key
                    ? 'bg-claw-500/20 text-white'
                    : 'text-gray-300 hover:bg-dark-600'
                )"
              >
                <MessageSquare :size="16" class="flex-shrink-0 text-gray-500" />
                <div class="flex-1 min-w-0">
                  <p class="text-sm truncate">{{ session.title }}</p>
                  <div class="flex gap-2 items-center mt-0.5">
                    <Clock :size="10" class="text-gray-600" />
                    <span class="text-xs text-gray-500">
                      {{ formatTime(session.updatedAt) }}
                    </span>
                    <span class="text-xs text-gray-600">
                      · {{ session.messageCount }}条
                    </span>
                  </div>
                </div>
                <button
                  @click="(e) => handleDeleteSession(e, session.key)"
                  class="p-1 rounded opacity-0 group-hover:opacity-100 transition-opacity hover:bg-dark-500"
                >
                  <Trash2 :size="14" class="text-gray-500 hover:text-red-400" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <button
    @click="store.toggleHistoryPanel"
    :class="clsx(
      'absolute top-1/2 z-10 flex justify-center items-center w-5 h-10 rounded-r-lg transition-all duration-300',
      store.historyPanelCollapsed ? 'left-0' : 'left-72',
      'bg-dark-600 border border-l-0 border-dark-500 hover:bg-dark-500'
    )"
    style="transform: translateY(-50%);"
  >
    <ChevronRight v-if="store.historyPanelCollapsed" :size="14" class="text-gray-400" />
    <ChevronLeft v-else :size="14" class="text-gray-400" />
  </button>
</template>

<style scoped>
.scroll-container::-webkit-scrollbar {
  width: 4px;
}

.scroll-container::-webkit-scrollbar-track {
  background: transparent;
}

.scroll-container::-webkit-scrollbar-thumb {
  background: #3d3d44;
  border-radius: 2px;
}

.scroll-container::-webkit-scrollbar-thumb:hover {
  background: #4d4d54;
}

.error-slide-enter-active,
.error-slide-leave-active {
  transition: all 0.3s ease-out;
}

.error-slide-enter-from,
.error-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
