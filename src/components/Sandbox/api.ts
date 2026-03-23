import { invoke } from '@tauri-apps/api/core';
import type {
  SandboxConfig,
  SandboxStatus,
  SandboxSecurityValidation,
  SandboxContainerInfo,
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

export async function listSandboxContainers(): Promise<SandboxContainerInfo[]> {
  return invoke<SandboxContainerInfo[]>('list_sandbox_containers');
}

export async function stopSandboxContainer(containerName: string): Promise<void> {
  return invoke('stop_sandbox_container', { containerName });
}

export async function removeSandboxContainer(containerName: string): Promise<void> {
  return invoke('remove_sandbox_container', { containerName });
}

export async function pruneSandboxContainers(): Promise<number> {
  return invoke<number>('prune_sandbox_containers');
}

export async function recreateSandboxContainer(containerName: string): Promise<void> {
  return invoke('recreate_sandbox_container', { containerName });
}

export async function checkDockerAvailable(): Promise<boolean> {
  return invoke<boolean>('check_docker_available_cmd');
}

export async function getDockerVersion(): Promise<string> {
  return invoke<string>('get_docker_version_cmd');
}

export async function pullSandboxImage(image: string): Promise<void> {
  return invoke('pull_sandbox_image', { image });
}

export async function buildSandboxImage(): Promise<string> {
  return invoke<string>('build_sandbox_image');
}
