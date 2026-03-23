import { invoke } from '@tauri-apps/api/core';
import { apiLogger } from './logger';

// 检查是否在 Tauri 环境中运行
export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

// 带日志的 invoke 封装（自动检查 Tauri 环境）
async function invokeWithLog<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error('不在 Tauri 环境中运行，请通过 Tauri 应用启动');
  }
  apiLogger.apiCall(cmd, args);
  try {
    const result = await invoke<T>(cmd, args);
    apiLogger.apiResponse(cmd, result);
    return result;
  } catch (error) {
    apiLogger.apiError(cmd, error);
    throw error;
  }
}

// 服务状态
export interface ServiceStatus {
  running: boolean;
  pid: number | null;
  port: number;
  uptime_seconds: number | null;
  memory_mb: number | null;
  cpu_percent: number | null;
}

// 系统信息
export interface SystemInfo {
  os: string;
  os_version: string;
  arch: string;
  openclaw_installed: boolean;
  openclaw_version: string | null;
  node_version: string | null;
  config_dir: string;
}

// AI Provider 选项（旧版兼容）
export interface AIProviderOption {
  id: string;
  name: string;
  icon: string;
  default_base_url: string | null;
  models: AIModelOption[];
  requires_api_key: boolean;
}

export interface AIModelOption {
  id: string;
  name: string;
  description: string | null;
  recommended: boolean;
}

// 官方 Provider 预设
export interface OfficialProvider {
  id: string;
  name: string;
  icon: string;
  default_base_url: string | null;
  api_type: string;
  suggested_models: SuggestedModel[];
  requires_api_key: boolean;
  docs_url: string | null;
}

export interface SuggestedModel {
  id: string;
  name: string;
  description: string | null;
  context_window: number | null;
  max_tokens: number | null;
  recommended: boolean;
}

// 已配置的 Provider
export interface ConfiguredProvider {
  name: string;
  base_url: string;
  api_key_masked: string | null;
  has_api_key: boolean;
  models: ConfiguredModel[];
}

export interface ConfiguredModel {
  full_id: string;
  id: string;
  name: string;
  api_type: string | null;
  context_window: number | null;
  max_tokens: number | null;
  is_primary: boolean;
}

// AI 配置概览
export interface AIConfigOverview {
  primary_model: string | null;
  configured_providers: ConfiguredProvider[];
  available_models: string[];
}

// 模型配置
export interface ModelConfig {
  id: string;
  name: string;
  api: string | null;
  input: string[];
  context_window: number | null;
  max_tokens: number | null;
  reasoning: boolean | null;
  cost: { input: number; output: number; cache_read: number; cache_write: number } | null;
}

// 渠道配置
export interface ChannelConfig {
  id: string;
  channel_type: string;
  enabled: boolean;
  config: Record<string, unknown>;
}

// 诊断结果
export interface DiagnosticResult {
  name: string;
  passed: boolean;
  message: string;
  suggestion: string | null;
}

// AI 测试结果
export interface AITestResult {
  success: boolean;
  provider: string;
  model: string;
  response: string | null;
  error: string | null;
  latency_ms: number | null;
}

