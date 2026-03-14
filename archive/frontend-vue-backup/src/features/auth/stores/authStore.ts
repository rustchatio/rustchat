// Auth Store - Pure state management for authentication
// No business logic - just state and simple mutations

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import type { User, UserId } from '../../../core/entities/User'
import type { AuthPolicy } from '../repositories/authRepository'

export const useAuthStore = defineStore('authStore', () => {
  // State
  const token = ref<string>('')
  const user = ref<User | null>(null)
  const authPolicy = ref<AuthPolicy | null>(null)
  const error = ref<string | null>(null)
  const isInitializing = ref(false)

  // Getters
  const isAuthenticated = computed(() => !!token.value && !!user.value)
  
  const currentUserId = computed((): UserId | null => {
    return user.value?.id || null
  })

  const isAdmin = computed(() => {
    return user.value?.role === 'system_admin' || user.value?.role === 'org_admin'
  })

  const isSystemAdmin = computed(() => {
    return user.value?.role === 'system_admin'
  })

  // Actions - Simple state mutations only
  function setToken(value: string) {
    token.value = value
  }

  function setUser(value: User | null) {
    user.value = value
  }

  function setAuthPolicy(value: AuthPolicy | null) {
    authPolicy.value = value
  }

  function setError(err: string | null) {
    error.value = err
  }

  function setInitializing(value: boolean) {
    isInitializing.value = value
  }

  function clearError() {
    error.value = null
  }

  function clear() {
    token.value = ''
    user.value = null
    error.value = null
  }

  return {
    // State (readonly)
    token: readonly(token),
    user: readonly(user),
    authPolicy: readonly(authPolicy),
    error: readonly(error),
    isInitializing: readonly(isInitializing),

    // Getters
    isAuthenticated,
    currentUserId,
    isAdmin,
    isSystemAdmin,

    // Actions
    setToken,
    setUser,
    setAuthPolicy,
    setError,
    setInitializing,
    clearError,
    clear
  }
})
