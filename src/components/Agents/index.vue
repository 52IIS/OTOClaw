<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  Loader2,
  Plus,
  Star,
  Bot,
  Pencil,
  Trash2,
  FolderOpen,
  Cpu,
  Sparkles,
  Link,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { api, isTauri, type AgentInfo, type AgentsListResult, type AIConfigOverview } from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'
import AgentDialog from './AgentDialog.vue'
import AgentChannelBindingDialog from './AgentChannelBindingDialog.vue'

const { alert, confirm } = useDialog()

const loading = ref(true)
const agentsResult = ref<AgentsListResult | null>(null)
const aiConfig = ref<AIConfigOverview | null>(null)
const error = ref<string | null>(null)

const showAddDialog = ref(false)
const editingAgent = ref<AgentInfo | null>(null)
const showChannelBindingDialog = ref(false)
const bindingAgent = ref<AgentInfo | null>(null)

const agents = computed(() => agentsResult.value?.agents || [])
const defaultId = computed(() => agentsResult.value?.defaultId)

const loadData = async () => {
  if (!isTauri()) {
    loading.value = false
    return
  }

  loading.value = true
  error.value = null

  try {
    const [agentsData, config] = await Promise.all([
      api.getAgentsList(),
      api.getAIConfig(),
    ])
    agentsResult.value = agentsData
    aiConfig.value = config
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadData()
})

const handleAddAgent = () => {
  editingAgent.value = null
  showAddDialog.value = true
}

const handleEditAgent = (agent: AgentInfo) => {
  editingAgent.value = agent
  showAddDialog.value = true
}

const handleCloseDialog = () => {
  showAddDialog.value = false
  editingAgent.value = null
}

const handleDialogSave = () => {
  loadData()
  handleCloseDialog()
}

const handleOpenChannelBinding = (agent: AgentInfo) => {
  bindingAgent.value = agent
  showChannelBindingDialog.value = true
}

const handleChannelBindingClose = () => {
  showChannelBindingDialog.value = false
  bindingAgent.value = null
}

const handleChannelBindingSave = () => {
  loadData()
  handleChannelBindingClose()
}

const handleSetDefault = async (agentId: string) => {
  try {
    await api.setDefaultAgent(agentId)
    await loadData()
  } catch (e) {
    await alert('设置默认智能体失败: ' + e, { variant: 'error', title: '操作失败' })
  }
}

const handleDeleteAgent = async (agent: AgentInfo) => {
  if (agent.isBuiltin) {
    await alert('内置智能体不能删除', { variant: 'warning', title: '无法删除' })
    return
  }

  const confirmed = await confirm(
    `确定要删除智能体 "${agent.name}" 吗？\n\n删除后将同时删除该智能体的工作区文件和会话记录。`,
    {
      title: '删除确认',
      variant: 'warning',
      confirmText: '删除',
      cancelText: '取消',
    }
  )

  if (!confirmed) return

  try {
    await api.deleteAgent(agent.id, true)
    await loadData()
  } catch (e) {
    await alert('删除智能体失败: ' + e, { variant: 'error', title: '删除失败' })
  }
}

const getAvatarDisplay = (agent: AgentInfo) => {
  if (agent.avatar) {
    return agent.avatar
  }
  return '🤖'
}

const getModelDisplay = (agent: AgentInfo) => {
  if (agent.model) {
    const parts = agent.model.split('/')
    return parts.length > 1 ? parts[1] : agent.model
  }
  return '未设置'
}
</script>