// API 封装（带日志）
export const api = {
  // 服务管理
  getServiceStatus: () => invokeWithLog<ServiceStatus>('get_service_status'),
  startService: () => invokeWithLog<string>('start_service'),
  stopService: () => invokeWithLog<string>('stop_service'),
  restartService: () => invokeWithLog<string>('restart_service'),
  getLogs: (lines?: number) => invokeWithLog<string[]>('get_logs', { lines }),

  // 系统信息
  getSystemInfo: () => invokeWithLog<SystemInfo>('get_system_info'),
  checkOpenclawInstalled: () => invokeWithLog<boolean>('check_openclaw_installed'),
  getOpenclawVersion: () => invokeWithLog<string | null>('get_openclaw_version'),

  // 配置管理
  getConfig: () => invokeWithLog<unknown>('get_config'),
  saveConfig: (config: unknown) => invokeWithLog<string>('save_config', { config }),
  getEnvValue: (key: string) => invokeWithLog<string | null>('get_env_value', { key }),
  saveEnvValue: (key: string, value: string) =>
    invokeWithLog<string>('save_env_value', { key, value }),

  // AI Provider（旧版兼容）
  getAIProviders: () => invokeWithLog<AIProviderOption[]>('get_ai_providers'),

  // AI 配置（新版）
  getOfficialProviders: () => invokeWithLog<OfficialProvider[]>('get_official_providers'),
  getAIConfig: () => invokeWithLog<AIConfigOverview>('get_ai_config'),
  saveProvider: (
    providerName: string,
    baseUrl: string,
    apiKey: string | null,
    apiType: string,
    models: ModelConfig[]
  ) =>
    invokeWithLog<string>('save_provider', {
      providerName,
      baseUrl,
      apiKey,
      apiType,
      models,
    }),
  deleteProvider: (providerName: string) =>
    invokeWithLog<string>('delete_provider', { providerName }),
  setPrimaryModel: (modelId: string) =>
    invokeWithLog<string>('set_primary_model', { modelId }),
  addAvailableModel: (modelId: string) =>
    invokeWithLog<string>('add_available_model', { modelId }),
  removeAvailableModel: (modelId: string) =>
    invokeWithLog<string>('remove_available_model', { modelId }),

  // 渠道
  getChannelsConfig: () => invokeWithLog<ChannelConfig[]>('get_channels_config'),
  saveChannelConfig: (channel: ChannelConfig) =>
    invokeWithLog<string>('save_channel_config', { channel }),

  // 诊断测试
  runDoctor: () => invokeWithLog<DiagnosticResult[]>('run_doctor'),
  testAIConnection: () => invokeWithLog<AITestResult>('test_ai_connection'),
  testChannel: (channelType: string) =>
    invokeWithLog<unknown>('test_channel', { channelType }),

  // 智能体管理
  getAgentsList: () => invokeWithLog<AgentsListResult>('get_agents_list'),
  createAgent: (params: CreateAgentParams) =>
    invokeWithLog<AgentInfo>('create_agent', { params }),
  updateAgent: (params: UpdateAgentParams) =>
    invokeWithLog<AgentInfo>('update_agent', { params }),
  deleteAgent: (agentId: string, deleteFiles?: boolean) =>
    invokeWithLog<string>('delete_agent', { params: { agentId, deleteFiles } }),
  setDefaultAgent: (agentId: string) =>
    invokeWithLog<string>('set_default_agent', { params: { agentId } }),
  getAgentById: (agentId: string) =>
    invokeWithLog<AgentInfo>('get_agent_by_id', { agentId }),
  getAgentBindings: (agentId: string) =>
    invokeWithLog<AgentBindingsResult>('get_agent_bindings', { agentId }),
  setAgentBindings: (params: SetAgentBindingsParams) =>
    invokeWithLog<string>('set_agent_bindings', { params }),
  getAvailableChannels: () =>
    invokeWithLog<string[]>('get_available_channels'),

  // 工作区文件管理
  getAgentWorkspaceFiles: (agentId: string) =>
    invokeWithLog<WorkspaceFilesResult>('get_agent_workspace_files', { agentId }),
  saveAgentWorkspaceFile: (params: SaveWorkspaceFileParams) =>
    invokeWithLog<string>('save_agent_workspace_file', { params }),

  // 技能管理
  getSkillsList: () => invokeWithLog<SkillsListResult>('get_skills_list'),
  getBuiltinSkills: () => invokeWithLog<SkillInfo[]>('get_builtin_skills'),
  checkSkillRequirements: (skillId: string) =>
    invokeWithLog<SkillRequirementsResult>('check_skill_requirements', { skillId }),
  getSkillDetail: (skillId: string) =>
    invokeWithLog<SkillDetail>('get_skill_detail', { skillId }),
  createSkill: (params: CreateSkillParams) =>
    invokeWithLog<SkillInfo>('create_skill', { params }),
  updateSkillConfig: (params: UpdateSkillConfigParams) =>
    invokeWithLog<string>('update_skill_config', { params }),
  deleteSkill: (skillId: string) =>
    invokeWithLog<string>('delete_skill', { skillId }),
  installSkillFromZip: (params: InstallSkillParams) =>
    invokeWithLog<InstallSkillResult>('install_skill_from_zip', { params }),
  exportSkill: (params: ExportSkillParams) =>
    invokeWithLog<ExportSkillResult>('export_skill', { params }),
  openSkillDirectory: (skillId: string) =>
    invokeWithLog<string>('open_skill_directory', { skillId }),
  installSkillDependency: (skillId: string, installId: string) =>
    invokeWithLog<SkillInstallResult>('install_skill_dependency', { skillId, installId }),
  getAgentSkills: (agentId: string) =>
    invokeWithLog<SkillInfo[]>('get_agent_skills', { agentId }),
  assignSkillToAgent: (skillId: string, agentId: string) =>
    invokeWithLog<string>('assign_skill_to_agent', { skillId, agentId }),
  removeSkillFromAgent: (skillId: string, agentId: string) =>
    invokeWithLog<string>('remove_skill_from_agent', { skillId, agentId }),

  // OTOClaw 更新管理
  getUpdateConfig: () => invokeWithLog<UpdateConfig>('get_update_config'),
  saveUpdateConfig: (config: UpdateConfig) =>
    invokeWithLog<string>('save_update_config_cmd', { config }),
  checkOTOClawUpdate: () => invokeWithLog<OTOClawUpdateInfo>('check_otoclaw_update'),
  downloadUpdate: (downloadUrl: string) =>
    invokeWithLog<string>('download_update', { downloadUrl }),
  installUpdate: (filePath: string) =>
    invokeWithLog<UpdateResult>('install_update', { filePath }),
  getAppVersion: () => invokeWithLog<VersionInfo>('get_app_version'),
  skipVersion: (version: string) =>
    invokeWithLog<string>('skip_version', { version }),
  cancelUpdate: () => invokeWithLog<string>('cancel_update'),
};

