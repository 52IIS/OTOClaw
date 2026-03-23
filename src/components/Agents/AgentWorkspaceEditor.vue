<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { 
  ChevronDown, 
  ChevronRight, 
  User, 
  Heart, 
  Shield, 
  Users, 
  Brain,
  Save,
  Loader2,
  FileText,
  RefreshCw
} from 'lucide-vue-next'
import clsx from 'clsx'
import { api, isTauri, type WorkspaceFilesResult } from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'

const props = defineProps<{
  agentId: string
  isNewAgent?: boolean
}>()

const emit = defineEmits<{
  saved: []
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
    color: 'text-blue-400'
  },
  {
    filename: 'SOUL.md',
    title: '人格特质',
    icon: Heart,
    description: '设定智能体的人格特质、语气风格与行为边界',
    color: 'text-pink-400'
  },
  {
    filename: 'AGENTS.md',
    title: '能力范围',
    icon: Shield,
    description: '规范智能体的能力范围与安全准则',
    color: 'text-green-400'
  },
  {
    filename: 'USER.md',
    title: '用户信息',
    icon: Users,
    description: '记录用户的个人信息与偏好',
    color: 'text-yellow-400'
  },
  {
    filename: 'MEMORY.md',
    title: '交互记忆',
    icon: Brain,
    description: '存储智能体与用户的长期交互记忆',
    color: 'text-purple-400'
  }
]

const loading = ref(false)
const saving = ref<string | null>(null)
const workspaceFiles = ref<Map<string, string>>(new Map())
const expandedFiles = ref<Set<string>>(new Set(['IDENTITY.md']))
const workspaceDir = ref('')

const loadWorkspaceFiles = async () => {
  if (!isTauri() || !props.agentId) return

  loading.value = true
  try {
    const result: WorkspaceFilesResult = await api.getAgentWorkspaceFiles(props.agentId)
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

onMounted(() => {
  loadWorkspaceFiles()
  expandedFiles.value = new Set(['IDENTITY.md'])
})

watch(() => props.agentId, () => {
  loadWorkspaceFiles()
  expandedFiles.value = new Set(['IDENTITY.md'])
})

watch(() => props.isNewAgent, () => {
  expandedFiles.value = new Set(['IDENTITY.md'])
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
  if (!isTauri() || !props.agentId) return

  const content = getFileContent(filename)
  saving.value = filename

  try {
    await api.saveAgentWorkspaceFile({
      agentId: props.agentId,
      filename,
      content
    })
    emit('saved')
  } catch (e) {
    await alert(`保存文件失败: ${e}`, { variant: 'error', title: '保存失败' })
  } finally {
    saving.value = null
  }
}
</script>

<template>
  <div class="space-y-3">
    <div class="flex justify-between items-center mb-4">
      <h4 class="text-sm font-medium text-gray-300">工作区文件</h4>
      <button
        @click="loadWorkspaceFiles"
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

    <div v-else class="space-y-2">
      <div
        v-for="config in fileConfigs"
        :key="config.filename"
        class="rounded-lg border bg-dark-700 border-dark-500 overflow-hidden"
      >
        <button
          @click="toggleFile(config.filename)"
          class="flex justify-between items-center w-full p-3 text-left transition-colors hover:bg-dark-600"
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

    <div v-if="workspaceDir" class="p-3 mt-4 rounded-lg bg-dark-700/50 border border-dark-600">
      <div class="flex gap-2 items-center text-xs text-gray-500">
        <FileText :size="12" />
        <span>工作区目录: </span>
        <code class="px-1.5 py-0.5 bg-dark-600 rounded text-claw-300 font-mono">{{ workspaceDir }}</code>
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
</style>
