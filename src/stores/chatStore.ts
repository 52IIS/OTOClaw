import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  ChatMessage,
  ChatSession,
  Agent,
  Model,
  GatewayConfig,
  GatewayStatus,
  ChatAttachment,
  StreamingState,
  ChatEventPayload,
} from '../components/Chat/types'
import { chatLogger } from '../lib/logger'

interface AgentsListResult {
  agents: Agent[]
  defaultId: string | null
}

interface ModelsListResult {
  models: Model[]
  defaultId: string | null
}

interface GatewayConfigInfo {
  url: string
  token_masked: string | null
  token_full: string | null
  password: string | null
}

export const useChatStore = defineStore('chat', () => {
  const sessions = ref<ChatSession[]>([])
  const currentSessionKey = ref<string | null>(null)
  const messages = ref<ChatMessage[]>([])
  const agents = ref<Agent[]>([])
  const models = ref<Model[]>([])
  const selectedAgentId = ref<string | null>(null)
  const selectedModelId = ref<string | null>(null)
  const gatewayConfig = ref<GatewayConfig>({
    url: 'ws://localhost:18789',
    token: '',
    password: '',
  })
  const gatewayStatus = ref<GatewayStatus>({
    connected: false,
  })
  const isSending = ref(false)
  const isStreaming = ref(false)
  const historyPanelCollapsed = ref(false)
  const gatewayConfigVisible = ref(false)
  const inputMessage = ref('')
  const attachments = ref<ChatAttachment[]>([])
  const showReasoning = ref(true)
  const isOptimizing = ref(false)
  const originalInputMessage = ref('')
  const streaming = ref<StreamingState>({
    isStreaming: false,
    streamText: '',
    streamThinking: '',
    runId: null,
    startedAt: null,
  })

  const currentSession = computed(() =>
    sessions.value.find(s => s.key === currentSessionKey.value)
  )

  const defaultAgent = computed(() =>
    agents.value.find(a => a.isDefault) || agents.value[0]
  )

  const defaultModel = computed(() =>
    models.value.find(m => m.isDefault) || models.value[0]
  )

  const selectedModel = computed(() =>
    models.value.find(m => m.id === selectedModelId.value)
  )

  const supportsVision = computed(() =>
    selectedModel.value?.supportsVision ?? false
  )

  const supportsThinking = computed(() =>
    selectedModel.value?.supportsThinking ?? false
  )

  const maskToken = (token: string): string => {
    if (!token) return ''
    const len = token.length
    if (len <= 8) return '****'
    if (len <= 16) return `${token.slice(0, 4)}****`
    return `${token.slice(0, 4)}****${token.slice(-4)}`
  }

  const loadAgents = async () => {
    chatLogger.info('加载智能体列表...')
    try {
      const result = await invoke<AgentsListResult>('get_agents')
      agents.value = result.agents
      if (result.defaultId) {
        selectedAgentId.value = result.defaultId
      } else if (defaultAgent.value) {
        selectedAgentId.value = defaultAgent.value.id
      }
      chatLogger.info(`成功加载 ${agents.value.length} 个智能体`)
    } catch (e) {
      chatLogger.error('加载智能体失败', e)
      agents.value = [
        { id: 'default', name: '默认助手', description: '通用AI助手', isDefault: true },
      ]
      selectedAgentId.value = 'default'
    }
  }

  const loadModels = async () => {
    chatLogger.info('加载模型列表...')
    try {
      const result = await invoke<ModelsListResult>('get_models')
      models.value = result.models
      if (result.defaultId) {
        selectedModelId.value = result.defaultId
      } else if (defaultModel.value) {
        selectedModelId.value = defaultModel.value.id
      }
      chatLogger.info(`成功加载 ${models.value.length} 个模型`)
    } catch (e) {
      chatLogger.error('加载模型失败', e)
      models.value = [
        { id: 'anthropic/claude-sonnet-4-5-20250929', name: 'Claude Sonnet 4.5', provider: 'anthropic', isDefault: true, supportsVision: true, supportsThinking: true },
        { id: 'openai/gpt-4o', name: 'GPT-4o', provider: 'openai', isDefault: false, supportsVision: true, supportsThinking: false },
      ]
      selectedModelId.value = 'anthropic/claude-sonnet-4-5-20250929'
    }
  }

  const loadGatewayConfig = async () => {
    chatLogger.info('加载Gateway配置...')
    try {
      const result = await invoke<GatewayConfigInfo>('get_gateway_config')
      gatewayConfig.value = {
        url: result.url,
        token: result.token_full || '',
        password: result.password || '',
      }
      chatLogger.info(`Gateway配置已加载: ${result.url}, Token: ${result.token_masked || '未设置'}`)
    } catch (e) {
      chatLogger.warn('加载Gateway配置失败，使用默认值', e)
      gatewayConfig.value = {
        url: 'ws://localhost:18789',
        token: '',
        password: '',
      }
    }
  }

  const loadSessions = async () => {
    chatLogger.info('加载会话列表...')
    try {
      const result = await invoke<{ sessions: ChatSession[] }>('get_sessions')
      sessions.value = result.sessions || []
    } catch (e) {
      chatLogger.error('加载会话列表失败', e)
      sessions.value = []
    }
  }

  const loadMessages = async (sessionKey: string) => {
    chatLogger.info('加载会话消息...', { sessionKey })
    try {
      const result = await invoke<{ messages: ChatMessage[] }>('get_session_messages', { sessionKey })
      messages.value = result.messages || []
      currentSessionKey.value = sessionKey
    } catch (e) {
      chatLogger.error('加载会话消息失败', e)
      messages.value = []
    }
  }

  const connectGateway = async () => {
    chatLogger.action('连接Gateway', { url: gatewayConfig.value.url })
    try {
      const result = await invoke<{ success: boolean; error?: string }>('connect_gateway', {
        config: gatewayConfig.value,
      })
      gatewayStatus.value = {
        connected: result.success,
        error: result.error,
        lastConnected: result.success ? Date.now() : undefined,
      }
      
      if (result.success) {
        chatLogger.info('Gateway连接成功，刷新会话列表...')
        await loadSessions()
      }
      
      return result.success
    } catch (e) {
      chatLogger.error('连接Gateway失败', e)
      gatewayStatus.value = {
        connected: false,
        error: String(e),
      }
      return false
    }
  }

  const disconnectGateway = async () => {
    chatLogger.action('断开Gateway连接')
    try {
      await invoke('disconnect_gateway')
      gatewayStatus.value = { connected: false }
    } catch (e) {
      chatLogger.error('断开Gateway失败', e)
    }
  }

  const saveGatewayConfig = async (config: Partial<GatewayConfig>) => {
    const newConfig = { ...gatewayConfig.value, ...config }
    gatewayConfig.value = newConfig
    
    try {
      await invoke<string>('save_gateway_config', {
        url: newConfig.url,
        token: newConfig.token,
        password: newConfig.password,
      })
      chatLogger.info('Gateway配置已保存')
    } catch (e) {
      chatLogger.error('保存Gateway配置失败', e)
    }
  }

  const createSession = async (agentId?: string, modelId?: string) => {
    chatLogger.action('创建新会话')
    try {
      const result = await invoke<{ session: ChatSession }>('create_session', {
        agentId: agentId || selectedAgentId.value,
        modelId: modelId || selectedModelId.value,
      })
      currentSessionKey.value = result.session.key
      messages.value = []
      await loadSessions()
      chatLogger.info('会话创建成功，已刷新会话列表')
      return result.session
    } catch (e) {
      chatLogger.error('创建会话失败', e)
      const newSession: ChatSession = {
        key: `session-${Date.now()}`,
        id: `session-${Date.now()}`,
        title: '新会话',
        agentId: agentId || selectedAgentId.value || 'default',
        modelId: modelId || selectedModelId.value || 'default',
        createdAt: Date.now(),
        updatedAt: Date.now(),
        messageCount: 0,
      }
      sessions.value.unshift(newSession)
      currentSessionKey.value = newSession.key
      messages.value = []
      return newSession
    }
  }

  const deleteSession = async (sessionKey: string) => {
    chatLogger.action('删除会话', { sessionKey })
    try {
      await invoke('delete_session', { sessionKey })
      if (currentSessionKey.value === sessionKey) {
        currentSessionKey.value = null
        messages.value = []
      }
      await loadSessions()
      chatLogger.info('会话删除成功，已刷新会话列表')
    } catch (e) {
      chatLogger.error('删除会话失败', e)
      sessions.value = sessions.value.filter(s => s.key !== sessionKey)
      if (currentSessionKey.value === sessionKey) {
        currentSessionKey.value = null
        messages.value = []
      }
    }
  }

  const handleChatEvent = (payload: ChatEventPayload) => {
    if (payload.state === 'thinking_delta') {
      streaming.value.streamThinking += payload.thinking || ''
      return
    }

    if (payload.state === 'thinking_final') {
      const lastMessage = messages.value[messages.value.length - 1]
      if (lastMessage?.role === 'assistant') {
        lastMessage.thinking = {
          text: streaming.value.streamThinking,
          isComplete: true,
        }
      }
      streaming.value.streamThinking = ''
      return
    }

    if (payload.state === 'delta') {
      if (payload.message?.content) {
        streaming.value.streamText = payload.message.content
        streaming.value.isStreaming = true
      }
      return
    }

    if (payload.state === 'final') {
      streaming.value.isStreaming = false
      streaming.value.streamText = ''
      streaming.value.runId = null
      
      if (payload.message) {
        const lastMessage = messages.value[messages.value.length - 1]
        if (lastMessage?.role === 'assistant') {
          lastMessage.content = payload.message.content
          lastMessage.isStreaming = false
          if (payload.message.thinking) {
            lastMessage.thinking = payload.message.thinking
          }
        }
      }
      isStreaming.value = false
      isSending.value = false
      return
    }

    if (payload.state === 'error') {
      streaming.value.isStreaming = false
      streaming.value.streamText = ''
      streaming.value.runId = null
      isStreaming.value = false
      isSending.value = false
      
      const lastMessage = messages.value[messages.value.length - 1]
      if (lastMessage?.role === 'assistant') {
        lastMessage.content = `错误: ${payload.errorMessage || '未知错误'}`
        lastMessage.isStreaming = false
      }
    }

    if (payload.state === 'aborted') {
      streaming.value.isStreaming = false
      streaming.value.runId = null
      isStreaming.value = false
      isSending.value = false
      
      const lastMessage = messages.value[messages.value.length - 1]
      if (lastMessage?.role === 'assistant') {
        lastMessage.isStreaming = false
      }
    }
  }

  const sendMessage = async (content: string, attachs?: ChatAttachment[]) => {
    if (!content.trim() && (!attachs || attachs.length === 0)) return

    const userMessage: ChatMessage = {
      id: `msg-${Date.now()}`,
      role: 'user',
      content: content.trim(),
      timestamp: Date.now(),
      attachments: attachs,
    }

    messages.value.push(userMessage)
    inputMessage.value = ''
    attachments.value = []
    isSending.value = true
    isStreaming.value = true

    const assistantMessage: ChatMessage = {
      id: `msg-${Date.now()}-assistant`,
      role: 'assistant',
      content: '',
      timestamp: Date.now(),
      isStreaming: true,
    }
    messages.value.push(assistantMessage)

    streaming.value = {
      isStreaming: true,
      streamText: '',
      streamThinking: '',
      runId: `run-${Date.now()}`,
      startedAt: Date.now(),
    }

    try {
      chatLogger.info('发送消息', { content: content.substring(0, 50), attachments: attachs?.length || 0 })
      
      const result = await invoke<{ content: string; thinking?: string }>('send_chat_message', {
        sessionKey: currentSessionKey.value,
        message: content,
        attachments: attachs,
        agentId: selectedAgentId.value,
        modelId: selectedModelId.value,
      })

      const lastMessage = messages.value[messages.value.length - 1]
      if (lastMessage.role === 'assistant') {
        lastMessage.content = result.content
        if (result.thinking) {
          lastMessage.thinking = {
            text: result.thinking,
            isComplete: true,
          }
        }
        lastMessage.isStreaming = false
      }
    } catch (e) {
      chatLogger.error('发送消息失败', e)
      const lastMessage = messages.value[messages.value.length - 1]
      if (lastMessage.role === 'assistant') {
        lastMessage.content = `发送失败: ${e}`
        lastMessage.isStreaming = false
      }
    } finally {
      isSending.value = false
      isStreaming.value = false
      streaming.value.isStreaming = false
    }
  }

  const stopStreaming = async () => {
    if (!streaming.value.runId) return
    
    try {
      await invoke('abort_chat', { runId: streaming.value.runId })
      chatLogger.info('已停止流式响应')
    } catch (e) {
      chatLogger.error('停止流式响应失败', e)
    }
    
    streaming.value.isStreaming = false
    streaming.value.runId = null
    isStreaming.value = false
    isSending.value = false
  }

  const addAttachment = (attachment: ChatAttachment) => {
    attachments.value.push(attachment)
  }

  const removeAttachment = (id: string) => {
    attachments.value = attachments.value.filter(a => a.id !== id)
  }

  const toggleHistoryPanel = () => {
    historyPanelCollapsed.value = !historyPanelCollapsed.value
  }

  const toggleGatewayConfig = () => {
    gatewayConfigVisible.value = !gatewayConfigVisible.value
  }

  const toggleReasoning = () => {
    showReasoning.value = !showReasoning.value
    chatLogger.info('切换推理显示', { showReasoning: showReasoning.value })
  }

  const setAgent = (agentId: string) => {
    selectedAgentId.value = agentId
    chatLogger.info('切换智能体', { agentId })
    
    // 自动更新模型为智能体的默认模型
    const agent = agents.value.find(a => a.id === agentId)
    if (agent?.model) {
      // 检查模型是否在可用列表中
      const modelExists = models.value.some(m => m.id === agent.model)
      if (modelExists) {
        selectedModelId.value = agent.model
        chatLogger.info('智能体模型已自动设置', { modelId: agent.model })
      } else {
        chatLogger.warn('智能体配置的模型不在可用列表中', { modelId: agent.model })
      }
    }
  }

  const setModel = (modelId: string) => {
    selectedModelId.value = modelId
    chatLogger.info('切换模型', { modelId })
  }

  const optimizePrompt = async () => {
    if (!inputMessage.value.trim()) return false
    
    if (isOptimizing.value) return false

    chatLogger.action('优化提示词')
    isOptimizing.value = true
    originalInputMessage.value = inputMessage.value

    try {
      const result = await invoke<{ optimized: string }>('optimize_prompt', {
        prompt: inputMessage.value,
      })
      inputMessage.value = result.optimized
      chatLogger.info('提示词优化完成')
      return true
    } catch (e) {
      chatLogger.error('提示词优化失败', e)
      return false
    } finally {
      isOptimizing.value = false
    }
  }

  const revertPrompt = () => {
    if (originalInputMessage.value) {
      inputMessage.value = originalInputMessage.value
      originalInputMessage.value = ''
      chatLogger.info('已撤回优化内容')
    }
  }

  const clearOptimization = () => {
    originalInputMessage.value = ''
    isOptimizing.value = false
  }

  const hasOptimized = computed(() => !!originalInputMessage.value)

  return {
    sessions,
    currentSessionKey,
    messages,
    agents,
    models,
    selectedAgentId,
    selectedModelId,
    gatewayConfig,
    gatewayStatus,
    isSending,
    isStreaming,
    historyPanelCollapsed,
    gatewayConfigVisible,
    inputMessage,
    attachments,
    showReasoning,
    isOptimizing,
    originalInputMessage,
    streaming,
    currentSession,
    defaultAgent,
    defaultModel,
    selectedModel,
    supportsVision,
    supportsThinking,
    maskToken,
    loadAgents,
    loadModels,
    loadGatewayConfig,
    loadSessions,
    loadMessages,
    connectGateway,
    disconnectGateway,
    saveGatewayConfig,
    createSession,
    deleteSession,
    handleChatEvent,
    sendMessage,
    stopStreaming,
    addAttachment,
    removeAttachment,
    toggleHistoryPanel,
    toggleGatewayConfig,
    toggleReasoning,
    setAgent,
    setModel,
    optimizePrompt,
    revertPrompt,
    clearOptimization,
    hasOptimized,
  }
})
