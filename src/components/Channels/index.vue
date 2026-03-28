<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  MessageCircle,
  Hash,
  Slack,
  MessagesSquare,
  MessageSquare,
  Check,
  X,
  Loader2,
  ChevronRight,
  Apple,
  Bell,
  Play,
  QrCode,
  CheckCircle,
  XCircle,
  Download,
  Package,
  AlertTriangle,
  Trash2,
} from 'lucide-vue-next'
import clsx from 'clsx'
import { useDialog } from '../../composables/useDialog'


const { alert } = useDialog()



interface FeishuPluginStatus {
  installed: boolean
  version: string | null
  plugin_name: string | null
}

interface ChannelConfig {
  id: string
  channel_type: string
  enabled: boolean
  config: Record<string, unknown>
}

interface ChannelField {
  key: string
  label: string
  type: 'text' | 'password' | 'select'
  placeholder?: string
  options?: { value: string; label: string }[]
  required?: boolean
}

interface ChannelInfo {
  name: string
  icon: any
  color: string
  fields: ChannelField[]
  helpText?: string
}

const channelInfo: Record<string, ChannelInfo> = {
  telegram: {
    name: 'Telegram',
    icon: MessageCircle,
    color: 'text-blue-400',
    fields: [
      { key: 'botToken', label: 'Bot Token', type: 'password', placeholder: '从 @BotFather 获取', required: true },
      { key: 'userId', label: 'User ID', type: 'text', placeholder: '你的 Telegram User ID', required: true },
      { key: 'dmPolicy', label: '私聊策略', type: 'select', options: [
        { value: 'pairing', label: '配对模式' },
        { value: 'open', label: '开放模式' },
        { value: 'disabled', label: '禁用' },
      ]},
      { key: 'groupPolicy', label: '群组策略', type: 'select', options: [
        { value: 'allowlist', label: '白名单' },
        { value: 'open', label: '开放' },
        { value: 'disabled', label: '禁用' },
      ]},
    ],
    helpText: '1. 搜索 @BotFather 发送 /newbot 获取 Token  2. 搜索 @userinfobot 获取 User ID',
  },
  discord: {
    name: 'Discord',
    icon: Hash,
    color: 'text-indigo-400',
    fields: [
      { key: 'botToken', label: 'Bot Token', type: 'password', placeholder: 'Discord Bot Token', required: true },
      { key: 'testChannelId', label: '测试 Channel ID', type: 'text', placeholder: '用于发送测试消息的频道 ID (可选)' },
      { key: 'dmPolicy', label: '私聊策略', type: 'select', options: [
        { value: 'pairing', label: '配对模式' },
        { value: 'open', label: '开放模式' },
        { value: 'disabled', label: '禁用' },
      ]},
    ],
    helpText: '从 Discord Developer Portal 获取，开启开发者模式可复制 Channel ID',
  },
  slack: {
    name: 'Slack',
    icon: Slack,
    color: 'text-purple-400',
    fields: [
      { key: 'botToken', label: 'Bot Token', type: 'password', placeholder: 'xoxb-...', required: true },
      { key: 'appToken', label: 'App Token', type: 'password', placeholder: 'xapp-...' },
      { key: 'testChannelId', label: '测试 Channel ID', type: 'text', placeholder: '用于发送测试消息的频道 ID (可选)' },
    ],
    helpText: '从 Slack API 后台获取，Channel ID 可从频道详情复制',
  },
  feishu: {
    name: '飞书',
    icon: MessagesSquare,
    color: 'text-blue-500',
    fields: [
      { key: 'appId', label: 'App ID', type: 'text', placeholder: '飞书应用 App ID', required: true },
      { key: 'appSecret', label: 'App Secret', type: 'password', placeholder: '飞书应用 App Secret', required: true },
      { key: 'testChatId', label: '测试 Chat ID', type: 'text', placeholder: '用于发送测试消息的群聊/用户 ID (可选)' },
      { key: 'connectionMode', label: '连接模式', type: 'select', options: [
        { value: 'websocket', label: 'WebSocket (推荐)' },
        { value: 'webhook', label: 'Webhook' },
      ]},
      { key: 'domain', label: '部署区域', type: 'select', options: [
        { value: 'feishu', label: '国内 (feishu.cn)' },
        { value: 'lark', label: '海外 (larksuite.com)' },
      ]},
      { key: 'requireMention', label: '需要 @提及', type: 'select', options: [
        { value: 'true', label: '是' },
        { value: 'false', label: '否' },
      ]},
    ],
    helpText: '从飞书开放平台获取凭证，Chat ID 可从群聊设置中获取',
  },
  imessage: {
    name: 'iMessage',
    icon: Apple,
    color: 'text-green-400',
    fields: [
      { key: 'dmPolicy', label: '私聊策略', type: 'select', options: [
        { value: 'pairing', label: '配对模式' },
        { value: 'open', label: '开放模式' },
        { value: 'disabled', label: '禁用' },
      ]},
      { key: 'groupPolicy', label: '群组策略', type: 'select', options: [
        { value: 'allowlist', label: '白名单' },
        { value: 'open', label: '开放' },
        { value: 'disabled', label: '禁用' },
      ]},
    ],
    helpText: '仅支持 macOS，需要授权消息访问权限',
  },
  whatsapp: {
    name: 'WhatsApp',
    icon: MessageCircle,
    color: 'text-green-500',
    fields: [
      { key: 'dmPolicy', label: '私聊策略', type: 'select', options: [
        { value: 'pairing', label: '配对模式' },
        { value: 'open', label: '开放模式' },
        { value: 'disabled', label: '禁用' },
      ]},
      { key: 'groupPolicy', label: '群组策略', type: 'select', options: [
        { value: 'allowlist', label: '白名单' },
        { value: 'open', label: '开放' },
        { value: 'disabled', label: '禁用' },
      ]},
    ],
    helpText: '需要扫描二维码登录，运行: openclaw channels login --channel whatsapp',
  },
  wecom: {
    name: '企业微信',
    icon: MessageSquare,
    color: 'text-blue-500',
    fields: [
      { key: 'botId', label: 'Bot ID', type: 'text', placeholder: '企业微信机器人 Bot ID', required: true },
      { key: 'secret', label: 'Secret', type: 'password', placeholder: '企业微信机器人 Secret', required: true },
      { key: 'websocketUrl', label: 'WebSocket URL', type: 'text', placeholder: 'wss://openws.work.weixin.qq.com', required: true },
    ],
    helpText: '企业微信机器人配置，需要先安装官方插件: openclaw plugins install @wecom/wecom-openclaw-plugin',
  },
  qq: {
    name: 'QQ',
    icon: MessageCircle,
    color: 'text-green-500',
    fields: [
      { key: 'appId', label: 'App ID', type: 'text', placeholder: 'QQ 机器人 App ID', required: true },
      { key: 'appSecret', label: 'App Secret', type: 'password', placeholder: 'QQ 机器人 App Secret', required: true },
      { key: 'botId', label: 'Bot ID', type: 'text', placeholder: 'QQ 机器人 Bot ID', required: true },
    ],
    helpText: 'QQ 机器人配置，需要先安装官方插件: openclaw plugins install @sliverp/qqbot',
  },
  dingtalk: {
    name: '钉钉',
    icon: Bell,
    color: 'text-blue-600',
    fields: [
      { key: 'appKey', label: 'App Key', type: 'text', placeholder: '钉钉应用 App Key' },
      { key: 'appSecret', label: 'App Secret', type: 'password', placeholder: '钉钉应用 App Secret' },
    ],
    helpText: '从钉钉开放平台获取',
  },
}

