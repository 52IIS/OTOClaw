<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { X, Loader2, Bot, FolderOpen, Cpu, Sparkles, Check, AlertCircle, ExternalLink, Info, FileText, Settings } from 'lucide-vue-next'
import clsx from 'clsx'
import { api, isTauri, type AgentInfo, type CreateAgentParams, type UpdateAgentParams, type SkillInfo } from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'
import AgentWorkspaceEditor from './AgentWorkspaceEditor.vue'

const props = defineProps<{
  editingAgent: AgentInfo | null
  availableModels: string[]
}>()

const emit = defineEmits<{
  close: []
  save: []
}>()

const { alert } = useDialog()

const isEditing = computed(() => props.editingAgent !== null)
const dialogTitle = computed(() => isEditing.value ? '编辑智能体' : '创建新智能体')

const activeTab = ref<'basic' | 'workspace'>('basic')
const createdAgentId = ref<string | null>(null)
const showWorkspaceEditor = ref(false)

const form = ref({
  name: '',
  description: '',
  avatar: '',
  model: '',
  workspace: '',
  selectedSkills: [] as string[],
})

const loading = ref(false)
const errors = ref<Record<string, string>>({})
const skillsLoading = ref(false)
const availableSkills = ref<SkillInfo[]>([])
const selectAllSkills = ref(true)

const loadSkills = async () => {
  if (!isTauri()) return
  
  skillsLoading.value = true
  try {
    const result = await api.getSkillsList()
    availableSkills.value = result.skills
  } catch (e) {
    console.error('加载技能列表失败:', e)
  } finally {
    skillsLoading.value = false
  }
}

onMounted(() => {
  loadSkills()
})

watch(
  () => props.editingAgent,
  (agent) => {
    if (agent) {
      form.value = {
        name: agent.name,
        description: agent.description || '',
        avatar: agent.avatar || '',
        model: agent.model || '',
        workspace: agent.workspace || '',
        selectedSkills: agent.skills || [],
      }
      selectAllSkills.value = form.value.selectedSkills.length === 0
      createdAgentId.value = agent.id
      showWorkspaceEditor.value = true
    } else {
      form.value = {
        name: '',
        description: '',
        avatar: '',
        model: props.availableModels[0] || '',
        workspace: '',
        selectedSkills: [],
      }
      selectAllSkills.value = true
      createdAgentId.value = null
      showWorkspaceEditor.value = false
    }
    activeTab.value = 'basic'
    errors.value = {}
  },
  { immediate: true }
)

watch(selectAllSkills, (newVal) => {
  if (newVal) {
    form.value.selectedSkills = []
  }
})

const avatarOptions = [
  { value: '🤖', label: '机器人' },
  { value: '🧠', label: '大脑' },
  { value: '💡', label: '灯泡' },
  { value: '🔮', label: '水晶球' },
  { value: '⚡', label: '闪电' },
  { value: '🎯', label: '靶心' },
  { value: '🚀', label: '火箭' },
  { value: '🌟', label: '星星' },
  { value: '🎭', label: '面具' },
  { value: '🐱', label: '猫咪' },
  { value: '🦊', label: '狐狸' },
  { value: '🐉', label: '龙' },
]

const workspaceExamples = [
  { label: 'Windows 示例', value: 'C:\\Users\\用户名\\.openclaw\\workspace-my-agent' },
  { label: 'macOS/Linux 示例', value: '~/.openclaw/workspace-my-agent' },
  { label: '项目目录示例', value: './workspace/my-agent' },
]

const toggleSkill = (skillId: string) => {
  if (selectAllSkills.value) {
    selectAllSkills.value = false
  }
  
  const index = form.value.selectedSkills.indexOf(skillId)
  if (index > -1) {
    form.value.selectedSkills.splice(index, 1)
  } else {
    form.value.selectedSkills.push(skillId)
  }
  
  if (form.value.selectedSkills.length === 0) {
    selectAllSkills.value = true
  }
}

const isSkillSelected = (skillId: string) => {
  return selectAllSkills.value || form.value.selectedSkills.includes(skillId)
}

