export interface ChatMessage {
  id: string
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  timestamp: number
  isStreaming?: boolean
  attachments?: ChatAttachment[]
  thinking?: ThinkingContent
  toolCalls?: ToolCall[]
}

export interface ThinkingContent {
  text: string
  isComplete: boolean
  signature?: string
}

export interface ChatAttachment {
  id: string
  name: string
  mimeType: string
  size: number
  dataUrl: string
  type?: 'image' | 'video' | 'file' | 'audio'
}

export interface ToolCall {
  id: string
  name: string
  arguments: Record<string, unknown>
  result?: unknown
}

export interface ChatSession {
  key: string
  id: string
  title: string
  agentId?: string
  modelId?: string
  createdAt: number
  updatedAt: number
  messageCount: number
  preview?: string
}

export interface Agent {
  id: string
  name: string
  description?: string
  avatar?: string
  isDefault?: boolean
  model?: string
}

export interface Model {
  id: string
  name: string
  provider?: string
  isDefault?: boolean
  supportsVision?: boolean
  supportsThinking?: boolean
}

export interface GatewayConfig {
  url: string
  token: string
  password: string
}

export interface GatewayStatus {
  connected: boolean
  error?: string
  lastConnected?: number
}

export interface StreamingState {
  isStreaming: boolean
  streamText: string
  streamThinking: string
  runId: string | null
  startedAt: number | null
}

export interface ChatState {
  sessions: ChatSession[]
  currentSessionKey: string | null
  messages: ChatMessage[]
  agents: Agent[]
  models: Model[]
  selectedAgentId: string | null
  selectedModelId: string | null
  gatewayConfig: GatewayConfig
  gatewayStatus: GatewayStatus
  isSending: boolean
  isStreaming: boolean
  historyPanelCollapsed: boolean
  gatewayConfigVisible: boolean
  inputMessage: string
  attachments: ChatAttachment[]
  showReasoning: boolean
  streaming: StreamingState
}

export type ChatEventState = 'delta' | 'final' | 'aborted' | 'error' | 'thinking_delta' | 'thinking_final'

export interface ChatEventPayload {
  runId: string
  sessionKey: string
  state: ChatEventState
  message?: ChatMessage
  errorMessage?: string
  thinking?: string
}

export interface ContentBlock {
  type: 'text' | 'image' | 'video' | 'file' | 'thinking'
  text?: string
  source?: {
    type: 'base64' | 'url'
    media_type?: string
    data?: string
    url?: string
  }
  thinking?: string
}

export function extractThinkingFromContent(content: string): { thinking: string | null; response: string } {
  if (!content) return { thinking: null, response: content }
  
  const thinkingRegex = /<\s*think(?:ing)?\s*>([\s\S]*?)<\s*\/\s*think(?:ing)?\s*>/gi
  const matches = [...content.matchAll(thinkingRegex)]
  
  if (matches.length === 0) {
    return { thinking: null, response: content }
  }
  
  const thinkingParts: string[] = []
  let responseText = content
  
  for (const match of matches) {
    thinkingParts.push(match[1].trim())
    responseText = responseText.replace(match[0], '')
  }
  
  return {
    thinking: thinkingParts.join('\n\n'),
    response: responseText.trim()
  }
}

export function formatThinkingContent(thinking: string): string {
  if (!thinking) return ''
  return thinking
    .split('\n')
    .map(line => `> ${line}`)
    .join('\n')
}

export function isImageMimeType(mimeType: string): boolean {
  return mimeType.startsWith('image/')
}

export function isVideoMimeType(mimeType: string): boolean {
  return mimeType.startsWith('video/')
}

export function isAudioMimeType(mimeType: string): boolean {
  return mimeType.startsWith('audio/')
}

export function getFileTypeFromMime(mimeType: string): 'image' | 'video' | 'audio' | 'file' {
  if (isImageMimeType(mimeType)) return 'image'
  if (isVideoMimeType(mimeType)) return 'video'
  if (isAudioMimeType(mimeType)) return 'audio'
  return 'file'
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}
