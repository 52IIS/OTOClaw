<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  X,
  Loader2,
  AlertCircle,
  CheckCircle,
} from 'lucide-vue-next'
import clsx from 'clsx'
import {
  api,
  type CronJob,
  type CronSchedule,
  type CronPayload,
  type CronValidateResult,
  type AgentInfo,
} from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'

const props = defineProps<{
  editingJob: CronJob | null
  agents: AgentInfo[]
}>()

const emit = defineEmits<{
  close: []
  save: []
}>()

const { alert } = useDialog()

const saving = ref(false)
const validating = ref(false)
const validateResult = ref<CronValidateResult | null>(null)

const formData = ref({
  name: '',
  enabled: true,
  scheduleKind: 'cron' as 'at' | 'every' | 'cron',
  scheduleAt: '',
  scheduleEveryMs: 3600000,
  scheduleExpr: '0 * * * *',
  scheduleTz: '',
  payloadKind: 'agentTurn' as 'systemEvent' | 'agentTurn',
  payloadText: '',
  payloadMessage: '',
  payloadModel: '',
  agentId: '',
  sessionTarget: 'main',
  wakeMode: 'next-heartbeat',
  deleteAfterRun: false,
})

const isEditing = computed(() => !!props.editingJob)

const validateCronExpression = async () => {
  if (formData.value.scheduleKind !== 'cron' || !formData.value.scheduleExpr) {
    validateResult.value = null
    return
  }

  validating.value = true
  try {
    const result = await api.validateCronExpression(formData.value.scheduleExpr)
    validateResult.value = result
  } catch (e) {
    validateResult.value = {
      valid: false,
      error: String(e),
      nextRuns: [],
    }
  } finally {
    validating.value = false
  }
}

watch(
  () => props.editingJob,
  (job) => {
    if (job) {
      const schedule = job.schedule
      const payload = job.payload
      
      formData.value = {
        name: job.name,
        enabled: job.enabled,
        scheduleKind: job.schedule.kind,
        scheduleAt: 'at' in schedule ? (schedule.at || '') : '',
        scheduleEveryMs: 'everyMs' in schedule ? (schedule.everyMs || 3600000) : 3600000,
        scheduleExpr: 'expr' in schedule ? (schedule.expr || '0 * * * *') : '0 * * * *',
        scheduleTz: 'tz' in schedule ? (schedule.tz || '') : '',
        payloadKind: job.payload.kind,
        payloadText: 'text' in payload ? (payload.text || '') : '',
        payloadMessage: 'message' in payload ? (payload.message || '') : '',
        payloadModel: 'model' in payload ? (payload.model || '') : '',
        agentId: job.agentId || '',
        sessionTarget: job.sessionTarget,
        wakeMode: job.wakeMode,
        deleteAfterRun: job.deleteAfterRun,
      }
      validateCronExpression()
    }
  },
  { immediate: true }
)

const scheduleOptions = [
  { value: 'cron', label: 'Cron 表达式', desc: '使用 Cron 表达式定义复杂调度' },
  { value: 'every', label: '间隔执行', desc: '按固定时间间隔重复执行' },
  { value: 'at', label: '指定时间', desc: '在指定时间执行一次' },
]

const payloadOptions = [
  { value: 'agentTurn', label: 'Agent 消息', desc: '发送消息给智能体执行' },
  { value: 'systemEvent', label: '系统事件', desc: '触发系统事件' },
]

const everyOptions = [
  { value: 60000, label: '每分钟' },
  { value: 300000, label: '每5分钟' },
  { value: 900000, label: '每15分钟' },
  { value: 1800000, label: '每30分钟' },
  { value: 3600000, label: '每小时' },
  { value: 21600000, label: '每6小时' },
  { value: 43200000, label: '每12小时' },
  { value: 86400000, label: '每天' },
]

watch(
  () => formData.value.scheduleExpr,
  () => {
    validateCronExpression()
  }
)

const buildSchedule = (): CronSchedule => {
  switch (formData.value.scheduleKind) {
    case 'at':
      return { kind: 'at', at: formData.value.scheduleAt }
    case 'every':
      return { kind: 'every', everyMs: formData.value.scheduleEveryMs }
    case 'cron':
      return {
        kind: 'cron',
        expr: formData.value.scheduleExpr,
        tz: formData.value.scheduleTz || undefined,
      }
  }
}

const buildPayload = (): CronPayload => {
  switch (formData.value.payloadKind) {
    case 'systemEvent':
      return { kind: 'systemEvent', text: formData.value.payloadText }
    case 'agentTurn':
      return {
        kind: 'agentTurn',
        message: formData.value.payloadMessage,
        model: formData.value.payloadModel || undefined,
      }
  }
}