interface TestResult {
  success: boolean
  message: string
  error: string | null
}

const channels = ref<ChannelConfig[]>([])
const loading = ref(true)
const selectedChannel = ref<string | null>(null)
const configForm = ref<Record<string, string>>({})
const saving = ref(false)
const testingChannel = ref(false)
const testResult = ref<TestResult | null>(null)
const loginLoading = ref(false)
const clearing = ref(false)
const showClearConfirm = ref(false)

const feishuPluginStatus = ref<FeishuPluginStatus | null>(null)
const feishuPluginLoading = ref(false)
const feishuPluginInstalling = ref(false)

const wecomPluginStatus = ref<FeishuPluginStatus | null>(null)
const wecomPluginLoading = ref(false)
const wecomPluginInstalling = ref(false)

const qqPluginStatus = ref<FeishuPluginStatus | null>(null)
const qqPluginLoading = ref(false)
const qqPluginInstalling = ref(false)

const visiblePasswords = ref<Set<string>>(new Set())

const togglePasswordVisibility = (fieldKey: string) => {
  if (visiblePasswords.value.has(fieldKey)) {
    visiblePasswords.value.delete(fieldKey)
  } else {
    visiblePasswords.value.add(fieldKey)
  }
}

const checkFeishuPlugin = async () => {
  feishuPluginLoading.value = true
  try {
    const status = await invoke<FeishuPluginStatus>('check_feishu_plugin')
    feishuPluginStatus.value = status
  } catch (e) {
    console.error('检查飞书插件失败:', e)
    feishuPluginStatus.value = { installed: false, version: null, plugin_name: null }
  } finally {
    feishuPluginLoading.value = false
  }
}

