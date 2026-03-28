<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { X, Bot, Cpu, Loader2, Check } from 'lucide-vue-next'
import clsx from 'clsx'
import { useChatStore } from '../../stores/chatStore'

defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
  create: [{ agentId: string; modelId: string }]
}>()

const store = useChatStore()

const selectedAgentId = ref<string | null>(null)
const selectedModelId = ref<string | null>(null)
const loading = ref(false)

const canCreate = computed(() =>
  selectedAgentId.value && selectedModelId.value && !loading.value
)

const handleSelectAgent = (agentId: string) => {
  selectedAgentId.value = agentId
  const agent = store.agents.find(a => a.id === agentId)
  if (agent?.model && store.models.some(m => m.id === agent.model)) {
    selectedModelId.value = agent.model
  }
}

const handleSelectModel = (modelId: string) => {
  selectedModelId.value = modelId
}

const handleCreate = async () => {
  if (!canCreate.value) return

  loading.value = true
  try {
    emit('create', {
      agentId: selectedAgentId.value!,
      modelId: selectedModelId.value!,
    })
  } finally {
    loading.value = false
  }
}

const handleClose = () => {
  selectedAgentId.value = null
  selectedModelId.value = null
  emit('close')
}

onMounted(() => {
  if (store.agents.length === 0) {
    store.loadAgents()
  }
  if (store.models.length === 0) {
    store.loadModels()
  }
  if (store.selectedAgentId) {
    selectedAgentId.value = store.selectedAgentId
  }
  if (store.selectedModelId) {
    selectedModelId.value = store.selectedModelId
  }
})
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog">
      <div
        v-if="visible"
        class="flex fixed inset-0 z-50 justify-center items-center p-4 bg-black/60 backdrop-blur-sm"
        @click.self="handleClose"
      >
        <div
          class="w-full max-w-2xl bg-dark-800 rounded-2xl border border-dark-500 shadow-2xl overflow-hidden"
          @click.stop
        >
          <div class="flex items-center justify-between px-6 py-4 border-b border-dark-600">
            <div>
              <h3 class="text-xl font-semibold text-white">新建会话</h3>
              <p class="mt-1 text-sm text-gray-400">选择智能体和模型开始新对话</p>
            </div>
            <button
              @click="handleClose"
              class="p-2 text-gray-400 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
            >
              <X :size="20" />
            </button>
          </div>

          <div class="p-6">
            <div class="grid grid-cols-2 gap-6">
              <div>
                <label class="flex items-center gap-2 mb-3 text-sm font-medium text-gray-300">
                  <Bot :size="16" class="text-claw-400" />
                  选择智能体
                </label>
                <div class="space-y-2 max-h-80 overflow-y-auto pr-2">
                  <button
                    v-for="agent in store.agents"
                    :key="agent.id"
                    @click="handleSelectAgent(agent.id)"
                    :class="clsx(
                      'flex gap-3 items-center w-full p-4 rounded-xl border transition-all text-left',
                      selectedAgentId === agent.id
                        ? 'bg-claw-500/20 border-claw-500/50 ring-1 ring-claw-500/30'
                        : 'bg-dark-700 border-dark-500 hover:border-dark-400 hover:bg-dark-600'
                    )"
                  >
                    <span class="flex justify-center items-center w-12 h-12 text-2xl rounded-xl bg-dark-600 shrink-0">
                      {{ agent.avatar || '🤖' }}
                    </span>
                    <div class="flex-1 min-w-0">
                      <p class="font-medium text-white truncate">{{ agent.name }}</p>
                      <p v-if="agent.description" class="mt-0.5 text-xs text-gray-500 truncate">
                        {{ agent.description }}
                      </p>
                    </div>
                    <Check
                      v-if="selectedAgentId === agent.id"
                      :size="18"
                      class="text-claw-400 shrink-0"
                    />
                  </button>
                </div>
              </div>

              <div>
                <label class="flex items-center gap-2 mb-3 text-sm font-medium text-gray-300">
                  <Cpu :size="16" class="text-accent-purple" />
                  选择模型
                </label>
                <div class="space-y-2 max-h-80 overflow-y-auto pr-2">
                  <button
                    v-for="model in store.models"
                    :key="model.id"
                    @click="handleSelectModel(model.id)"
                    :class="clsx(
                      'flex gap-3 items-center w-full p-4 rounded-xl border transition-all text-left',
                      selectedModelId === model.id
                        ? 'bg-claw-500/20 border-claw-500/50 ring-1 ring-claw-500/30'
                        : 'bg-dark-700 border-dark-500 hover:border-dark-400 hover:bg-dark-600'
                    )"
                  >
                    <div class="flex-1 min-w-0">
                      <p class="font-medium text-white truncate">{{ model.name }}</p>
                      <p v-if="model.provider" class="mt-0.5 text-xs text-gray-500 truncate">
                        {{ model.provider }}
                      </p>
                    </div>
                    <Check
                      v-if="selectedModelId === model.id"
                      :size="18"
                      class="text-claw-400 shrink-0"
                    />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div class="flex gap-3 justify-end px-6 py-4 border-t border-dark-600 bg-dark-700/50">
            <button
              @click="handleClose"
              class="px-5 py-2.5 text-gray-400 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
            >
              取消
            </button>
            <button
              @click="handleCreate"
              :disabled="!canCreate"
              :class="clsx(
                'flex gap-2 items-center px-6 py-2.5 rounded-lg font-medium transition-all',
                'bg-claw-500 text-white hover:bg-claw-600',
                'disabled:opacity-50 disabled:cursor-not-allowed'
              )"
            >
              <Loader2 v-if="loading" :size="18" class="animate-spin" />
              创建会话
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
