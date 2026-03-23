<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  Shield,
  Loader2,
  Save,
  RefreshCw,
  Server,
  Container,
  Globe,
  Settings2,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Trash2,
  Square,
  Download,
  Hammer,
  Info,
} from 'lucide-vue-next';
import {
  getSandboxStatus,
  getSandboxConfig,
  saveSandboxConfig,
  validateSandboxConfig,
  listSandboxContainers,
  stopSandboxContainer,
  removeSandboxContainer,
  pruneSandboxContainers,
  checkDockerAvailable,
  pullSandboxImage,
  buildSandboxImage,
} from './api';
import type {
  SandboxConfig,
  SandboxStatus,
  SandboxContainerInfo,
  SandboxSecurityValidation,
} from './types';
import {
  DEFAULT_SANDBOX_CONFIG,
  SANDBOX_MODE_OPTIONS,
  SANDBOX_SCOPE_OPTIONS,
  WORKSPACE_ACCESS_OPTIONS,
  NETWORK_MODE_OPTIONS,
} from './types';

const loading = ref(true);
const saving = ref(false);
const status = ref<SandboxStatus | null>(null);
const config = ref<SandboxConfig>(DEFAULT_SANDBOX_CONFIG);
const validation = ref<SandboxSecurityValidation | null>(null);
const containers = ref<SandboxContainerInfo[]>([]);
const dockerAvailable = ref(false);
const error = ref<string | null>(null);
const success = ref<string | null>(null);
const activeTab = ref<'general' | 'docker' | 'browser' | 'containers'>('general');

async function loadStatus() {
  try {
    status.value = await getSandboxStatus();
    dockerAvailable.value = status.value.docker_available;
  } catch (e) {
    console.error('加载沙箱状态失败:', e);
  }
}

async function loadConfig() {
  try {
    config.value = await getSandboxConfig();
  } catch (e) {
    console.error('加载沙箱配置失败:', e);
    config.value = DEFAULT_SANDBOX_CONFIG;
  }
}

async function loadContainers() {
  try {
    containers.value = await listSandboxContainers();
  } catch (e) {
    console.error('加载容器列表失败:', e);
    containers.value = [];
  }
}

async function validateConfig() {
  try {
    validation.value = await validateSandboxConfig(config.value);
  } catch (e) {
    console.error('验证配置失败:', e);
  }
}

async function saveConfig() {
  saving.value = true;
  error.value = null;
  success.value = null;
  
  try {
    await validateConfig();
    if (validation.value && !validation.value.valid) {
      error.value = `配置验证失败: ${validation.value.errors.join(', ')}`;
      return;
    }
    
    await saveSandboxConfig(config.value);
    success.value = '沙箱配置已保存';
    
    if (validation.value && validation.value.warnings.length > 0) {
      error.value = `警告: ${validation.value.warnings.join(', ')}`;
    }
  } catch (e) {
    error.value = `保存失败: ${e}`;
  } finally {
    saving.value = false;
  }
}

async function handleStopContainer(container: SandboxContainerInfo) {
  try {
    await stopSandboxContainer(container.name);
    success.value = `容器 ${container.name} 已停止`;
    await loadContainers();
    await loadStatus();
  } catch (e) {
    error.value = `停止容器失败: ${e}`;
  }
}

async function handleRemoveContainer(container: SandboxContainerInfo) {
  try {
    await removeSandboxContainer(container.name);
    success.value = `容器 ${container.name} 已删除`;
    await loadContainers();
    await loadStatus();
  } catch (e) {
    error.value = `删除容器失败: ${e}`;
  }
}

async function handlePruneContainers() {
  try {
    const removed = await pruneSandboxContainers();
    success.value = `已清理 ${removed} 个容器`;
    await loadContainers();
    await loadStatus();
  } catch (e) {
    error.value = `清理容器失败: ${e}`;
  }
}

