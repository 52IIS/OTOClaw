<script setup lang="ts">
import { computed } from 'vue'
import { Brain } from 'lucide-vue-next'
import clsx from 'clsx'
import { useChatStore } from '../../stores/chatStore'

const store = useChatStore()

const isActive = computed(() => store.showReasoning)

const handleClick = () => {
  store.toggleReasoning()
}
</script>

<template>
  <button
    @click="handleClick"
    :class="clsx(
      'relative flex justify-center items-center w-8 h-8 rounded-lg border transition-all duration-200',
      isActive
        ? 'bg-accent-purple/20 border-accent-purple/50 text-accent-purple'
        : 'bg-dark-700 border-dark-500 text-gray-500 hover:border-dark-400 hover:text-gray-400'
    )"
    :title="isActive ? '隐藏推理过程' : '显示推理过程'"
  >
    <Brain
      :size="16"
      :class="clsx(
        'transition-all duration-200',
        isActive ? 'text-accent-purple' : 'opacity-50'
      )"
    />
    
    <div
      v-if="isActive"
      class="absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full bg-accent-purple animate-pulse"
    />
  </button>
</template>

<style scoped>
@keyframes pulse-ring {
  0% {
    transform: scale(0.8);
    opacity: 1;
  }
  100% {
    transform: scale(1.5);
    opacity: 0;
  }
}
</style>
