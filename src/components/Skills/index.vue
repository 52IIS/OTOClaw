<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  Loader2,
  Plus,
  Sparkles,
  Package,
  FolderOpen,
  Download,
  Upload,
  Pencil,
  Trash2,
  Key,
  CheckCircle,
  XCircle,
  AlertCircle,
  Settings,
  ExternalLink,
} from 'lucide-vue-next'
import clsx from 'clsx'
import {
  api,
  isTauri,
  type SkillInfo,
  type SkillsListResult,
  type AgentsListResult,
  type InstallSkillResult,
} from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'
import SkillDialog from './SkillDialog.vue'
import SkillDetailDialog from './SkillDetailDialog.vue'

const { alert, confirm } = useDialog()

const loading = ref(true)
const skillsResult = ref<SkillsListResult | null>(null)
const agentsResult = ref<AgentsListResult | null>(null)
const error = ref<string | null>(null)

const showAddDialog = ref(false)
const showDetailDialog = ref(false)
const showInstallDialog = ref(false)
const selectedSkill = ref<SkillInfo | null>(null)
const installZipPath = ref('')
const installAgentId = ref<string | undefined>(undefined)
const installing = ref(false)

const skills = computed(() => skillsResult.value?.skills || [])
const eligibleCount = computed(() => skillsResult.value?.eligibleCount || 0)
const totalCount = computed(() => skillsResult.value?.total || 0)
const agents = computed(() => agentsResult.value?.agents || [])

const loadData = async () => {
  if (!isTauri()) {
    loading.value = false
    return
  }

  loading.value = true
  error.value = null

  try {
    const [skillsData, agentsData] = await Promise.all([
      api.getSkillsList(),
      api.getAgentsList(),
    ])
    skillsResult.value = skillsData
    agentsResult.value = agentsData
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadData()
})

const handleAddSkill = () => {
  selectedSkill.value = null
  showAddDialog.value = true
}

const handleEditSkill = (skill: SkillInfo) => {
  selectedSkill.value = skill
  showAddDialog.value = true
}

const handleViewDetail = (skill: SkillInfo) => {
  selectedSkill.value = skill
  showDetailDialog.value = true
}

const handleCloseDialog = () => {
  showAddDialog.value = false
  showDetailDialog.value = false
  selectedSkill.value = null
}

const handleDialogSave = () => {
  loadData()
  handleCloseDialog()
}

const handleDeleteSkill = async (skill: SkillInfo) => {
  if (skill.bundled) {
    await alert('内置技能不能删除', { variant: 'warning', title: '无法删除' })
    return
  }

  const confirmed = await confirm(
    `确定要删除技能 "${skill.name}" 吗？\n\n删除后将移除技能文件和相关配置。`,
    {
      title: '删除确认',
      variant: 'warning',
      confirmText: '删除',
      cancelText: '取消',
    }
  )

  if (!confirmed) return

  try {
    await api.deleteSkill(skill.id)
    await loadData()
  } catch (e) {
    await alert('删除技能失败: ' + e, { variant: 'error', title: '删除失败' })
  }
}

const handleOpenDirectory = async (skill: SkillInfo) => {
  try {
    await api.openSkillDirectory(skill.id)
  } catch (e) {
    await alert('打开目录失败: ' + e, { variant: 'error', title: '操作失败' })
  }
}

const handleExportSkill = async (skill: SkillInfo) => {
  try {
    const result = await api.exportSkill({ skillId: skill.id })
    if (result.success) {
      await alert(`技能已导出到:\n${result.outputPath}`, {
        variant: 'success',
        title: '导出成功',
      })
    }
  } catch (e) {
    await alert('导出技能失败: ' + e, { variant: 'error', title: '导出失败' })
  }
}