const handleInstallFeishuPlugin = async () => {
  feishuPluginInstalling.value = true
  try {
    const result = await invoke<string>('install_feishu_plugin')
    await alert(result, { title: '安装结果', variant: 'success' })
    await checkFeishuPlugin()
  } catch (e) {
    await alert('安装失败: ' + e, { title: '安装失败', variant: 'error' })
  } finally {
    feishuPluginInstalling.value = false
  }
}

const checkWecomPlugin = async () => {
  wecomPluginLoading.value = true
  try {
    const status = await invoke<FeishuPluginStatus>('check_feishu_plugin', { pluginName: '@wecom/wecom-openclaw-plugin' })
    wecomPluginStatus.value = status
  } catch (e) {
    console.error('检查企业微信插件失败:', e)
    wecomPluginStatus.value = { installed: false, version: null, plugin_name: null }
  } finally {
    wecomPluginLoading.value = false
  }
}

const handleInstallWecomPlugin = async () => {
  wecomPluginInstalling.value = true
  try {
    const result = await invoke<string>('install_feishu_plugin', { pluginName: '@wecom/wecom-openclaw-plugin' })
    await alert(result, { title: '安装结果', variant: 'success' })
    await checkWecomPlugin()
  } catch (e) {
    await alert('安装失败: ' + e, { title: '安装失败', variant: 'error' })
  } finally {
    wecomPluginInstalling.value = false
  }
}

const checkQQPlugin = async () => {
  qqPluginLoading.value = true
  try {
    const status = await invoke<FeishuPluginStatus>('check_feishu_plugin', { pluginName: '@sliverp/qqbot' })
    qqPluginStatus.value = status
  } catch (e) {
    console.error('检查QQ插件失败:', e)
    qqPluginStatus.value = { installed: false, version: null, plugin_name: null }
  } finally {
    qqPluginLoading.value = false
  }
}

const handleInstallQQPlugin = async () => {
  qqPluginInstalling.value = true
  try {
    const result = await invoke<string>('install_feishu_plugin', { pluginName: '@sliverp/qqbot' })
    await alert(result, { title: '安装结果', variant: 'success' })
    await checkQQPlugin()
  } catch (e) {
    await alert('安装失败: ' + e, { title: '安装失败', variant: 'error' })
  } finally {
    qqPluginInstalling.value = false
  }
}

const handleClearConfig = async () => {
  if (!selectedChannel.value) return
  
  const channel = channels.value.find((c) => c.id === selectedChannel.value)
  const channelName = channel ? channelInfo[channel.channel_type]?.name || channel.channel_type : selectedChannel.value
  
  showClearConfirm.value = false
  clearing.value = true
  try {
    await invoke('clear_channel_config', { channelId: selectedChannel.value })
    configForm.value = {}
    await fetchChannels()
    testResult.value = {
      success: true,
      message: `${channelName} 配置已清空`,
      error: null,
    }
  } catch (e) {
    testResult.value = {
      success: false,
      message: '清空失败',
      error: String(e),
    }
  } finally {
    clearing.value = false
  }
}

