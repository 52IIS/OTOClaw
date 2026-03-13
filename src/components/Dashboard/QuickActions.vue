<script setup lang="ts">
import { computed } from 'vue'
import { Play, Square, RotateCcw, Stethoscope } from 'lucide-vue-next'
import clsx from 'clsx'
import type { ServiceStatus } from '../../vite-env.d'

interface Props {
  status: ServiceStatus | null
  loading: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  start: []
  stop: []
  restart: []
}>()

const isRunning = computed(() => props.status?.running || false)
</script>

<template>
  <div class="bg-dark-700 rounded-2xl p-6 border border-dark-500">
    <h3 class="text-lg font-semibold text-white mb-4">快捷操作</h3>

    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <button
        @click="emit('start')"
        :disabled="loading || isRunning"
        :class="clsx(
          'flex flex-col items-center gap-3 p-4 rounded-xl transition-all',
          'border border-dark-500',
          isRunning
            ? 'bg-dark-600 opacity-50 cursor-not-allowed'
            : 'bg-dark-600 hover:bg-green-500/20 hover:border-green-500/50'
        )"
      >
        <div
          :class="clsx(
            'w-12 h-12 rounded-full flex items-center justify-center',
            isRunning ? 'bg-dark-500' : 'bg-green-500/20'
          )"
        >
          <Play
            :size="20"
            :class="isRunning ? 'text-gray-500' : 'text-green-400'"
          />
        </div>
        <span
          :class="clsx(
            'text-sm font-medium',
            isRunning ? 'text-gray-500' : 'text-gray-300'
          )"
        >
          启动
        </span>
      </button>

      <button
        @click="emit('stop')"
        :disabled="loading || !isRunning"
        :class="clsx(
          'flex flex-col items-center gap-3 p-4 rounded-xl transition-all',
          'border border-dark-500',
          !isRunning
            ? 'bg-dark-600 opacity-50 cursor-not-allowed'
            : 'bg-dark-600 hover:bg-red-500/20 hover:border-red-500/50'
        )"
      >
        <div
          :class="clsx(
            'w-12 h-12 rounded-full flex items-center justify-center',
            !isRunning ? 'bg-dark-500' : 'bg-red-500/20'
          )"
        >
          <Square
            :size="20"
            :class="!isRunning ? 'text-gray-500' : 'text-red-400'"
          />
        </div>
        <span
          :class="clsx(
            'text-sm font-medium',
            !isRunning ? 'text-gray-500' : 'text-gray-300'
          )"
        >
          停止
        </span>
      </button>

      <button
        @click="emit('restart')"
        :disabled="loading"
        :class="clsx(
          'flex flex-col items-center gap-3 p-4 rounded-xl transition-all',
          'border border-dark-500',
          'bg-dark-600 hover:bg-amber-500/20 hover:border-amber-500/50'
        )"
      >
        <div class="w-12 h-12 rounded-full flex items-center justify-center bg-amber-500/20">
          <RotateCcw
            :size="20"
            :class="clsx('text-amber-400', loading && 'animate-spin')"
          />
        </div>
        <span class="text-sm font-medium text-gray-300">重启</span>
      </button>

      <button
        :disabled="loading"
        :class="clsx(
          'flex flex-col items-center gap-3 p-4 rounded-xl transition-all',
          'border border-dark-500',
          'bg-dark-600 hover:bg-purple-500/20 hover:border-purple-500/50'
        )"
      >
        <div class="w-12 h-12 rounded-full flex items-center justify-center bg-purple-500/20">
          <Stethoscope :size="20" class="text-purple-400" />
        </div>
        <span class="text-sm font-medium text-gray-300">诊断</span>
      </button>
    </div>
  </div>
</template>