const handleInstallFromZip = async () => {
  if (!installZipPath.value) {
    await alert('请选择 ZIP 文件', { variant: 'warning', title: '提示' })
    return
  }

  installing.value = true
  try {
    const result: InstallSkillResult = await api.installSkillFromZip({
      zipPath: installZipPath.value,
      agentId: installAgentId.value,
    })

    if (result.success) {
      let message = `技能 "${result.name}" 安装成功！`
      if (result.warnings.length > 0) {
        message += '\n\n警告:\n' + result.warnings.join('\n')
      }
      await alert(message, { variant: 'success', title: '安装成功' })
      showInstallDialog.value = false
      installZipPath.value = ''
      installAgentId.value = undefined
      await loadData()
    } else {
      await alert('安装失败: ' + result.error, { variant: 'error', title: '安装失败' })
    }
  } catch (e) {
    await alert('安装技能失败: ' + e, { variant: 'error', title: '安装失败' })
  } finally {
    installing.value = false
  }
}

const selectZipFile = async () => {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selected = await open({
    multiple: false,
    filters: [{ name: 'ZIP', extensions: ['zip'] }],
  })
  if (selected) {
    installZipPath.value = selected as string
  }
}

const getEmojiDisplay = (skill: SkillInfo) => {
  return skill.emoji || '📦'
}

const getSourceLabel = (source: string) => {
  switch (source) {
    case 'bundled':
      return '内置'
    case 'managed':
      return '托管'
    case 'extra':
      return '外部'
    default:
      if (source.startsWith('agent-')) {
        return '智能体'
      }
      return source
  }
}

