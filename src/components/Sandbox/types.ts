export type SandboxMode = 'off' | 'non-main' | 'all';

export type SandboxScope = 'session' | 'agent' | 'shared';

export type SandboxWorkspaceAccess = 'none' | 'ro' | 'rw';

export interface UlimitConfig {
  soft?: number;
  hard?: number;
}

export interface SandboxDockerConfig {
  image: string;
  container_prefix: string;
  workdir: string;
  read_only_root: boolean;
  tmpfs: string[];
  network?: string;
  user?: string;
  cap_drop: string[];
  env: Record<string, string>;
  setup_command?: string;
  pids_limit?: number;
  memory?: string;
  memory_swap?: string;
  cpus?: number;
  ulimits: Record<string, UlimitConfig>;
  seccomp_profile?: string;
  apparmor_profile?: string;
  dns: string[];
  extra_hosts: string[];
  binds: string[];
  dangerously_allow_reserved_container_targets: boolean;
  dangerously_allow_external_bind_sources: boolean;
  dangerously_allow_container_namespace_join: boolean;
}

export interface SandboxBrowserConfig {
  enabled: boolean;
  image: string;
  container_prefix: string;
  network: string;
  cdp_port: number;
  cdp_source_range?: string;
  vnc_port: number;
  no_vnc_port: number;
  headless: boolean;
  enable_no_vnc: boolean;
  allow_host_control: boolean;
  auto_start: boolean;
  auto_start_timeout_ms: number;
  binds: string[];
}

export interface SandboxPruneConfig {
  idle_hours: number;
  max_age_days: number;
}

export interface SandboxToolPolicy {
  allow: string[];
  deny: string[];
}

export interface SandboxConfig {
  mode: SandboxMode;
  scope: SandboxScope;
  workspace_access: SandboxWorkspaceAccess;
  workspace_root?: string;
  docker: SandboxDockerConfig;
  browser: SandboxBrowserConfig;
  tools: SandboxToolPolicy;
  prune: SandboxPruneConfig;
}

export interface SandboxContainerInfo {
  name: string;
  session_key: string;
  image: string;
  status: string;
  created_at: string;
  last_used_at: string;
  config_hash?: string;
}

export interface SandboxStatus {
  enabled: boolean;
  mode: SandboxMode;
  docker_available: boolean;
  docker_version?: string;
  running_containers: number;
  containers: SandboxContainerInfo[];
}

export interface SandboxSecurityValidation {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

export const DEFAULT_SANDBOX_CONFIG: SandboxConfig = {
  mode: 'off',
  scope: 'session',
  workspace_access: 'none',
  workspace_root: undefined,
  docker: {
    image: 'openclaw-sandbox:bookworm-slim',
    container_prefix: 'openclaw-sbx-',
    workdir: '/workspace',
    read_only_root: true,
    tmpfs: ['/tmp', '/run', '/var/run'],
    network: 'none',
    user: undefined,
    cap_drop: ['ALL'],
    env: {},
    setup_command: undefined,
    pids_limit: 256,
    memory: '512m',
    memory_swap: undefined,
    cpus: 1.0,
    ulimits: {},
    seccomp_profile: undefined,
    apparmor_profile: undefined,
    dns: [],
    extra_hosts: [],
    binds: [],
    dangerously_allow_reserved_container_targets: false,
    dangerously_allow_external_bind_sources: false,
    dangerously_allow_container_namespace_join: false,
  },
  browser: {
    enabled: false,
    image: 'openclaw-sandbox-browser:bookworm-slim',
    container_prefix: 'openclaw-sbx-browser-',
    network: 'openclaw-sandbox-browser',
    cdp_port: 9222,
    cdp_source_range: undefined,
    vnc_port: 5900,
    no_vnc_port: 6080,
    headless: true,
    enable_no_vnc: false,
    allow_host_control: false,
    auto_start: true,
    auto_start_timeout_ms: 12000,
    binds: [],
  },
  tools: {
    allow: [],
    deny: [],
  },
  prune: {
    idle_hours: 24,
    max_age_days: 7,
  },
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

export const NETWORK_MODE_OPTIONS = [
  { value: 'none', label: '无网络', description: '完全隔离网络（最安全）' },
  { value: 'bridge', label: '桥接', description: '使用 Docker 桥接网络' },
  { value: 'host', label: '主机', description: '使用主机网络（不安全，不推荐）' },
];
