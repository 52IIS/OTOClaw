<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  Plus,
  Edit3,
  Trash2,
  Check,
  X,
  Loader2,
  Users,
  Eye,
  EyeOff,
  AlertCircle,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { useDialog } from '../../composables/useDialog'

const { alert, confirm } = useDialog()

interface WecomAccount {
  accountId: string
  name: string
  enabled: boolean
  configured: boolean
  botId: string
  websocketUrl: string
  agentConfigured: boolean
  callbackConfigured: boolean
}

interface AccountFormData {
  accountId: string
  name: string
  botId: string
  secret: string
  websocketUrl: string
  enabled: boolean
  agent: {
    corpId: string
    corpSecret: string
    agentId: string
    callback: {
      token: string
      encodingAESKey: string
      path: string
    }
  }
}

const accounts = ref<WecomAccount[]>([])
const loading = ref(false)
const saving = ref(false)
const showEditor = ref(false)
const editingAccount = ref<string | null>(null)
const formData = ref<AccountFormData>({
  accountId: '',
  name: '',
  botId: '',
  secret: '',
  websocketUrl: 'wss://openws.work.weixin.qq.com',
  enabled: true,
  agent: {
    corpId: '',
    corpSecret: '',
    agentId: '',
    callback: {
      token: '',
      encodingAESKey: '',
      path: '/api/channels/wecom/callback',
    },
  },
})

const visiblePasswords = ref<Set<string>>(new Set())

const togglePasswordVisibility = (fieldKey: string) => {
  if (visiblePasswords.value.has(fieldKey)) {
    visiblePasswords.value.delete(fieldKey)
  } else {
    visiblePasswords.value.add(fieldKey)
  }
}

const loadAccounts = async () => {
  loading.value = true
  try {
    const result = await invoke<WecomAccount[]>('get_wecom_accounts')
    accounts.value = result
  } catch (e) {
    console.error('加载企业微信账号列表失败:', e)
    accounts.value = []
  } finally {
    loading.value = false
  }
}

const resetForm = () => {
  formData.value = {
    accountId: '',
    name: '',
    botId: '',
    secret: '',
    websocketUrl: 'wss://openws.work.weixin.qq.com',
    enabled: true,
    agent: {
      corpId: '',
      corpSecret: '',
      agentId: '',
      callback: {
        token: '',
        encodingAESKey: '',
        path: '/api/channels/wecom/callback',
      },
    },
  }
  editingAccount.value = null
}

const handleAddAccount = () => {
  resetForm()
  showEditor.value = true
}

const handleEditAccount = (account: WecomAccount) => {
  editingAccount.value = account.accountId
  invoke<AccountFormData>('get_wecom_account', { accountId: account.accountId })
    .then((data) => {
      formData.value = data
      showEditor.value = true
    })
    .catch((e) => {
      alert('加载账号详情失败: ' + e, { variant: 'error', title: '加载失败' })
    })
}

const handleDeleteAccount = async (account: WecomAccount) => {
  const confirmed = await confirm(`确定要删除账号 "${account.name}" 吗？此操作不可撤销。`, {
    title: '删除确认',
    variant: 'error',
  })
  
  if (!confirmed) return
  
  try {
    await invoke('delete_wecom_account', { accountId: account.accountId })
    await loadAccounts()
    await alert('账号已删除', { variant: 'success', title: '删除成功' })
  } catch (e) {
    await alert('删除失败: ' + e, { variant: 'error', title: '删除失败' })
  }
}

const handleSaveAccount = async () => {
  if (!formData.value.accountId.trim()) {
    await alert('请输入账号 ID', { variant: 'warning', title: '验证失败' })
    return
  }
  
  if (!formData.value.botId.trim() || !formData.value.secret.trim()) {
    await alert('Bot ID 和 Secret 为必填项', { variant: 'warning', title: '验证失败' })
    return
  }
  
  saving.value = true
  try {
    if (editingAccount.value) {
      await invoke('update_wecom_account', {
        accountId: editingAccount.value,
        config: formData.value,
      })
    } else {
      await invoke('create_wecom_account', { config: formData.value })
    }
    showEditor.value = false
    await loadAccounts()
    await alert('账号保存成功', { variant: 'success', title: '保存成功' })
  } catch (e) {
    await alert('保存失败: ' + e, { variant: 'error', title: '保存失败' })
  } finally {
    saving.value = false
  }
}

const handleCancelEdit = () => {
  showEditor.value = false
  resetForm()
}