const handleQuickTest = async () => {
  if (!selectedChannel.value) return
  
  testingChannel.value = true
  testResult.value = null
  
  try {
    const result = await invoke<{
      success: boolean
      channel: string
      message: string
      error: string | null
    }>('test_channel', { channelType: selectedChannel.value })
    
    testResult.value = {
      success: result.success,
      message: result.message,
      error: result.error,
    }
  } catch (e) {
    testResult.value = {
      success: false,
      message: '测试失败',
      error: String(e),
    }
  } finally {
    testingChannel.value = false
  }
}

const handleWhatsAppLogin = async () => {
  loginLoading.value = true
  try {
    await invoke('start_channel_login', { channelType: 'whatsapp' })
    
    const pollInterval = setInterval(async () => {
      try {
        const result = await invoke<{
          success: boolean
          message: string
        }>('test_channel', { channelType: 'whatsapp' })
        
        if (result.success) {
          clearInterval(pollInterval)
          loginLoading.value = false
          await fetchChannels()
          testResult.value = {
            success: true,
            message: 'WhatsApp 登录成功！',
            error: null,
          }
        }
      } catch {
      }
    }, 3000)
    
    setTimeout(() => {
      clearInterval(pollInterval)
      loginLoading.value = false
    }, 60000)
    
    await alert('请在弹出的终端窗口中扫描二维码完成登录\n\n登录成功后界面会自动更新', {
      title: 'WhatsApp 登录',
      variant: 'info'
    })
  } catch (e) {
    await alert('启动登录失败: ' + e, { title: '登录失败', variant: 'error' })
    loginLoading.value = false
  }
}

const fetchChannels = async () => {
  try {
    const result = await invoke<ChannelConfig[]>('get_channels_config')
    channels.value = result
    return result
  } catch (e) {
    console.error('获取渠道配置失败:', e)
    return []
  }
}

onMounted(async () => {
  try {
    const result = await fetchChannels()
    
    const configured = result.find((c) => c.enabled)
    if (configured) {
      handleChannelSelect(configured.id, result)
    }
  } finally {
    loading.value = false
  }
})

const handleChannelSelect = (channelId: string, channelList?: ChannelConfig[]) => {
  selectedChannel.value = channelId
  testResult.value = null
  
  const list = channelList || channels.value
  const channel = list.find((c) => c.id === channelId)
  
  if (channel) {
    const form: Record<string, string> = {}
    Object.entries(channel.config).forEach(([key, value]) => {
      if (typeof value === 'boolean') {
        form[key] = value ? 'true' : 'false'
      } else {
        form[key] = String(value ?? '')
      }
    })
    configForm.value = form
    
    if (channel.channel_type === 'feishu') {
      checkFeishuPlugin()
    } else if (channel.channel_type === 'wecom') {
      checkWecomPlugin()
    } else if (channel.channel_type === 'qq') {
      checkQQPlugin()
    }
  } else {
    configForm.value = {}
  }
}

const handleSave = async () => {
  if (!selectedChannel.value) return
  
  saving.value = true
  try {
    const channel = channels.value.find((c) => c.id === selectedChannel.value)
    if (!channel) return
    
    const config: Record<string, unknown> = {}
    Object.entries(configForm.value).forEach(([key, value]) => {
      if (value === 'true') {
        config[key] = true
      } else if (value === 'false') {
        config[key] = false
      } else if (value) {
        config[key] = value
      }
    })
    
    await invoke('save_channel_config', {
      channel: {
        ...channel,
        config,
      },
    })
    
    await fetchChannels()
    
    await alert('渠道配置已保存！', { title: '保存成功', variant: 'success' })
  } catch (e) {
    console.error('保存失败:', e)
    await alert('保存失败: ' + e, { title: '保存失败', variant: 'error' })
  } finally {
    saving.value = false
  }
}

