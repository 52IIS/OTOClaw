<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { X, Loader2, Plus } from 'lucide-vue-next'
import { api, type SkillInfo, type AgentInfo, type CreateSkillParams } from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'

const props = defineProps<{
  editingSkill: SkillInfo | null
  agents: AgentInfo[]
}>()

const emit = defineEmits<{
  close: []
  save: []
}>()

const { alert } = useDialog()

const saving = ref(false)

const name = ref('')
const description = ref('')
const emoji = ref('')
const homepage = ref('')
const agentId = ref<string | undefined>(undefined)
const requiredEnv = ref<string[]>([])
const requiredBins = ref<string[]>([])
const newEnv = ref('')
const newBin = ref('')

const isEditing = computed(() => !!props.editingSkill)
const title = computed(() => isEditing.value ? '编辑技能' : '新建技能')

watch(() => props.editingSkill, (skill) => {
  if (skill) {
    name.value = skill.name
    description.value = skill.description
    emoji.value = skill.emoji || ''
    homepage.value = skill.homepage || ''
    requiredEnv.value = [...(skill.requiredEnv || [])]
    requiredBins.value = [...(skill.requiredBins || [])]
  } else {
    name.value = ''
    description.value = ''
    emoji.value = ''
    homepage.value = ''
    agentId.value = undefined
    requiredEnv.value = []
    requiredBins.value = []
  }
}, { immediate: true })

const addEnv = () => {
  if (newEnv.value && !requiredEnv.value.includes(newEnv.value)) {
    requiredEnv.value.push(newEnv.value)
    newEnv.value = ''
  }
}

const removeEnv = (index: number) => {
  requiredEnv.value.splice(index, 1)
}

const addBin = () => {
  if (newBin.value && !requiredBins.value.includes(newBin.value)) {
    requiredBins.value.push(newBin.value)
    newBin.value = ''
  }
}

const removeBin = (index: number) => {
  requiredBins.value.splice(index, 1)
}

