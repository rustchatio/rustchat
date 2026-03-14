// Preferences Store - Pure state management for user preferences

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import type { UserStatus, UserPreferences, StatusPreset } from '../repositories/preferencesRepository'

export const usePreferencesStore = defineStore('preferencesStore', () => {
  // State
  const status = ref<UserStatus | null>(null)
  const preferences = ref<UserPreferences | null>(null)
  const statusPresets = ref<StatusPreset[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const hasStatus = computed(() =>
    status.value && (status.value.text || status.value.emoji)
  )

  const statusDisplay = computed(() => {
    if (!status.value) return null
    if (status.value.emoji && status.value.text) {
      return `${status.value.emoji} ${status.value.text}`
    }
    return status.value.emoji || status.value.text
  })

  // Actions
  function setStatus(value: UserStatus | null) {
    status.value = value
  }

  function setPreferences(value: UserPreferences | null) {
    preferences.value = value
  }

  function setStatusPresets(value: StatusPreset[]) {
    statusPresets.value = value
  }

  function setLoading(value: boolean) {
    loading.value = value
  }

  function setError(err: string | null) {
    error.value = err
  }

  function clearError() {
    error.value = null
  }

  function clear() {
    status.value = null
    preferences.value = null
    statusPresets.value = []
    error.value = null
  }

  return {
    // State (readonly)
    status: readonly(status),
    preferences: readonly(preferences),
    statusPresets: readonly(statusPresets),
    loading: readonly(loading),
    error: readonly(error),

    // Getters
    hasStatus,
    statusDisplay,

    // Actions
    setStatus,
    setPreferences,
    setStatusPresets,
    setLoading,
    setError,
    clearError,
    clear
  }
})
