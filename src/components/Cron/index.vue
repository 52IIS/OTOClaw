<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  Loader2,
  Plus,
  Clock,
  Play,
  Pause,
  Pencil,
  Trash2,
  Copy,
  CheckCircle,
  XCircle,
  Calendar,
  Timer,
  Download,
  Upload,
  History,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { save, open } from '@tauri-apps/plugin-dialog'
import { writeFile, readFile } from '@tauri-apps/plugin-fs'
import {
  api,
  isTauri,
  type CronJob,
  type CronJobsListResult,
  type CronStats,
  type AgentsListResult,
} from '../../lib/tauri'
import { useDialog } from '../../composables/useDialog'
import CronJobDialog from './CronJobDialog.vue'
import RunLogDialog from './RunLogDialog.vue'
import RunLogFilesViewer from './RunLogFilesViewer.vue'

const { alert, confirm } = useDialog()

const loading = ref(true)
const jobsResult = ref<CronJobsListResult | null>(null)
const stats = ref<CronStats | null>(null)
const agentsResult = ref<AgentsListResult | null>(null)
const error = ref<string | null>(null)

const showAddDialog = ref(false)
const showEditDialog = ref(false)
const showLogDialog = ref(false)
const showLogFilesViewer = ref(false)
const selectedJob = ref<CronJob | null>(null)
const runningJobId = ref<string | null>(null)

const jobs = computed(() => jobsResult.value?.jobs || [])
const totalCount = computed(() => jobsResult.value?.total || 0)
const enabledCount = computed(() => jobsResult.value?.enabledCount || 0)
const runningCount = computed(() => jobsResult.value?.runningCount || 0)
const agents = computed(() => agentsResult.value?.agents || [])

const loadData = async () => {
  if (!isTauri()) {
    loading.value = false
    return
  }

  loading.value = true
  error.value = null

  try {
    const [jobsData, statsData, agentsData] = await Promise.all([
      api.getCronJobs(),
      api.getCronStats(),
      api.getAgentsList(),
    ])
    jobsResult.value = jobsData
    stats.value = statsData
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

const handleAddJob = () => {
  selectedJob.value = null
  showAddDialog.value = true
}

const handleEditJob = (job: CronJob) => {
  selectedJob.value = job
  showEditDialog.value = true
}

const handleViewLogs = (job: CronJob) => {
  selectedJob.value = job
  showLogDialog.value = true
}

const handleCloseLogDialog = () => {
  showLogDialog.value = false
  selectedJob.value = null
}

const handleOpenLogFilesViewer = () => {
  showLogFilesViewer.value = true
}

const handleCloseLogFilesViewer = () => {
  showLogFilesViewer.value = false
}

const handleCloseDialog = () => {
  showAddDialog.value = false
  showEditDialog.value = false
  selectedJob.value = null
}

const handleDialogSave = () => {
  loadData()
  handleCloseDialog()
}

const handleToggleJob = async (job: CronJob) => {
  try {
    await api.toggleCronJob(job.id, !job.enabled)
    await loadData()
  } catch (e) {
    await alert('切换任务状态失败: ' + e, { variant: 'error', title: '操作失败' })
  }
}

const handleRunJob = async (job: CronJob) => {
  const confirmed = await confirm(
    `确定要立即执行任务 "${job.name}" 吗？`,
    {
      title: '执行确认',
      variant: 'info',
      confirmText: '执行',
      cancelText: '取消',
    }
  )

  if (!confirmed) return

  runningJobId.value = job.id
  try {
    const result = await api.runCronJob(job.id)
    if (result.status === 'enqueued') {
      await alert(result.summary || '已加入任务运行队列，运行结果请查看执行日志', {
        variant: 'info',
        title: '任务已提交',
      })
    } else if (result.status === 'ok') {
      await alert(`任务执行成功\n${result.summary || ''}`, {
        variant: 'success',
        title: '执行成功',
      })
    } else {
      await alert(`任务执行失败: ${result.error || '未知错误'}`, {
        variant: 'error',
        title: '执行失败',
      })
    }
    await loadData()
  } catch (e) {
    await alert('执行任务失败: ' + e, { variant: 'error', title: '执行失败' })
  } finally {
    runningJobId.value = null
  }
}

const handleDeleteJob = async (job: CronJob) => {
  const confirmed = await confirm(
    `确定要删除任务 "${job.name}" 吗？\n\n删除后无法恢复。`,
    {
      title: '删除确认',
      variant: 'warning',
      confirmText: '删除',
      cancelText: '取消',
    }
  )

  if (!confirmed) return

  try {
    await api.deleteCronJob(job.id)
    await loadData()
  } catch (e) {
    await alert('删除任务失败: ' + e, { variant: 'error', title: '删除失败' })
  }
}

const handleDuplicateJob = async (job: CronJob) => {
  try {
    await api.duplicateCronJob(job.id)
    await loadData()
    await alert('任务已复制', { variant: 'success', title: '复制成功' })
  } catch (e) {
    await alert('复制任务失败: ' + e, { variant: 'error', title: '复制失败' })
  }
}

const handleExportJobs = async () => {
  try {
    const json = await api.exportCronJobs()
    
    if (isTauri()) {
      const defaultPath = `cron-jobs-${new Date().toISOString().slice(0, 10)}.json`
      const filePath = await save({
        defaultPath,
        filters: [{ name: 'JSON', extensions: ['json'] }],
        title: '导出定时任务'
      })
      
      if (filePath) {
        const encoder = new TextEncoder()
        await writeFile(filePath, encoder.encode(json))
        await alert(`任务已导出到:\n${filePath}`, { variant: 'success', title: '导出成功' })
      }
    } else {
      const blob = new Blob([json], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `cron-jobs-${new Date().toISOString().slice(0, 10)}.json`
      a.click()
      URL.revokeObjectURL(url)
      await alert('任务已导出', { variant: 'success', title: '导出成功' })
    }
  } catch (e) {
    await alert('导出任务失败: ' + e, { variant: 'error', title: '导出失败' })
  }
}

const handleImportJobs = async () => {
  if (isTauri()) {
    try {
      const filePath = await open({
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }],
        title: '导入定时任务'
      })
      
      if (filePath && typeof filePath === 'string') {
        const fileData = await readFile(filePath)
        const text = new TextDecoder().decode(fileData)
        await api.importCronJobs(text)
        await loadData()
        await alert('任务已导入', { variant: 'success', title: '导入成功' })
      }
    } catch (e) {
      await alert('导入任务失败: ' + e, { variant: 'error', title: '导入失败' })
    }
  } else {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.json'
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (!file) return

      try {
        const text = await file.text()
        await api.importCronJobs(text)
        await loadData()
        await alert('任务已导入', { variant: 'success', title: '导入成功' })
      } catch (e) {
        await alert('导入任务失败: ' + e, { variant: 'error', title: '导入失败' })
      }
    }
    input.click()
  }
}

