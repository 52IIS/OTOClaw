<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import {
  Send,
  Paperclip,
  X,
  Loader2,
  Image as ImageIcon,
  Video,
  Music,
  FileText,
  Sparkles,
  Undo2,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { useChatStore } from '../../stores/chatStore'
import type { ChatAttachment } from './types'
import { isImageMimeType, isVideoMimeType, isAudioMimeType, formatFileSize, getFileTypeFromMime } from './types'

const store = useChatStore()

const textareaRef = ref<HTMLTextAreaElement | null>(null)
const fileInputRef = ref<HTMLInputElement | null>(null)
const dragOver = ref(false)

const MIN_HEIGHT = 44
const MAX_HEIGHT = 200

const textareaHeight = ref(MIN_HEIGHT)

const canSend = computed(() => {
  return (store.inputMessage.trim() || store.attachments.length > 0) && !store.isSending
})

const adjustTextareaHeight = () => {
  if (!textareaRef.value) return
  
  textareaRef.value.style.height = 'auto'
  const scrollHeight = textareaRef.value.scrollHeight
  const newHeight = Math.min(Math.max(scrollHeight, MIN_HEIGHT), MAX_HEIGHT)
  textareaHeight.value = newHeight
  textareaRef.value.style.height = newHeight + 'px'
}

const handleInput = () => {
  adjustTextareaHeight()
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

const handleSend = () => {
  if (!canSend.value) return
  store.sendMessage(store.inputMessage, store.attachments.length > 0 ? [...store.attachments] : undefined)
  resetInputHeight()
}

const resetInputHeight = () => {
  textareaHeight.value = MIN_HEIGHT
  if (textareaRef.value) {
    textareaRef.value.style.height = MIN_HEIGHT + 'px'
  }
  store.clearOptimization()
}

const handleFileSelect = () => {
  fileInputRef.value?.click()
}

const handleFileChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  const files = target.files
  if (files) {
    processFiles(files)
  }
  target.value = ''
}

const processFiles = async (files: FileList) => {
  const maxFileSize = 20 * 1024 * 1024
  
  for (const file of files) {
    if (file.size > maxFileSize) {
      console.warn('文件过大，跳过:', file.name)
      continue
    }

    const reader = new FileReader()
    reader.onload = () => {
      const attachment: ChatAttachment = {
        id: `att-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        name: file.name,
        mimeType: file.type || 'application/octet-stream',
        size: file.size,
        dataUrl: reader.result as string,
        type: getFileTypeFromMime(file.type || 'application/octet-stream'),
      }
      store.addAttachment(attachment)
    }
    reader.readAsDataURL(file)
  }
}

const handleDrop = (e: DragEvent) => {
  e.preventDefault()
  dragOver.value = false
  const files = e.dataTransfer?.files
  if (files && files.length > 0) {
    processFiles(files)
  }
}

const handleDragOver = (e: DragEvent) => {
  e.preventDefault()
  dragOver.value = true
}

const handleDragLeave = () => {
  dragOver.value = false
}

const handleRemoveAttachment = (id: string) => {
  store.removeAttachment(id)
}

const getAttachmentIcon = (mimeType: string) => {
  if (isImageMimeType(mimeType)) return ImageIcon
  if (isVideoMimeType(mimeType)) return Video
  if (isAudioMimeType(mimeType)) return Music
  return FileText
}

const isImage = (mimeType: string) => isImageMimeType(mimeType)

const handleOptimize = async () => {
  if (store.hasOptimized) {
    store.revertPrompt()
  } else {
    await store.optimizePrompt()
  }
}

const optimizeButtonTitle = computed(() => {
  if (store.isOptimizing) {
    return '优化中...'
  }
  return store.hasOptimized ? '撤回优化内容' : '优化输入内容'
})

const canOptimize = computed(() => {
  return store.inputMessage.trim().length > 0 && !store.isSending
})

onMounted(() => {
  nextTick(() => {
    adjustTextareaHeight()
  })
})

watch(() => store.inputMessage, () => {
  nextTick(() => {
    adjustTextareaHeight()
  })
})
</script>

<template>
  <div class="p-4 border-t border-dark-600">
    <div
      v-if="store.attachments.length > 0"
      class="flex flex-wrap gap-2 mb-3"
    >
      <div
        v-for="attachment in store.attachments"
        :key="attachment.id"
        class="relative group"
      >
        <div
          v-if="isImage(attachment.mimeType)"
          class="overflow-hidden relative rounded-lg"
        >
          <img
            :src="attachment.dataUrl"
            :alt="attachment.name"
            class="object-cover h-16 rounded-lg"
          />
          <button
            @click="handleRemoveAttachment(attachment.id)"
            class="absolute top-1 right-1 p-1 rounded-full opacity-0 transition-opacity bg-dark-900/80 group-hover:opacity-100"
          >
            <X :size="12" class="text-white" />
          </button>
        </div>
        <div
          v-else
          class="flex gap-2 items-center px-3 py-2 rounded-lg bg-dark-600"
        >
          <component :is="getAttachmentIcon(attachment.mimeType)" :size="16" class="text-gray-500" />
          <div class="flex-1 min-w-0">
            <p class="text-xs text-white truncate">{{ attachment.name }}</p>
            <p class="text-xs text-gray-500">{{ formatFileSize(attachment.size) }}</p>
          </div>
          <button
            @click="handleRemoveAttachment(attachment.id)"
            class="p-1 rounded transition-colors hover:bg-dark-500"
          >
            <X :size="14" class="text-gray-500" />
          </button>
        </div>
      </div>
    </div>

    <div
      :class="clsx(
        'relative flex gap-2 items-end p-3 rounded-xl border transition-colors',
        dragOver
          ? 'border-claw-500 bg-claw-500/10'
          : 'border-dark-500 bg-dark-700'
      )"
      @drop="handleDrop"
      @dragover="handleDragOver"
      @dragleave="handleDragLeave"
    >
      <input
        ref="fileInputRef"
        type="file"
        multiple
        accept="image/*,video/*,audio/*,.pdf,.txt,.md,.json,.csv,.doc,.docx,.xls,.xlsx,.ppt,.pptx"
        class="hidden"
        @change="handleFileChange"
      />

      <button
        @click="handleFileSelect"
        class="flex-shrink-0 p-2 rounded-lg transition-colors hover:bg-dark-600"
        title="添加附件（图片、视频、文件）"
      >
        <Paperclip :size="18" class="text-gray-500" />
      </button>

      <textarea
        ref="textareaRef"
        v-model="store.inputMessage"
        @input="handleInput"
        @keydown="handleKeydown"
        placeholder="输入消息... (Shift+Enter 换行)"
        rows="1"
        :style="{ height: textareaHeight + 'px' }"
        class="flex-1 overflow-y-auto py-2 text-sm text-white placeholder-gray-500 bg-transparent resize-none focus:outline-none transition-[height] duration-150 ease-out"
      />

      <button
        @click="handleOptimize"
        :disabled="!canOptimize"
        :class="clsx(
          'flex-shrink-0 p-2 rounded-lg transition-colors',
          canOptimize
            ? 'hover:bg-claw-500/20 text-gray-400 hover:text-claw-400'
            : 'text-gray-600 cursor-not-allowed'
        )"
        :title="optimizeButtonTitle"
      >
        <Loader2 v-if="store.isOptimizing" :size="18" class="animate-spin" />
        <Undo2 v-else-if="store.hasOptimized" :size="18" />
        <Sparkles v-else :size="18" />
      </button>

      <button
        @click="handleSend"
        :disabled="!canSend"
        :class="clsx(
          'flex-shrink-0 flex justify-center items-center w-10 h-10 rounded-lg transition-colors',
          canSend
            ? 'bg-claw-500 text-white hover:bg-claw-600'
            : 'bg-dark-600 text-gray-500 cursor-not-allowed'
        )"
      >
        <Loader2 v-if="store.isSending" :size="18" class="animate-spin" />
        <Send v-else :size="18" />
      </button>
    </div>

    <div class="mt-2 text-xs text-gray-600">
      支持拖拽上传图片、视频、音频和文件（最大 20MB）
      <span>
        回车直接发送消息
      </span>
    </div>
  </div>
</template>