const handleSave = async () => {
  if (!formData.value.name.trim()) {
    await alert('请输入任务名称', { variant: 'warning', title: '验证失败' })
    return
  }

  if (formData.value.scheduleKind === 'cron' && validateResult.value && !validateResult.value.valid) {
    await alert('Cron 表达式无效，请检查', { variant: 'warning', title: '验证失败' })
    return
  }

  if (formData.value.payloadKind === 'systemEvent' && !formData.value.payloadText.trim()) {
    await alert('请输入系统事件文本', { variant: 'warning', title: '验证失败' })
    return
  }

  if (formData.value.payloadKind === 'agentTurn' && !formData.value.payloadMessage.trim()) {
    await alert('请输入 Agent 消息', { variant: 'warning', title: '验证失败' })
    return
  }

  saving.value = true
  try {
    const schedule = buildSchedule()
    const payload = buildPayload()

    if (isEditing.value && props.editingJob) {
      await api.updateCronJob({
        jobId: props.editingJob.id,
        name: formData.value.name,
        enabled: formData.value.enabled,
        schedule,
        payload,
        agentId: formData.value.agentId || undefined,
        sessionTarget: formData.value.sessionTarget,
        wakeMode: formData.value.wakeMode,
        deleteAfterRun: formData.value.deleteAfterRun,
      })
    } else {
      await api.createCronJob({
        name: formData.value.name,
        schedule,
        payload,
        enabled: formData.value.enabled,
        agentId: formData.value.agentId || undefined,
        sessionTarget: formData.value.sessionTarget,
        wakeMode: formData.value.wakeMode,
        deleteAfterRun: formData.value.deleteAfterRun,
      })
    }

    emit('save')
  } catch (e) {
    await alert('保存失败: ' + e, { variant: 'error', title: '保存失败' })
  } finally {
    saving.value = false
  }
}

const formatTimestamp = (ms: number) => {
  return new Date(ms).toLocaleString('zh-CN')
}
</script>