// 智能体相关类型
export interface AgentInfo {
  id: string;
  name: string;
  description: string | null;
  avatar: string | null;
  isDefault: boolean;
  isBuiltin: boolean;
  workspace: string | null;
  model: string | null;
  skills: string[];
  createdAt: string | null;
  updatedAt: string | null;
}

export interface AgentsListResult {
  agents: AgentInfo[];
  defaultId: string | null;
}

export interface CreateAgentParams {
  name: string;
  workspace?: string;
  model?: string;
  description?: string;
  avatar?: string;
  skills?: string[];
}

export interface UpdateAgentParams {
  agentId: string;
  name?: string;
  workspace?: string;
  model?: string;
  description?: string;
  avatar?: string;
  skills?: string[];
}

export interface AgentChannelBinding {
  channel: string;
  accountId?: string;
}

export interface AgentBindingsResult {
  agentId: string;
  bindings: AgentChannelBinding[];
}

export interface SetAgentBindingsParams {
  agentId: string;
  bindings: AgentChannelBinding[];
}

// 工作区文件相关类型
export interface WorkspaceFile {
  filename: string;
  content: string;
  exists: boolean;
}

export interface WorkspaceFilesResult {
  workspaceDir: string;
  files: WorkspaceFile[];
}

export interface SaveWorkspaceFileParams {
  agentId: string;
  filename: string;
  content: string;
}

// 技能相关类型
export interface SkillInfo {
  id: string;
  name: string;
  description: string;
  emoji: string | null;
  homepage: string | null;
  source: string;
  bundled: boolean;
  eligible: boolean;
  disabled: boolean;
  tags: string[];
  version: string | null;
  author: string | null;
  requiredEnv: string[];
  requiredBins: string[];
  installOptions: SkillInstallOption[];
}

export interface SkillInstallOption {
  id: string;
  kind: string;
  label: string;
  bins: string[];
}

export interface SkillsListResult {
  skills: SkillInfo[];
  total: number;
  eligibleCount: number;
}

export interface SkillRequirementsResult {
  skillId: string;
  eligible: boolean;
  missingBins: string[];
  missingEnv: string[];
  installOptions: SkillInstallOption[];
}

export interface SkillConfigEntry {
  enabled: boolean;
  apiKey: string | null;
  env: Record<string, string>;
  config: unknown;
}

export interface SkillDetail {
  id: string;
  name: string;
  description: string;
  emoji: string | null;
  homepage: string | null;
  source: string;
  bundled: boolean;
  eligible: boolean;
  disabled: boolean;
  tags: string[];
  version: string | null;
  author: string | null;
  requiredEnv: string[];
  requiredBins: string[];
  installOptions: SkillInstallOption[];
  config: SkillConfigEntry | null;
  path: string;
  skillMdContent: string | null;
  requiresApiKey: boolean;
  primaryEnv: string | null;
}

export interface CreateSkillParams {
  name: string;
  description: string;
  emoji?: string;
  homepage?: string;
  requiredEnv?: string[];
  requiredBins?: string[];
  skillMdContent?: string;
  agentId?: string;
}

export interface UpdateSkillConfigParams {
  skillId: string;
  enabled?: boolean;
  apiKey?: string;
  env?: Record<string, string>;
  config?: unknown;
}

export interface InstallSkillParams {
  zipPath: string;
  agentId?: string;
}

export interface InstallSkillResult {
  success: boolean;
  skillId: string | null;
  name: string | null;
  error: string | null;
  warnings: string[];
}

export interface ExportSkillParams {
  skillId: string;
  outputDir?: string;
}

export interface ExportSkillResult {
  success: boolean;
  outputPath: string | null;
  error: string | null;
}

export interface SkillInstallResult {
  success: boolean;
  message: string;
  installId: string | null;
}

export type UpdateMode = 'auto' | 'prompt';

export interface UpdateConfig {
  mode: UpdateMode;
  check_on_startup: boolean;
  last_check_time: string | null;
  skipped_version: string | null;
}

export interface OTOClawUpdateInfo {
  update_available: boolean;
  current_version: string;
  latest_version: string;
  release_notes: string | null;
  download_url: string | null;
  file_size: number | null;
  error: string | null;
}

export interface DownloadProgress {
  downloaded: number;
  total: number;
  percentage: number;
  speed: string;
}

export interface UpdateResult {
  success: boolean;
  message: string;
  error?: string;
}

export interface VersionInfo {
  version: string;
  build_number: number;
  release_date: string;
  release_notes: string | null;
}
