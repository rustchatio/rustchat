// Preferences Repository - Data access for user preferences

import { preferencesApi } from '../../../api/preferences'
import { withRetry } from '../../../core/services/retry'

export interface UserStatus {
  emoji?: string
  text?: string
  expiresAt?: string
}

export interface StatusPreset {
  emoji: string
  text: string
  durationMinutes?: number
}

export interface UserPreferences {
  [key: string]: unknown
  // Common preferences
  theme?: string
  timezone?: string
  locale?: string
  collapsedCategories?: string[]
}

export const preferencesRepository = {
  // Get current user status
  async getMyStatus(): Promise<UserStatus> {
    return withRetry(async () => {
      const response = await preferencesApi.getMyStatus()
      return {
        emoji: response.data.emoji ?? undefined,
        text: response.data.text ?? undefined,
        expiresAt: response.data.expires_at ?? undefined
      }
    })
  },

  // Update user status
  async updateMyStatus(data: {
    text?: string
    emoji?: string
    durationMinutes?: number
  }): Promise<UserStatus> {
    return withRetry(async () => {
      const response = await preferencesApi.updateMyStatus({
        text: data.text,
        emoji: data.emoji,
        duration_minutes: data.durationMinutes
      })
      return {
        emoji: response.data.emoji ?? undefined,
        text: response.data.text ?? undefined,
        expiresAt: response.data.expires_at ?? undefined
      }
    })
  },

  // Clear user status
  async clearMyStatus(): Promise<UserStatus> {
    return withRetry(async () => {
      const response = await preferencesApi.clearMyStatus()
      return {
        emoji: response.data.emoji ?? undefined,
        text: response.data.text ?? undefined,
        expiresAt: response.data.expires_at ?? undefined
      }
    })
  },

  // Get all user preferences
  async getMyPreferences(): Promise<UserPreferences> {
    return withRetry(async () => {
      const response = await preferencesApi.getMyPreferences()
      return normalizePreferences(response.data)
    })
  },

  // Update user preferences
  async updateMyPreferences(data: UserPreferences): Promise<UserPreferences> {
    return withRetry(async () => {
      const response = await preferencesApi.updateMyPreferences(data as any)
      return normalizePreferences(response.data)
    })
  },

  // Get status presets
  async listStatusPresets(): Promise<StatusPreset[]> {
    return withRetry(async () => {
      const response = await preferencesApi.listStatusPresets()
      return response.data.map((p: any) => ({
        emoji: p.emoji,
        text: p.text,
        durationMinutes: p.duration_minutes
      }))
    })
  }
}

function normalizePreferences(raw: any): UserPreferences {
  const prefs: UserPreferences = { ...raw }
  
  // Normalize common fields
  if (raw.collapsed_categories) {
    prefs.collapsedCategories = raw.collapsed_categories
  }
  
  return prefs
}