async function handlePullImage() {
  try {
    await pullSandboxImage(config.value.docker.image);
    success.value = `镜像 ${config.value.docker.image} 拉取成功`;
  } catch (e) {
    error.value = `拉取镜像失败: ${e}`;
  }
}

async function handleBuildImage() {
  try {
    const image = await buildSandboxImage();
    success.value = `镜像 ${image} 构建成功`;
  } catch (e) {
    error.value = `构建镜像失败: ${e}`;
  }
}

function formatDateTime(dateStr: string): string {
  if (!dateStr) return '-';
  try {
    return new Date(dateStr).toLocaleString('zh-CN');
  } catch {
    return dateStr;
  }
}

onMounted(async () => {
  loading.value = true;
  try {
    dockerAvailable.value = await checkDockerAvailable();
    await Promise.all([loadStatus(), loadConfig(), loadContainers()]);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="overflow-y-auto pr-2 h-full scroll-container">
    <div class="space-y-6">
      <Transition name="fade">
        <div v-if="error" class="p-4 text-red-300 rounded-xl border bg-red-500/20 border-red-500/50">
          <div class="flex gap-2 items-center">
            <AlertTriangle :size="18" />
            <p>{{ error }}</p>
          </div>
        </div>
      </Transition>

      <Transition name="fade">
        <div v-if="success" class="p-4 text-green-300 rounded-xl border bg-green-500/20 border-green-500/50">
          <div class="flex gap-2 items-center">
            <CheckCircle :size="18" />
            <p>{{ success }}</p>
          </div>
        </div>
      </Transition>

      <div class="p-6 bg-gradient-to-br rounded-2xl border from-dark-700 to-dark-800 border-dark-500">
        <div class="flex justify-between items-start mb-4">
          <div>
            <h2 class="flex gap-2 items-center text-xl font-semibold text-white">
              <Shield :size="22" class="text-claw-400" />
              沙箱管理
            </h2>
            <p class="mt-1 text-sm text-gray-500">配置 Docker 沙箱环境，实现安全隔离的 AI 执行环境</p>
          </div>
          <div class="flex gap-2 items-center">
            <span
              :class="[
                'px-3 py-1.5 rounded-lg text-sm font-medium flex items-center gap-1.5',
                dockerAvailable
                  ? 'bg-green-500/20 text-green-400 border border-green-500/30'
                  : 'bg-red-500/20 text-red-400 border border-red-500/30'
              ]"
            >
              <CheckCircle v-if="dockerAvailable" :size="14" />
              <XCircle v-else :size="14" />
              Docker: {{ dockerAvailable ? '可用' : '不可用' }}
            </span>
            <span
              v-if="status?.docker_version"
              class="px-3 py-1.5 rounded-lg text-sm bg-dark-600 text-gray-300 border border-dark-500"
            >
              {{ status.docker_version }}
            </span>
          </div>
        </div>

        <div v-if="!dockerAvailable" class="p-4 rounded-xl bg-yellow-500/10 border border-yellow-500/30">
          <div class="flex gap-3 items-start">
            <AlertTriangle :size="20" class="text-yellow-400 mt-0.5" />
            <div>
              <h3 class="font-medium text-yellow-300">Docker 不可用</h3>
              <p class="mt-1 text-sm text-yellow-400/80">
                沙箱功能需要 Docker 支持。请先安装并启动 Docker Desktop。
              </p>
            </div>
          </div>
        </div>

        <div v-else class="grid grid-cols-3 gap-4">
          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-claw-500/20">
                <Shield :size="20" class="text-claw-400" />
              </div>
              <div>
                <p class="text-sm text-gray-500">沙箱模式</p>
                <p class="text-lg font-medium text-white capitalize">{{ config.mode || 'off' }}</p>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-green-500/20">
                <Container :size="20" class="text-green-400" />
              </div>
              <div>
                <p class="text-sm text-gray-500">运行中容器</p>
                <p class="text-lg font-medium text-white">{{ status?.running_containers || 0 }}</p>
              </div>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-dark-600/50">
            <div class="flex gap-3 items-center">
              <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-blue-500/20">
                <Server :size="20" class="text-blue-400" />
              </div>
              <div>
                <p class="text-sm text-gray-500">总容器数</p>
                <p class="text-lg font-medium text-white">{{ containers.length }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-if="loading" class="flex justify-center items-center py-12">
        <Loader2 :size="24" class="animate-spin text-claw-400" />
      </div>

      <template v-else>
        <div class="border-b border-dark-600">
          <nav class="flex gap-1">
            <button
              v-for="tab in ['general', 'docker', 'browser', 'containers'] as const"
              :key="tab"
              @click="activeTab = tab"
              :class="[
                'flex gap-2 items-center px-4 py-3 text-sm font-medium border-b-2 transition-colors -mb-px',
                activeTab === tab
                  ? 'border-claw-500 text-claw-400'
                  : 'border-transparent text-gray-500 hover:text-gray-300'
              ]"
            >
              <component
                :is="{
                  general: Settings2,
                  docker: Container,
                  browser: Globe,
                  containers: Server
                }[tab]"
                :size="16"
              />
              {{ { general: '基本设置', docker: 'Docker 配置', browser: '浏览器配置', containers: '容器管理' }[tab] }}
            </button>
          </nav>
        </div>

        <div v-if="activeTab === 'general'" class="space-y-6">
          <div class="p-6 rounded-xl border bg-dark-700 border-dark-500">
            <h3 class="flex gap-2 items-center mb-4 text-lg font-medium text-white">
              <Settings2 :size="18" class="text-gray-500" />
              基本配置
            </h3>

            <div class="space-y-4">
              <div>
                <label class="block mb-2 text-sm text-gray-400">沙箱模式</label>
                <div class="space-y-2">
                  <label
                    v-for="opt in SANDBOX_MODE_OPTIONS"
                    :key="opt.value"
                    class="flex gap-3 items-center p-4 rounded-lg cursor-pointer transition-colors bg-dark-600 hover:bg-dark-500"
                    :class="{ 'ring-2 ring-claw-500': config.mode === opt.value }"
                  >
                    <input
                      type="radio"
                      v-model="config.mode"
                      :value="opt.value"
                      class="w-4 h-4 text-claw-500 bg-dark-500 border-dark-400 focus:ring-claw-500"
                    />
                    <div>
                      <p class="text-sm font-medium text-white">{{ opt.label }}</p>
                      <p class="text-xs text-gray-500">{{ opt.description }}</p>
                    </div>
                  </label>
                </div>
              </div>

              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block mb-2 text-sm text-gray-400">容器作用域</label>
                  <select
                    v-model="config.scope"
                    class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                  >
                    <option v-for="opt in SANDBOX_SCOPE_OPTIONS" :key="opt.value" :value="opt.value">
                      {{ opt.label }}
                    </option>
                  </select>
                  <p class="mt-1.5 text-xs text-gray-500">
                    {{ SANDBOX_SCOPE_OPTIONS.find(o => o.value === config.scope)?.description }}
                  </p>
                </div>

                <div>
                  <label class="block mb-2 text-sm text-gray-400">工作区访问权限</label>
                  <select
                    v-model="config.workspace_access"
                    class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                  >
                    <option v-for="opt in WORKSPACE_ACCESS_OPTIONS" :key="opt.value" :value="opt.value">
                      {{ opt.label }}
                    </option>
                  </select>
                  <p class="mt-1.5 text-xs text-gray-500">
                    {{ WORKSPACE_ACCESS_OPTIONS.find(o => o.value === config.workspace_access)?.description }}
                  </p>
                </div>
              </div>

              <div>
                <label class="block mb-2 text-sm text-gray-400">工作区根目录</label>
                <input
                  v-model="config.workspace_root"
                  type="text"
                  placeholder="留空使用默认目录"
                  class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                />
              </div>
            </div>
          </div>

          <div class="p-6 rounded-xl border bg-dark-700 border-dark-500">
            <h3 class="flex gap-2 items-center mb-4 text-lg font-medium text-white">
              <RefreshCw :size="18" class="text-gray-500" />
              清理策略
            </h3>

            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block mb-2 text-sm text-gray-400">空闲清理时间（小时）</label>
                <input
                  v-model.number="config.prune.idle_hours"
                  type="number"
                  min="0"
                  class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500"
                />
                <p class="mt-1.5 text-xs text-gray-500">0 表示禁用空闲清理</p>
              </div>

              <div>
                <label class="block mb-2 text-sm text-gray-400">最大存活天数</label>
                <input
                  v-model.number="config.prune.max_age_days"
                  type="number"
                  min="0"
                  class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500"
                />
                <p class="mt-1.5 text-xs text-gray-500">0 表示禁用过期清理</p>
              </div>
            </div>
          </div>
        </div>

        <div v-if="activeTab === 'docker'" class="space-y-6">
          <div class="p-6 rounded-xl border bg-dark-700 border-dark-500">
            <h3 class="flex gap-2 items-center mb-4 text-lg font-medium text-white">
              <Container :size="18" class="text-gray-500" />
              容器配置
            </h3>

            <div class="space-y-4">
              <div>
                <label class="block mb-2 text-sm text-gray-400">Docker 镜像</label>
                <div class="flex gap-2">
                  <input
                    v-model="config.docker.image"
                    type="text"
                    placeholder="openclaw-sandbox:bookworm-slim"
                    class="flex-1 px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                  />
                  <button
                    @click="handlePullImage"
                    :disabled="!dockerAvailable"
                    class="flex gap-2 items-center px-4 py-2 text-sm text-white rounded-lg transition-colors bg-dark-600 hover:bg-dark-500 disabled:opacity-50"
                    title="拉取镜像"
                  >
                    <Download :size="16" />
                    拉取
                  </button>
                  <button
                    @click="handleBuildImage"
                    :disabled="!dockerAvailable"
                    class="flex gap-2 items-center px-4 py-2 text-sm text-white rounded-lg transition-colors bg-claw-600 hover:bg-claw-500 disabled:opacity-50"
                    title="构建镜像"
                  >
                    <Hammer :size="16" />
                    构建
                  </button>
                </div>
              </div>

              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block mb-2 text-sm text-gray-400">容器名前缀</label>
                  <input
                    v-model="config.docker.container_prefix"
                    type="text"
                    class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                  />
                </div>

                <div>
                  <label class="block mb-2 text-sm text-gray-400">容器工作目录</label>
                  <input
                    v-model="config.docker.workdir"
                    type="text"
                    class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                  />
                </div>
              </div>

              <div>
                <label class="block mb-2 text-sm text-gray-400">网络模式</label>
                <select
                  v-model="config.docker.network"
                  class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500"
                >
                  <option :value="undefined">默认</option>
                  <option v-for="opt in NETWORK_MODE_OPTIONS" :key="opt.value" :value="opt.value">
                    {{ opt.label }} - {{ opt.description }}
                  </option>
                </select>
              </div>
            </div>
          </div>

          <div class="p-6 rounded-xl border bg-dark-700 border-dark-500">
            <h3 class="flex gap-2 items-center mb-4 text-lg font-medium text-white">
              <Server :size="18" class="text-gray-500" />
              资源限制
            </h3>

            <div class="grid grid-cols-3 gap-4">
              <div>
                <label class="block mb-2 text-sm text-gray-400">内存限制</label>
                <input
                  v-model="config.docker.memory"
                  type="text"
                  placeholder="如 512m, 2g"
                  class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                />
              </div>

              <div>
                <label class="block mb-2 text-sm text-gray-400">CPU 限制</label>
                <input
                  v-model.number="config.docker.cpus"
                  type="number"
                  min="0.1"
                  step="0.1"
                  placeholder="如 1.0, 2.0"
                  class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                />
              </div>

              <div>
                <label class="block mb-2 text-sm text-gray-400">PID 限制</label>
                <input
                  v-model.number="config.docker.pids_limit"
                  type="number"
                  min="0"
                  class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500"
                />
              </div>
            </div>
          </div>

          <div class="p-6 rounded-xl border bg-dark-700 border-dark-500">
            <h3 class="flex gap-2 items-center mb-4 text-lg font-medium text-white">
              <Shield :size="18" class="text-gray-500" />
              安全配置
            </h3>

            <div class="space-y-4">
              <div class="flex justify-between items-center p-4 rounded-lg bg-dark-600">
                <div>
                  <p class="text-sm text-white">只读根文件系统</p>
                  <p class="text-xs text-gray-500">增强容器安全性，防止文件系统被篡改</p>
                </div>
                <label class="inline-flex relative items-center cursor-pointer">
                  <input type="checkbox" v-model="config.docker.read_only_root" class="sr-only peer" />
                  <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
                </label>
              </div>

              <div>
                <label class="block mb-2 text-sm text-gray-400">tmpfs 挂载点</label>
                <textarea
                  :value="config.docker.tmpfs.join('\n')"
                  @change="config.docker.tmpfs = ($event.target as HTMLTextAreaElement).value.split('\n').filter(Boolean)"
                  rows="3"
                  placeholder="每行一个路径"
                  class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500 resize-none"
                ></textarea>
              </div>

              <div>
                <label class="block mb-2 text-sm text-gray-400">丢弃的 Linux 能力</label>
                <input
                  :value="config.docker.cap_drop.join(', ')"
                  @change="config.docker.cap_drop = ($event.target as HTMLInputElement).value.split(',').map(s => s.trim()).filter(Boolean)"
                  type="text"
                  placeholder="如 ALL, NET_ADMIN"
                  class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                />
              </div>
            </div>
          </div>

          <div class="p-6 rounded-xl border bg-dark-700 border-red-900/30">
            <div class="flex gap-3 items-center mb-4">
              <div class="flex justify-center items-center w-10 h-10 rounded-xl bg-red-500/20">
                <AlertTriangle :size="20" class="text-red-400" />
              </div>
              <div>
                <h3 class="text-lg font-semibold text-white">危险选项</h3>
                <p class="text-xs text-gray-500">以下选项可能削弱沙箱安全性，请谨慎启用</p>
              </div>
            </div>

            <div class="space-y-3">
              <div class="flex justify-between items-center p-4 rounded-lg border bg-red-950/30 border-red-900/30">
                <div>
                  <p class="text-sm text-red-300">允许保留容器目标路径</p>
                  <p class="text-xs text-red-400/70">可能覆盖沙箱挂载</p>
                </div>
                <label class="inline-flex relative items-center cursor-pointer">
                  <input type="checkbox" v-model="config.docker.dangerously_allow_reserved_container_targets" class="sr-only peer" />
                  <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-red-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-red-500"></div>
                </label>
              </div>

              <div class="flex justify-between items-center p-4 rounded-lg border bg-red-950/30 border-red-900/30">
                <div>
                  <p class="text-sm text-red-300">允许外部绑定源</p>
                  <p class="text-xs text-red-400/70">可能访问工作区外的文件</p>
                </div>
                <label class="inline-flex relative items-center cursor-pointer">
                  <input type="checkbox" v-model="config.docker.dangerously_allow_external_bind_sources" class="sr-only peer" />
                  <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-red-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-red-500"></div>
                </label>
              </div>

              <div class="flex justify-between items-center p-4 rounded-lg border bg-red-950/30 border-red-900/30">
                <div>
                  <p class="text-sm text-red-300">允许容器命名空间加入</p>
                  <p class="text-xs text-red-400/70">可能绕过网络隔离</p>
                </div>
                <label class="inline-flex relative items-center cursor-pointer">
                  <input type="checkbox" v-model="config.docker.dangerously_allow_container_namespace_join" class="sr-only peer" />
                  <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-red-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-red-500"></div>
                </label>
              </div>
            </div>
          </div>
        </div>

        <div v-if="activeTab === 'browser'" class="space-y-6">
          <div class="p-6 rounded-xl border bg-dark-700 border-dark-500">
            <h3 class="flex gap-2 items-center mb-4 text-lg font-medium text-white">
              <Globe :size="18" class="text-gray-500" />
              浏览器配置
            </h3>

            <div class="flex justify-between items-center p-4 rounded-lg bg-dark-600">
              <div>
                <p class="text-sm text-white">启用沙箱浏览器</p>
                <p class="text-xs text-gray-500">在沙箱中运行 Chromium 浏览器</p>
              </div>
              <label class="inline-flex relative items-center cursor-pointer">
                <input type="checkbox" v-model="config.browser.enabled" class="sr-only peer" />
                <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
              </label>
            </div>

            <div v-if="config.browser.enabled" class="mt-4 space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block mb-2 text-sm text-gray-400">浏览器镜像</label>
                  <input
                    v-model="config.browser.image"
                    type="text"
                    class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                  />
                </div>

                <div>
                  <label class="block mb-2 text-sm text-gray-400">网络模式</label>
                  <input
                    v-model="config.browser.network"
                    type="text"
                    class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500 placeholder:text-gray-500"
                  />
                </div>

                <div>
                  <label class="block mb-2 text-sm text-gray-400">CDP 端口</label>
                  <input
                    v-model.number="config.browser.cdp_port"
                    type="number"
                    min="1"
                    max="65535"
                    class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500"
                  />
                </div>

                <div>
                  <label class="block mb-2 text-sm text-gray-400">noVNC 端口</label>
                  <input
                    v-model.number="config.browser.no_vnc_port"
                    type="number"
                    min="1"
                    max="65535"
                    class="w-full px-4 py-3 rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500"
                  />
                </div>
              </div>

              <div class="space-y-3">
                <div class="flex justify-between items-center p-4 rounded-lg bg-dark-600">
                  <div>
                    <p class="text-sm text-white">无头模式</p>
                    <p class="text-xs text-gray-500">不显示浏览器窗口</p>
                  </div>
                  <label class="inline-flex relative items-center cursor-pointer">
                    <input type="checkbox" v-model="config.browser.headless" class="sr-only peer" />
                    <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
                  </label>
                </div>

                <div class="flex justify-between items-center p-4 rounded-lg bg-dark-600">
                  <div>
                    <p class="text-sm text-white">启用 noVNC</p>
                    <p class="text-xs text-gray-500">通过 Web 界面访问浏览器</p>
                  </div>
                  <label class="inline-flex relative items-center cursor-pointer">
                    <input type="checkbox" v-model="config.browser.enable_no_vnc" class="sr-only peer" />
                    <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
                  </label>
                </div>

                <div class="flex justify-between items-center p-4 rounded-lg bg-dark-600">
                  <div>
                    <p class="text-sm text-white">允许主机控制</p>
                    <p class="text-xs text-gray-500">允许沙箱访问主机浏览器控制</p>
                  </div>
                  <label class="inline-flex relative items-center cursor-pointer">
                    <input type="checkbox" v-model="config.browser.allow_host_control" class="sr-only peer" />
                    <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
                  </label>
                </div>

                <div class="flex justify-between items-center p-4 rounded-lg bg-dark-600">
                  <div>
                    <p class="text-sm text-white">自动启动</p>
                    <p class="text-xs text-gray-500">需要时自动启动浏览器容器</p>
                  </div>
                  <label class="inline-flex relative items-center cursor-pointer">
                    <input type="checkbox" v-model="config.browser.auto_start" class="sr-only peer" />
                    <div class="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="activeTab === 'containers'" class="space-y-6">
          <div class="p-6 rounded-xl border bg-dark-700 border-dark-500">
            <div class="flex justify-between items-center mb-4">
              <h3 class="flex gap-2 items-center text-lg font-medium text-white">
                <Server :size="18" class="text-gray-500" />
                容器列表
              </h3>
              <button
                @click="handlePruneContainers"
                :disabled="!dockerAvailable || containers.length === 0"
                class="flex gap-2 items-center px-4 py-2 text-sm text-red-400 rounded-lg transition-colors bg-red-500/10 hover:bg-red-500/20 disabled:opacity-50"
              >
                <Trash2 :size="16" />
                清理所有容器
              </button>
            </div>

            <div v-if="containers.length === 0" class="py-12 text-center">
              <div class="flex justify-center items-center mx-auto mb-4 w-16 h-16 rounded-full bg-dark-600">
                <Container :size="24" class="text-gray-500" />
              </div>
              <p class="text-gray-400">暂无沙箱容器</p>
              <p class="mt-1 text-sm text-gray-500">启用沙箱模式后，容器将在此显示</p>
            </div>

            <div v-else class="space-y-3">
              <div
                v-for="container in containers"
                :key="container.name"
                class="p-4 rounded-xl border bg-dark-600 border-dark-500"
              >
                <div class="flex gap-4 items-start">
                  <div class="flex justify-center items-center w-10 h-10 rounded-lg bg-dark-500">
                    <Container :size="20" class="text-gray-400" />
                  </div>

                  <div class="flex-1 min-w-0">
                    <div class="flex gap-2 items-center mb-1">
                      <h4 class="font-mono text-sm font-medium text-white">{{ container.name }}</h4>
                      <span
                        :class="[
                          'px-2 py-0.5 text-xs rounded-full',
                          container.status === 'running'
                            ? 'bg-green-500/20 text-green-400'
                            : 'bg-gray-500/20 text-gray-400'
                        ]"
                      >
                        {{ container.status }}
                      </span>
                    </div>
                    <p class="text-xs text-gray-500">{{ container.image }}</p>
                    <div class="flex gap-4 mt-2 text-xs text-gray-500">
                      <span>创建: {{ formatDateTime(container.created_at) }}</span>
                      <span v-if="container.session_key">会话: {{ container.session_key }}</span>
                    </div>
                  </div>

                  <div class="flex gap-2">
                    <button
                      v-if="container.status === 'running'"
                      @click="handleStopContainer(container)"
                      class="flex gap-1 items-center px-3 py-1.5 text-sm text-yellow-400 rounded-lg transition-colors hover:bg-yellow-500/10"
                    >
                      <Square :size="14" />
                      停止
                    </button>
                    <button
                      @click="handleRemoveContainer(container)"
                      class="flex gap-1 items-center px-3 py-1.5 text-sm text-red-400 rounded-lg transition-colors hover:bg-red-500/10"
                    >
                      <Trash2 :size="14" />
                      删除
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="p-4 rounded-xl border bg-dark-700/50 border-dark-500">
          <h4 class="flex gap-2 items-center mb-2 text-sm font-medium text-gray-400">
            <Info :size="16" />
            配置说明
          </h4>
          <ul class="space-y-1 text-sm text-gray-500">
            <li>• 沙箱配置保存在 <code class="text-claw-400">~/.openclaw/sandbox.json</code></li>
            <li>• 沙箱模式 "non-main" 仅对非主会话启用隔离，推荐用于生产环境</li>
            <li>• 只读根文件系统 + tmpfs 是推荐的安全配置</li>
            <li>• 修改配置后，新会话将使用新配置，现有容器需手动重建</li>
          </ul>
        </div>

        <div class="flex justify-end gap-3">
          <button
            @click="loadConfig"
            class="flex gap-2 items-center px-4 py-2.5 text-white rounded-lg transition-colors bg-dark-600 hover:bg-dark-500"
          >
            <RefreshCw :size="16" />
            重置
          </button>
          <button
            @click="saveConfig"
            :disabled="saving"
            class="flex gap-2 items-center btn-primary"
          >
            <Loader2 v-if="saving" :size="16" class="animate-spin" />
            <Save v-else :size="16" />
            {{ saving ? '保存中...' : '保存配置' }}
          </button>
        </div>
      </template>
    </div>
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
