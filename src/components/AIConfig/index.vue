<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  Loader2,
  Plus,
  Star,
  Server,
  Sparkles,
  Zap,
  CheckCircle,
  XCircle,
  Cpu,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { aiLogger } from '../../lib/logger'

interface OfficialProvider {
  id: string
  name: string
  icon: string
  default_base_url: string | null
  api_type: string
  suggested_models: SuggestedModel[]
  requires_api_key: boolean
  docs_url: string | null
}

interface SuggestedModel {
  id: string
  name: string
  description: string | null
  context_window: number | null
  max_tokens: number | null
  recommended: boolean
}

interface ConfiguredProvider {
  name: string
  base_url: string
  api_key_masked: string | null
  has_api_key: boolean
  models: ConfiguredModel[]
}

interface ConfiguredModel {
  full_id: string
  id: string
  name: string
  api_type: string | null
  context_window: number | null
  max_tokens: number | null
  is_primary: boolean
}

interface AIConfigOverview {
  primary_model: string | null
  configured_providers: ConfiguredProvider[]
  available_models: string[]
}

interface AITestResult {
  success: boolean
  provider: string
  model: string
  response: string | null
  error: string | null
  latency_ms: number | null
}

const loading = ref(true)
const officialProviders = ref<OfficialProvider[]>([])
const aiConfig = ref<AIConfigOverview | null>(null)
const error = ref<string | null>(null)
const testing = ref(false)
const testResult = ref<AITestResult | null>(null)

const expandedProviders = ref<Set<string>>(new Set())

const loadData = async () => {
  aiLogger.info('AIConfig 组件加载数据...')
  error.value = null
  
  try {
    const [officials, config] = await Promise.all([
      invoke<OfficialProvider[]>('get_official_providers'),
      invoke<AIConfigOverview>('get_ai_config'),
    ])
    officialProviders.value = officials
    aiConfig.value = config
    config.configured_providers.forEach(p => expandedProviders.value.add(p.name))
  } catch (e) {
    aiLogger.error('加载 AI 配置失败', e)
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadData()
})

const runAITest = async () => {
  aiLogger.action('测试 AI 连接')
  testing.value = true
  testResult.value = null
  try {
    const result = await invoke<AITestResult>('test_ai_connection')
    testResult.value = result
  } catch (e) {
    testResult.value = {
      success: false,
      provider: 'unknown',
      model: 'unknown',
      response: null,
      error: String(e),
      latency_ms: null,
    }
  } finally {
    testing.value = false
  }
}

const handleSetPrimary = async (modelId: string) => {
  try {
    await invoke('set_primary_model', { modelId })
    loadData()
  } catch (e) {
    alert('设置失败: ' + e)
  }
}

const toggleProviderExpand = (name: string) => {
  if (expandedProviders.value.has(name)) {
    expandedProviders.value.delete(name)
  } else {
    expandedProviders.value.add(name)
  }
}

const getProviderIcon = (provider: ConfiguredProvider) => {
  const official = officialProviders.value.find(p => 
    provider.name.includes(p.id) || p.id === provider.name
  )
  return official?.icon || '🔌'
}

const handleAddProvider = () => {
  alert('添加 Provider 功能将在后续版本中完善')
}

const handleEditProvider = (provider: ConfiguredProvider) => {
  alert(`编辑 Provider: ${provider.name} 功能将在后续版本中完善`)
}

const handleDeleteProvider = async (providerName: string) => {
  if (!confirm(`确定要删除 Provider "${providerName}" 吗？`)) return
  try {
    await invoke('delete_provider', { providerName })
    loadData()
  } catch (e) {
    alert('删除失败: ' + e)
  }
}
</script>

