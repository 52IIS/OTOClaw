<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { X, Loader2, Key, CheckCircle, XCircle, AlertCircle, ExternalLink, FolderOpen, Download, Settings, UserPlus } from 'lucide-vue-next'
import { api, type SkillInfo, type AgentInfo, type SkillDetail, type UpdateSkillConfigParams } from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'

const props = defineProps<{
  skill: SkillInfo | null
  agents: AgentInfo[]
}>()

const emit = defineEmits<{
  close: []
  save: []
}>()

const { alert, confirm } = useDialog()

const loading = ref(true)
const saving = ref(false)
const detail = ref<SkillDetail | null>(null)

const enabled = ref(true)
const apiKey = ref('')
const showApiKey = ref(false)

const selectedAgentId = ref<string | undefined>(undefined)
const assigning = ref(false)

const loadDetail = async () => {
  if (!props.skill) return

  loading.value = true
  try {
    detail.value = await api.getSkillDetail(props.skill.id)
    enabled.value = detail.value.config?.enabled ?? true
    apiKey.value = detail.value.config?.apiKey || ''
  } catch (e) {
    await alert('加载技能详情失败: ' + e, { variant: 'error', title: '加载失败' })
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadDetail()
})

watch(() => props.skill, () => {
  loadDetail()
})

const handleSaveConfig = async () => {
  if (!props.skill) return

  saving.value = true
  try {
    const params: UpdateSkillConfigParams = {
      skillId: props.skill.id,
      enabled: enabled.value,
      apiKey: apiKey.value || undefined,
    }
    await api.updateSkillConfig(params)
    await alert('配置已保存', { variant: 'success', title: '保存成功' })
    emit('save')
  } catch (e) {
    await alert('保存配置失败: ' + e, { variant: 'error', title: '保存失败' })
  } finally {
    saving.value = false
  }
}

const handleOpenDirectory = async () => {
  if (!props.skill) return
  try {
    await api.openSkillDirectory(props.skill.id)
  } catch (e) {
    await alert('打开目录失败: ' + e, { variant: 'error', title: '操作失败' })
  }
}

const handleExport = async () => {
  if (!props.skill) return
  try {
    const result = await api.exportSkill({ skillId: props.skill.id })
    if (result.success) {
      await alert(`技能已导出到:\n${result.outputPath}`, { variant: 'success', title: '导出成功' })
    }
  } catch (e) {
    await alert('导出技能失败: ' + e, { variant: 'error', title: '导出失败' })
  }
}

const handleAssignToAgent = async () => {
  if (!props.skill || !selectedAgentId.value) return

  const agent = props.agents.find(a => a.id === selectedAgentId.value)
  if (!agent) return

  const confirmed = await confirm(
    `确定要将技能 "${props.skill.name}" 分配给智能体 "${agent.name}" 吗？\n\n这将在智能体工作区创建技能副本。`,
    {
      title: '分配确认',
      variant: 'info',
      confirmText: '分配',
      cancelText: '取消',
    }
  )

  if (!confirmed) return

  assigning.value = true
  try {
    await api.assignSkillToAgent(props.skill.id, selectedAgentId.value)
    await alert(`技能已分配给智能体 "${agent.name}"`, { variant: 'success', title: '分配成功' })
    selectedAgentId.value = undefined
  } catch (e) {
    await alert('分配技能失败: ' + e, { variant: 'error', title: '分配失败' })
  } finally {
    assigning.value = false
  }
}

