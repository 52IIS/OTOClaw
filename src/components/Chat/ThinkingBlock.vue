<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Brain, ChevronDown, ChevronRight, Sparkles, Loader2 } from 'lucide-vue-next'
import clsx from 'clsx'
import type { ThinkingContent } from './types'

interface Props {
  thinking: ThinkingContent | string | null | undefined
  isStreaming?: boolean
  showContent?: boolean
  collapsed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isStreaming: false,
  showContent: true,
  collapsed: undefined,
})

const emit = defineEmits<{
  (e: 'toggle'): void
}>()

const isExpanded = ref(props.collapsed === undefined ? true : !props.collapsed)
const autoCollapsed = ref(false)

const thinkingText = computed(() => {
  if (!props.thinking) return ''
  if (typeof props.thinking === 'string') return props.thinking
  return props.thinking.text || ''
})

const isComplete = computed(() => {
  if (!props.thinking) return true
  if (typeof props.thinking === 'string') return true
  return props.thinking.isComplete
})

const shouldShow = computed(() => {
  return props.showContent && thinkingText.value
})

const shouldAutoCollapse = computed(() => {
  return isComplete.value && thinkingText.value && !props.isStreaming
})

watch(shouldAutoCollapse, (autoCollapse) => {
  if (autoCollapse && props.collapsed === undefined) {
    autoCollapsed.value = true
    isExpanded.value = false
  }
}, { immediate: true })

watch(() => props.collapsed, (collapsed) => {
  if (collapsed !== undefined) {
    isExpanded.value = !collapsed
  }
})

const handleToggle = () => {
  isExpanded.value = !isExpanded.value
  autoCollapsed.value = false
  emit('toggle')
}

const formatThinkingLine = (line: string) => {
  return line.replace(/^(>\s*)?/, '')
}
</script>

<template>
  <div v-if="shouldShow" class="thinking-block">
    <button
      @click="handleToggle"
      :class="clsx(
        'flex gap-2 items-center w-full px-2 py-1.5 text-left rounded-lg transition-colors',
        isExpanded ? 'bg-dark-600/50' : 'hover:bg-dark-600/30'
      )"
    >
      <div class="flex gap-2 items-center flex-1 min-w-0">
        <Brain
          :size="14"
          :class="clsx(
            'flex-shrink-0 transition-colors',
            isStreaming && !isComplete ? 'text-accent-purple animate-pulse' : 'text-accent-purple/70'
          )"
        />
        <span class="text-xs font-medium text-accent-purple/90 truncate">
          {{ isStreaming && !isComplete ? '思考中...' : '推理过程' }}
        </span>
        <Loader2
          v-if="isStreaming && !isComplete"
          :size="12"
          class="flex-shrink-0 text-accent-purple animate-spin"
        />
        <Sparkles
          v-else-if="isComplete"
          :size="12"
          class="flex-shrink-0 text-accent-purple/50"
        />
      </div>
      <ChevronDown
        v-if="isExpanded"
        :size="14"
        class="flex-shrink-0 text-gray-500"
      />
      <ChevronRight
        v-else
        :size="14"
        class="flex-shrink-0 text-gray-500"
      />
    </button>

    <Transition name="expand">
      <div
        v-if="isExpanded"
        class="mt-1 pl-4 pr-2 overflow-hidden border-l-2 border-accent-purple/30"
      >
        <div class="py-2 text-xs text-gray-400 italic leading-relaxed whitespace-pre-wrap break-words">
          <template v-for="(line, index) in thinkingText.split('\n')" :key="index">
            <span v-if="line.trim()">{{ formatThinkingLine(line) }}</span>
            <br v-if="index < thinkingText.split('\n').length - 1" />
          </template>
          <span
            v-if="isStreaming && !isComplete"
            class="inline-block w-1.5 h-3 ml-0.5 bg-accent-purple/50 animate-pulse rounded-sm"
          />
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.thinking-block {
  margin-bottom: 0.5rem;
}

.expand-enter-active,
.expand-leave-active {
  transition: all 0.25s ease-out;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.expand-enter-to,
.expand-leave-from {
  opacity: 1;
  max-height: 500px;
}

@keyframes pulse-subtle {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}
</style>