const validate = (): boolean => {
  errors.value = {}

  if (!form.value.name.trim()) {
    errors.value.name = '请输入智能体名称'
  } else if (form.value.name.length > 50) {
    errors.value.name = '名称不能超过50个字符'
  }

  if (form.value.description.length > 200) {
    errors.value.description = '描述不能超过200个字符'
  }

  return Object.keys(errors.value).length === 0
}

const handleSubmit = async () => {
  if (!validate()) return

  if (!isTauri()) {
    emit('save')
    return
  }

  loading.value = true

  try {
    const skillsArray = selectAllSkills.value ? [] : form.value.selectedSkills

    if (isEditing.value && props.editingAgent) {
      const params: UpdateAgentParams = {
        agentId: props.editingAgent.id,
        name: form.value.name.trim(),
        description: form.value.description.trim() || undefined,
        avatar: form.value.avatar || undefined,
        model: form.value.model || undefined,
        workspace: form.value.workspace.trim() || undefined,
        skills: skillsArray.length > 0 ? skillsArray : undefined,
      }
      await api.updateAgent(params)
      createdAgentId.value = props.editingAgent.id
      showWorkspaceEditor.value = true
      activeTab.value = 'workspace'
    } else {
      const params: CreateAgentParams = {
        name: form.value.name.trim(),
        description: form.value.description.trim() || undefined,
        avatar: form.value.avatar || undefined,
        model: form.value.model || undefined,
        workspace: form.value.workspace.trim() || undefined,
        skills: skillsArray,
      }
      const result = await api.createAgent(params)
      createdAgentId.value = result.id
      showWorkspaceEditor.value = true
      activeTab.value = 'workspace'
    }
  } catch (e) {
    await alert('操作失败: ' + e, { variant: 'error', title: '操作失败' })
  } finally {
    loading.value = false
  }
}

const handleClose = () => {
  if (showWorkspaceEditor.value && createdAgentId.value) {
    emit('save')
  }
  emit('close')
}

const handleWorkspaceSaved = () => {
}

