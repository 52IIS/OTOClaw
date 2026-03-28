<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  X,
  Loader2,
  Bot,
  ChevronDown,
  ChevronRight,
  User,
  Heart,
  Shield,
  Users,
  Brain,
  Save,
  FileText,
  RefreshCw,
  Sparkles,
  AlertCircle,
  CheckCircle,
  Settings,
} from 'lucide-vue-next'
import clsx from 'clsx'
import {
  api,
  isTauri,
  type BuiltinAgentInfo,
  type WorkspaceFilesResult,
  type SkillInfo,
  type SkillsListResult,
} from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'

const emit = defineEmits<{
  close: []
  save: []
}>()

const { alert } = useDialog()

interface FileConfig {
  filename: string
  title: string
  icon: typeof User
  description: string
  color: string
}

const fileConfigs: FileConfig[] = [
  {
    filename: 'IDENTITY.md',
    title: '身份信息',
    icon: User,
    description: '定义智能体的基础身份信息，包括名称、角色、专长和目标',
    color: 'text-blue-400',
  },
  {
    filename: 'SOUL.md',
    title: '人格特质',
    icon: Heart,
    description: '设定智能体的人格特质、语气风格与行为边界',
    color: 'text-pink-400',
  },
  {
    filename: 'AGENTS.md',
    title: '能力范围',
    icon: Shield,
    description: '规范智能体的能力范围与安全准则',
    color: 'text-green-400',
  },
  {
    filename: 'USER.md',
    title: '用户信息',
    icon: Users,
    description: '记录用户的个人信息与偏好',
    color: 'text-yellow-400',
  },
  {
    filename: 'MEMORY.md',
    title: '交互记忆',
    icon: Brain,
    description: '存储智能体与用户的长期交互记忆',
    color: 'text-purple-400',
  },
]

const activeTab = ref<'description' | 'workspace' | 'skills'>('description')
const loading = ref(false)
const saving = ref<string | null>(null)
const workspaceFiles = ref<Map<string, string>>(new Map())
const expandedFiles = ref<Set<string>>(new Set(['IDENTITY.md']))
const workspaceDir = ref('')
const skillsLoading = ref(false)
const availableSkills = ref<SkillInfo[]>([])
const agentSkills = ref<string[]>([])

const descriptionExpanded = ref(true)
const builtinAgents = ref<BuiltinAgentInfo[]>([])
const currentAgent = ref<BuiltinAgentInfo | null>(null)

const hasAgent = computed(() => currentAgent.value !== null)

const loadBuiltinAgents = async () => {
  if (!isTauri()) return

  loading.value = true
  try {
    const result = await api.getBuiltinAgents()
    builtinAgents.value = result.agents
    if (result.agents.length > 0) {
      currentAgent.value = result.agents[0]
      await loadWorkspaceFiles(result.agents[0].id)
      await loadAgentSkills(result.agents[0].id)
    }
  } catch (e) {
    console.error('加载内置智能体失败:', e)
  } finally {
    loading.value = false
  }
}

const loadWorkspaceFiles = async (agentId: string) => {
  if (!isTauri() || !agentId) return

  loading.value = true
  try {
    const result: WorkspaceFilesResult = await api.getBuiltinAgentWorkspaceFiles(agentId)
    workspaceDir.value = result.workspaceDir

    const newFiles = new Map<string, string>()
    for (const file of result.files) {
      newFiles.set(file.filename, file.content)
    }
    workspaceFiles.value = newFiles
  } catch (e) {
    console.error('加载工作区文件失败:', e)
  } finally {
    loading.value = false
  }
}

const loadSkills = async () => {
  if (!isTauri()) return

  skillsLoading.value = true
  try {
    const result: SkillsListResult = await api.getSkillsList()
    availableSkills.value = result.skills
  } catch (e) {
    console.error('加载技能列表失败:', e)
  } finally {
    skillsLoading.value = false
  }
}