const getScheduleDisplay = (job: CronJob) => {
  const { schedule } = job
  switch (schedule.kind) {
    case 'at':
      return `一次性: ${formatTime(schedule.at || '')}`
    case 'every':
      return `间隔: ${formatDuration(schedule.everyMs || 0)}`
    case 'cron':
      return `Cron: ${schedule.expr}`
    default:
      return '未知'
  }
}

const formatTime = (timestamp: string) => {
  try {
    return new Date(timestamp).toLocaleString('zh-CN')
  } catch {
    return timestamp
  }
}

const formatDuration = (ms: number) => {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${Math.floor(ms / 1000)}秒`
  if (ms < 3600000) return `${Math.floor(ms / 60000)}分钟`
  if (ms < 86400000) return `${Math.floor(ms / 3600000)}小时`
  return `${Math.floor(ms / 86400000)}天`
}

const formatNextRun = (ms?: number) => {
  if (!ms) return '未计划'
  const now = Date.now()
  const diff = ms - now
  if (diff < 0) return '已过期'
  if (diff < 60000) return '即将执行'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟后`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时后`
  return new Date(ms).toLocaleString('zh-CN')
}

const getStatusBadge = (job: CronJob) => {
  if (!job || !job.state) {
    return { text: '未知', class: 'text-gray-400 bg-gray-500/20', icon: Clock }
  }
  if (job.state.runningAtMs) {
    return { text: '运行中', class: 'text-blue-400 bg-blue-500/20', icon: Loader2 }
  }
  if (!job.enabled) {
    return { text: '已暂停', class: 'text-gray-400 bg-gray-500/20', icon: Pause }
  }
  if (job.state.lastRunStatus === 'error') {
    return { text: '错误', class: 'text-red-400 bg-red-500/20', icon: XCircle }
  }
  if (job.state.lastRunStatus === 'ok') {
    return { text: '正常', class: 'text-green-400 bg-green-500/20', icon: CheckCircle }
  }
  return { text: '待执行', class: 'text-yellow-400 bg-yellow-500/20', icon: Clock }
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
              <Clock :size="22" class="text-claw-400" />
              定时任务管理
            </h2>
            <p class="mt-1 text-sm text-gray-500">创建和管理定时任务，自动执行周期性操作</p>
          </div>
          <div class="flex gap-2">
            <button @click="handleImportJobs" class="flex gap-2 items-center px-4 py-2 text-sm text-gray-300 rounded-lg transition-colors bg-dark-600 hover:bg-dark-500">
              <Upload :size="16" />
              导入
            </button>
            <button @click="handleExportJobs" class="flex gap-2 items-center px-4 py-2 text-sm text-gray-300 rounded-lg transition-colors bg-dark-600 hover:bg-dark-500">
              <Download :size="16" />
              导出
            </button>
            <button @click="handleAddJob" class="flex gap-2 items-center btn-primary">
              <Plus :size="16" />
              新建任务
            </button>
          </div>
        </div>

        <div class="grid grid-cols-4 gap-4">
          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-claw-500/20">
                <Clock :size="20" class="text-claw-400" />
              </div>
              <div>
                <p class="text-2xl font-bold text-white">{{ totalCount }}</p>
                <p class="text-sm text-gray-500">任务总数</p>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-green-500/20">
                <CheckCircle :size="20" class="text-green-400" />
              </div>
              <div>
                <p class="text-2xl font-bold text-white">{{ enabledCount }}</p>
                <p class="text-sm text-gray-500">已启用</p>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-blue-500/20">
                <Play :size="20" class="text-blue-400" />
              </div>
              <div>
                <p class="text-2xl font-bold text-white">{{ runningCount }}</p>
                <p class="text-sm text-gray-500">运行中</p>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-purple-500/20">
                <Timer :size="20" class="text-purple-400" />
              </div>
              <div>
                <p class="text-2xl font-bold text-white">{{ stats?.runsLast24h || 0 }}</p>
                <p class="text-sm text-gray-500">24h执行</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="space-y-4">
        <h3 class="flex gap-2 items-center text-lg font-medium text-white">
          <Clock :size="18" class="text-gray-500" />
          任务列表
        </h3>

        <div v-if="loading" class="flex justify-center items-center py-12">
          <Loader2 :size="24" class="animate-spin text-claw-400" />
        </div>

        <div v-else-if="jobs.length === 0" class="p-8 text-center rounded-xl border bg-dark-700 border-dark-500">
          <div class="flex justify-center items-center mx-auto mb-4 w-16 h-16 rounded-full bg-dark-600">
            <Clock :size="24" class="text-gray-500" />
          </div>
          <p class="mb-4 text-gray-400">还没有任何定时任务</p>
          <button @click="handleAddJob" class="btn-primary">创建第一个任务</button>
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="job in jobs"
            :key="job.id"
            :class="clsx(
              'rounded-xl border transition-all',
              job.enabled
                ? 'bg-dark-700 border-dark-500 hover:border-dark-400'
                : 'bg-dark-700/50 border-dark-600'
            )"
          >
            <div class="p-4">
              <div class="flex gap-4 items-start">
                <div class="flex justify-center items-center w-12 h-12 text-xl rounded-xl bg-dark-600">
                  <Clock :size="24" class="text-claw-400" />
                </div>

                <div class="flex-1 min-w-0">
                  <div class="flex gap-2 items-center mb-1">
                    <h4 class="font-medium text-white">{{ job.name }}</h4>
                    <span
                      :class="clsx(
                        'flex gap-1 items-center px-2 py-0.5 text-xs rounded',
                        getStatusBadge(job).class
                      )"
                    >
                      <component :is="getStatusBadge(job).icon" :size="10" :class="{ 'animate-spin': job.state.runningAtMs }" />
                      {{ getStatusBadge(job).text }}
                    </span>
                  </div>

                  <p class="mb-2 text-sm text-gray-500">
                    {{ getScheduleDisplay(job) }}
                  </p>

                  <div class="flex flex-wrap gap-4 text-xs text-gray-500">
                    <span class="flex gap-1 items-center">
                      <Calendar :size="12" />
                      下次执行: {{ formatNextRun(job.state.nextRunAtMs) }}
                    </span>
                    <span v-if="job.state.lastRunAtMs" class="flex gap-1 items-center">
                      <Timer :size="12" />
                      上次: {{ new Date(job.state.lastRunAtMs).toLocaleString('zh-CN') }}
                    </span>
                    <span v-if="job.state.lastDurationMs" class="flex gap-1 items-center">
                      耗时: {{ formatDuration(job.state.lastDurationMs) }}
                    </span>
                    <span v-if="job.agentId" class="flex gap-1 items-center text-claw-400">
                      Agent: {{ agents.find(a => a.id === job.agentId)?.name || job.agentId }}
                    </span>
                  </div>

                  <div v-if="job.state.lastError" class="p-2 mt-2 text-xs text-red-400 rounded bg-red-500/10">
                    上次错误: {{ job.state.lastError }}
                  </div>
                </div>

                <div class="flex gap-2 items-center" @click.stop>
                  <button
                    @click="handleToggleJob(job)"
                    :class="clsx(
                      'flex gap-1 items-center px-3 py-1.5 text-sm rounded-lg transition-colors',
                      job.enabled
                        ? 'text-yellow-400 hover:bg-yellow-500/10'
                        : 'text-green-400 hover:bg-green-500/10'
                    )"
                    :title="job.enabled ? '暂停' : '启用'"
                  >
                    <component :is="job.enabled ? Pause : Play" :size="14" />
                  </button>

                  <button
                    @click="handleRunJob(job)"
                    :disabled="runningJobId === job.id || !!job.state.runningAtMs"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-blue-400 rounded-lg transition-colors hover:bg-blue-500/10 disabled:opacity-50"
                    title="立即执行"
                  >
                    <Loader2 v-if="runningJobId === job.id" :size="14" class="animate-spin" />
                    <Play v-else :size="14" />
                  </button>

                  <button
                    @click="handleDuplicateJob(job)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-dark-600 hover:text-claw-400"
                    title="复制"
                  >
                    <Copy :size="14" />
                  </button>

                  <button
                    @click="handleViewLogs(job)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-dark-600 hover:text-claw-400"
                    title="查看日志"
                  >
                    <History :size="14" />
                  </button>

                  <button
                    @click="handleEditJob(job)"
                    class="flex gap-1 items-center px-3 py-1.5 text-sm text-gray-400 rounded-lg transition-colors hover:bg-dark-600 hover:text-claw-400"
                    title="编辑"
                  >
                    <Pencil :size="14" />
                  </button>

                  <button
                    @click="handleDeleteJob(job)"
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
        <h4 class="mb-2 text-sm font-medium text-gray-400">任务说明</h4>
        <ul class="space-y-1 text-sm text-gray-500">
          <li>• 定时任务支持三种调度方式：指定时间、间隔执行、Cron表达式</li>
          <li>• 任务可以绑定到特定智能体，执行Agent消息或系统事件</li>
          <li>• 支持任务的启用/暂停、立即执行、复制等操作</li>
          <li>• 可以导入/导出任务配置，方便备份和迁移</li>
        </ul>
      </div>

      <div class="p-4 rounded-xl border bg-dark-700/50 border-dark-500">
        <div class="flex justify-between items-center mb-2">
          <h4 class="text-sm font-medium text-gray-400">执行日志文件</h4>
          <button
            @click="handleOpenLogFilesViewer"
            class="flex gap-2 items-center px-3 py-1.5 text-sm rounded-lg transition-colors text-claw-400 hover:bg-dark-600"
          >
            <History :size="14" />
            查看所有日志文件
          </button>
        </div>
        <p class="text-sm text-gray-500">
          浏览 ~/.openclaw/cron/runs 目录下的所有执行日志文件，查看 OpenClaw 自动产生的任务执行记录。
        </p>
      </div>
    </div>

    <Teleport to="body">
      <Transition name="fade">
        <CronJobDialog
          v-if="showAddDialog || showEditDialog"
          :editing-job="selectedJob"
          :agents="agents"
          @close="handleCloseDialog"
          @save="handleDialogSave"
        />
      </Transition>
    </Teleport>

    <RunLogDialog
      v-if="showLogDialog && selectedJob"
      :visible="showLogDialog"
      :job-id="selectedJob.id"
      :job-name="selectedJob.name"
      @close="handleCloseLogDialog"
    />

    <RunLogFilesViewer
      :visible="showLogFilesViewer"
      @close="handleCloseLogFilesViewer"
    />
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
</style>