<template>
  <div class="h-full overflow-y-auto scroll-container pr-2">
    <div class="max-w-4xl space-y-6">
      <Transition name="fade">
        <div v-if="error" class="bg-red-500/20 border border-red-500/50 rounded-xl p-4 text-red-300">
          <p class="font-medium mb-1">加载配置失败</p>
          <p class="text-sm text-red-400">{{ error }}</p>
          <button @click="loadData" class="mt-2 text-sm text-red-300 hover:text-white underline">重试</button>
        </div>
      </Transition>

      <div class="bg-gradient-to-br from-dark-700 to-dark-800 rounded-2xl p-6 border border-dark-500">
        <div class="flex items-start justify-between mb-4">
          <div>
            <h2 class="text-xl font-semibold text-white flex items-center gap-2">
              <Sparkles :size="22" class="text-claw-400" />
              AI 模型配置
            </h2>
            <p class="text-sm text-gray-500 mt-1">管理 OpenClaw 使用的 AI Provider 和模型</p>
          </div>
          <button @click="handleAddProvider" class="btn-primary flex items-center gap-2">
            <Plus :size="16" />
            添加 Provider
          </button>
        </div>

        <div class="bg-dark-600/50 rounded-xl p-4 flex items-center gap-4">
          <div class="w-12 h-12 rounded-xl bg-claw-500/20 flex items-center justify-center">
            <Star :size="24" class="text-claw-400" />
          </div>
          <div class="flex-1">
            <p class="text-sm text-gray-400">当前主模型</p>
            <p v-if="aiConfig?.primary_model" class="text-lg font-medium text-white">{{ aiConfig.primary_model }}</p>
            <p v-else class="text-lg text-gray-500">未设置</p>
          </div>
          <div class="text-right mr-4">
            <p class="text-sm text-gray-500">{{ aiConfig?.configured_providers.length || 0 }} 个 Provider</p>
            <p class="text-sm text-gray-500">{{ aiConfig?.available_models.length || 0 }} 个可用模型</p>
          </div>
          <button @click="runAITest" :disabled="testing || !aiConfig?.primary_model" class="btn-secondary flex items-center gap-2">
            <Loader2 v-if="testing" :size="16" class="animate-spin" />
            <Zap v-else :size="16" />
            测试连接
          </button>
        </div>

        <Transition name="fade">
          <div v-if="testResult" :class="['mt-4 p-4 rounded-xl', testResult.success ? 'bg-green-500/10 border border-green-500/30' : 'bg-red-500/10 border border-red-500/30']">
            <div class="flex items-center gap-3 mb-2">
              <CheckCircle v-if="testResult.success" :size="20" class="text-green-400" />
              <XCircle v-else :size="20" class="text-red-400" />
              <div class="flex-1">
                <p :class="['font-medium', testResult.success ? 'text-green-400' : 'text-red-400']">
                  {{ testResult.success ? '连接成功' : '连接失败' }}
                </p>
                <p v-if="testResult.latency_ms" class="text-xs text-gray-400">响应时间: {{ testResult.latency_ms }}ms</p>
              </div>
              <button @click="testResult = null" class="text-gray-500 hover:text-white text-sm">关闭</button>
            </div>
            <div v-if="testResult.response" class="mt-2 p-3 bg-dark-700 rounded-lg">
              <p class="text-xs text-gray-400 mb-1">AI 响应:</p>
              <p class="text-sm text-white whitespace-pre-wrap">{{ testResult.response }}</p>
            </div>
            <div v-if="testResult.error" class="mt-2 p-3 bg-red-500/10 rounded-lg">
              <p class="text-xs text-red-400 mb-1">错误信息:</p>
              <p class="text-sm text-red-300 whitespace-pre-wrap">{{ testResult.error }}</p>
            </div>
          </div>
        </Transition>
      </div>

      <div class="space-y-4">
        <h3 class="text-lg font-medium text-white flex items-center gap-2">
          <Server :size="18" class="text-gray-500" />
          已配置的 Provider
        </h3>

        <div v-if="aiConfig?.configured_providers.length === 0" class="bg-dark-700 rounded-xl border border-dark-500 p-8 text-center">
          <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-dark-600 flex items-center justify-center">
            <Plus :size="24" class="text-gray-500" />
          </div>
          <p class="text-gray-400 mb-4">还没有配置任何 AI Provider</p>
          <button @click="handleAddProvider" class="btn-primary">添加第一个 Provider</button>
        </div>

        <div v-else class="space-y-3">
          <div v-for="provider in aiConfig?.configured_providers" :key="provider.name" class="bg-dark-700 rounded-xl border border-dark-500 overflow-hidden">
            <div class="flex items-center gap-3 p-4 cursor-pointer hover:bg-dark-600/50 transition-colors" @click="toggleProviderExpand(provider.name)">
              <span class="text-xl">{{ getProviderIcon(provider) }}</span>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <h3 class="font-medium text-white">{{ provider.name }}</h3>
                  <span v-if="provider.has_api_key" class="px-1.5 py-0.5 bg-green-500/20 text-green-400 text-xs rounded">已配置</span>
                </div>
                <p class="text-xs text-gray-500 truncate">{{ provider.base_url }}</p>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-sm text-gray-500">{{ provider.models.length }} 模型</span>
              </div>
            </div>

            <Transition name="expand">
              <div v-show="expandedProviders.has(provider.name)" class="border-t border-dark-600">
                <div class="p-4 space-y-3">
                  <div v-if="provider.api_key_masked" class="flex items-center gap-2 text-sm">
                    <span class="text-gray-500">API Key:</span>
                    <code class="px-2 py-0.5 bg-dark-600 rounded text-gray-400">{{ provider.api_key_masked }}</code>
                  </div>

                  <div class="space-y-2">
                    <div v-for="model in provider.models" :key="model.full_id" :class="clsx('flex items-center justify-between p-3 rounded-lg border transition-all', model.is_primary ? 'bg-claw-500/10 border-claw-500/50' : 'bg-dark-600 border-dark-500')">
                      <div class="flex items-center gap-3">
                        <Cpu :size="16" :class="model.is_primary ? 'text-claw-400' : 'text-gray-500'" />
                        <div>
                          <p :class="['text-sm font-medium', model.is_primary ? 'text-white' : 'text-gray-300']">
                            {{ model.name }}
                            <span v-if="model.is_primary" class="ml-2 text-xs text-claw-400">
                              <Star :size="12" class="inline -mt-0.5" /> 主模型
                            </span>
                          </p>
                          <p class="text-xs text-gray-500">{{ model.full_id }}</p>
                        </div>
                      </div>
                      <button v-if="!model.is_primary" @click.stop="handleSetPrimary(model.full_id)" class="text-xs text-gray-500 hover:text-claw-400 transition-colors">设为主模型</button>
                    </div>
                  </div>

                  <div class="flex justify-end gap-4 pt-2">
                    <button @click.stop="handleEditProvider(provider)" class="flex items-center gap-1 text-sm text-claw-400 hover:text-claw-300 transition-colors">编辑</button>
                    <button @click.stop="handleDeleteProvider(provider.name)" class="flex items-center gap-1 text-sm text-red-400 hover:text-red-300 transition-colors">删除</button>
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </div>
      </div>

      <div v-if="aiConfig && aiConfig.available_models.length > 0" class="space-y-4">
        <h3 class="text-lg font-medium text-white flex items-center gap-2">
          <Cpu :size="18" class="text-gray-500" />
          可用模型列表
          <span class="text-sm font-normal text-gray-500">({{ aiConfig.available_models.length }} 个)</span>
        </h3>
        <div class="bg-dark-700 rounded-xl border border-dark-500 p-4">
          <div class="flex flex-wrap gap-2">
            <span v-for="modelId in aiConfig.available_models" :key="modelId" :class="['inline-flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm', modelId === aiConfig.primary_model ? 'bg-claw-500/20 text-claw-300 border border-claw-500/30' : 'bg-dark-600 text-gray-300']">
              <Star v-if="modelId === aiConfig.primary_model" :size="12" />
              {{ modelId }}
            </span>
          </div>
        </div>
      </div>

      <div class="bg-dark-700/50 rounded-xl p-4 border border-dark-500">
        <h4 class="text-sm font-medium text-gray-400 mb-2">配置说明</h4>
        <ul class="text-sm text-gray-500 space-y-1">
          <li>• Provider 配置保存在 <code class="text-claw-400">~/.openclaw/openclaw.json</code></li>
          <li>• 支持官方 Provider（Anthropic、OpenAI、Kimi 等）和自定义 OpenAI/Anthropic 兼容 API</li>
          <li>• 主模型用于 Agent 的默认推理，可随时切换</li>
          <li>• 修改配置后需要重启服务生效</li>
        </ul>
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

.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}
</style>
