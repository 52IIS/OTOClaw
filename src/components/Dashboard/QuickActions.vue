<script setup lang="ts">
import { computed } from 'vue'
import { Play, Square, RotateCcw, Stethoscope, Wrench } from 'lucide-vue-next'
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
  diagnose: []
  fix: []
}>()

const isRunning = computed(() => props.status?.running || false)
</script>

<template>
  <div class="p-6 rounded-2xl border bg-dark-700 border-dark-500">
    <h3 class="mb-4 text-lg font-semibold text-white">快捷操作</h3>

    <div class="grid grid-cols-5 gap-3">
      <button
        @click="emit('start')"
        :disabled="loading || isRunning"
        :class="clsx(
          'flex flex-col items-center gap-2 p-3 rounded-xl transition-all',
          'border border-dark-500',
          isRunning
            ? 'bg-dark-600 opacity-50 cursor-not-allowed'
            : 'bg-dark-600 hover:bg-green-500/20 hover:border-green-500/50'
        )"
      >
        <div
          :class="clsx(
            'w-10 h-10 rounded-full flex items-center justify-center',
            isRunning ? 'bg-dark-500' : 'bg-green-500/20'
          )"
        >
          <Play
            :size="18"
            :class="isRunning ? 'text-gray-500' : 'text-green-400'"
          />
        </div>
        <span
          :class="clsx(
            'text-xs font-medium',
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
          'flex flex-col items-center gap-2 p-3 rounded-xl transition-all',
          'border border-dark-500',
          !isRunning
            ? 'bg-dark-600 opacity-50 cursor-not-allowed'
            : 'bg-dark-600 hover:bg-red-500/20 hover:border-red-500/50'
        )"
      >
        <div
          :class="clsx(
            'w-10 h-10 rounded-full flex items-center justify-center',
            !isRunning ? 'bg-dark-500' : 'bg-red-500/20'
          )"
        >
          <Square
            :size="18"
            :class="!isRunning ? 'text-gray-500' : 'text-red-400'"
          />
        </div>
        <span
          :class="clsx(
            'text-xs font-medium',
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
          'flex flex-col items-center gap-2 p-3 rounded-xl transition-all',
          'border border-dark-500',
          'bg-dark-600 hover:bg-amber-500/20 hover:border-amber-500/50'
        )"
      >
        <div class="flex justify-center items-center w-10 h-10 rounded-full bg-amber-500/20">
          <RotateCcw
            :size="18"
            :class="clsx('text-amber-400', loading && 'animate-spin')"
          />
        </div>
        <span class="text-xs font-medium text-gray-300">重启</span>
      </button>

      <button
        @click="emit('diagnose')"
        :disabled="loading"
        :class="clsx(
          'flex flex-col items-center gap-2 p-3 rounded-xl transition-all',
          'border border-dark-500',
          'bg-dark-600 hover:bg-cyan-500/20 hover:border-cyan-500/50'
        )"
      >
        <div class="flex justify-center items-center w-10 h-10 rounded-full bg-cyan-500/20">
          <Stethoscope :size="18" class="text-cyan-400" />
        </div>
        <span class="text-xs font-medium text-gray-300">诊断</span>
      </button>

      <button
        @click="emit('fix')"
        :disabled="loading"
        :class="clsx(
          'flex flex-col items-center gap-2 p-3 rounded-xl transition-all',
          'border border-dark-500',
          'bg-dark-600 hover:bg-purple-500/20 hover:border-purple-500/50'
        )"
      >
        <div class="flex justify-center items-center w-10 h-10 rounded-full bg-purple-500/20">
          <Wrench :size="18" class="text-purple-400" />
        </div>
        <span class="text-xs font-medium text-gray-300">修复</span>
      </button>
    </div>
  </div>
</template>
