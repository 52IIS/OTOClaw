<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { AlertCircle, CheckCircle, Info, AlertTriangle, X, ChevronDown, ChevronUp, Copy, Check } from 'lucide-vue-next'

export interface DialogOptions {
  type?: 'alert' | 'confirm'
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: 'info' | 'success' | 'warning' | 'error'
  details?: string
  detailsLabel?: string
}

const props = defineProps<{
  visible: boolean
  options: DialogOptions
}>()

const emit = defineEmits<{
  (e: 'confirm'): void
  (e: 'cancel'): void
  (e: 'close'): void
}>()

const detailsExpanded = ref(false)
const copied = ref(false)

const dialogType = computed(() => props.options.type || 'alert')
const variant = computed(() => props.options.variant || 'info')
const title = computed(() => props.options.title || (dialogType.value === 'confirm' ? '确认' : '提示'))
const confirmText = computed(() => props.options.confirmText || '确定')
const cancelText = computed(() => props.options.cancelText || '取消')
const hasDetails = computed(() => !!props.options.details)
const detailsLabel = computed(() => props.options.detailsLabel || '详细日志')

const iconComponent = computed(() => {
  switch (variant.value) {
    case 'success': return CheckCircle
    case 'warning': return AlertTriangle
    case 'error': return AlertCircle
    default: return Info
  }
})

const iconColor = computed(() => {
  switch (variant.value) {
    case 'success': return 'text-green-400'
    case 'warning': return 'text-yellow-400'
    case 'error': return 'text-red-400'
    default: return 'text-claw-400'
  }
})

const iconBg = computed(() => {
  switch (variant.value) {
    case 'success': return 'bg-green-500/20'
    case 'warning': return 'bg-yellow-500/20'
    case 'error': return 'bg-red-500/20'
    default: return 'bg-claw-500/20'
  }
})

const confirmButtonClass = computed(() => {
  const base = 'px-6 py-2.5 rounded-lg font-medium transition-all duration-200'
  if (variant.value === 'error' || variant.value === 'warning') {
    return `${base} bg-red-500 text-white hover:bg-red-600 active:bg-red-700`
  }
  return `${base} bg-claw-500 text-white hover:bg-claw-600 active:bg-claw-700`
})

const handleConfirm = () => {
  emit('confirm')
  emit('close')
}

const handleCancel = () => {
  emit('cancel')
  emit('close')
}

const handleBackdropClick = () => {
  if (dialogType.value === 'alert') {
    handleConfirm()
  }
}

const toggleDetails = () => {
  detailsExpanded.value = !detailsExpanded.value
}

const copyDetails = async () => {
  if (!props.options.details) return
  try {
    await navigator.clipboard.writeText(props.options.details)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (e) {
    console.error('复制失败', e)
  }
}

const getLogLineClass = (line: string) => {
  const lowerLine = line.toLowerCase()
  if (lowerLine.includes('error') || lowerLine.includes('失败') || lowerLine.includes('✗')) {
    return 'text-red-400'
  }
  if (lowerLine.includes('warn') || lowerLine.includes('警告')) {
    return 'text-yellow-400'
  }
  if (lowerLine.includes('success') || lowerLine.includes('成功') || lowerLine.includes('✓') || lowerLine.includes('通过')) {
    return 'text-green-400'
  }
  if (lowerLine.includes('info') || lowerLine.includes('信息')) {
    return 'text-blue-400'
  }
  return 'text-gray-400'
}

const handleKeydown = (e: KeyboardEvent) => {
  if (!props.visible) return
  
  if (e.key === 'Escape') {
    handleCancel()
  } else if (e.key === 'Enter') {
    handleConfirm()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog">
      <div
        v-if="visible"
        class="flex fixed inset-0 z-[100] justify-center items-center p-4 backdrop-blur-sm bg-black/60"
        @click="handleBackdropClick"
      >
        <div
          :class="[
            'bg-dark-800 rounded-xl border border-dark-600 w-full overflow-hidden shadow-2xl',
            hasDetails ? 'max-w-2xl' : 'max-w-md'
          ]"
          @click.stop
          role="dialog"
          aria-modal="true"
          :aria-labelledby="title"
        >
          <div class="p-6">
            <div class="flex gap-4 items-start">
              <div :class="['flex shrink-0 justify-center items-center w-12 h-12 rounded-full', iconBg]">
                <component :is="iconComponent" :size="24" :class="iconColor" />
              </div>
              <div class="flex-1 min-w-0">
                <h3 class="text-lg font-semibold text-white">
                  {{ title }}
                </h3>
                <p class="mt-2 text-sm text-gray-300 whitespace-pre-wrap break-words">
                  {{ options.message }}
                </p>
              </div>
              <button
                @click="handleCancel"
                class="shrink-0 p-1 text-gray-500 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
                aria-label="关闭"
              >
                <X :size="18" />
              </button>
            </div>

            <Transition name="expand">
              <div v-if="hasDetails && detailsExpanded" class="mt-4">
                <div class="overflow-hidden rounded-lg border border-dark-500 bg-dark-900">
                  <div class="flex justify-between items-center px-3 py-2 border-b border-dark-600 bg-dark-700/50">
                    <span class="text-xs font-medium text-gray-400">{{ detailsLabel }}</span>
                    <button
                      @click="copyDetails"
                      class="flex gap-1 items-center text-xs text-gray-500 transition-colors hover:text-gray-300"
                    >
                      <Check v-if="copied" :size="12" class="text-green-400" />
                      <Copy v-else :size="12" />
                      {{ copied ? '已复制' : '复制' }}
                    </button>
                  </div>
                  <div class="overflow-y-auto p-3 max-h-64 font-mono text-xs leading-relaxed">
                    <div
                      v-for="(line, index) in options.details?.split('\n') || []"
                      :key="index"
                      :class="['py-0.5 whitespace-pre-wrap break-all', getLogLineClass(line)]"
                    >
                      {{ line }}
                    </div>
                  </div>
                </div>
              </div>
            </Transition>

            <button
              v-if="hasDetails"
              @click="toggleDetails"
              class="flex gap-1 items-center mt-3 text-xs text-gray-500 transition-colors hover:text-gray-300"
            >
              <ChevronDown v-if="!detailsExpanded" :size="14" />
              <ChevronUp v-else :size="14" />
              {{ detailsExpanded ? '收起' : '展开' }}{{ detailsLabel }}
            </button>
          </div>

          <div class="flex justify-end gap-3 px-6 py-4 border-t border-dark-600 bg-dark-700/50">
            <button
              v-if="dialogType === 'confirm'"
              @click="handleCancel"
              class="px-6 py-2.5 rounded-lg font-medium text-white border border-dark-400 transition-all duration-200 bg-dark-600 hover:bg-dark-500 active:bg-dark-400"
            >
              {{ cancelText }}
            </button>
            <button
              @click="handleConfirm"
              :class="confirmButtonClass"
            >
              {{ confirmText }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.dialog-enter-active,
.dialog-leave-active {
  transition: all 0.2s ease;
}

.dialog-enter-active .bg-dark-800,
.dialog-leave-active .bg-dark-800 {
  transition: all 0.2s ease;
}

.dialog-enter-from,
.dialog-leave-to {
  opacity: 0;
}

.dialog-enter-from .bg-dark-800,
.dialog-leave-to .bg-dark-800 {
  transform: scale(0.95);
  opacity: 0;
}

.expand-enter-active,
.expand-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
}

.expand-enter-to,
.expand-leave-from {
  max-height: 300px;
}
</style>
