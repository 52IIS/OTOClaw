<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  Loader2,
  Plus,
  Check,
  Eye,
  EyeOff,
  Settings2,
  ExternalLink,
  ChevronRight,
  XCircle,
} from 'lucide-vue-next'
import { aiLogger } from '../../lib/logger'
import type {
  OfficialProvider,
  ConfiguredProvider,
  ModelConfig,
} from './types'

const props = defineProps<{
  officialProviders: OfficialProvider[]
  editingProvider?: ConfiguredProvider | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'save'): void
}>()

const isEditing = computed(() => !!props.editingProvider)

const getInitialOfficial = (): OfficialProvider | null => {
  if (props.editingProvider) {
    return props.officialProviders.find(p => 
      props.editingProvider!.name.includes(p.id) || p.id === props.editingProvider!.name
    ) || null
  }
  return null
}

const getInitialApiType = (): string => {
  if (props.editingProvider && props.editingProvider.models.length > 0) {
    return props.editingProvider.models[0].api_type || 'openai-completions'
  }
  return 'openai-completions'
}

const getInitialModels = (): string[] => {
  if (props.editingProvider) {
    return props.editingProvider.models.map(m => m.id)
  }
  return []
}

const step = ref<'select' | 'configure'>(isEditing.value ? 'configure' : 'select')

const selectedOfficial = ref<OfficialProvider | null>(getInitialOfficial())

const providerName = ref(props.editingProvider?.name || '')
const baseUrl = ref(props.editingProvider?.base_url || '')
const apiKey = ref('')
const apiType = ref(getInitialApiType())
const showApiKey = ref(false)
const selectedModels = ref<string[]>(getInitialModels())
const customModelId = ref('')
const saving = ref(false)
const formError = ref<string | null>(null)
const showCustomUrlWarning = ref(false)

const isCustomUrlWithOfficialName = computed(() => {
  const official = props.officialProviders.find(p => p.id === providerName.value)
  if (official && official.default_base_url && baseUrl.value !== official.default_base_url) {
    return true
  }
  return false
})

const suggestedName = computed(() => {
  if (isCustomUrlWithOfficialName.value && selectedOfficial.value) {
    return `${selectedOfficial.value.id}-custom`
  }
  return null
})

const handleSelectOfficial = (provider: OfficialProvider) => {
  selectedOfficial.value = provider
  providerName.value = provider.id
  baseUrl.value = provider.default_base_url || ''
  apiType.value = provider.api_type
  const recommended = provider.suggested_models.filter(m => m.recommended).map(m => m.id)
  selectedModels.value = recommended.length > 0 ? recommended : [provider.suggested_models[0]?.id].filter(Boolean) as string[]
  formError.value = null
  showCustomUrlWarning.value = false
  step.value = 'configure'
}

const handleSelectCustom = () => {
  selectedOfficial.value = null
  providerName.value = ''
  baseUrl.value = ''
  apiType.value = 'openai-completions'
  selectedModels.value = []
  formError.value = null
  showCustomUrlWarning.value = false
  step.value = 'configure'
}

const toggleModel = (modelId: string) => {
  formError.value = null
  if (selectedModels.value.includes(modelId)) {
    selectedModels.value = selectedModels.value.filter(id => id !== modelId)
  } else {
    selectedModels.value.push(modelId)
  }
}

const addCustomModel = () => {
  if (customModelId.value && !selectedModels.value.includes(customModelId.value)) {
    formError.value = null
    selectedModels.value.push(customModelId.value)
    customModelId.value = ''
  }
}

const handleApplySuggestedName = () => {
  if (suggestedName.value) {
    providerName.value = suggestedName.value
  }
}