const getSourceColor = (source: string) => {
  switch (source) {
    case 'bundled':
      return 'text-blue-400 bg-blue-500/20'
    case 'managed':
      return 'text-green-400 bg-green-500/20'
    case 'extra':
      return 'text-purple-400 bg-purple-500/20'
    default:
      return 'text-orange-400 bg-orange-500/20'
  }
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

      <div class="flex gap-3">
        <a
          href="https://clawhub.ai/"
          target="_blank"
          class="flex gap-2 items-center px-4 py-3 text-sm rounded-xl border transition-colors text-claw-400 bg-claw-500/10 border-claw-500/30 hover:bg-claw-500/20"
        >
          <ExternalLink :size="16" />
          ClawHub 官方社区
        </a>
        <a
          href="https://skillhub.tencent.com/"
          target="_blank"
          class="flex gap-2 items-center px-4 py-3 text-sm text-gray-300 rounded-xl border transition-colors bg-dark-600 border-dark-500 hover:bg-dark-500"
        >
          <ExternalLink :size="16" />
          SkillHub 技能商店
        </a>
      </div>

      <div class="p-6 bg-gradient-to-br rounded-2xl border from-dark-700 to-dark-800 border-dark-500">
        <div class="flex justify-between items-start mb-4">
          <div>
            <h2 class="flex gap-2 items-center text-xl font-semibold text-white">
              <Sparkles :size="22" class="text-claw-400" />
              技能管理
            </h2>
            <p class="mt-1 text-sm text-gray-500">管理AI技能，扩展智能体的能力范围</p>
          </div>
          <div class="flex gap-2">
            <button @click="showInstallDialog = true" class="flex gap-2 items-center px-4 py-2 text-sm text-gray-300 rounded-lg transition-colors bg-dark-600 hover:bg-dark-500">
              <Upload :size="16" />
              安装技能
            </button>
            <button @click="handleAddSkill" class="flex gap-2 items-center btn-primary">
              <Plus :size="16" />
              新建技能
            </button>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-4">
          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-claw-500/20">
                <Package :size="20" class="text-claw-400" />
              </div>
              <div>
                <p class="text-2xl font-bold text-white">{{ totalCount }}</p>
                <p class="text-sm text-gray-500">技能总数</p>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-green-500/20">
                <CheckCircle :size="20" class="text-green-400" />
              </div>
              <div>
                <p class="text-2xl font-bold text-white">{{ eligibleCount }}</p>
                <p class="text-sm text-gray-500">可用技能</p>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-blue-500/20">
                <Sparkles :size="20" class="text-blue-400" />
              </div>
              <div>
                <p class="text-lg font-medium text-white">{{ skills.filter(s => s.bundled).length }}</p>
                <p class="text-sm text-gray-500">内置技能</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="space-y-4">
        <h3 class="flex gap-2 items-center text-lg font-medium text-white">
          <Package :size="18" class="text-gray-500" />
          技能列表
        </h3>

        <div v-if="loading" class="flex justify-center items-center py-12">
          <Loader2 :size="24" class="animate-spin text-claw-400" />
        </div>

        <div v-else-if="skills.length === 0" class="p-8 text-center rounded-xl border bg-dark-700 border-dark-500">
          <div class="flex justify-center items-center mx-auto mb-4 w-16 h-16 rounded-full bg-dark-600">
            <Package :size="24" class="text-gray-500" />
          </div>
          <p class="mb-4 text-gray-400">还没有任何技能</p>
          <button @click="handleAddSkill" class="btn-primary">创建第一个技能</button>
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="skill in skills"
            :key="skill.id"
            :class="clsx(
              'rounded-xl border transition-all cursor-pointer',
              skill.eligible
                ? 'bg-dark-700 border-dark-500 hover:border-dark-400'
                : 'bg-dark-700/50 border-dark-600'
            )"
            @click="handleViewDetail(skill)"
          >
            <div class="p-4">
              <div class="flex gap-4 items-start">
                <div class="flex justify-center items-center w-12 h-12 text-2xl rounded-xl bg-dark-600">
                  {{ getEmojiDisplay(skill) }}
                </div>

                <div class="flex-1 min-w-0">
                  <div class="flex gap-2 items-center mb-1">
                    <h4 class="font-medium text-white">{{ skill.name }}</h4>
                    <span
                      :class="clsx(
                        'px-2 py-0.5 text-xs rounded',
                        getSourceColor(skill.source)
                      )"
                    >
                      {{ getSourceLabel(skill.source) }}
                    </span>
                    <span
                      v-if="skill.eligible"
                      class="flex gap-1 items-center px-2 py-0.5 text-xs text-green-400 rounded bg-green-500/20"
                    >
                      <CheckCircle :size="10" /> 可用
                    </span>
                    <span
                      v-else
                      class="flex gap-1 items-center px-2 py-0.5 text-xs text-yellow-400 rounded bg-yellow-500/20"
                    >
                      <AlertCircle :size="10" /> 需配置
                    </span>
                    <span
                      v-if="skill.disabled"
                      class="flex gap-1 items-center px-2 py-0.5 text-xs text-red-400 rounded bg-red-500/20"
                    >
                      <XCircle :size="10" /> 已禁用
                    </span>
                  </div>

                  <p class="mb-2 text-sm text-gray-500 line-clamp-1">
                    {{ skill.description || '暂无描述' }}
                  </p>

                  <div class="flex flex-wrap gap-3 text-xs text-gray-500">
                    <span v-if="(skill.requiredEnv?.length || 0) > 0" class="flex gap-1 items-center">
                      <Key :size="12" />
                      {{ skill.requiredEnv?.length || 0 }} 个环境变量
                    </span>
                    <span v-if="(skill.requiredBins?.length || 0) > 0" class="flex gap-1 items-center">
                      <Settings :size="12" />
                      {{ skill.requiredBins?.length || 0 }} 个依赖
                    </span>
                    <span v-if="skill.homepage" class="flex gap-1 items-center text-claw-400">
                      <ExternalLink :size="12" />
                      主页
                    </span>
                  </div>
                </div>

                <div class="flex gap-2 items-center" @click.stop>
                  <button
                    @click="handleOpenDirectory(skill)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-dark-600 hover:text-claw-400"
                    title="打开目录"
                  >
                    <FolderOpen :size="14" />
                  </button>

                  <button
                    @click="handleExportSkill(skill)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-dark-600 hover:text-claw-400"
                    title="导出"
                  >
                    <Download :size="14" />
                  </button>

                  <button
                    v-if="!skill.bundled"
                    @click="handleEditSkill(skill)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-dark-600 hover:text-claw-400"
                    title="编辑"
                  >
                    <Pencil :size="14" />
                  </button>

                  <button
                    v-if="!skill.bundled"
                    @click="handleDeleteSkill(skill)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-red-500/10 hover:text-red-400"
                    title="删除"
                  >
                    <Trash2 :size="14" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="p-4 rounded-xl border bg-dark-700/50 border-dark-500">
        <h4 class="mb-2 text-sm font-medium text-gray-400">技能说明</h4>
        <ul class="space-y-1 text-sm text-gray-500">
          <li>• 技能定义了AI可以执行的操作和使用的工具</li>
          <li>• 内置技能随OpenClaw安装，托管技能由用户管理</li>
          <li>• 部分技能需要配置API密钥或安装依赖才能使用</li>
          <li>• 可以将技能分配给特定智能体，实现个性化配置</li>
        </ul>
      </div>
    </div>

    <Teleport to="body">
      <Transition name="fade">
        <SkillDialog
          v-if="showAddDialog"
          :editing-skill="selectedSkill"
          :agents="agents"
          @close="handleCloseDialog"
          @save="handleDialogSave"
        />
      </Transition>

      <Transition name="fade">
        <SkillDetailDialog
          v-if="showDetailDialog"
          :skill="selectedSkill"
          :agents="agents"
          @close="handleCloseDialog"
          @save="handleDialogSave"
        />
      </Transition>

      <Transition name="fade">
        <div v-if="showInstallDialog" class="flex fixed inset-0 z-50 justify-center items-center bg-black/50">
          <div class="p-6 mx-4 w-full max-w-md rounded-2xl border bg-dark-800 border-dark-500">
            <h3 class="mb-4 text-lg font-semibold text-white">安装技能</h3>
            
            <div class="space-y-4">
              <div>
                <label class="block mb-2 text-sm text-gray-400">ZIP 文件路径</label>
                <div class="flex gap-2">
                  <input
                    v-model="installZipPath"
                    type="text"
                    placeholder="选择或输入 ZIP 文件路径"
                    class="flex-1 px-3 py-2 text-sm text-white rounded-lg border bg-dark-600 border-dark-500 focus:border-claw-500 focus:outline-none"
                  />
                  <button
                    @click="selectZipFile"
                    class="px-4 py-2 text-sm text-gray-300 rounded-lg bg-dark-600 hover:bg-dark-500"
                  >
                    浏览
                  </button>
                </div>
              </div>

              <div>
                <label class="block mb-2 text-sm text-gray-400">安装到智能体（可选）</label>
                <select
                  v-model="installAgentId"
                  class="px-3 py-2 w-full text-sm text-white rounded-lg border bg-dark-600 border-dark-500 focus:border-claw-500 focus:outline-none"
                >
                  <option :value="undefined">全局技能库</option>
                  <option v-for="agent in agents" :key="agent.id" :value="agent.id">
                    {{ agent.name }}
                  </option>
                </select>
                <p class="mt-1 text-xs text-gray-500">默认安装到全局技能库，选择智能体则仅该智能体可用</p>
              </div>
            </div>

            <div class="flex gap-3 justify-end mt-6">
              <button
                @click="showInstallDialog = false; installZipPath = ''; installAgentId = undefined"
                class="px-4 py-2 text-sm text-gray-400 rounded-lg hover:text-white"
              >
                取消
              </button>
              <button
                @click="handleInstallFromZip"
                :disabled="installing || !installZipPath"
                class="flex gap-2 items-center px-4 py-2 text-sm text-white rounded-lg bg-claw-500 hover:bg-claw-600 disabled:opacity-50"
              >
                <Loader2 v-if="installing" :size="16" class="animate-spin" />
                <Upload v-else :size="16" />
                安装
              </button>
            </div>
          </div>
        </div>
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