const handleSave = async () => {
  if (!name.value.trim()) {
    await alert('请输入技能名称', { variant: 'warning', title: '提示' })
    return
  }

  if (!description.value.trim()) {
    await alert('请输入技能描述', { variant: 'warning', title: '提示' })
    return
  }

  saving.value = true

  try {
    if (isEditing.value && props.editingSkill) {
      await api.updateSkillConfig({
        skillId: props.editingSkill.id,
        enabled: true,
      })
    } else {
      const params: CreateSkillParams = {
        name: name.value.trim(),
        description: description.value.trim(),
        emoji: emoji.value.trim() || undefined,
        homepage: homepage.value.trim() || undefined,
        requiredEnv: requiredEnv.value,
        requiredBins: requiredBins.value,
        agentId: agentId.value,
      }
      await api.createSkill(params)
    }

    emit('save')
  } catch (e) {
    await alert('保存技能失败: ' + e, { variant: 'error', title: '保存失败' })
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div class="w-full max-w-lg mx-4 rounded-2xl bg-dark-800 border border-dark-500 max-h-[90vh] overflow-hidden flex flex-col">
      <div class="flex justify-between items-center p-4 border-b border-dark-500">
        <h3 class="text-lg font-semibold text-white">{{ title }}</h3>
        <button
          @click="emit('close')"
          class="p-1 text-gray-400 rounded-lg hover:text-white hover:bg-dark-600"
        >
          <X :size="20" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <div>
          <label class="block mb-2 text-sm text-gray-400">
            技能名称 <span class="text-red-400">*</span>
          </label>
          <input
            v-model="name"
            type="text"
            placeholder="输入技能名称"
            :disabled="isEditing"
            class="w-full px-3 py-2 text-sm text-white rounded-lg bg-dark-600 border border-dark-500 focus:border-claw-500 focus:outline-none disabled:opacity-50"
          />
        </div>

        <div>
          <label class="block mb-2 text-sm text-gray-400">
            技能描述 <span class="text-red-400">*</span>
          </label>
          <textarea
            v-model="description"
            placeholder="描述这个技能的功能和用途"
            rows="3"
            class="w-full px-3 py-2 text-sm text-white rounded-lg bg-dark-600 border border-dark-500 focus:border-claw-500 focus:outline-none resize-none"
          />
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block mb-2 text-sm text-gray-400">图标 (Emoji)</label>
            <input
              v-model="emoji"
              type="text"
              placeholder="如: 🚀"
              class="w-full px-3 py-2 text-sm text-white rounded-lg bg-dark-600 border border-dark-500 focus:border-claw-500 focus:outline-none"
            />
          </div>

          <div>
            <label class="block mb-2 text-sm text-gray-400">主页 URL</label>
            <input
              v-model="homepage"
              type="text"
              placeholder="https://..."
              class="w-full px-3 py-2 text-sm text-white rounded-lg bg-dark-600 border border-dark-500 focus:border-claw-500 focus:outline-none"
            />
          </div>
        </div>

        <div v-if="!isEditing">
          <label class="block mb-2 text-sm text-gray-400">安装到智能体</label>
          <select
            v-model="agentId"
            class="w-full px-3 py-2 text-sm text-white rounded-lg bg-dark-600 border border-dark-500 focus:border-claw-500 focus:outline-none"
          >
            <option :value="undefined">全局技能库</option>
            <option v-for="agent in agents" :key="agent.id" :value="agent.id">
              {{ agent.name }}
            </option>
          </select>
          <p class="mt-1 text-xs text-gray-500">默认安装到全局技能库，选择智能体则仅该智能体可用</p>
        </div>

        <div>
          <label class="block mb-2 text-sm text-gray-400">所需环境变量</label>
          <div class="flex gap-2 mb-2">
            <input
              v-model="newEnv"
              type="text"
              placeholder="如: GEMINI_API_KEY"
              class="flex-1 px-3 py-2 text-sm text-white rounded-lg bg-dark-600 border border-dark-500 focus:border-claw-500 focus:outline-none"
              @keyup.enter="addEnv"
            />
            <button
              @click="addEnv"
              class="px-3 py-2 text-sm text-white rounded-lg bg-dark-600 hover:bg-dark-500"
            >
              <Plus :size="16" />
            </button>
          </div>
          <div v-if="requiredEnv.length > 0" class="flex flex-wrap gap-2">
            <span
              v-for="(env, index) in requiredEnv"
              :key="env"
              class="flex gap-1 items-center px-2 py-1 text-xs text-claw-400 rounded bg-claw-500/20"
            >
              {{ env }}
              <button @click="removeEnv(index)" class="hover:text-white">
                <X :size="12" />
              </button>
            </span>
          </div>
        </div>

        <div>
          <label class="block mb-2 text-sm text-gray-400">所需二进制文件</label>
          <div class="flex gap-2 mb-2">
            <input
              v-model="newBin"
              type="text"
              placeholder="如: node, python"
              class="flex-1 px-3 py-2 text-sm text-white rounded-lg bg-dark-600 border border-dark-500 focus:border-claw-500 focus:outline-none"
              @keyup.enter="addBin"
            />
            <button
              @click="addBin"
              class="px-3 py-2 text-sm text-white rounded-lg bg-dark-600 hover:bg-dark-500"
            >
              <Plus :size="16" />
            </button>
          </div>
          <div v-if="requiredBins.length > 0" class="flex flex-wrap gap-2">
            <span
              v-for="(bin, index) in requiredBins"
              :key="bin"
              class="flex gap-1 items-center px-2 py-1 text-xs text-blue-400 rounded bg-blue-500/20"
            >
              {{ bin }}
              <button @click="removeBin(index)" class="hover:text-white">
                <X :size="12" />
              </button>
            </span>
          </div>
        </div>
      </div>

      <div class="flex justify-end gap-3 p-4 border-t border-dark-500">
        <button
          @click="emit('close')"
          class="px-4 py-2 text-sm text-gray-400 rounded-lg hover:text-white"
        >
          取消
        </button>
        <button
          @click="handleSave"
          :disabled="saving || !name.trim() || !description.trim()"
          class="flex gap-2 items-center px-4 py-2 text-sm text-white rounded-lg bg-claw-500 hover:bg-claw-600 disabled:opacity-50"
        >
          <Loader2 v-if="saving" :size="16" class="animate-spin" />
          {{ isEditing ? '保存' : '创建' }}
        </button>
      </div>
    </div>
  </div>
</template>
