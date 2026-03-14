// Auth Repository - Data access for authentication

import client from '../../../api/client'
import type { User, UserId } from '../../../core/entities/User'
import { withRetry } from '../../../core/services/retry'

export interface LoginCredentials {
  email?: string
  username?: string
  password: string
}

export interface LoginResponse {
  token: string
  user: User
}

export interface AuthPolicy {
  allowSignup: boolean
  requireEmailVerification: boolean
  allowedDomains?: string[]
  minPasswordLength: number
}

export interface UpdateStatusRequest {
  presence?: 'online' | 'away' | 'dnd' | 'offline'
  text?: string
  emoji?: string
  durationMinutes?: number
}

// Cookie management helpers
const AUTH_COOKIE_NAME = 'MMAUTHTOKEN'
const AUTH_COOKIE_OPTIONS = 'path=/; SameSite=Strict'

export const authRepository = {
  // Login with credentials
  async login(credentials: LoginCredentials): Promise<LoginResponse> {
    return withRetry(async () => {
      const response = await client.post('/auth/login', credentials)
      
      // Normalize user data
      const user = normalizeUser(response.data.user)
      
      return {
        token: response.data.token,
        user
      }
    })
  },

  // Fetch current user profile
  async fetchMe(): Promise<User> {
    return withRetry(async () => {
      const response = await client.get('/auth/me')
      return normalizeUser(response.data)
    })
  },

  // Logout on server (optional, mainly for session invalidation)
  async logout(): Promise<void> {
    // Server-side logout if needed
    try {
      await client.post('/auth/logout')
    } catch {
      // Ignore errors during logout
    }
  },

  // Update user status/presence
  async updateStatus(request: UpdateStatusRequest): Promise<{
    presence?: string
    text?: string
    emoji?: string
    expiresAt?: string
  }> {
    return withRetry(async () => {
      const response = await client.put('/users/me/status', {
        presence: request.presence,
        text: request.text,
        emoji: request.emoji,
        duration_minutes: request.durationMinutes
      })
      return response.data
    })
  },

  // Get auth policy (signup settings, etc)
  async getAuthPolicy(): Promise<AuthPolicy> {
    return withRetry(async () => {
      const response = await client.get('/auth/policy')
      return {
        allowSignup: response.data.allow_signup ?? true,
        requireEmailVerification: response.data.require_email_verification ?? false,
        allowedDomains: response.data.allowed_domains,
        minPasswordLength: response.data.min_password_length ?? 8
      }
    })
  },

  // Cookie management
  setAuthCookie(token: string): void {
    document.cookie = `${AUTH_COOKIE_NAME}=${token}; ${AUTH_COOKIE_OPTIONS}`
  },

  clearAuthCookie(): void {
    document.cookie = `${AUTH_COOKIE_NAME}=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT`
  },

  // Token storage (localStorage)
  getStoredToken(): string | null {
    try {
      return localStorage.getItem('auth_token')
    } catch {
      return null
    }
  },

  setStoredToken(token: string): void {
    try {
      localStorage.setItem('auth_token', token)
    } catch {
      // Ignore storage errors
    }
  },

  clearStoredToken(): void {
    try {
      localStorage.removeItem('auth_token')
    } catch {
      // Ignore storage errors
    }
  }
}

// Normalize API user response to domain entity
function normalizeUser(raw: any): User {
  const customStatus = raw.custom_status || {
    text: raw.status_text,
    emoji: raw.status_emoji,
    expires_at: raw.status_expires_at
  }

  return {
    id: raw.id as UserId,
    username: raw.username,
    email: raw.email,
    displayName: raw.display_name,
    avatarUrl: raw.avatar_url || raw.profile_image,
    role: raw.role || 'user',
    presence: raw.presence || 'offline',
    isBot: raw.is_bot || false,
    timezone: raw.timezone,
    locale: raw.locale,
    customStatus: customStatus.text || customStatus.emoji ? {
      emoji: customStatus.emoji,
      text: customStatus.text,
      expiresAt: customStatus.expires_at ? new Date(customStatus.expires_at) : undefined
    } : undefined,
    createdAt: new Date(raw.created_at || Date.now()),
    updatedAt: new Date(raw.updated_at || raw.created_at || Date.now())
  }
}
