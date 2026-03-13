import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ServiceStatus, SystemInfo } from '../lib/tauri'

interface Notification {
  id: string
  type: 'success' | 'error' | 'warning' | 'info'
  title: string
  message?: string
}

export const useAppStore = defineStore('app', () => {
  const serviceStatus = ref<ServiceStatus | null>(null)
  const systemInfo = ref<SystemInfo | null>(null)
  const loading = ref(false)
  const notifications = ref<Notification[]>([])

  const setServiceStatus = (status: ServiceStatus | null) => {
    serviceStatus.value = status
  }

  const setSystemInfo = (info: SystemInfo | null) => {
    systemInfo.value = info
  }

  const setLoading = (value: boolean) => {
    loading.value = value
  }

  const addNotification = (notification: Omit<Notification, 'id'>) => {
    notifications.value.push({
      ...notification,
      id: Date.now().toString(),
    })
  }

  const removeNotification = (id: string) => {
    notifications.value = notifications.value.filter((n) => n.id !== id)
  }

  return {
    serviceStatus,
    systemInfo,
    loading,
    notifications,
    setServiceStatus,
    setSystemInfo,
    setLoading,
    addNotification,
    removeNotification,
  }
})