const handleSave = async (forceOverride: boolean = false) => {
  formError.value = null
  
  if (!providerName.value || !baseUrl.value || selectedModels.value.length === 0) {
    formError.value = '请填写完整的 Provider 信息和至少选择一个模型'
    return
  }

  if (isCustomUrlWithOfficialName.value && !forceOverride) {
    showCustomUrlWarning.value = true
    return
  }
  
  saving.value = true
  showCustomUrlWarning.value = false
  try {
    const models: ModelConfig[] = selectedModels.value.map(modelId => {
      const suggested = selectedOfficial.value?.suggested_models.find(m => m.id === modelId)
      const existingModel = props.editingProvider?.models.find(m => m.id === modelId)
      return {
        id: modelId,
        name: suggested?.name || existingModel?.name || modelId,
        api: apiType.value,
        input: ['text', 'image'],
        context_window: suggested?.context_window || existingModel?.context_window || 200000,
        max_tokens: suggested?.max_tokens || existingModel?.max_tokens || 8192,
        reasoning: false,
        cost: null,
      }
    })

    await invoke('save_provider', {
      providerName: providerName.value,
      baseUrl: baseUrl.value,
      apiKey: apiKey.value || null,
      apiType: apiType.value,
      models,
    })

    aiLogger.info(`✓ Provider ${providerName.value} 已${isEditing.value ? '更新' : '保存'}`)
    emit('save')
    emit('close')
  } catch (e) {
    aiLogger.error('保存 Provider 失败', e)
    formError.value = '保存失败: ' + String(e)
  } finally {
    saving.value = false
  }
}

const handleClose = () => {
  emit('close')
}
</script>

