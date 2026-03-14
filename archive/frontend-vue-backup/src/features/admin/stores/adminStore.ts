// Admin Store - Pure state management for admin

import { defineStore } from 'pinia'
import { ref, readonly } from 'vue'
import type { ServerConfig, AdminUser, AuditLog, SystemStats, HealthStatus } from '../../../api/admin'

export const useAdminStore = defineStore('adminStore', () => {
  // State
  const config = ref<ServerConfig | null>(null)
  const users = ref<AdminUser[]>([])
  const usersTotal = ref(0)
  const auditLogs = ref<AuditLog[]>([])
  const stats = ref<SystemStats | null>(null)
  const health = ref<HealthStatus | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Actions
  function setConfig(value: ServerConfig | null) {
    config.value = value
  }

  function setUsers(value: AdminUser[]) {
    users.value = value
  }

  function addUser(user: AdminUser) {
    users.value.unshift(user)
    usersTotal.value++
  }

  function updateUser(user: AdminUser) {
    const index = users.value.findIndex(u => u.id === user.id)
    if (index !== -1) {
      users.value[index] = user
    }
  }

  function updateUserStatus(id: string, isActive: boolean) {
    const user = users.value.find(u => u.id === id)
    if (user) {
      user.is_active = isActive
    }
  }

  function setUsersTotal(value: number) {
    usersTotal.value = value
  }

  function setAuditLogs(value: AuditLog[]) {
    auditLogs.value = value
  }

  function setStats(value: SystemStats | null) {
    stats.value = value
  }

  function setHealth(value: HealthStatus | null) {
    health.value = value
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

  return {
    // State (readonly)
    config: readonly(config),
    users: readonly(users),
    usersTotal: readonly(usersTotal),
    auditLogs: readonly(auditLogs),
    stats: readonly(stats),
    health: readonly(health),
    loading: readonly(loading),
    error: readonly(error),

    // Actions
    setConfig,
    setUsers,
    addUser,
    updateUser,
    updateUserStatus,
    setUsersTotal,
    setAuditLogs,
    setStats,
    setHealth,
    setLoading,
    setError,
    clearError
  }
})