const handleInstallDependency = async (installId: string) => {
  if (!props.skill) return

  try {
    const result = await api.installSkillDependency(props.skill.id, installId)
    if (result.success) {
      await alert(result.message, { variant: 'success', title: '安装成功' })
      loadDetail()
    }
  } catch (e) {
    await alert('安装依赖失败: ' + e, { variant: 'error', title: '安装失败' })
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div class="w-full max-w-2xl mx-4 rounded-2xl bg-dark-800 border border-dark-500 max-h-[90vh] overflow-hidden flex flex-col">
      <div class="flex justify-between items-center p-4 border-b border-dark-500">
        <div class="flex gap-3 items-center">
          <span class="text-2xl">{{ skill?.emoji || '📦' }}</span>
          <div>
            <h3 class="text-lg font-semibold text-white">{{ skill?.name }}</h3>
            <p class="text-sm text-gray-500">{{ skill?.source }}</p>
          </div>
        </div>
        <button
          @click="emit('close')"
          class="p-1 text-gray-400 rounded-lg hover:text-white hover:bg-dark-600"
        >
          <X :size="20" />
        </button>
      </div>

      <div v-if="loading" class="flex justify-center items-center py-12">
        <Loader2 :size="24" class="animate-spin text-claw-400" />
      </div>

      <div v-else class="flex-1 overflow-y-auto p-4 space-y-6">
        <div>
          <h4 class="mb-2 text-sm font-medium text-gray-400">描述</h4>
          <p class="text-sm text-white">{{ detail?.description || skill?.description || '暂无描述' }}</p>
        </div>

        <div v-if="skill?.homepage" class="flex gap-2 items-center">
          <ExternalLink :size="16" class="text-claw-400" />
          <a :href="skill.homepage" target="_blank" class="text-sm text-claw-400 hover:underline">
            {{ skill.homepage }}
          </a>
        </div>

        <div class="flex gap-4 items-center">
          <span
            v-if="skill?.eligible"
            class="flex gap-1 items-center px-3 py-1 text-sm text-green-400 rounded-lg bg-green-500/20"
          >
            <CheckCircle :size="16" /> 可用
          </span>
          <span
            v-else
            class="flex gap-1 items-center px-3 py-1 text-sm text-yellow-400 rounded-lg bg-yellow-500/20"
          >
            <AlertCircle :size="16" /> 需要配置
          </span>
          <span
            v-if="skill?.disabled"
            class="flex gap-1 items-center px-3 py-1 text-sm text-red-400 rounded-lg bg-red-500/20"
          >
            <XCircle :size="16" /> 已禁用
          </span>
        </div>

        <div class="p-4 rounded-xl bg-dark-700 border border-dark-500">
          <h4 class="mb-4 text-sm font-medium text-white">技能配置</h4>

          <div class="space-y-4">
            <div class="flex justify-between items-center">
              <div>
                <p class="text-sm text-white">启用技能</p>
                <p class="text-xs text-gray-500">禁用后智能体将无法使用此技能</p>
              </div>
              <button
                @click="enabled = !enabled"
                :class="[
                  'relative w-12 h-6 rounded-full transition-colors',
                  enabled ? 'bg-claw-500' : 'bg-dark-500'
                ]"
              >
                <span
                  :class="[
                    'absolute top-1 w-4 h-4 rounded-full bg-white transition-transform',
                    enabled ? 'left-7' : 'left-1'
                  ]"
                />
              </button>
            </div>

            <div v-if="detail?.requiresApiKey">
              <label class="block mb-2 text-sm text-gray-400">
                <Key :size="14" class="inline mr-1" />
                API 密钥
                <span v-if="detail.primaryEnv" class="text-xs text-gray-500">({{ detail.primaryEnv }})</span>
              </label>
              <div class="relative">
                <input
                  v-model="apiKey"
                  :type="showApiKey ? 'text' : 'password'"
                  placeholder="输入 API 密钥"
                  class="w-full px-3 py-2 pr-20 text-sm text-white rounded-lg bg-dark-600 border border-dark-500 focus:border-claw-500 focus:outline-none"
                />
                <button
                  @click="showApiKey = !showApiKey"
                  class="absolute right-2 top-1/2 -translate-y-1/2 px-2 py-1 text-xs text-gray-400 hover:text-white"
                >
                  {{ showApiKey ? '隐藏' : '显示' }}
                </button>
              </div>
              <p class="mt-1 text-xs text-gray-500">密钥将安全存储在配置文件中</p>
            </div>
          </div>
        </div>

        <div v-if="(skill?.requiredEnv?.length || 0) > 0 || (skill?.requiredBins?.length || 0) > 0" class="p-4 rounded-xl bg-dark-700 border border-dark-500">
          <h4 class="mb-4 text-sm font-medium text-white">依赖要求</h4>

          <div v-if="skill && skill.requiredEnv && skill.requiredEnv.length > 0" class="mb-4">
            <p class="mb-2 text-xs text-gray-400">环境变量</p>
            <div class="flex flex-wrap gap-2">
              <span
                v-for="env in skill.requiredEnv"
                :key="env"
                class="px-2 py-1 text-xs rounded bg-dark-600 text-gray-300"
              >
                {{ env }}
              </span>
            </div>
          </div>

          <div v-if="skill && skill.requiredBins && skill.requiredBins.length > 0">
            <p class="mb-2 text-xs text-gray-400">二进制文件</p>
            <div class="flex flex-wrap gap-2">
              <span
                v-for="bin in skill.requiredBins"
                :key="bin"
                class="px-2 py-1 text-xs rounded bg-dark-600 text-gray-300"
              >
                {{ bin }}
              </span>
            </div>
          </div>

          <div v-if="detail?.installOptions && detail.installOptions.length > 0" class="mt-4">
            <p class="mb-2 text-xs text-gray-400">安装选项</p>
            <div class="space-y-2">
              <button
                v-for="option in detail.installOptions"
                :key="option.id"
                @click="handleInstallDependency(option.id)"
                class="flex gap-2 items-center w-full px-3 py-2 text-sm text-left text-white rounded-lg bg-dark-600 hover:bg-dark-500"
              >
                <Settings :size="14" class="text-claw-400" />
                {{ option.label }}
              </button>
            </div>
          </div>
        </div>

        <div class="p-4 rounded-xl bg-dark-700 border border-dark-500">
          <h4 class="mb-4 text-sm font-medium text-white">分配到智能体</h4>
          <p class="mb-3 text-xs text-gray-500">将技能分配给特定智能体，仅该智能体可用</p>
          
          <div class="flex gap-2">
            <select
              v-model="selectedAgentId"
              class="flex-1 px-3 py-2 text-sm text-white rounded-lg bg-dark-600 border border-dark-500 focus:border-claw-500 focus:outline-none"
            >
              <option :value="undefined">选择智能体...</option>
              <option v-for="agent in agents" :key="agent.id" :value="agent.id">
                {{ agent.name }}
              </option>
            </select>
            <button
              @click="handleAssignToAgent"
              :disabled="!selectedAgentId || assigning"
              class="flex gap-2 items-center px-4 py-2 text-sm text-white rounded-lg bg-claw-500 hover:bg-claw-600 disabled:opacity-50"
            >
              <Loader2 v-if="assigning" :size="16" class="animate-spin" />
              <UserPlus v-else :size="16" />
              分配
            </button>
          </div>
        </div>

        <div class="flex gap-2">
          <button
            @click="handleOpenDirectory"
            class="flex gap-2 items-center px-4 py-2 text-sm text-gray-300 rounded-lg bg-dark-600 hover:bg-dark-500"
          >
            <FolderOpen :size="16" />
            打开目录
          </button>
          <button
            @click="handleExport"
            class="flex gap-2 items-center px-4 py-2 text-sm text-gray-300 rounded-lg bg-dark-600 hover:bg-dark-500"
          >
            <Download :size="16" />
            导出技能
          </button>
        </div>

        <div v-if="detail?.skillMdContent" class="p-4 rounded-xl bg-dark-700 border border-dark-500">
          <h4 class="mb-2 text-sm font-medium text-gray-400">SKILL.md 内容</h4>
          <pre class="p-3 overflow-x-auto text-xs text-gray-300 rounded-lg bg-dark-600 max-h-60">{{ detail.skillMdContent }}</pre>
        </div>
      </div>

      <div class="flex justify-end gap-3 p-4 border-t border-dark-500">
        <button
          @click="emit('close')"
          class="px-4 py-2 text-sm text-gray-400 rounded-lg hover:text-white"
        >
          关闭
        </button>
        <button
          @click="handleSaveConfig"
          :disabled="saving"
          class="flex gap-2 items-center px-4 py-2 text-sm text-white rounded-lg bg-claw-500 hover:bg-claw-600 disabled:opacity-50"
        >
          <Loader2 v-if="saving" :size="16" class="animate-spin" />
          保存配置
        </button>
      </div>
    </div>
  </div>
</template>
