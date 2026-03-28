<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  Shield,
  Loader2,
  Save,
  RefreshCw,
  Info,
  ExternalLink,
} from 'lucide-vue-next';
import {
  getSandboxStatus,
  getSandboxConfig,
  saveSandboxConfig,
  validateSandboxConfig,
} from './api';
import type {
  SandboxConfig,
  SandboxStatus,
  SandboxSecurityValidation,
} from './types';
import {
  DEFAULT_SANDBOX_CONFIG,
  SANDBOX_MODE_OPTIONS,
  SANDBOX_SCOPE_OPTIONS,
  WORKSPACE_ACCESS_OPTIONS,
} from './types';

const loading = ref(true);
const saving = ref(false);
const status = ref<SandboxStatus | null>(null);
const config = ref<SandboxConfig>(DEFAULT_SANDBOX_CONFIG);
const validation = ref<SandboxSecurityValidation | null>(null);
const error = ref<string | null>(null);
const success = ref<string | null>(null);

async function loadStatus() {
  try {
    status.value = await getSandboxStatus();
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
      saving.value = false;
      return;
    }

    await saveSandboxConfig(config.value);
    success.value = '沙箱配置已保存到 ~/.openclaw/sandbox.json';

    if (validation.value && validation.value.warnings.length > 0) {
      success.value += ` (警告: ${validation.value.warnings.join(', ')})`;
    }
  } catch (e) {
    error.value = `保存失败: ${e}`;
  } finally {
    saving.value = false;
  }
}

function resetConfig() {
  config.value = DEFAULT_SANDBOX_CONFIG;
}

onMounted(async () => {
  loading.value = true;
  try {
    await Promise.all([loadStatus(), loadConfig()]);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="overflow-y-auto pr-2 h-full scroll-container">
    <div class="space-y-4">
      <Transition name="fade">
        <div v-if="error" class="p-3 text-sm text-red-300 rounded-lg border bg-red-500/20 border-red-500/50">
          <div class="flex gap-2 items-center">
            <Info :size="16" />
            <p>{{ error }}</p>
          </div>
        </div>
      </Transition>

      <Transition name="fade">
        <div v-if="success" class="p-3 text-sm text-green-300 rounded-lg border bg-green-500/20 border-green-500/50">
          <div class="flex gap-2 items-center">
            <Info :size="16" />
            <p>{{ success }}</p>
          </div>
        </div>
      </Transition>

      <div class="p-4 rounded-xl bg-gradient-to-br from-dark-700 to-dark-800 border border-dark-500">
        <div class="flex items-center gap-3 mb-3">
          <Shield :size="20" class="text-claw-400" />
          <h2 class="text-lg font-semibold text-white">沙箱管理</h2>
        </div>
        <p class="text-sm text-gray-400">
          为了更安全使用 OpenClaw，可以在 Docker 容器内运行工具以减少影响范围。
          沙箱配置通过 <code class="text-claw-400">agents.defaults.sandbox</code> 进行管理。
        </p>
      </div>

      <div v-if="loading" class="flex justify-center items-center py-8">
        <Loader2 :size="24" class="animate-spin text-claw-400" />
      </div>

      <template v-else>
        <div class="p-4 rounded-xl border bg-dark-700 border-dark-500">
          <h3 class="flex gap-2 items-center mb-4 text-sm font-medium text-gray-400">
            <Info :size="16" />
            沙箱配置选项
          </h3>

          <div class="grid grid-cols-3 gap-4">
            <div class="space-y-2">
              <label class="text-sm text-gray-400">沙箱模式 (mode)</label>
              <select
                v-model="config.mode"
                class="w-full px-3 py-2 text-sm rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500"
              >
                <option v-for="opt in SANDBOX_MODE_OPTIONS" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
              <p class="text-xs text-gray-500">
                {{ SANDBOX_MODE_OPTIONS.find(o => o.value === config.mode)?.description }}
              </p>
            </div>

            <div class="space-y-2">
              <label class="text-sm text-gray-400">容器作用域 (scope)</label>
              <select
                v-model="config.scope"
                class="w-full px-3 py-2 text-sm rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500"
              >
                <option v-for="opt in SANDBOX_SCOPE_OPTIONS" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
              <p class="text-xs text-gray-500">
                {{ SANDBOX_SCOPE_OPTIONS.find(o => o.value === config.scope)?.description }}
              </p>
            </div>

            <div class="space-y-2">
              <label class="text-sm text-gray-400">工作区访问权限</label>
              <select
                v-model="config.workspace_access"
                class="w-full px-3 py-2 text-sm rounded-lg bg-dark-600 text-white border-0 focus:ring-2 focus:ring-claw-500"
              >
                <option v-for="opt in WORKSPACE_ACCESS_OPTIONS" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
              <p class="text-xs text-gray-500">
                {{ WORKSPACE_ACCESS_OPTIONS.find(o => o.value === config.workspace_access)?.description }}
              </p>
            </div>
          </div>

          <div class="flex justify-between items-center mt-4 pt-4 border-t border-dark-600">
            <div class="flex gap-2 items-center text-xs text-gray-500">
              <span>当前配置:</span>
              <code class="px-1.5 py-0.5 rounded bg-dark-600 text-claw-400">
                mode={{ config.mode }}, scope={{ config.scope }}, workspaceAccess={{ config.workspace_access }}
              </code>
            </div>
            <div class="flex gap-2">
              <button
                @click="resetConfig"
                class="flex gap-1.5 items-center px-3 py-1.5 text-sm text-white rounded-lg transition-colors bg-dark-600 hover:bg-dark-500"
              >
                <RefreshCw :size="14" />
                重置
              </button>
              <button
                @click="saveConfig"
                :disabled="saving"
                class="flex gap-1.5 items-center px-3 py-1.5 text-sm text-white rounded-lg transition-colors btn-primary"
              >
                <Loader2 v-if="saving" :size="14" class="animate-spin" />
                <Save v-else :size="14" />
                {{ saving ? '保存中...' : '保存配置' }}
              </button>
            </div>
          </div>
        </div>

        <div class="p-4 rounded-xl border bg-dark-700 border-dark-500">
          <h3 class="flex gap-2 items-center mb-3 text-sm font-medium text-gray-400">
            <ExternalLink :size="16" />
            参考文档
          </h3>
          <div class="grid grid-cols-2 gap-4 text-sm text-gray-400">
            <ul class="space-y-1.5">
              <li>• 详细配置请参阅 <a href="https://docs.openclaw.ai/zh-CN/gateway/sandboxing" target="_blank" class="text-claw-400 hover:underline">OpenClaw 官方沙箱文档</a></li>
              <li>• "non-main" 模式仅对非主会话启用隔离</li>
            </ul>
            <ul class="space-y-1.5">
              <li>• Gateway 网关进程本身不被沙箱隔离</li>
              <li>• tools.elevated 会绕过沙箱隔离</li>
            </ul>
          </div>
        </div>

        <div class="p-3 rounded-lg bg-dark-700/50 border border-dark-500">
          <p class="text-xs text-gray-500">
            <Info :size="14" class="inline mr-1" />
            配置保存后需重启 OpenClaw 服务使配置生效
          </p>
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