const getSkillTagsDisplay = (skill: SkillInfo) => {
  const tags = [...skill.tags]
  if (skill.bundled) {
    tags.unshift('内置')
  }
  if (!skill.eligible) {
    tags.push('需配置')
  }
  return tags.slice(0, 4)
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
    <div
      class="w-full max-w-4xl max-h-[90vh] bg-dark-800 rounded-2xl border border-dark-500 shadow-2xl flex flex-col"
      @click.stop
    >
      <div class="flex items-center justify-between p-4 border-b border-dark-600 shrink-0">
        <div class="flex items-center gap-2">
          <Bot :size="20" class="text-claw-400" />
          <h3 class="text-lg font-semibold text-white">{{ dialogTitle }}</h3>
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
          @click="activeTab = 'basic'"
          :class="clsx(
            'flex gap-2 items-center px-4 py-2 rounded-lg text-sm font-medium transition-all',
            activeTab === 'basic'
              ? 'bg-claw-500/20 text-claw-400'
              : 'text-gray-400 hover:text-white hover:bg-dark-600'
          )"
        >
          <Settings :size="16" />
          基础设置
        </button>
        <button
          @click="activeTab = 'workspace'"
          :disabled="!showWorkspaceEditor"
          :class="clsx(
            'flex gap-2 items-center px-4 py-2 rounded-lg text-sm font-medium transition-all',
            activeTab === 'workspace'
              ? 'bg-claw-500/20 text-claw-400'
              : 'text-gray-400 hover:text-white hover:bg-dark-600',
            !showWorkspaceEditor && 'opacity-50 cursor-not-allowed'
          )"
        >
          <FileText :size="16" />
          工作区文件
          <span v-if="!showWorkspaceEditor" class="text-xs text-gray-500">(创建后可用)</span>
        </button>
      </div>

      <div class="flex-1 overflow-y-auto">
        <form v-show="activeTab === 'basic'" @submit.prevent="handleSubmit" class="p-4 space-y-5">
          <div>
            <label class="block mb-1 text-sm font-medium text-gray-300">
              名称 <span class="text-red-400">*</span>
            </label>
            <input
              v-model="form.name"
              type="text"
              placeholder="给智能体起个名字"
              :class="clsx(
                'w-full px-3 py-2 bg-dark-700 border rounded-lg text-white placeholder-gray-500',
                'focus:outline-none focus:ring-2 focus:ring-claw-500/50 focus:border-claw-500',
                errors.name ? 'border-red-500' : 'border-dark-500'
              )"
            />
            <p v-if="errors.name" class="mt-1 text-xs text-red-400">{{ errors.name }}</p>
          </div>

          <div>
            <label class="block mb-1 text-sm font-medium text-gray-300">
              头像
            </label>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="option in avatarOptions"
                :key="option.value"
                type="button"
                @click="form.avatar = option.value"
                :class="clsx(
                  'w-10 h-10 text-xl rounded-lg border transition-all',
                  form.avatar === option.value
                    ? 'bg-claw-500/20 border-claw-500 scale-110'
                    : 'bg-dark-700 border-dark-500 hover:border-dark-400'
                )"
              >
                {{ option.value }}
              </button>
            </div>
          </div>

          <div>
            <label class="block mb-1 text-sm font-medium text-gray-300">
              描述
            </label>
            <textarea
              v-model="form.description"
              placeholder="描述这个智能体的用途和特点"
              rows="2"
              :class="clsx(
                'w-full px-3 py-2 bg-dark-700 border rounded-lg text-white placeholder-gray-500 resize-none',
                'focus:outline-none focus:ring-2 focus:ring-claw-500/50 focus:border-claw-500',
                errors.description ? 'border-red-500' : 'border-dark-500'
              )"
            />
            <p v-if="errors.description" class="mt-1 text-xs text-red-400">{{ errors.description }}</p>
          </div>

          <div>
            <label class="block mb-1 text-sm font-medium text-gray-300">
              <Cpu :size="14" class="inline mr-1" />
              主模型
            </label>
            <select
              v-model="form.model"
              :class="clsx(
                'w-full px-3 py-2 bg-dark-700 border rounded-lg text-white',
                'focus:outline-none focus:ring-2 focus:ring-claw-500/50 focus:border-claw-500',
                'border-dark-500'
              )"
            >
              <option value="">使用默认模型</option>
              <option v-for="model in availableModels" :key="model" :value="model">
                {{ model }}
              </option>
            </select>
          </div>

          <div>
            <label class="block mb-1 text-sm font-medium text-gray-300">
              <FolderOpen :size="14" class="inline mr-1" />
              工作区目录
            </label>
            <input
              v-model="form.workspace"
              type="text"
              placeholder="留空则自动生成在 ~/.openclaw/workspace-{智能体ID}"
              :class="clsx(
                'w-full px-3 py-2 bg-dark-700 border rounded-lg text-white placeholder-gray-500',
                'focus:outline-none focus:ring-2 focus:ring-claw-500/50 focus:border-claw-500',
                'border-dark-500'
              )"
            />
            <div class="mt-2 p-3 rounded-lg bg-dark-700/50 border border-dark-600">
              <div class="flex gap-1 items-center mb-2 text-xs text-gray-400">
                <Info :size="12" />
                目录填写示例
              </div>
              <div class="space-y-1">
                <div
                  v-for="example in workspaceExamples"
                  :key="example.label"
                  class="flex gap-2 items-center text-xs"
                >
                  <span class="text-gray-500 w-28 shrink-0">{{ example.label }}:</span>
                  <code class="px-1.5 py-0.5 bg-dark-600 rounded text-claw-300 font-mono">{{ example.value }}</code>
                </div>
              </div>
              <p class="mt-2 text-xs text-gray-500">
                工作区用于存储智能体的配置文件、会话记录和身份文件
              </p>
            </div>
          </div>

          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="text-sm font-medium text-gray-300">
                <Sparkles :size="14" class="inline mr-1" />
                技能选择
              </label>
              <label class="flex gap-2 items-center text-sm text-gray-400 cursor-pointer">
                <input
                  type="checkbox"
                  v-model="selectAllSkills"
                  class="w-4 h-4 rounded border-dark-500 bg-dark-700 text-claw-500 focus:ring-claw-500/50"
                />
                全选（不限制技能）
              </label>
            </div>

            <div v-if="skillsLoading" class="flex justify-center items-center py-8">
              <Loader2 :size="24" class="animate-spin text-claw-400" />
            </div>

            <div v-else-if="availableSkills.length === 0" class="py-8 text-center text-gray-500">
              <AlertCircle :size="24" class="mx-auto mb-2 opacity-50" />
              <p>暂无可用技能</p>
              <p class="text-xs mt-1">请确保 OpenClaw 已正确安装并配置</p>
            </div>

            <div v-else class="grid grid-cols-2 gap-3 max-h-64 overflow-y-auto pr-1">
              <div
                v-for="skill in availableSkills"
                :key="skill.id"
                @click="toggleSkill(skill.id)"
                :class="clsx(
                  'p-3 rounded-xl border cursor-pointer transition-all',
                  isSkillSelected(skill.id)
                    ? 'bg-claw-500/10 border-claw-500/50 hover:border-claw-500'
                    : 'bg-dark-700 border-dark-500 hover:border-dark-400 opacity-60'
                )"
              >
                <div class="flex gap-3 items-start">
                  <div class="flex justify-center items-center w-10 h-10 text-xl rounded-lg bg-dark-600 shrink-0">
                    {{ skill.emoji || '🔧' }}
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex gap-2 items-center">
                      <h4 class="font-medium text-white text-sm truncate">{{ skill.name }}</h4>
                      <Check v-if="isSkillSelected(skill.id)" :size="14" class="text-claw-400 shrink-0" />
                    </div>
                    <p class="mt-0.5 text-xs text-gray-500 line-clamp-2">
                      {{ skill.description || '暂无描述' }}
                    </p>
                    <div class="flex flex-wrap gap-1 mt-2">
                      <span
                        v-for="tag in getSkillTagsDisplay(skill)"
                        :key="tag"
                        :class="clsx(
                          'px-1.5 py-0.5 text-[10px] rounded',
                          tag === '内置' ? 'bg-blue-500/20 text-blue-400' :
                          tag === '需配置' ? 'bg-yellow-500/20 text-yellow-400' :
                          'bg-dark-600 text-gray-400'
                        )"
                      >
                        {{ tag }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="flex gap-4 justify-between items-center mt-3 p-2 rounded-lg bg-dark-700/50">
              <div class="text-xs text-gray-500">
                <span v-if="selectAllSkills">已选择：全部技能（无限制）</span>
                <span v-else>已选择：{{ form.selectedSkills.length }} / {{ availableSkills.length }} 个技能</span>
              </div>
              <a
                v-if="availableSkills.find(s => s.homepage)"
                :href="availableSkills.find(s => s.homepage)?.homepage || '#'"
                target="_blank"
                class="flex gap-1 items-center text-xs text-claw-400 hover:text-claw-300"
              >
                了解更多
                <ExternalLink :size="12" />
              </a>
            </div>
          </div>
        </form>

        <div v-show="activeTab === 'workspace'" class="p-4">
          <AgentWorkspaceEditor
            v-if="showWorkspaceEditor && createdAgentId"
            :agent-id="createdAgentId"
            :is-new-agent="!isEditing"
            @saved="handleWorkspaceSaved"
          />
          <div v-else class="py-12 text-center text-gray-500">
            <FileText :size="48" class="mx-auto mb-4 opacity-30" />
            <p>请先创建智能体后再编辑工作区文件</p>
          </div>
        </div>
      </div>

      <div class="flex gap-3 justify-end p-4 border-t border-dark-600 shrink-0">
        <button
          type="button"
          @click="handleClose"
          class="px-4 py-2 text-gray-400 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
        >
          {{ showWorkspaceEditor ? '完成' : '取消' }}
        </button>
        <button
          v-if="activeTab === 'basic'"
          @click="handleSubmit"
          :disabled="loading || !form.name.trim()"
          :class="clsx(
            'flex gap-2 items-center px-4 py-2 rounded-lg font-medium transition-all',
            'bg-claw-500 text-white hover:bg-claw-600',
            'disabled:opacity-50 disabled:cursor-not-allowed'
          )"
        >
          <Loader2 v-if="loading" :size="16" class="animate-spin" />
          <Bot v-else :size="16" />
          {{ isEditing ? '保存并编辑文件' : '创建并编辑文件' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