const currentChannel = computed(() => channels.value.find((c) => c.id === selectedChannel.value))
const currentInfo = computed(() => currentChannel.value ? channelInfo[currentChannel.value.channel_type] : null)

const hasValidConfig = (channel: ChannelConfig) => {
  const info = channelInfo[channel.channel_type]
  if (!info) return channel.enabled
  
  const requiredFields = info.fields.filter((f) => f.required)
  if (requiredFields.length === 0) return channel.enabled
  
  return requiredFields.some((field) => {
    const value = channel.config[field.key]
    return value !== undefined && value !== null && value !== ''
  })
}
</script>

<template>
  <div class="overflow-y-auto pr-2 h-full scroll-container">
    <div>
      <div class="grid grid-cols-1 gap-6 md:grid-cols-3">
        <div class="space-y-2 md:col-span-1">
          <h3 class="px-1 mb-3 text-sm font-medium text-gray-400">消息渠道</h3>
          <button
            v-for="channel in channels"
            :key="channel.id"
            @click="handleChannelSelect(channel.id)"
            :class="clsx(
              'w-full flex items-center gap-3 p-4 rounded-xl border transition-all',
              selectedChannel === channel.id
                ? 'bg-dark-600 border-claw-500'
                : 'bg-dark-700 border-dark-500 hover:border-dark-400'
            )"
          >
            <div
              :class="clsx(
                'w-10 h-10 rounded-lg flex items-center justify-center',
                hasValidConfig(channel) ? 'bg-dark-500' : 'bg-dark-600'
              )"
            >
              <component
                :is="channelInfo[channel.channel_type]?.icon || MessageSquare"
                :size="20"
                :class="channelInfo[channel.channel_type]?.color || 'text-gray-400'"
              />
            </div>
            <div class="flex-1 text-left">
              <p :class="['text-sm font-medium', selectedChannel === channel.id ? 'text-white' : 'text-gray-300']">
                {{ channelInfo[channel.channel_type]?.name || channel.channel_type }}
              </p>
              <div class="flex gap-2 items-center mt-1">
                <template v-if="hasValidConfig(channel)">
                  <Check :size="12" class="text-green-400" />
                  <span class="text-xs text-green-400">已配置</span>
                </template>
                <template v-else>
                  <X :size="12" class="text-gray-500" />
                  <span class="text-xs text-gray-500">未配置</span>
                </template>
              </div>
            </div>
            <ChevronRight
              :size="16"
              :class="selectedChannel === channel.id ? 'text-claw-400' : 'text-gray-600'"
            />
          </button>
        </div>

        <div class="md:col-span-2">
          <template v-if="currentChannel && currentInfo">
            <Transition name="fade-slide" mode="out-in">
              <div :key="selectedChannel ?? ''" class="p-6 rounded-2xl border bg-dark-700 border-dark-500">
                <div class="flex gap-3 items-center mb-4">
                  <div :class="['w-10 h-10 rounded-lg flex items-center justify-center bg-dark-500', currentInfo.color]">
                    <component :is="currentInfo.icon" :size="20" />
                  </div>
                  <div>
                    <h3 class="text-lg font-semibold text-white">配置 {{ currentInfo.name }}</h3>
                    <p v-if="currentInfo.helpText" class="text-xs text-gray-500">{{ currentInfo.helpText }}</p>
                  </div>
                </div>

                <template v-if="currentChannel.channel_type === 'feishu'">
                  <div class="mb-4">
                    <div v-if="feishuPluginLoading" class="flex gap-3 items-center p-4 rounded-xl border bg-dark-600 border-dark-500">
                      <Loader2 :size="20" class="text-gray-400 animate-spin" />
                      <span class="text-gray-400">正在检查飞书插件状态...</span>
                    </div>
                    <div v-else-if="feishuPluginStatus?.installed" class="flex gap-3 items-center p-4 rounded-xl border bg-green-500/10 border-green-500/30">
                      <Package :size="20" class="text-green-400" />
                      <div class="flex-1">
                        <p class="font-medium text-green-400">飞书插件已安装</p>
                        <p class="mt-0.5 text-xs text-gray-400">
                          {{ feishuPluginStatus.plugin_name || '@m1heng-clawd/feishu' }}
                          {{ feishuPluginStatus.version ? ` v${feishuPluginStatus.version}` : '' }}
                        </p>
                      </div>
                      <CheckCircle :size="16" class="text-green-400" />
                    </div>
                    <div v-else class="p-4 rounded-xl border bg-amber-500/10 border-amber-500/30">
                      <div class="flex gap-3 items-start">
                        <AlertTriangle :size="20" class="mt-0.5 text-amber-400" />
                        <div class="flex-1">
                          <p class="font-medium text-amber-400">需要安装飞书插件</p>
                          <p class="mt-1 text-xs text-gray-400">飞书渠道需要先安装 @m1heng-clawd/feishu 插件才能使用。</p>
                          <div class="flex flex-wrap gap-2 mt-3">
                            <button
                              @click="handleInstallFeishuPlugin"
                              :disabled="feishuPluginInstalling"
                              class="flex gap-2 items-center py-2 text-sm btn-primary"
                            >
                              <Loader2 v-if="feishuPluginInstalling" :size="14" class="animate-spin" />
                              <Download v-else :size="14" />
                              {{ feishuPluginInstalling ? '安装中...' : '一键安装插件' }}
                            </button>
                            <button
                              @click="checkFeishuPlugin"
                              :disabled="feishuPluginLoading"
                              class="flex gap-2 items-center py-2 text-sm btn-secondary"
                            >
                              刷新状态
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </template>

                <template v-if="currentChannel.channel_type === 'wecom'">
                  <div class="mb-4">
                    <div v-if="wecomPluginLoading" class="flex gap-3 items-center p-4 rounded-xl border bg-dark-600 border-dark-500">
                      <Loader2 :size="20" class="text-gray-400 animate-spin" />
                      <span class="text-gray-400">正在检查企业微信插件状态...</span>
                    </div>
                    <div v-else-if="wecomPluginStatus?.installed" class="flex gap-3 items-center p-4 rounded-xl border bg-green-500/10 border-green-500/30">
                      <Package :size="20" class="text-green-400" />
                      <div class="flex-1">
                        <p class="font-medium text-green-400">企业微信插件已安装</p>
                        <p class="mt-0.5 text-xs text-gray-400">
                          {{ wecomPluginStatus.plugin_name || '@wecom/wecom-openclaw-plugin' }}
                          {{ wecomPluginStatus.version ? ` v${wecomPluginStatus.version}` : '' }}
                        </p>
                      </div>
                      <CheckCircle :size="16" class="text-green-400" />
                    </div>
                    <div v-else class="p-4 rounded-xl border bg-amber-500/10 border-amber-500/30">
                      <div class="flex gap-3 items-start">
                        <AlertTriangle :size="20" class="mt-0.5 text-amber-400" />
                        <div class="flex-1">
                          <p class="font-medium text-amber-400">需要安装企业微信插件</p>
                          <p class="mt-1 text-xs text-gray-400">企业微信渠道需要先安装 @wecom/wecom-openclaw-plugin 插件才能使用。</p>
                          <div class="flex flex-wrap gap-2 mt-3">
                            <button
                              @click="handleInstallWecomPlugin"
                              :disabled="wecomPluginInstalling"
                              class="flex gap-2 items-center py-2 text-sm btn-primary"
                            >
                              <Loader2 v-if="wecomPluginInstalling" :size="14" class="animate-spin" />
                              <Download v-else :size="14" />
                              {{ wecomPluginInstalling ? '安装中...' : '一键安装插件' }}
                            </button>
                            <button
                              @click="checkWecomPlugin"
                              :disabled="wecomPluginLoading"
                              class="flex gap-2 items-center py-2 text-sm btn-secondary"
                            >
                              刷新状态
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </template>

                <template v-if="currentChannel.channel_type === 'qq'">
                  <div class="mb-4">
                    <div v-if="qqPluginLoading" class="flex gap-3 items-center p-4 rounded-xl border bg-dark-600 border-dark-500">
                      <Loader2 :size="20" class="text-gray-400 animate-spin" />
                      <span class="text-gray-400">正在检查QQ插件状态...</span>
                    </div>
                    <div v-else-if="qqPluginStatus?.installed" class="flex gap-3 items-center p-4 rounded-xl border bg-green-500/10 border-green-500/30">
                      <Package :size="20" class="text-green-400" />
                      <div class="flex-1">
                        <p class="font-medium text-green-400">QQ插件已安装</p>
                        <p class="mt-0.5 text-xs text-gray-400">
                          {{ qqPluginStatus.plugin_name || '@sliverp/qqbot' }}
                          {{ qqPluginStatus.version ? ` v${qqPluginStatus.version}` : '' }}
                        </p>
                      </div>
                      <CheckCircle :size="16" class="text-green-400" />
                    </div>
                    <div v-else class="p-4 rounded-xl border bg-amber-500/10 border-amber-500/30">
                      <div class="flex gap-3 items-start">
                        <AlertTriangle :size="20" class="mt-0.5 text-amber-400" />
                        <div class="flex-1">
                          <p class="font-medium text-amber-400">需要安装QQ插件</p>
                          <p class="mt-1 text-xs text-gray-400">QQ渠道需要先安装 @sliverp/qqbot 插件才能使用。</p>
                          <div class="flex flex-wrap gap-2 mt-3">
                            <button
                              @click="handleInstallQQPlugin"
                              :disabled="qqPluginInstalling"
                              class="flex gap-2 items-center py-2 text-sm btn-primary"
                            >
                              <Loader2 v-if="qqPluginInstalling" :size="14" class="animate-spin" />
                              <Download v-else :size="14" />
                              {{ qqPluginInstalling ? '安装中...' : '一键安装插件' }}
                            </button>
                            <button
                              @click="checkQQPlugin"
                              :disabled="qqPluginLoading"
                              class="flex gap-3 items-center py-2 text-sm btn-secondary"
                            >
                              刷新状态
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </template>


                <div class="space-y-4">
                  <div v-for="field in currentInfo.fields" :key="field.key">
                    <label class="block mb-2 text-sm text-gray-400">
                      {{ field.label }}
                      <span v-if="field.required" class="ml-1 text-red-400">*</span>
                      <span v-if="configForm[field.key]" class="ml-2 text-xs text-green-500">✓</span>
                    </label>
                    
                    <select
                      v-if="field.type === 'select'"
                      v-model="configForm[field.key]"
                      class="input-base"
                    >
                      <option value="">请选择...</option>
                      <option v-for="opt in field.options" :key="opt.value" :value="opt.value">
                        {{ opt.label }}
                      </option>
                    </select>
                    
                    <div v-else-if="field.type === 'password'" class="relative">
                      <input
                        :type="visiblePasswords.has(field.key) ? 'text' : 'password'"
                        v-model="configForm[field.key]"
                        :placeholder="field.placeholder"
                        class="pr-10 input-base"
                      />
                      <button
                        type="button"
                        @click="togglePasswordVisibility(field.key)"
                        class="absolute right-3 top-1/2 text-gray-500 transition-colors -translate-y-1/2 hover:text-white"
                        :title="visiblePasswords.has(field.key) ? '隐藏' : '显示'"
                      >
                        <EyeOff v-if="visiblePasswords.has(field.key)" :size="18" />
                        <Eye v-else :size="18" />
                      </button>
                    </div>
                    
                    <input
                      v-else
                      :type="field.type"
                      v-model="configForm[field.key]"
                      :placeholder="field.placeholder"
                      class="input-base"
                    />
                  </div>
                </div>

                <template v-if="currentChannel.channel_type === 'whatsapp'">
                    <div class="p-4 rounded-xl border bg-green-500/10 border-green-500/30">
                      <div class="flex gap-3 items-center mb-3">
                        <QrCode :size="24" class="text-green-400" />
                        <div>
                          <p class="font-medium text-white">扫码登录</p>
                          <p class="text-xs text-gray-400">WhatsApp 需要扫描二维码登录</p>
                        </div>
                      </div>
                      <div class="flex gap-2">
                        <button
                          @click="handleWhatsAppLogin"
                          :disabled="loginLoading"
                          class="flex flex-1 gap-2 justify-center items-center btn-secondary"
                        >
                          <Loader2 v-if="loginLoading" :size="16" class="animate-spin" />
                          <QrCode v-else :size="16" />
                          {{ loginLoading ? '等待登录...' : '启动扫码登录' }}
                        </button>
                        <button
                          @click="async () => { await fetchChannels(); handleQuickTest(); }"
                          :disabled="testingChannel"
                          class="flex gap-2 justify-center items-center px-4 btn-secondary"
                          title="刷新状态"
                        >
                          <Loader2 v-if="testingChannel" :size="16" class="animate-spin" />
                          <Check v-else :size="16" />
                        </button>
                      </div>
                      <p class="mt-2 text-xs text-center text-gray-500">
                        登录成功后点击右侧按钮刷新状态，或运行: openclaw channels login --channel whatsapp
                      </p>
                    </div>
                  </template>

                  <div class="flex flex-wrap gap-3 items-center pt-4 border-t border-dark-500">
                    <button
                      @click="handleSave"
                      :disabled="saving"
                      class="flex gap-2 items-center btn-primary"
                    >
                      <Loader2 v-if="saving" :size="16" class="animate-spin" />
                      <Check v-else :size="16" />
                      保存配置
                    </button>
                    
                    <button
                      @click="handleQuickTest"
                      :disabled="testingChannel"
                      class="flex gap-2 items-center btn-secondary"
                    >
                      <Loader2 v-if="testingChannel" :size="16" class="animate-spin" />
                      <Play v-else :size="16" />
                      快速测试
                    </button>
                    
                    <template v-if="!showClearConfirm">
                      <button
                        @click="showClearConfirm = true"
                        :disabled="clearing"
                        class="flex gap-2 items-center text-red-400 btn-secondary hover:text-red-300 hover:border-red-500/50"
                      >
                        <Loader2 v-if="clearing" :size="16" class="animate-spin" />
                        <Trash2 v-else :size="16" />
                        清空配置
                      </button>
                    </template>
                    <template v-else>
                      <div class="flex gap-2 items-center px-3 py-1.5 rounded-lg border bg-red-500/20 border-red-500/50">
                        <span class="text-sm text-red-300">确定清空？</span>
                        <button
                          @click="handleClearConfig"
                          class="px-2 py-1 text-xs text-white bg-red-500 rounded transition-colors hover:bg-red-600"
                        >
                          确定
                        </button>
                        <button
                          @click="showClearConfirm = false"
                          class="px-2 py-1 text-xs text-gray-300 rounded transition-colors bg-dark-600 hover:bg-dark-500"
                        >
                          取消
                        </button>
                      </div>
                    </template>
                  </div>
                  
                  <Transition name="fade">
                    <div
                      v-if="testResult"
                      :class="[
                        'mt-4 p-4 rounded-xl flex items-start gap-3',
                        testResult.success ? 'bg-green-500/10' : 'bg-red-500/10'
                      ]"
                    >
                      <CheckCircle v-if="testResult.success" :size="20" class="mt-0.5 text-green-400" />
                      <XCircle v-else :size="20" class="mt-0.5 text-red-400" />
                      <div class="flex-1">
                        <p :class="['font-medium', testResult.success ? 'text-green-400' : 'text-red-400']">
                          {{ testResult.success ? '测试成功' : '测试失败' }}
                        </p>
                        <p class="mt-1 text-sm text-gray-400">{{ testResult.message }}</p>
                        <p v-if="testResult.error" class="mt-2 text-xs text-red-300 whitespace-pre-wrap">
                          {{ testResult.error }}
                        </p>
                      </div>
                    </div>
                  </Transition>
                </div>
            </Transition>
          </template>
          <template v-else>
            <div class="flex justify-center items-center h-full text-gray-500">
              <p>选择左侧渠道进行配置</p>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