<template>
  <div class="flex fixed inset-0 z-50 justify-center items-center bg-black/50">
    <div class="mx-4 my-8 w-full max-w-2xl max-h-[90vh] rounded-2xl border bg-dark-800 border-dark-500 overflow-hidden flex flex-col">
      <div class="flex justify-between items-center px-6 py-4 border-b border-dark-600">
        <h3 class="text-lg font-semibold text-white">
          {{ isEditing ? '编辑定时任务' : '新建定时任务' }}
        </h3>
        <button @click="emit('close')" class="p-1 text-gray-400 rounded-lg hover:text-white hover:bg-dark-600">
          <X :size="20" />
        </button>
      </div>

      <div class="overflow-y-auto flex-1 p-6 space-y-6">
        <div>
          <label class="block mb-2 text-sm font-medium text-gray-300">任务名称 *</label>
          <input
            v-model="formData.name"
            type="text"
            placeholder="输入任务名称"
            class="input-base"
          />
        </div>

        <div class="flex gap-4 items-center">
          <label class="flex gap-2 items-center cursor-pointer">
            <input
              v-model="formData.enabled"
              type="checkbox"
              class="w-4 h-4 rounded border-dark-500 bg-dark-700 text-claw-500 focus:ring-claw-500"
            />
            <span class="text-sm text-gray-300">启用任务</span>
          </label>
          <label class="flex gap-2 items-center cursor-pointer">
            <input
              v-model="formData.deleteAfterRun"
              type="checkbox"
              class="w-4 h-4 rounded border-dark-500 bg-dark-700 text-claw-500 focus:ring-claw-500"
            />
            <span class="text-sm text-gray-300">执行后删除</span>
          </label>
        </div>

        <div>
          <label class="block mb-2 text-sm font-medium text-gray-300">调度方式</label>
          <div class="grid grid-cols-3 gap-3">
            <button
              v-for="opt in scheduleOptions"
              :key="opt.value"
              @click="formData.scheduleKind = opt.value as any"
              :class="clsx(
                'p-3 rounded-lg border text-left transition-all',
                formData.scheduleKind === opt.value
                  ? 'border-claw-500 bg-claw-500/10'
                  : 'border-dark-500 bg-dark-700 hover:border-dark-400'
              )"
            >
              <p class="text-sm font-medium text-white">{{ opt.label }}</p>
              <p class="mt-1 text-xs text-gray-500">{{ opt.desc }}</p>
            </button>
          </div>
        </div>

        <div v-if="formData.scheduleKind === 'cron'" class="space-y-4">
          <div>
            <label class="block mb-2 text-sm font-medium text-gray-300">Cron 表达式 *</label>
            <div class="flex gap-2">
              <input
                v-model="formData.scheduleExpr"
                type="text"
                placeholder="例如: 0 * * * * (每小时)"
                class="flex-1 input-base"
              />
              <div v-if="validating" class="flex items-center px-3">
                <Loader2 :size="16" class="animate-spin text-claw-400" />
              </div>
              <div v-else-if="validateResult" :class="clsx(
                'flex gap-1 items-center px-3 rounded-lg',
                validateResult.valid ? 'text-green-400' : 'text-red-400'
              )">
                <component :is="validateResult.valid ? CheckCircle : AlertCircle" :size="16" />
              </div>
            </div>
            <div v-if="validateResult?.error" class="mt-1 text-xs text-red-400">
              {{ validateResult.error }}
            </div>
            <div v-if="validateResult?.description" class="mt-1 text-xs text-gray-500">
              {{ validateResult.description }}
            </div>
          </div>

          <div>
            <label class="block mb-2 text-sm font-medium text-gray-300">时区（可选）</label>
            <input
              v-model="formData.scheduleTz"
              type="text"
              placeholder="例如: Asia/Shanghai，留空使用系统时区"
              class="input-base"
            />
          </div>

          <div v-if="validateResult?.valid && validateResult.nextRuns.length > 0" class="p-3 rounded-lg bg-dark-700">
            <p class="mb-2 text-xs font-medium text-gray-400">下次执行时间预览</p>
            <ul class="space-y-1">
              <li v-for="(ts, i) in validateResult.nextRuns.slice(0, 5)" :key="i" class="text-xs text-gray-500">
                {{ formatTimestamp(ts) }}
              </li>
            </ul>
          </div>
        </div>

        <div v-else-if="formData.scheduleKind === 'every'" class="space-y-4">
          <div>
            <label class="block mb-2 text-sm font-medium text-gray-300">执行间隔</label>
            <select v-model="formData.scheduleEveryMs" class="input-base">
              <option v-for="opt in everyOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
        </div>

        <div v-else-if="formData.scheduleKind === 'at'" class="space-y-4">
          <div>
            <label class="block mb-2 text-sm font-medium text-gray-300">执行时间</label>
            <input
              v-model="formData.scheduleAt"
              type="datetime-local"
              class="input-base"
            />
            <p class="mt-1 text-xs text-gray-500">任务将在指定时间执行一次</p>
          </div>
        </div>

        <div>
          <label class="block mb-2 text-sm font-medium text-gray-300">任务类型</label>
          <div class="grid grid-cols-2 gap-3">
            <button
              v-for="opt in payloadOptions"
              :key="opt.value"
              @click="formData.payloadKind = opt.value as any"
              :class="clsx(
                'p-3 rounded-lg border text-left transition-all',
                formData.payloadKind === opt.value
                  ? 'border-claw-500 bg-claw-500/10'
                  : 'border-dark-500 bg-dark-700 hover:border-dark-400'
              )"
            >
              <p class="text-sm font-medium text-white">{{ opt.label }}</p>
              <p class="mt-1 text-xs text-gray-500">{{ opt.desc }}</p>
            </button>
          </div>
        </div>

        <div v-if="formData.payloadKind === 'systemEvent'">
          <label class="block mb-2 text-sm font-medium text-gray-300">系统事件文本 *</label>
          <textarea
            v-model="formData.payloadText"
            rows="3"
            placeholder="输入要触发的系统事件内容"
            class="resize-none input-base"
          />
        </div>

        <div v-else-if="formData.payloadKind === 'agentTurn'" class="space-y-4">
          <div>
            <label class="block mb-2 text-sm font-medium text-gray-300">绑定智能体</label>
            <select v-model="formData.agentId" class="input-base">
              <option value="">默认智能体</option>
              <option v-for="agent in agents" :key="agent.id" :value="agent.id">
                {{ agent.name }}
              </option>
            </select>
          </div>

          <div>
            <label class="block mb-2 text-sm font-medium text-gray-300">消息内容 *</label>
            <textarea
              v-model="formData.payloadMessage"
              rows="3"
              placeholder="输入要发送给智能体的消息"
              class="resize-none input-base"
            />
          </div>

          <div>
            <label class="block mb-2 text-sm font-medium text-gray-300">指定模型（可选）</label>
            <input
              v-model="formData.payloadModel"
              type="text"
              placeholder="例如: openai/gpt-4，留空使用默认模型"
              class="input-base"
            />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block mb-2 text-sm font-medium text-gray-300">会话目标</label>
            <select v-model="formData.sessionTarget" class="input-base">
              <option value="main">主会话</option>
              <option value="isolated">隔离会话</option>
            </select>
          </div>

          <div>
            <label class="block mb-2 text-sm font-medium text-gray-300">唤醒模式</label>
            <select v-model="formData.wakeMode" class="input-base">
              <option value="next-heartbeat">下次心跳</option>
              <option value="now">立即唤醒</option>
            </select>
          </div>
        </div>
      </div>

      <div class="flex gap-3 justify-end px-6 py-4 border-t border-dark-600">
        <button
          @click="emit('close')"
          class="px-4 py-2 text-sm text-gray-400 rounded-lg hover:text-white"
        >
          取消
        </button>
        <button
          @click="handleSave"
          :disabled="saving"
          class="flex gap-2 items-center px-4 py-2 text-sm text-white rounded-lg bg-claw-500 hover:bg-claw-600 disabled:opacity-50"
        >
          <Loader2 v-if="saving" :size="16" class="animate-spin" />
          {{ saving ? '保存中...' : '保存' }}
        </button>
      </div>
    </div>
  </div>
</template>