<template>
  <div class="flex fixed inset-0 z-50 justify-center items-center p-4 backdrop-blur-sm bg-black/60" @click="handleClose">
    <div class="bg-dark-800 rounded-2xl border border-dark-600 w-full max-w-2xl max-h-[85vh] overflow-hidden" @click.stop>
      <div class="flex justify-between items-center px-6 py-4 border-b border-dark-600">
        <h2 class="flex gap-2 items-center text-lg font-semibold text-white">
          <component :is="isEditing ? Settings2 : Plus" :size="20" class="text-claw-400" />
          {{ isEditing ? '编辑 Provider: ' + editingProvider?.name : (step === 'select' ? '添加 AI Provider' : '配置 ' + (selectedOfficial?.name || '自定义 Provider')) }}
        </h2>
        <button @click="handleClose" class="text-gray-500 hover:text-white">✕</button>
      </div>

      <div class="p-6 overflow-y-auto max-h-[calc(85vh-140px)]">
        <Transition name="slide-fade" mode="out-in">
          <div v-if="step === 'select'" key="select" class="space-y-4">
            <div class="space-y-3">
              <h3 class="text-sm font-medium text-gray-400">官方 Provider</h3>
              <div class="grid grid-cols-2 gap-3">
                <button
                  v-for="provider in officialProviders"
                  :key="provider.id"
                  @click="handleSelectOfficial(provider)"
                  class="flex gap-3 items-center p-4 text-left rounded-xl border transition-all bg-dark-700 border-dark-500 hover:border-claw-500/50 hover:bg-dark-600 group"
                >
                  <span class="text-2xl">{{ provider.icon }}</span>
                  <div class="flex-1 min-w-0">
                    <p class="font-medium text-white truncate">{{ provider.name }}</p>
                    <p class="text-xs text-gray-500 truncate">{{ provider.suggested_models.length }} 个模型</p>
                  </div>
                  <ChevronRight :size="16" class="text-gray-500 transition-colors group-hover:text-claw-400" />
                </button>
              </div>
            </div>

            <div class="pt-4 border-t border-dark-600">
              <button
                @click="handleSelectCustom"
                class="flex gap-2 justify-center items-center p-4 w-full text-gray-400 rounded-xl border-2 border-dashed transition-all border-dark-500 hover:border-claw-500/50 hover:text-white"
              >
                <Settings2 :size="18" />
                <span>自定义 Provider (兼容 OpenAI/Anthropic API)</span>
              </button>
            </div>
          </div>

          <div v-else key="configure" class="space-y-5">
            <div>
              <label class="block mb-2 text-sm text-gray-400">
                Provider 名称
                <span class="ml-2 text-xs text-gray-600">(用于配置标识，如 anthropic-custom)</span>
              </label>
              <input
                type="text"
                v-model="providerName"
                placeholder="如: anthropic-custom, my-openai"
                :class="['input-base', isCustomUrlWithOfficialName && 'border-yellow-500/50']"
                :disabled="isEditing"
                @input="formError = null"
              />
              <p v-if="isEditing" class="mt-1 text-xs text-gray-500">
                Provider 名称不可修改，如需更改请删除后重新创建
              </p>
              <div v-if="isCustomUrlWithOfficialName && !isEditing" class="p-2 mt-2 rounded-lg border bg-yellow-500/10 border-yellow-500/30">
                <p class="text-xs text-yellow-400">
                  ⚠️ 您使用的是官方 Provider 名称，但修改了 API 地址。建议使用不同的名称以避免配置冲突。
                </p>
                <button
                  type="button"
                  @click="handleApplySuggestedName"
                  class="mt-1 text-xs text-yellow-300 underline hover:text-yellow-200"
                >
                  使用建议名称: {{ suggestedName }}
                </button>
              </div>
            </div>

            <div>
              <label class="block mb-2 text-sm text-gray-400">API 地址</label>
              <input
                type="text"
                v-model="baseUrl"
                placeholder="https://api.example.com/v1"
                class="input-base"
                @input="formError = null"
              />
            </div>

            <div>
              <label class="block mb-2 text-sm text-gray-400">
                API Key
                <span v-if="!selectedOfficial?.requires_api_key" class="ml-2 text-xs text-gray-600">(可选)</span>
              </label>
              <div v-if="isEditing && editingProvider?.has_api_key" class="flex gap-2 items-center mb-2 text-sm">
                <span class="text-gray-500">当前:</span>
                <code class="px-2 py-0.5 text-gray-400 rounded bg-dark-600">
                  {{ editingProvider.api_key_masked }}
                </code>
                <span class="text-xs text-green-400">✓ 已配置</span>
              </div>
              <div class="relative">
                <input
                  :type="showApiKey ? 'text' : 'password'"
                  v-model="apiKey"
                  :placeholder="isEditing && editingProvider?.has_api_key ? '留空保持原有 API Key 不变，或输入新的 Key' : 'sk-...'"
                  class="pr-10 input-base"
                />
                <button
                  type="button"
                  @click="showApiKey = !showApiKey"
                  class="absolute right-3 top-1/2 text-gray-500 -translate-y-1/2 hover:text-white"
                >
                  <component :is="showApiKey ? EyeOff : Eye" :size="18" />
                </button>
              </div>
              <p v-if="isEditing && editingProvider?.has_api_key" class="mt-1 text-xs text-gray-500">
                💡 如果不需要更改 API Key，请保持为空
              </p>
            </div>

            <div>
              <label class="block mb-2 text-sm text-gray-400">API 类型</label>
              <select v-model="apiType" class="input-base">
                <option value="openai-completions">OpenAI 兼容 (openai-completions)</option>
                <option value="anthropic-messages">Anthropic 兼容 (anthropic-messages)</option>
              </select>
            </div>

            <div>
              <label class="block mb-2 text-sm text-gray-400">
                选择模型
                <span class="ml-2 text-xs text-gray-600">(已选 {{ selectedModels.length }} 个)</span>
              </label>
              
              <div v-if="selectedOfficial" class="mb-3 space-y-2">
                <button
                  v-for="model in selectedOfficial.suggested_models"
                  :key="model.id"
                  @click="toggleModel(model.id)"
                  :class="[
                    'w-full flex items-center justify-between p-3 rounded-lg border transition-all text-left',
                    selectedModels.includes(model.id)
                      ? 'bg-claw-500/20 border-claw-500'
                      : 'bg-dark-700 border-dark-500 hover:border-dark-400'
                  ]"
                >
                  <div>
                    <p :class="['text-sm font-medium', selectedModels.includes(model.id) ? 'text-white' : 'text-gray-300']">
                      {{ model.name }}
                      <span v-if="model.recommended" class="ml-2 text-xs text-claw-400">推荐</span>
                    </p>
                    <p v-if="model.description" class="mt-0.5 text-xs text-gray-500">{{ model.description }}</p>
                  </div>
                  <Check v-if="selectedModels.includes(model.id)" :size="16" class="text-claw-400" />
                </button>
              </div>

              <div class="flex gap-2">
                <input
                  type="text"
                  v-model="customModelId"
                  placeholder="输入自定义模型 ID"
                  class="flex-1 input-base"
                  @keydown.enter="addCustomModel"
                />
                <button @click="addCustomModel" :disabled="!customModelId" class="px-4 btn-secondary">
                  <Plus :size="16" />
                </button>
              </div>

              <div v-if="selectedModels.filter(id => !selectedOfficial?.suggested_models.find(m => m.id === id)).length > 0" class="flex flex-wrap gap-2 mt-3">
                <span
                  v-for="modelId in selectedModels.filter(id => !selectedOfficial?.suggested_models.find(m => m.id === id))"
                  :key="modelId"
                  class="inline-flex gap-1 items-center px-2 py-1 text-sm text-gray-300 rounded-lg bg-dark-600"
                >
                  {{ modelId }}
                  <button @click="toggleModel(modelId)" class="text-gray-500 hover:text-red-400">✕</button>
                </span>
              </div>
            </div>

            <a
              v-if="selectedOfficial?.docs_url"
              :href="selectedOfficial.docs_url"
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex gap-1 items-center text-sm text-claw-400 hover:text-claw-300"
            >
              <ExternalLink :size="14" />
              查看官方文档
            </a>

            <Transition name="fade">
              <div v-if="formError" class="p-3 rounded-lg border bg-red-500/10 border-red-500/30">
                <p class="flex gap-2 items-center text-sm text-red-400">
                  <XCircle :size="16" />
                  {{ formError }}
                </p>
              </div>
            </Transition>

            <Transition name="fade">
              <div v-if="showCustomUrlWarning" class="p-4 space-y-3 rounded-lg border bg-yellow-500/10 border-yellow-500/30">
                <p class="text-sm text-yellow-400">
                  ⚠️ 您使用的是官方 Provider 名称 "{{ providerName }}"，但修改了 API 地址。
                  这可能导致配置被 OpenClaw 内置设置覆盖。
                </p>
                <p class="text-sm text-yellow-300">
                  建议使用不同的名称，如 "{{ suggestedName }}"
                </p>
                <div class="flex gap-2 pt-2">
                  <button @click="handleApplySuggestedName" class="px-3 py-2 text-sm btn-secondary">
                    使用建议名称
                  </button>
                  <button @click="handleSave(true)" class="px-3 py-2 text-sm btn-primary">
                    仍然保存
                  </button>
                  <button @click="showCustomUrlWarning = false" class="px-3 text-sm text-gray-400 hover:text-white">
                    取消
                  </button>
                </div>
              </div>
            </Transition>
          </div>
        </Transition>
      </div>

      <div class="flex justify-between px-6 py-4 border-t border-dark-600">
        <button v-if="step === 'configure' && !isEditing" @click="step = 'select'" class="btn-secondary">
          返回
        </button>
        <div v-else class="flex-1" />
        <div class="flex gap-3">
          <button @click="handleClose" class="btn-secondary">取消</button>
          <button
            v-if="step === 'configure' && !showCustomUrlWarning"
            @click="handleSave()"
            :disabled="saving || !providerName || !baseUrl || selectedModels.length === 0"
            class="flex gap-2 items-center btn-primary"
          >
            <Loader2 v-if="saving" :size="16" class="animate-spin" />
            <Check v-else :size="16" />
            {{ isEditing ? '更新' : '保存' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.2s ease;
}

.slide-fade-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.slide-fade-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}
</style>
