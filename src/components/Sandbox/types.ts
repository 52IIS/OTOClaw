export type SandboxMode = 'off' | 'non-main' | 'all';

export type SandboxScope = 'session' | 'agent' | 'shared';

export type SandboxWorkspaceAccess = 'none' | 'ro' | 'rw';

export interface SandboxStatus {
  enabled: boolean;
  mode: SandboxMode;
  docker_available: boolean;
  docker_version?: string;
  running_containers: number;
}

export interface SandboxSecurityValidation {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

export interface SandboxConfig {
  mode: SandboxMode;
  scope: SandboxScope;
  workspace_access: SandboxWorkspaceAccess;
}

export const DEFAULT_SANDBOX_CONFIG: SandboxConfig = {
  mode: 'off',
  scope: 'session',
  workspace_access: 'none',
};

export const SANDBOX_MODE_OPTIONS = [
  { value: 'off', label: '关闭', description: '不使用沙箱隔离' },
  { value: 'non-main', label: '非主会话', description: '仅对非主会话启用沙箱（推荐）' },
  { value: 'all', label: '全部', description: '所有会话都在沙箱中运行' },
];

export const SANDBOX_SCOPE_OPTIONS = [
  { value: 'session', label: '会话级', description: '每个会话一个容器（推荐）' },
  { value: 'agent', label: '智能体级', description: '每个智能体一个容器' },
  { value: 'shared', label: '共享', description: '所有会话共享一个容器' },
];

export const WORKSPACE_ACCESS_OPTIONS = [
  { value: 'none', label: '无访问', description: '工具看到沙箱工作区（最安全）' },
  { value: 'ro', label: '只读', description: '只读挂载智能体工作区到 /agent' },
  { value: 'rw', label: '读写', description: '读写挂载智能体工作区到 /workspace' },
];