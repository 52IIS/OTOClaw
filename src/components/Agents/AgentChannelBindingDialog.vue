<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { X, Loader2, Link, Check, AlertCircle } from 'lucide-vue-next'
import clsx from 'clsx'
import { api, isTauri, type AgentInfo, type AgentChannelBinding } from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'

const props = defineProps<{
  agent: AgentInfo
}>()

const emit = defineEmits<{
  close: []
  save: []
}>()

const { alert } = useDialog()

const loading = ref(false)
const saving = ref(false)
const availableChannels = ref<string[]>([])
const selectedChannels = ref<string[]>([])

const channelNames: Record<string, string> = {
  telegram: 'Telegram',
  discord: 'Discord',
  slack: 'Slack',
  feishu: '飞书',
  imessage: 'iMessage',
  whatsapp: 'WhatsApp',
  wecom: '企业微信',
  dingtalk: '钉钉',
}

const loadData = async () => {
  if (!isTauri()) return

  loading.value = true
  try {
    const [channels, bindings] = await Promise.all([
      api.getAvailableChannels(),
      api.getAgentBindings(props.agent.id),
    ])
    availableChannels.value = channels
    selectedChannels.value = bindings.bindings.map((b) => b.channel)
  } catch (e) {
    console.error('加载渠道关联数据失败:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadData()
})

watch(
  () => props.agent,
  () => {
    loadData()
  }
)

const toggleChannel = (channel: string) => {
  const index = selectedChannels.value.indexOf(channel)
  if (index > -1) {
    selectedChannels.value.splice(index, 1)
  } else {
    selectedChannels.value.push(channel)
  }
}

const isChannelSelected = (channel: string) => {
  return selectedChannels.value.includes(channel)
}

const handleSave = async () => {
  if (!isTauri()) {
    emit('save')
    return
  }

  saving.value = true
  try {
    const bindings: AgentChannelBinding[] = selectedChannels.value.map((channel) => ({
      channel,
    }))
    await api.setAgentBindings({
      agentId: props.agent.id,
      bindings,
    })
    emit('save')
  } catch (e) {
    await alert('保存失败: ' + e, { variant: 'error', title: '保存失败' })
  } finally {
    saving.value = false
  }
}

const handleClose = () => {
  emit('close')
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
    <div
      class="w-full max-w-lg max-h-[90vh] bg-dark-800 rounded-2xl border border-dark-500 shadow-2xl flex flex-col"
      @click.stop
    >
      <div class="flex items-center justify-between p-4 border-b border-dark-600 shrink-0">
        <div class="flex items-center gap-2">
          <Link :size="20" class="text-claw-400" />
          <h3 class="text-lg font-semibold text-white">关联渠道 - {{ agent.name }}</h3>
        </div>
        <button
          @click="handleClose"
          class="p-1 text-gray-400 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
        >
          <X :size="20" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-4">
        <div v-if="loading" class="flex justify-center items-center py-12">
          <Loader2 :size="24" class="animate-spin text-claw-400" />
        </div>

        <div v-else-if="availableChannels.length === 0" class="py-12 text-center text-gray-500">
          <AlertCircle :size="24" class="mx-auto mb-2 opacity-50" />
          <p>暂无可用渠道</p>
          <p class="text-xs mt-1">请先在渠道管理中配置至少一个渠道</p>
        </div>

        <div v-else class="space-y-3">
          <p class="text-sm text-gray-400 mb-4">
            选择要与智能体 <span class="text-claw-400">{{ agent.name }}</span> 关联的渠道。
            关联后，来自这些渠道的消息将由此智能体处理。
          </p>

          <div
            v-for="channel in availableChannels"
            :key="channel"
            @click="toggleChannel(channel)"
            :class="clsx(
              'p-4 rounded-xl border cursor-pointer transition-all',
              isChannelSelected(channel)
                ? 'bg-claw-500/10 border-claw-500/50 hover:border-claw-500'
                : 'bg-dark-700 border-dark-500 hover:border-dark-400'
            )"
          >
            <div class="flex gap-3 items-center">
              <div
                :class="clsx(
                  'flex justify-center items-center w-10 h-10 rounded-lg shrink-0',
                  isChannelSelected(channel) ? 'bg-claw-500/20' : 'bg-dark-600'
                )"
              >
                <Check v-if="isChannelSelected(channel)" :size="20" class="text-claw-400" />
                <Link v-else :size="20" class="text-gray-500" />
              </div>
              <div class="flex-1">
                <h4 class="font-medium text-white">
                  {{ channelNames[channel] || channel }}
                </h4>
                <p class="text-sm text-gray-500">
                  {{ isChannelSelected(channel) ? '已关联' : '点击关联' }}
                </p>
              </div>
            </div>
          </div>

          <div class="mt-4 p-3 rounded-lg bg-dark-700/50 border border-dark-600">
            <p class="text-xs text-gray-500">
              已选择 {{ selectedChannels.length }} / {{ availableChannels.length }} 个渠道
            </p>
          </div>
        </div>
      </div>

      <div class="flex gap-3 justify-end p-4 border-t border-dark-600 shrink-0">
        <button
          type="button"
          @click="handleClose"
          class="px-4 py-2 text-gray-400 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
        >
          取消
        </button>
        <button
          @click="handleSave"
          :disabled="saving || loading"
          :class="clsx(
            'flex gap-2 items-center px-4 py-2 rounded-lg font-medium transition-all',
            'bg-claw-500 text-white hover:bg-claw-600',
            'disabled:opacity-50 disabled:cursor-not-allowed'
          )"
        >
          <Loader2 v-if="saving" :size="16" class="animate-spin" />
          <Check v-else :size="16" />
          保存关联
        </button>
      </div>
    </div>
  </div>
</template>
