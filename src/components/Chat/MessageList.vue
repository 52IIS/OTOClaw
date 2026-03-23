<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import {
  User,
  Bot,
  Copy,
  Loader2,
  Image as ImageIcon,
  Wrench,
  Video,
  FileText,
  Music,
  StopCircle,
  Check,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { useChatStore } from '../../stores/chatStore'
import type { ChatMessage } from './types'
import { isImageMimeType, isVideoMimeType, isAudioMimeType } from './types'
import ThinkingBlock from './ThinkingBlock.vue'

const store = useChatStore()

const messagesContainer = ref<HTMLElement | null>(null)
const copiedMessageId = ref<string | null>(null)

const scrollToBottom = async () => {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

watch(
  () => store.messages.length,
  () => {
    scrollToBottom()
  }
)

watch(
  () => store.streaming.streamText,
  () => {
    scrollToBottom()
  }
)

watch(
  () => store.streaming.streamThinking,
  () => {
    scrollToBottom()
  }
)

const formatTime = (timestamp: number) => {
  return new Date(timestamp).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  })
}

const handleCopy = async (content: string, messageId: string) => {
  try {
    await navigator.clipboard.writeText(content)
    copiedMessageId.value = messageId
    setTimeout(() => {
      copiedMessageId.value = null
    }, 2000)
  } catch (e) {
    console.error('复制失败', e)
  }
}

const handleStopStreaming = () => {
  store.stopStreaming()
}

const renderContent = (content: string) => {
  if (!content) return ''
  return content.replace(/\n/g, '<br>')
}

const getAttachmentIcon = (mimeType: string) => {
  if (isImageMimeType(mimeType)) return ImageIcon
  if (isVideoMimeType(mimeType)) return Video
  if (isAudioMimeType(mimeType)) return Music
  return FileText
}

const groupedMessages = computed(() => {
  const groups: Array<{ role: 'user' | 'assistant'; messages: ChatMessage[] }> = []
  let currentGroup: { role: 'user' | 'assistant'; messages: ChatMessage[] } | null = null

  store.messages.forEach(message => {
    if (message.role === 'system' || message.role === 'tool') return

    if (!currentGroup || currentGroup.role !== message.role) {
      currentGroup = { role: message.role as 'user' | 'assistant', messages: [] }
      groups.push(currentGroup)
    }
    currentGroup.messages.push(message)
  })

  return groups
})

const hasStreamingMessage = computed(() => {
  return store.streaming.isStreaming && store.streaming.streamText
})

const hasStreamingThinking = computed(() => {
  return store.streaming.isStreaming && store.streaming.streamThinking
})
</script>

