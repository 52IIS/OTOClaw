import { ref } from 'vue'
import type { DialogOptions } from '../components/Dialog.vue'

export interface DialogState {
  visible: boolean
  options: DialogOptions
  resolve: ((value: boolean) => void) | null
}

const state = ref<DialogState>({
  visible: false,
  options: {
    message: '',
  },
  resolve: null,
})

export function useDialog() {
  const show = (options: DialogOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      state.value = {
        visible: true,
        options,
        resolve,
      }
    })
  }

  const alert = (
    message: string,
    options?: Partial<Omit<DialogOptions, 'message' | 'type'>>
  ): Promise<void> => {
    return new Promise((resolve) => {
      state.value = {
        visible: true,
        options: {
          message,
          type: 'alert',
          ...options,
        },
        resolve: () => {
          resolve()
          return true
        },
      }
    })
  }

  const confirm = (
    message: string,
    options?: Partial<Omit<DialogOptions, 'message' | 'type'>>
  ): Promise<boolean> => {
    return new Promise((resolve) => {
      state.value = {
        visible: true,
        options: {
          message,
          type: 'confirm',
          ...options,
        },
        resolve,
      }
    })
  }

  const handleConfirm = () => {
    if (state.value.resolve) {
      state.value.resolve(true)
    }
    state.value.visible = false
    state.value.resolve = null
  }

  const handleCancel = () => {
    if (state.value.resolve) {
      state.value.resolve(false)
    }
    state.value.visible = false
    state.value.resolve = null
  }

  const close = () => {
    if (state.value.resolve) {
      state.value.resolve(false)
    }
    state.value.visible = false
    state.value.resolve = null
  }

  return {
    state,
    show,
    alert,
    confirm,
    handleConfirm,
    handleCancel,
    close,
  }
}
