<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { AlertCircle, CheckCircle, Info, AlertTriangle, X } from 'lucide-vue-next'

export interface DialogOptions {
  type?: 'alert' | 'confirm'
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: 'info' | 'success' | 'warning' | 'error'
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

const dialogType = computed(() => props.options.type || 'alert')
const variant = computed(() => props.options.variant || 'info')
const title = computed(() => props.options.title || (dialogType.value === 'confirm' ? '确认' : '提示'))
const confirmText = computed(() => props.options.confirmText || '确定')
const cancelText = computed(() => props.options.cancelText || '取消')

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
          class="bg-dark-800 rounded-xl border border-dark-600 w-full max-w-md overflow-hidden shadow-2xl"
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
</style>