const loadAgentSkills = async (agentId: string) => {
  if (!isTauri() || !agentId) return

  try {
    const skills = await api.getBuiltinAgentSkills(agentId)
    agentSkills.value = skills
  } catch (e) {
    console.error('加载智能体技能失败:', e)
  }
}

onMounted(async () => {
  await loadBuiltinAgents()
  await loadSkills()
})

const toggleFile = (filename: string) => {
  const newExpanded = new Set(expandedFiles.value)
  if (newExpanded.has(filename)) {
    newExpanded.delete(filename)
  } else {
    newExpanded.add(filename)
  }
  expandedFiles.value = newExpanded
}

const isExpanded = (filename: string) => expandedFiles.value.has(filename)

const getFileContent = (filename: string): string => {
  return workspaceFiles.value.get(filename) || ''
}

const setFileContent = (filename: string, content: string) => {
  workspaceFiles.value.set(filename, content)
}

const saveFile = async (filename: string) => {
  if (!isTauri() || !currentAgent.value?.id) return

  const content = getFileContent(filename)
  saving.value = filename

  try {
    await api.saveBuiltinAgentWorkspaceFile({
      agentId: currentAgent.value.id,
      filename,
      content,
    })
    emit('save')
  } catch (e) {
    await alert(`保存文件失败: ${e}`, { variant: 'error', title: '保存失败' })
  } finally {
    saving.value = null
  }
}

const handleAssignSkill = async (skillId: string) => {
  if (!currentAgent.value?.id) return

  try {
    await api.assignSkillToBuiltinAgent(skillId, currentAgent.value.id)
    await loadAgentSkills(currentAgent.value.id)
    await alert('技能分配成功', { variant: 'success', title: '操作成功' })
  } catch (e) {
    await alert(`分配技能失败: ${e}`, { variant: 'error', title: '操作失败' })
  }
}

const handleRemoveSkill = async (skillId: string) => {
  if (!currentAgent.value?.id) return

  try {
    await api.removeSkillFromBuiltinAgent(skillId, currentAgent.value.id)
    await loadAgentSkills(currentAgent.value.id)
    await alert('技能移除成功', { variant: 'success', title: '操作成功' })
  } catch (e) {
    await alert(`移除技能失败: ${e}`, { variant: 'error', title: '操作失败' })
  }
}

const isSkillAssigned = (skillId: string) => {
  return agentSkills.value.includes(skillId)
}

const handleClose = () => {
  emit('close')
}

const handleSelectAgent = async (agent: BuiltinAgentInfo) => {
  currentAgent.value = agent
  await loadWorkspaceFiles(agent.id)
  await loadAgentSkills(agent.id)
}
</script>

