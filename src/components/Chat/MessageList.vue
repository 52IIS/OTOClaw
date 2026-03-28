<script setup lang="ts">
import { ref, computed, nextTick, watch, onMounted } from 'vue'
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
  ChevronDown,
  ChevronRight,
  Brain,
  Sparkles,
  AlertCircle,
  File,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { useChatStore } from '../../stores/chatStore'
import type { ChatMessage } from './types'
import { isImageMimeType, isVideoMimeType, isAudioMimeType } from './types'
import { renderMarkdown } from './utils'

const store = useChatStore()

const messagesContainer = ref<HTMLElement | null>(null)
const copiedMessageId = ref<string | null>(null)
const collapsedMessages = ref<Set<string>>(new Set())
const collapsedTools = ref<Set<string>>(new Set())
const collapsedArtifacts = ref<Set<string>>(new Set())
const collapsedErrors = ref<Set<string>>(new Set())

const scrollToBottom = async () => {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

onMounted(() => {
  store.messages.forEach(msg => {
    if (msg.thinking) {
      collapsedMessages.value.add(msg.id)
    }
  })
})

watch(
  () => store.messages.length,
  () => {
    scrollToBottom()
    store.messages.forEach(msg => {
      if (msg.thinking && !collapsedMessages.value.has(msg.id)) {
        collapsedMessages.value.add(msg.id)
      }
    })
  }
)

watch(
  () => store.streaming.streamText,
  (newText) => {
    console.log('[DEBUG: REPEAT_ISSUE] MessageList 流式内容更新', { streamText: newText, messagesLength: store.messages.length })
    scrollToBottom()
  }
)

watch(
  () => store.streaming.streamThinking,
  (newThinking) => {
    console.log('[DEBUG: REPEAT_ISSUE] MessageList 思考内容更新', { streamThinking: newThinking })
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
  return renderMarkdown(content)
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

  // 如果在流式传输，需要排除最后一条正在流式的 assistant 消息
  const messagesToRender = store.streaming.isStreaming && store.messages.length > 0
    ? store.messages.filter((msg, idx) => {
        // 如果是最后一条消息且是 assistant 角色且正在流式传输，则不渲染
        if (idx === store.messages.length - 1 && msg.role === 'assistant' && msg.isStreaming) {
          console.log('[DEBUG: REPEAT_ISSUE] 跳过渲染正在流式的消息', { messageId: msg.id, content: msg.content })
          return false
        }
        return true
      })
    : store.messages

  messagesToRender.forEach(message => {
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
  const hasStreamText = store.streaming.isStreaming && store.streaming.streamText
  console.log('[DEBUG: REPEAT_ISSUE] hasStreamingMessage 计算', { 
    isStreaming: store.streaming.isStreaming, 
    streamText: store.streaming.streamText,
    hasStreamText,
    messagesLength: store.messages.length,
    lastMessage: store.messages[store.messages.length - 1]
  })
  return hasStreamText
})

const hasStreamingThinking = computed(() => {
  const hasStreamThinking = store.streaming.isStreaming && store.streaming.streamThinking
  console.log('[DEBUG: REPEAT_ISSUE] hasStreamingThinking 计算', {
    isStreaming: store.streaming.isStreaming,
    streamThinking: store.streaming.streamThinking,
    hasStreamThinking
  })
  return hasStreamThinking
})

const isMessageCollapsed = (messageId: string) => {
  return collapsedMessages.value.has(messageId)
}

const toggleMessageCollapse = (messageId: string) => {
  if (collapsedMessages.value.has(messageId)) {
    collapsedMessages.value.delete(messageId)
  } else {
    collapsedMessages.value.add(messageId)
  }
}

const isThinkingComplete = (message: ChatMessage) => {
  return message.thinking?.isComplete ?? true
}

const getToolCollapseKey = (messageId: string, toolId: string) => {
  return `${messageId}-tool-${toolId}`
}

const isToolCollapsed = (messageId: string, toolId: string) => {
  return collapsedTools.value.has(getToolCollapseKey(messageId, toolId))
}

const toggleToolCollapse = (messageId: string, toolId: string) => {
  const key = getToolCollapseKey(messageId, toolId)
  if (collapsedTools.value.has(key)) {
    collapsedTools.value.delete(key)
  } else {
    collapsedTools.value.add(key)
  }
}

const getArtifactCollapseKey = (messageId: string, artifactUri: string) => {
  return `${messageId}-artifact-${artifactUri}`
}

const isArtifactCollapsed = (messageId: string, artifactUri: string) => {
  return collapsedArtifacts.value.has(getArtifactCollapseKey(messageId, artifactUri))
}

const toggleArtifactCollapse = (messageId: string, artifactUri: string) => {
  const key = getArtifactCollapseKey(messageId, artifactUri)
  if (collapsedArtifacts.value.has(key)) {
    collapsedArtifacts.value.delete(key)
  } else {
    collapsedArtifacts.value.add(key)
  }
}

const isErrorCollapsed = (messageId: string) => {
  return collapsedErrors.value.has(messageId)
}

const toggleErrorCollapse = (messageId: string) => {
  if (collapsedErrors.value.has(messageId)) {
    collapsedErrors.value.delete(messageId)
  } else {
    collapsedErrors.value.add(messageId)
  }
}
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
          <template v-for="message in group.messages" :key="message.id">
            <div
              v-if="message.attachments && message.attachments.length > 0"
              class="flex flex-wrap gap-2"
            >
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

            <div
              v-if="group.role === 'assistant' && message.thinking && store.showReasoning"
              class="rounded-2xl px-4 py-3 bg-dark-700 text-gray-200"
            >
              <button
                @click="toggleMessageCollapse(message.id)"
                :class="clsx(
                  'flex gap-2 items-center w-full px-2 py-1.5 -mx-2 -my-1.5 text-left rounded-lg transition-colors',
                  isMessageCollapsed(message.id) ? 'hover:bg-dark-600/30' : 'bg-dark-600/50'
                )"
              >
                <div class="flex gap-2 items-center flex-1 min-w-0">
                  <Brain
                    :size="14"
                    :class="clsx(
                      'flex-shrink-0 transition-colors',
                      message.isStreaming && !isThinkingComplete(message) ? 'text-accent-purple animate-pulse' : 'text-accent-purple/70'
                    )"
                  />
                  <span class="text-xs font-medium text-accent-purple/90 truncate">
                    {{ message.isStreaming && !isThinkingComplete(message) ? '思考中...' : '推理过程' }}
                  </span>
                  <Loader2
                    v-if="message.isStreaming && !isThinkingComplete(message)"
                    :size="12"
                    class="flex-shrink-0 text-accent-purple animate-spin"
                  />
                  <Sparkles
                    v-else-if="isThinkingComplete(message)"
                    :size="12"
                    class="flex-shrink-0 text-accent-purple/50"
                  />
                </div>
                <ChevronDown
                  v-if="!isMessageCollapsed(message.id)"
                  :size="14"
                  class="flex-shrink-0 text-gray-500"
                />
                <ChevronRight
                  v-else
                  :size="14"
                  class="flex-shrink-0 text-gray-500"
                />
              </button>

              <div v-if="!isMessageCollapsed(message.id)" class="mt-1 pt-2 pl-4 pr-2 border-l-2 border-accent-purple/30">
                <div class="text-xs text-gray-400 italic leading-relaxed whitespace-pre-wrap break-words">
                  <template v-for="(line, index) in message.thinking.text.split('\n')" :key="index">
                    <span v-if="line.trim()">{{ line.replace(/^(>\s*)?/, '') }}</span>
                    <br v-if="index < message.thinking.text.split('\n').length - 1" />
                  </template>
                  <span
                    v-if="message.isStreaming && !isThinkingComplete(message)"
                    class="inline-block w-1.5 h-3 ml-0.5 bg-accent-purple/50 animate-pulse rounded-sm"
                  />
                </div>
              </div>
            </div>

            <div
              v-if="(message.toolCalls && message.toolCalls.length > 0) || (message.toolResults && message.toolResults.length > 0)"
              class="space-y-2"
            >
              <div
                v-for="tool in message.toolCalls"
                :key="tool.id"
                class="rounded-lg border bg-dark-600 border-dark-500 overflow-hidden"
              >
                <button
                  @click="toggleToolCollapse(message.id, tool.id)"
                  class="flex gap-2 items-center w-full px-3 py-2 text-left bg-dark-500/50 hover:bg-dark-500 transition-colors"
                >
                  <Wrench :size="14" class="text-accent-cyan flex-shrink-0" />
                  <span class="text-xs font-medium text-accent-cyan flex-1 min-w-0 truncate">工具调用: {{ tool.name }}</span>
                  <ChevronDown
                    v-if="!isToolCollapsed(message.id, tool.id)"
                    :size="14"
                    class="text-gray-500 flex-shrink-0"
                  />
                  <ChevronRight
                    v-else
                    :size="14"
                    class="text-gray-500 flex-shrink-0"
                  />
                </button>
                <div v-if="!isToolCollapsed(message.id, tool.id)" class="p-3">
                  <pre class="text-xs text-gray-400 overflow-x-auto whitespace-pre-wrap break-words">{{ JSON.stringify(tool.arguments, null, 2) }}</pre>
                </div>
              </div>

              <div
                v-for="result in message.toolResults"
                :key="result.id"
                class="rounded-lg border bg-dark-600 border-dark-500 overflow-hidden"
              >
                <button
                  @click="toggleToolCollapse(message.id, result.id)"
                  class="flex gap-2 items-center w-full px-3 py-2 text-left bg-dark-500/50 hover:bg-dark-500 transition-colors"
                >
                  <Check :size="14" class="text-green-400 flex-shrink-0" />
                  <span class="text-xs font-medium text-green-400 flex-1 min-w-0 truncate">结果: {{ result.name }}</span>
                  <ChevronDown
                    v-if="!isToolCollapsed(message.id, result.id)"
                    :size="14"
                    class="text-gray-500 flex-shrink-0"
                  />
                  <ChevronRight
                    v-else
                    :size="14"
                    class="text-gray-500 flex-shrink-0"
                  />
                </button>
                <div v-if="!isToolCollapsed(message.id, result.id)" class="p-3">
                  <pre class="text-xs text-gray-400 overflow-x-auto whitespace-pre-wrap break-words">{{ typeof result.result === 'string' ? result.result : JSON.stringify(result.result, null, 2) }}</pre>
                </div>
              </div>
            </div>

            <div
              v-if="message.artifacts && message.artifacts.length > 0"
              class="space-y-2"
            >
              <div
                v-for="artifact in message.artifacts"
                :key="artifact.id"
                class="rounded-lg border bg-dark-600 border-dark-500 overflow-hidden"
              >
                <button
                  @click="toggleArtifactCollapse(message.id, artifact.uri)"
                  class="flex gap-2 items-center w-full px-3 py-2 text-left bg-dark-500/50 hover:bg-dark-500 transition-colors"
                >
                  <ImageIcon v-if="artifact.kind === 'image'" :size="14" class="text-accent-yellow flex-shrink-0" />
                  <File v-else :size="14" class="text-accent-yellow flex-shrink-0" />
                  <span class="text-xs font-medium text-accent-yellow flex-1 min-w-0 truncate">
                    {{ artifact.kind === 'image' ? '图片' : '文件' }}: {{ artifact.uri }}
                  </span>
                  <ChevronDown
                    v-if="!isArtifactCollapsed(message.id, artifact.uri)"
                    :size="14"
                    class="text-gray-500 flex-shrink-0"
                  />
                  <ChevronRight
                    v-else
                    :size="14"
                    class="text-gray-500 flex-shrink-0"
                  />
                </button>
                <div v-if="!isArtifactCollapsed(message.id, artifact.uri)" class="p-3">
                  <img
                    v-if="artifact.kind === 'image'"
                    :src="artifact.uri"
                    class="max-w-full rounded-lg"
                  />
                  <a
                    v-else
                    :href="artifact.uri"
                    class="text-xs text-claw-400 hover:underline break-all"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {{ artifact.uri }}
                  </a>
                </div>
              </div>
            </div>

            <div
              v-if="message.error"
              class="rounded-lg border border-red-500/50 bg-red-500/10 overflow-hidden"
            >
              <button
                @click="toggleErrorCollapse(message.id)"
                class="flex gap-2 items-center w-full px-3 py-2 text-left hover:bg-red-500/20 transition-colors"
              >
                <AlertCircle :size="14" class="text-red-400 flex-shrink-0" />
                <span class="text-xs font-medium text-red-400 flex-1 min-w-0 truncate">错误</span>
                <ChevronDown
                  v-if="!isErrorCollapsed(message.id)"
                  :size="14"
                  class="text-gray-500 flex-shrink-0"
                />
                <ChevronRight
                  v-else
                  :size="14"
                  class="text-gray-500 flex-shrink-0"
                />
              </button>
              <div v-if="!isErrorCollapsed(message.id)" class="p-3">
                <pre class="text-xs text-red-300 whitespace-pre-wrap break-words">{{ message.error.message }}</pre>
              </div>
            </div>

            <div
              v-if="message.content"
              :class="clsx(
                'rounded-2xl px-4 py-3',
                group.role === 'user'
                  ? 'bg-claw-500 text-white'
                  : 'bg-dark-700 text-gray-200'
              )"
            >
              <div
                class="text-sm whitespace-pre-wrap break-words"
                v-html="renderContent(message.content)"
              />
            </div>

            <div
              v-if="message.isStreaming && !message.content && !store.streaming.streamText"
              :class="clsx(
                'rounded-2xl px-4 py-3',
                'bg-dark-700 text-gray-200'
              )"
            >
              <div class="flex gap-1 items-center">
                <Loader2 :size="14" class="animate-spin text-claw-400" />
                <span class="text-xs text-gray-500">等待回复...</span>
              </div>
            </div>
          </template>

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
            <button
              @click="() => {}"
              class="flex gap-2 items-center w-full px-2 py-1.5 -mx-2 -my-1.5 text-left rounded-lg bg-dark-600/50 transition-colors cursor-default"
            >
              <div class="flex gap-2 items-center flex-1 min-w-0">
                <Brain
                  :size="14"
                  class="flex-shrink-0 text-accent-purple animate-pulse"
                />
                <span class="text-xs font-medium text-accent-purple/90 truncate">
                  思考中...
                </span>
                <Loader2
                  :size="12"
                  class="flex-shrink-0 text-accent-purple animate-spin"
                />
              </div>
            </button>

            <div class="mt-1 pt-2 pl-4 pr-2 border-l-2 border-accent-purple/30">
              <div class="text-xs text-gray-400 italic leading-relaxed whitespace-pre-wrap break-words">
                <template v-for="(line, index) in store.streaming.streamThinking.split('\n')" :key="index">
                  <span v-if="line.trim()">{{ line.replace(/^(>\s*)?/, '') }}</span>
                  <br v-if="index < store.streaming.streamThinking.split('\n').length - 1" />
                </template>
                <span class="inline-block w-1.5 h-3 ml-0.5 bg-accent-purple/50 animate-pulse rounded-sm" />
              </div>
            </div>
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