<template>
  <div
    ref="messagesContainer"
    class="overflow-y-auto flex-1 p-4 space-y-4 scroll-container"
  >
    <div v-if="store.messages.length === 0" class="flex flex-col justify-center items-center h-full text-center">
      <Bot :size="48" class="mb-4 text-gray-600" />
      <h3 class="mb-2 text-lg font-medium text-white">开始新对话</h3>
      <p class="max-w-sm text-sm text-gray-500">
        选择智能体和模型，输入您的问题开始对话
      </p>
      <p class="mt-2 text-xs text-gray-600">
        支持文本、图片、视频和文件附件
      </p>
    </div>

    <div v-else>
      <div
        v-for="(group, groupIndex) in groupedMessages"
        :key="groupIndex"
        :class="clsx(
          'flex gap-3',
          group.role === 'user' ? 'justify-end' : 'justify-start'
        )"
      >
        <div
          v-if="group.role === 'assistant'"
          class="flex-shrink-0 flex justify-center items-center w-8 h-8 rounded-full bg-claw-500/20"
        >
          <Bot :size="18" class="text-claw-400" />
        </div>

        <div class="flex flex-col gap-2 max-w-[80%]">
          <div
            v-for="message in group.messages"
            :key="message.id"
            :class="clsx(
              'rounded-2xl px-4 py-3',
              group.role === 'user'
                ? 'bg-claw-500 text-white'
                : 'bg-dark-700 text-gray-200'
            )"
          >
            <div v-if="message.attachments && message.attachments.length > 0" class="flex flex-wrap gap-2 mb-2">
              <div
                v-for="attachment in message.attachments"
                :key="attachment.id"
                class="relative overflow-hidden rounded-lg"
              >
                <img
                  v-if="isImageMimeType(attachment.mimeType)"
                  :src="attachment.dataUrl"
                  :alt="attachment.name"
                  class="h-32 object-cover rounded-lg cursor-pointer hover:opacity-90 transition-opacity"
                  @click="$emit('preview-image', attachment.dataUrl)"
                />
                <video
                  v-else-if="isVideoMimeType(attachment.mimeType)"
                  :src="attachment.dataUrl"
                  class="h-32 object-cover rounded-lg"
                  controls
                />
                <div
                  v-else
                  class="flex gap-2 items-center px-3 py-2 rounded-lg bg-dark-600"
                >
                  <component :is="getAttachmentIcon(attachment.mimeType)" :size="16" class="text-gray-500" />
                  <span class="text-xs text-gray-400">{{ attachment.name }}</span>
                </div>
              </div>
            </div>

            <ThinkingBlock
              v-if="group.role === 'assistant' && message.thinking && store.showReasoning"
              :thinking="message.thinking"
              :is-streaming="message.isStreaming"
            />

            <div
              v-if="message.toolCalls && message.toolCalls.length > 0"
              class="mb-2 space-y-2"
            >
              <div
                v-for="tool in message.toolCalls"
                :key="tool.id"
                class="p-2 rounded-lg border bg-dark-600 border-dark-500"
              >
                <div class="flex gap-2 items-center mb-1">
                  <Wrench :size="12" class="text-accent-cyan" />
                  <span class="text-xs font-medium text-accent-cyan">{{ tool.name }}</span>
                </div>
                <pre class="overflow-x-auto text-xs text-gray-500">{{ JSON.stringify(tool.arguments, null, 2) }}</pre>
              </div>
            </div>

            <div
              v-if="message.content || message.isStreaming"
              class="text-sm whitespace-pre-wrap break-words"
              v-html="renderContent(message.content)"
            />

            <div
              v-if="message.isStreaming && !message.content"
              class="flex gap-1 items-center"
            >
              <Loader2 :size="14" class="animate-spin text-claw-400" />
              <span class="text-xs text-gray-500">思考中...</span>
            </div>
          </div>

          <div
            :class="clsx(
              'flex gap-2 items-center text-xs text-gray-500',
              group.role === 'user' ? 'justify-end' : 'justify-start'
            )"
          >
            <span>{{ formatTime(group.messages[group.messages.length - 1].timestamp) }}</span>
            <button
              v-if="group.role === 'assistant'"
              @click="handleCopy(group.messages[group.messages.length - 1].content, group.messages[group.messages.length - 1].id)"
              class="p-1 rounded transition-colors hover:bg-dark-600"
            >
              <Check v-if="copiedMessageId === group.messages[group.messages.length - 1].id" :size="12" class="text-green-400" />
              <Copy v-else :size="12" />
            </button>
          </div>
        </div>

        <div
          v-if="group.role === 'user'"
          class="flex-shrink-0 flex justify-center items-center w-8 h-8 rounded-full bg-dark-600"
        >
          <User :size="18" class="text-gray-400" />
        </div>
      </div>

      <div
        v-if="hasStreamingThinking && store.showReasoning"
        class="flex gap-3 justify-start"
      >
        <div class="flex-shrink-0 flex justify-center items-center w-8 h-8 rounded-full bg-claw-500/20">
          <Bot :size="18" class="text-claw-400" />
        </div>
        <div class="flex flex-col gap-2 max-w-[80%]">
          <div class="rounded-2xl px-4 py-3 bg-dark-700 text-gray-200">
            <ThinkingBlock
              :thinking="store.streaming.streamThinking"
              :is-streaming="true"
            />
          </div>
        </div>
      </div>

      <div
        v-if="hasStreamingMessage"
        class="flex gap-3 justify-start"
      >
        <div class="flex-shrink-0 flex justify-center items-center w-8 h-8 rounded-full bg-claw-500/20">
          <Bot :size="18" class="text-claw-400" />
        </div>
        <div class="flex flex-col gap-2 max-w-[80%]">
          <div class="rounded-2xl px-4 py-3 bg-dark-700 text-gray-200">
            <div
              class="text-sm whitespace-pre-wrap break-words"
              v-html="renderContent(store.streaming.streamText)"
            />
            <span class="inline-block w-1.5 h-3 ml-0.5 bg-claw-400 animate-pulse rounded-sm" />
          </div>
          <div class="flex gap-2 items-center text-xs text-gray-500">
            <button
              @click="handleStopStreaming"
              class="flex gap-1 items-center px-2 py-1 rounded transition-colors hover:bg-dark-600"
            >
              <StopCircle :size="12" />
              停止生成
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.scroll-container::-webkit-scrollbar {
  width: 6px;
}

.scroll-container::-webkit-scrollbar-track {
  background: transparent;
}

.scroll-container::-webkit-scrollbar-thumb {
  background: #3d3d44;
  border-radius: 3px;
}

.scroll-container::-webkit-scrollbar-thumb:hover {
  background: #4d4d54;
}
</style>
