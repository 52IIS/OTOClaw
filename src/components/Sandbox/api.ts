import { invoke } from '@tauri-apps/api/core';
import type {
  SandboxConfig,
  SandboxStatus,
  SandboxSecurityValidation,
} from './types';

export async function getSandboxStatus(): Promise<SandboxStatus> {
  return invoke<SandboxStatus>('get_sandbox_status');
}

export async function getSandboxConfig(): Promise<SandboxConfig> {
  return invoke<SandboxConfig>('get_sandbox_config');
}

export async function saveSandboxConfig(config: SandboxConfig): Promise<void> {
  return invoke('save_sandbox_config', { config });
}

export async function validateSandboxConfig(config: SandboxConfig): Promise<SandboxSecurityValidation> {
  return invoke<SandboxSecurityValidation>('validate_sandbox_config_cmd', { config });
}