<template>
  <div class="flex fixed inset-0 z-50 justify-center items-center p-4 backdrop-blur-sm bg-black/60">
    <div
      class="w-full max-w-4xl max-h-[90vh] bg-dark-800 rounded-2xl border border-dark-500 shadow-2xl flex flex-col"
      @click.stop
    >
      <div class="flex justify-between items-center p-4 border-b border-dark-600 shrink-0">
        <div class="flex gap-2 items-center">
          <Bot :size="20" class="text-claw-400" />
          <h3 class="text-lg font-semibold text-white">管理内置智能体</h3>
        </div>
        <button
          @click="handleClose"
          class="p-1 text-gray-400 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
        >
          <X :size="20" />
        </button>
      </div>

      <div class="flex gap-1 p-2 border-b border-dark-600 shrink-0">
        <button
          @click="activeTab = 'description'"
          :class="clsx(
            'flex gap-2 items-center px-4 py-2 rounded-lg text-sm font-medium transition-all',
            activeTab === 'description'
              ? 'bg-claw-500/20 text-claw-400'
              : 'text-gray-400 hover:text-white hover:bg-dark-600'
          )"
        >
          <User :size="16" />
          描述信息
        </button>
        <button
          @click="activeTab = 'workspace'"
          :disabled="!hasAgent"
          :class="clsx(
            'flex gap-2 items-center px-4 py-2 rounded-lg text-sm font-medium transition-all',
            activeTab === 'workspace'
              ? 'bg-claw-500/20 text-claw-400'
              : 'text-gray-400 hover:text-white hover:bg-dark-600',
            !hasAgent && 'opacity-50 cursor-not-allowed'
          )"
        >
          <FileText :size="16" />
          工作区文件
        </button>
        <button
          @click="activeTab = 'skills'"
          :disabled="!hasAgent"
          :class="clsx(
            'flex gap-2 items-center px-4 py-2 rounded-lg text-sm font-medium transition-all',
            activeTab === 'skills'
              ? 'bg-claw-500/20 text-claw-400'
              : 'text-gray-400 hover:text-white hover:bg-dark-600',
            !hasAgent && 'opacity-50 cursor-not-allowed'
          )"
        >
          <Sparkles :size="16" />
          技能管理
        </button>
      </div>

      <div class="overflow-y-auto flex-1">
        <div v-show="activeTab === 'description'" class="p-4">
          <div class="space-y-4">
            <div v-if="loading" class="flex justify-center items-center py-8">
              <Loader2 :size="24" class="animate-spin text-claw-400" />
            </div>

            <div v-else-if="builtinAgents.length === 0" class="p-8 text-center text-gray-500 rounded-xl border bg-dark-700 border-dark-500">
              <Bot :size="48" class="mx-auto mb-4 opacity-30" />
              <p>暂无内置智能体</p>
            </div>

            <template v-else>
              <div class="mb-4">
                <label class="block mb-2 text-sm text-gray-400">选择内置智能体</label>
                <div class="flex gap-2">
                  <button
                    v-for="agent in builtinAgents"
                    :key="agent.id"
                    @click="handleSelectAgent(agent)"
                    :class="clsx(
                      'flex gap-2 items-center px-4 py-2 rounded-lg text-sm font-medium transition-all',
                      currentAgent?.id === agent.id
                        ? 'bg-claw-500/20 text-claw-400 border border-claw-500/50'
                        : 'bg-dark-700 text-gray-300 border border-dark-500 hover:border-dark-400'
                    )"
                  >
                    <span class="text-lg">{{ agent.avatar || '🤖' }}</span>
                    <span>{{ agent.name }}</span>
                  </button>
                </div>
              </div>

              <div v-if="currentAgent" class="p-4 rounded-xl border bg-dark-700 border-dark-500">
                <button
                  @click="descriptionExpanded = !descriptionExpanded"
                  class="flex justify-between items-center w-full text-left"
                >
                  <div class="flex gap-3 items-center">
                    <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-claw-500/20">
                      <Bot :size="20" class="text-claw-400" />
                    </div>
                    <div>
                      <h4 class="font-medium text-white">{{ currentAgent.name }}</h4>
                      <p class="text-sm text-gray-500">{{ currentAgent.model || '未设置模型' }}</p>
                    </div>
                  </div>
                  <ChevronDown
                    v-if="descriptionExpanded"
                    :size="20"
                    class="text-gray-400 transition-transform"
                  />
                  <ChevronRight
                    v-else
                    :size="20"
                    class="text-gray-400 transition-transform"
                  />
                </button>

                <Transition name="expand">
                  <div v-if="descriptionExpanded" class="pt-4 mt-4 border-t border-dark-500">
                    <p class="text-sm text-gray-300">
                      {{ currentAgent.description || '暂无描述' }}
                    </p>
                  </div>
                </Transition>
              </div>

              <div v-if="currentAgent" class="p-4 rounded-xl border bg-dark-700 border-dark-500">
                <div class="flex gap-2 items-center mb-3">
                  <Settings :size="16" class="text-gray-400" />
                  <h4 class="text-sm font-medium text-white">智能体信息</h4>
                </div>
                <div class="space-y-2 text-sm">
                  <div class="flex justify-between">
                    <span class="text-gray-500">ID</span>
                    <span class="font-mono text-white">{{ currentAgent.id }}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-gray-500">默认</span>
                    <span :class="currentAgent.isDefault ? 'text-green-400' : 'text-gray-400'">
                      {{ currentAgent.isDefault ? '是' : '否' }}
                    </span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-gray-500">工作区</span>
                    <span class="text-white font-mono text-xs max-w-[300px] truncate">
                      {{ currentAgent.workspace || '未设置' }}
                    </span>
                  </div>
                </div>
              </div>
            </template>
          </div>
        </div>

        <div v-show="activeTab === 'workspace'" class="p-4">
          <div class="space-y-3">
            <div class="flex justify-between items-center mb-4">
              <h4 class="text-sm font-medium text-gray-300">工作区文件</h4>
              <button
                v-if="currentAgent?.id"
                @click="loadWorkspaceFiles(currentAgent.id)"
                :disabled="loading"
                class="flex gap-1 items-center p-1.5 text-xs text-gray-400 rounded transition-colors hover:text-white hover:bg-dark-600 disabled:opacity-50"
              >
                <RefreshCw :size="14" :class="{ 'animate-spin': loading }" />
                刷新
              </button>
            </div>

            <div v-if="loading" class="flex justify-center items-center py-8">
              <Loader2 :size="24" class="animate-spin text-claw-400" />
            </div>

            <div v-else-if="!currentAgent" class="py-12 text-center text-gray-500">
              <Bot :size="48" class="mx-auto mb-4 opacity-30" />
              <p>请先选择要管理的内置智能体</p>
            </div>

            <div v-else class="space-y-2">
              <div
                v-for="config in fileConfigs"
                :key="config.filename"
                class="overflow-hidden rounded-lg border bg-dark-700 border-dark-500"
              >
                <button
                  @click="toggleFile(config.filename)"
                  class="flex justify-between items-center p-3 w-full text-left transition-colors hover:bg-dark-600"
                >
                  <div class="flex gap-3 items-center">
                    <component :is="config.icon" :size="18" :class="config.color" />
                    <div>
                      <div class="flex gap-2 items-center">
                        <span class="font-medium text-white">{{ config.title }}</span>
                        <span class="text-xs text-gray-500">{{ config.filename }}</span>
                      </div>
                      <p class="mt-0.5 text-xs text-gray-500">{{ config.description }}</p>
                    </div>
                  </div>
                  <ChevronDown
                    v-if="isExpanded(config.filename)"
                    :size="18"
                    class="text-gray-400"
                  />
                  <ChevronRight
                    v-else
                    :size="18"
                    class="text-gray-400"
                  />
                </button>

                <Transition name="expand">
                  <div v-if="isExpanded(config.filename)" class="border-t border-dark-500">
                    <div class="p-3">
                      <div class="flex justify-between items-center mb-2">
                        <div class="flex gap-2 items-center text-xs text-gray-500">
                          <FileText :size="12" />
                          <span>Markdown 格式</span>
                        </div>
                        <button
                          @click.stop="saveFile(config.filename)"
                          :disabled="saving === config.filename"
                          :class="clsx(
                            'flex gap-1.5 items-center px-3 py-1.5 text-xs font-medium rounded-lg transition-all',
                            'bg-claw-500/20 text-claw-400 hover:bg-claw-500/30',
                            'disabled:opacity-50 disabled:cursor-not-allowed'
                          )"
                        >
                          <Loader2 v-if="saving === config.filename" :size="12" class="animate-spin" />
                          <Save v-else :size="12" />
                          {{ saving === config.filename ? '保存中...' : '保存' }}
                        </button>
                      </div>
                      <textarea
                        :value="getFileContent(config.filename)"
                        @input="(e) => setFileContent(config.filename, (e.target as HTMLTextAreaElement).value)"
                        :placeholder="`在此编辑 ${config.filename} 的内容...`"
                        rows="12"
                        :class="clsx(
                          'w-full px-3 py-2 bg-dark-800 border rounded-lg text-white text-sm font-mono',
                          'placeholder-gray-600 resize-y min-h-[200px]',
                          'focus:outline-none focus:ring-2 focus:ring-claw-500/50 focus:border-claw-500',
                          'border-dark-500'
                        )"
                      />
                    </div>
                  </div>
                </Transition>
              </div>
            </div>

            <div v-if="workspaceDir" class="p-3 mt-4 rounded-lg border bg-dark-700/50 border-dark-600">
              <div class="flex gap-2 items-center text-xs text-gray-500">
                <FileText :size="12" />
                <span>工作区目录: </span>
                <code class="px-1.5 py-0.5 font-mono rounded bg-dark-600 text-claw-300">{{ workspaceDir }}</code>
              </div>
            </div>
          </div>
        </div>

        <div v-show="activeTab === 'skills'" class="p-4">
          <div v-if="!currentAgent" class="py-12 text-center text-gray-500">
            <Bot :size="48" class="mx-auto mb-4 opacity-30" />
            <p>请先选择要管理的内置智能体</p>
          </div>

          <template v-else>
            <div class="mb-4">
              <h4 class="mb-2 text-sm font-medium text-gray-300">已分配技能</h4>
              <div v-if="agentSkills.length === 0" class="p-4 text-sm text-center text-gray-500 rounded-lg border bg-dark-700 border-dark-500">
                暂无分配任何技能
              </div>
              <div v-else class="flex flex-wrap gap-2">
                <div
                  v-for="skillId in agentSkills"
                  :key="skillId"
                  class="flex gap-2 items-center px-3 py-2 rounded-lg border bg-dark-700 border-dark-500"
                >
                  <span class="text-sm text-white">{{ availableSkills.find(s => s.id === skillId)?.name || skillId }}</span>
                  <button
                    @click="handleRemoveSkill(skillId)"
                    class="p-1 text-gray-400 transition-colors hover:text-red-400"
                  >
                    <X :size="14" />
                  </button>
                </div>
              </div>
            </div>

            <div>
              <div class="flex gap-2 justify-between items-center mb-2">
                <h4 class="text-sm font-medium text-gray-300">可分配技能</h4>
                <span class="text-xs text-gray-500">{{ availableSkills.length }} 个技能</span>
              </div>
              <div v-if="skillsLoading" class="flex justify-center items-center py-8">
                <Loader2 :size="24" class="animate-spin text-claw-400" />
              </div>
              <div v-else-if="availableSkills.length === 0" class="p-8 text-center text-gray-500 rounded-lg border bg-dark-700 border-dark-500">
                <AlertCircle :size="24" class="mx-auto mb-2 opacity-50" />
                <p>暂无可用技能</p>
              </div>
              <div v-else class="overflow-y-auto pr-1 grid grid-cols-2 gap-2 max-h-80">
                <div
                  v-for="skill in availableSkills"
                  :key="skill.id"
                  :class="clsx(
                    'p-2 rounded-lg border transition-all cursor-pointer',
                    isSkillAssigned(skill.id)
                      ? 'bg-claw-500/10 border-claw-500/50'
                      : 'bg-dark-700 border-dark-500 hover:border-dark-400'
                  )"
                  @click="isSkillAssigned(skill.id) ? handleRemoveSkill(skill.id) : handleAssignSkill(skill.id)"
                >
                  <div class="flex gap-2 items-center">
                    <div class="flex justify-center items-center w-8 h-8 text-lg rounded-lg bg-dark-600 shrink-0">
                      {{ skill.emoji || '🔧' }}
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="flex gap-1 justify-between items-center">
                        <h4 class="text-xs font-medium text-white truncate">{{ skill.name }}</h4>
                        <CheckCircle v-if="isSkillAssigned(skill.id)" :size="12" class="text-claw-400 shrink-0" />
                      </div>
                      <p class="mt-0.5 text-[10px] text-gray-500 truncate">
                        {{ skill.description || '暂无描述' }}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </template>
        </div>
      </div>

      <div class="flex gap-3 justify-end p-4 border-t border-dark-600 shrink-0">
        <button
          @click="handleClose"
          class="px-4 py-2 text-gray-400 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
        >
          关闭
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
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
  opacity: 1;
  max-height: 1000px;
}

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>