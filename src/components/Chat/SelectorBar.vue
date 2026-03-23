<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ChevronDown, Bot, Cpu, Check } from 'lucide-vue-next'
import clsx from 'clsx'
import { useChatStore } from '../../stores/chatStore'

const store = useChatStore()

const agentDropdownOpen = ref(false)
const modelDropdownOpen = ref(false)

const selectedAgent = computed(() =>
  store.agents.find(a => a.id === store.selectedAgentId)
)

const selectedModel = computed(() =>
  store.models.find(m => m.id === store.selectedModelId)
)

const handleSelectAgent = (agentId: string) => {
  store.setAgent(agentId)
  agentDropdownOpen.value = false
}

const handleSelectModel = (modelId: string) => {
  store.setModel(modelId)
  modelDropdownOpen.value = false
}

onMounted(() => {
  if (store.agents.length === 0) {
    store.loadAgents()
  }
  if (store.models.length === 0) {
    store.loadModels()
  }
})
</script>

<template>
  <div class="flex gap-3 items-center">
    <div class="relative">
      <button
        @click="agentDropdownOpen = !agentDropdownOpen"
        class="flex gap-2 items-center px-3 py-2 rounded-lg bg-dark-700 border border-dark-500 transition-colors hover:border-dark-400"
      >
        <Bot :size="16" class="text-claw-400" />
        <span class="text-sm text-white">{{ selectedAgent?.name || '选择智能体' }}</span>
        <ChevronDown
          :size="14"
          :class="['text-gray-500 transition-transform', agentDropdownOpen && 'rotate-180']"
        />
      </button>

      <Transition name="dropdown">
        <div
          v-if="agentDropdownOpen"
          class="absolute top-full left-0 z-20 mt-1 w-48 overflow-hidden rounded-lg border bg-dark-700 border-dark-500 shadow-lg"
        >
          <div class="py-1 max-h-64 overflow-y-auto">
            <button
              v-for="agent in store.agents"
              :key="agent.id"
              @click="handleSelectAgent(agent.id)"
              :class="clsx(
                'flex gap-2 items-center w-full px-3 py-2 text-left transition-colors',
                store.selectedAgentId === agent.id
                  ? 'bg-claw-500/20 text-white'
                  : 'text-gray-300 hover:bg-dark-600'
              )"
            >
              <span class="flex-shrink-0">{{ agent.avatar || '🤖' }}</span>
              <div class="flex-1 min-w-0">
                <p class="text-sm truncate">{{ agent.name }}</p>
                <p v-if="agent.description" class="text-xs text-gray-500 truncate">
                  {{ agent.description }}
                </p>
              </div>
              <Check
                v-if="store.selectedAgentId === agent.id"
                :size="14"
                class="flex-shrink-0 text-claw-400"
              />
            </button>
          </div>
        </div>
      </Transition>
    </div>

    <div class="relative">
      <button
        @click="modelDropdownOpen = !modelDropdownOpen"
        class="flex gap-2 items-center px-3 py-2 rounded-lg bg-dark-700 border border-dark-500 transition-colors hover:border-dark-400"
      >
        <Cpu :size="16" class="text-accent-purple" />
        <span class="text-sm text-white">{{ selectedModel?.name || '选择模型' }}</span>
        <ChevronDown
          :size="14"
          :class="['text-gray-500 transition-transform', modelDropdownOpen && 'rotate-180']"
        />
      </button>

      <Transition name="dropdown">
        <div
          v-if="modelDropdownOpen"
          class="absolute top-full left-0 z-20 mt-1 w-56 overflow-hidden rounded-lg border bg-dark-700 border-dark-500 shadow-lg"
        >
          <div class="py-1 max-h-64 overflow-y-auto">
            <button
              v-for="model in store.models"
              :key="model.id"
              @click="handleSelectModel(model.id)"
              :class="clsx(
                'flex gap-2 items-center w-full px-3 py-2 text-left transition-colors',
                store.selectedModelId === model.id
                  ? 'bg-claw-500/20 text-white'
                  : 'text-gray-300 hover:bg-dark-600'
              )"
            >
              <div class="flex-1 min-w-0">
                <p class="text-sm truncate">{{ model.name }}</p>
                <p v-if="model.provider" class="text-xs text-gray-500 truncate">
                  {{ model.provider }}
                </p>
              </div>
              <Check
                v-if="store.selectedModelId === model.id"
                :size="14"
                class="flex-shrink-0 text-claw-400"
              />
            </button>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