<template>
  <div class="overflow-y-auto pr-2 h-full scroll-container">
    <div class="space-y-6">
      <Transition name="fade">
        <div v-if="error" class="p-4 text-red-300 rounded-xl border bg-red-500/20 border-red-500/50">
          <p class="mb-1 font-medium">加载配置失败</p>
          <p class="text-sm text-red-400">{{ error }}</p>
          <button @click="loadData" class="mt-2 text-sm text-red-300 underline hover:text-white">重试</button>
        </div>
      </Transition>

      <div class="p-6 bg-gradient-to-br rounded-2xl border from-dark-700 to-dark-800 border-dark-500">
        <div class="flex justify-between items-start mb-4">
          <div>
            <h2 class="flex gap-2 items-center text-xl font-semibold text-white">
              <Bot :size="22" class="text-claw-400" />
              智能体管理
            </h2>
            <p class="mt-1 text-sm text-gray-500">创建和管理多个AI智能体，每个智能体可以有独立的配置和工作区</p>
          </div>
          <button @click="handleAddAgent" class="flex gap-2 items-center btn-primary">
            <Plus :size="16" />
            新建智能体
          </button>
        </div>

        <div class="grid grid-cols-3 gap-4">
          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-claw-500/20">
                <Bot :size="20" class="text-claw-400" />
              </div>
              <div>
                <p class="text-2xl font-bold text-white">{{ agents.length }}</p>
                <p class="text-sm text-gray-500">智能体总数</p>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-green-500/20">
                <Star :size="20" class="text-green-400" />
              </div>
              <div>
                <p class="text-lg font-medium text-white">{{ agents.find(a => a.id === defaultId)?.name || '未设置' }}</p>
                <p class="text-sm text-gray-500">默认智能体</p>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-blue-500/20">
                <Cpu :size="20" class="text-blue-400" />
              </div>
              <div>
                <p class="text-lg font-medium text-white">{{ aiConfig?.available_models.length || 0 }}</p>
                <p class="text-sm text-gray-500">可用模型</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="space-y-4">
        <h3 class="flex gap-2 items-center text-lg font-medium text-white">
          <Sparkles :size="18" class="text-gray-500" />
          智能体列表
        </h3>

        <div v-if="loading" class="flex justify-center items-center py-12">
          <Loader2 :size="24" class="animate-spin text-claw-400" />
        </div>

        <div v-else-if="agents.length === 0" class="p-8 text-center rounded-xl border bg-dark-700 border-dark-500">
          <div class="flex justify-center items-center mx-auto mb-4 w-16 h-16 rounded-full bg-dark-600">
            <Bot :size="24" class="text-gray-500" />
          </div>
          <p class="mb-4 text-gray-400">还没有创建任何智能体</p>
          <button @click="handleAddAgent" class="btn-primary">创建第一个智能体</button>
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="agent in agents"
            :key="agent.id"
            :class="clsx(
              'rounded-xl border transition-all',
              agent.isDefault
                ? 'bg-claw-500/5 border-claw-500/30 hover:border-claw-500/50'
                : 'bg-dark-700 border-dark-500 hover:border-dark-400'
            )"
          >
            <div class="p-4">
              <div class="flex gap-4 items-start">
                <div class="flex justify-center items-center w-12 h-12 text-2xl rounded-xl bg-dark-600">
                  {{ getAvatarDisplay(agent) }}
                </div>

                <div class="flex-1 min-w-0">
                  <div class="flex gap-2 items-center mb-1">
                    <h4 class="font-medium text-white">{{ agent.name }}</h4>
                    <span
                      v-if="agent.isBuiltin"
                      class="px-2 py-0.5 text-xs text-blue-400 rounded bg-blue-500/20"
                    >
                      内置
                    </span>
                    <span
                      v-if="agent.isDefault"
                      class="px-2 py-0.5 text-xs text-claw-400 rounded bg-claw-500/20"
                    >
                      <Star :size="10" class="inline -mt-0.5" /> 默认
                    </span>
                  </div>

                  <p class="mb-2 text-sm text-gray-500 line-clamp-1">
                    {{ agent.description || '暂无描述' }}
                  </p>

                  <div class="flex flex-wrap gap-3 text-xs text-gray-500">
                    <span class="flex gap-1 items-center">
                      <Cpu :size="12" />
                      {{ getModelDisplay(agent) }}
                    </span>
                    <span v-if="agent.workspace" class="flex gap-1 items-center">
                      <FolderOpen :size="12" />
                      {{ agent.workspace }}
                    </span>
                    <span v-if="agent.skills.length > 0" class="flex gap-1 items-center">
                      <Sparkles :size="12" />
                      {{ agent.skills.length }} 技能
                    </span>
                  </div>
                </div>

                <div class="flex gap-2 items-center">
                  <button
                    @click="handleOpenChannelBinding(agent)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-dark-600 hover:text-claw-400"
                  >
                    <Link :size="14" />
                    关联渠道
                  </button>

                  <button
                    v-if="!agent.isDefault"
                    @click="handleSetDefault(agent.id)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-dark-600 hover:text-claw-400"
                  >
                    <Star :size="14" />
                    设为默认
                  </button>

                  <button
                    @click="handleEditAgent(agent)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-dark-600 hover:text-claw-400"
                  >
                    <Pencil :size="14" />
                    编辑
                  </button>

                  <button
                    v-if="!agent.isBuiltin"
                    @click="handleDeleteAgent(agent)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-red-500/10 hover:text-red-400"
                  >
                    <Trash2 :size="14" />
                    删除
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="p-4 rounded-xl border bg-dark-700/50 border-dark-500">
        <h4 class="mb-2 text-sm font-medium text-gray-400">智能体说明</h4>
        <ul class="space-y-1 text-sm text-gray-500">
          <li>• 每个智能体可以有独立的工作区目录、模型配置和身份设定</li>
          <li>• 内置智能体（如"默认助手"）是系统预置的，不可删除</li>
          <li>• 默认智能体是系统使用的默认智能体，可在设置中修改</li>
          <li>• 通过"关联渠道"功能可以将渠道绑定到特定智能体</li>
          <li>• 智能体的身份信息保存在工作区的 IDENTITY.md 文件中</li>
          <li>• 删除智能体会同时删除其工作区文件和会话记录</li>
        </ul>
      </div>
    </div>

    <Teleport to="body">
      <Transition name="fade">
        <AgentDialog
          v-if="showAddDialog"
          :editing-agent="editingAgent"
          :available-models="aiConfig?.available_models || []"
          @close="handleCloseDialog"
          @save="handleDialogSave"
        />
      </Transition>
    </Teleport>

    <Teleport to="body">
      <Transition name="fade">
        <AgentChannelBindingDialog
          v-if="showChannelBindingDialog && bindingAgent"
          :agent="bindingAgent"
          @close="handleChannelBindingClose"
          @save="handleChannelBindingSave"
        />
      </Transition>
    </Teleport>
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

.line-clamp-1 {
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