onMounted(() => {
  loadAccounts()
})
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Users :size="18" class="text-blue-400" />
        <h4 class="text-sm font-medium text-white">多账号管理</h4>
        <span class="text-xs text-gray-500">({{ accounts.length }} 个账号)</span>
      </div>
      <button
        @click="handleAddAccount"
        class="btn-secondary flex items-center gap-1.5 text-sm py-1.5 px-3"
      >
        <Plus :size="14" />
        添加账号
      </button>
    </div>

    <div v-if="loading" class="flex justify-center items-center py-8">
      <Loader2 :size="24" class="animate-spin text-blue-400" />
    </div>

    <div v-else-if="accounts.length === 0" class="py-8 text-center text-gray-500">
      <AlertCircle :size="24" class="mx-auto mb-2 opacity-50" />
      <p class="text-sm">暂无企业微信账号</p>
      <p class="text-xs mt-1">点击上方"添加账号"按钮创建第一个账号</p>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="account in accounts"
        :key="account.accountId"
        class="p-4 bg-dark-700 rounded-xl border border-dark-500 hover:border-dark-400 transition-colors"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1">
            <div class="flex items-center gap-2">
              <h5 class="font-medium text-white">{{ account.name }}</h5>
              <span
                :class="clsx(
                  'px-2 py-0.5 text-xs rounded-full',
                  account.enabled
                    ? 'bg-green-500/20 text-green-400'
                    : 'bg-gray-500/20 text-gray-400'
                )"
              >
                {{ account.enabled ? '已启用' : '已禁用' }}
              </span>
            </div>
            <p class="text-xs text-gray-500 mt-1">ID: {{ account.accountId }}</p>
            <div class="flex flex-wrap gap-2 mt-2">
              <span
                :class="clsx(
                  'px-2 py-0.5 text-xs rounded',
                  account.configured
                    ? 'bg-blue-500/20 text-blue-400'
                    : 'bg-gray-500/20 text-gray-400'
                )"
              >
                {{ account.configured ? '基础配置完成' : '基础配置未完成' }}
              </span>
              <span
                v-if="account.agentConfigured"
                class="px-2 py-0.5 text-xs rounded bg-purple-500/20 text-purple-400"
              >
                Agent 已配置
              </span>
              <span
                v-if="account.callbackConfigured"
                class="px-2 py-0.5 text-xs rounded bg-amber-500/20 text-amber-400"
              >
                回调已配置
              </span>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button
              @click="handleEditAccount(account)"
              class="p-2 text-gray-400 hover:text-white hover:bg-dark-600 rounded-lg transition-colors"
              title="编辑"
            >
              <Edit3 :size="16" />
            </button>
            <button
              @click="handleDeleteAccount(account)"
              class="p-2 text-gray-400 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
              title="删除"
            >
              <Trash2 :size="16" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <Transition name="fade">
      <div
        v-if="showEditor"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm"
        @click.self="handleCancelEdit"
      >
        <div
          class="w-full max-w-2xl max-h-[90vh] bg-dark-800 rounded-2xl border border-dark-500 shadow-2xl flex flex-col"
          @click.stop
        >
          <div class="flex items-center justify-between p-4 border-b border-dark-600 shrink-0">
            <h3 class="text-lg font-semibold text-white">
              {{ editingAccount ? '编辑账号' : '添加账号' }}
            </h3>
            <button
              @click="handleCancelEdit"
              class="p-1 text-gray-400 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
            >
              <X :size="20" />
            </button>
          </div>

          <div class="flex-1 overflow-y-auto p-4 space-y-4">
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm text-gray-400 mb-2">
                  账号 ID <span class="text-red-400">*</span>
                </label>
                <input
                  v-model="formData.accountId"
                  :disabled="!!editingAccount"
                  type="text"
                  placeholder="唯一标识符 (如: default, account1)"
                  class="input-base disabled:opacity-50 disabled:cursor-not-allowed"
                />
              </div>
              <div>
                <label class="block text-sm text-gray-400 mb-2">
                  账号名称 <span class="text-red-400">*</span>
                </label>
                <input
                  v-model="formData.name"
                  type="text"
                  placeholder="显示名称"
                  class="input-base"
                />
              </div>
            </div>

            <div class="pt-2 border-t border-dark-600">
              <h4 class="text-sm font-medium text-white mb-3">基础配置</h4>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm text-gray-400 mb-2">
                    Bot ID <span class="text-red-400">*</span>
                  </label>
                  <input
                    v-model="formData.botId"
                    type="text"
                    placeholder="企业微信机器人 Bot ID"
                    class="input-base"
                  />
                </div>
                <div class="relative">
                  <label class="block text-sm text-gray-400 mb-2">
                    Secret <span class="text-red-400">*</span>
                  </label>
                  <input
                    :type="visiblePasswords.has('secret') ? 'text' : 'password'"
                    v-model="formData.secret"
                    placeholder="企业微信机器人 Secret"
                    class="input-base pr-10"
                  />
                  <button
                    type="button"
                    @click="togglePasswordVisibility('secret')"
                    class="absolute right-3 top-9 text-gray-500 hover:text-white transition-colors"
                  >
                    <EyeOff v-if="visiblePasswords.has('secret')" :size="18" />
                    <Eye v-else :size="18" />
                  </button>
                </div>
              </div>
              <div class="mt-4">
                <label class="block text-sm text-gray-400 mb-2">WebSocket URL</label>
                <input
                  v-model="formData.websocketUrl"
                  type="text"
                  placeholder="wss://openws.work.weixin.qq.com"
                  class="input-base"
                />
              </div>
            </div>

            <div class="pt-2 border-t border-dark-600">
              <h4 class="text-sm font-medium text-white mb-3">Agent 配置 (可选)</h4>
              <div class="grid grid-cols-3 gap-4">
                <div>
                  <label class="block text-sm text-gray-400 mb-2">Corp ID</label>
                  <input
                    v-model="formData.agent.corpId"
                    type="text"
                    placeholder="企业 ID"
                    class="input-base"
                  />
                </div>
                <div class="relative">
                  <label class="block text-sm text-gray-400 mb-2">Corp Secret</label>
                  <input
                    :type="visiblePasswords.has('corpSecret') ? 'text' : 'password'"
                    v-model="formData.agent.corpSecret"
                    placeholder="应用 Secret"
                    class="input-base pr-10"
                  />
                  <button
                    type="button"
                    @click="togglePasswordVisibility('corpSecret')"
                    class="absolute right-3 top-9 text-gray-500 hover:text-white transition-colors"
                  >
                    <EyeOff v-if="visiblePasswords.has('corpSecret')" :size="18" />
                    <Eye v-else :size="18" />
                  </button>
                </div>
                <div>
                  <label class="block text-sm text-gray-400 mb-2">Agent ID</label>
                  <input
                    v-model="formData.agent.agentId"
                    type="text"
                    placeholder="应用 AgentId"
                    class="input-base"
                  />
                </div>
              </div>
            </div>

            <div class="pt-2 border-t border-dark-600">
              <h4 class="text-sm font-medium text-white mb-3">回调配置 (可选)</h4>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm text-gray-400 mb-2">回调 Token</label>
                  <input
                    v-model="formData.agent.callback.token"
                    type="text"
                    placeholder="回调验证 Token"
                    class="input-base"
                  />
                </div>
                <div>
                  <label class="block text-sm text-gray-400 mb-2">Encoding AES Key</label>
                  <input
                    v-model="formData.agent.callback.encodingAESKey"
                    type="text"
                    placeholder="消息加密密钥"
                    class="input-base"
                  />
                </div>
              </div>
              <div class="mt-4">
                <label class="block text-sm text-gray-400 mb-2">回调路径</label>
                <input
                  v-model="formData.agent.callback.path"
                  type="text"
                  placeholder="/api/channels/wecom/callback"
                  class="input-base"
                />
              </div>
            </div>

            <div class="flex items-center gap-2 pt-2">
              <input
                type="checkbox"
                id="account-enabled"
                v-model="formData.enabled"
                class="w-4 h-4 rounded border-dark-500 bg-dark-700 text-blue-500 focus:ring-blue-500"
              />
              <label for="account-enabled" class="text-sm text-gray-300">启用此账号</label>
            </div>
          </div>

          <div class="flex gap-3 justify-end p-4 border-t border-dark-600 shrink-0">
            <button
              type="button"
              @click="handleCancelEdit"
              class="px-4 py-2 text-gray-400 rounded-lg transition-colors hover:text-white hover:bg-dark-600"
            >
              取消
            </button>
            <button
              @click="handleSaveAccount"
              :disabled="saving"
              :class="clsx(
                'flex gap-2 items-center px-4 py-2 rounded-lg font-medium transition-all',
                'bg-blue-500 text-white hover:bg-blue-600',
                'disabled:opacity-50 disabled:cursor-not-allowed'
              )"
            >
              <Loader2 v-if="saving" :size="16" class="animate-spin" />
              <Check v-else :size="16" />
              {{ editingAccount ? '保存修改' : '创建账号' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
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
