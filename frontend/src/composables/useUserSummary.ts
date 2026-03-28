import { computed, ref, watchEffect } from 'vue'
import { usersApi } from '../api/users'
import { usePresenceStore } from '../features/presence'
import { normalizePresenceStatus } from '../features/presence/presencePresentation'
import { useTeamStore } from '../stores/teams'

export interface UserSummary {
  id: string
  username: string
  displayName?: string
  email?: string
  nickname?: string
  firstName?: string
  lastName?: string
  position?: string
  avatarUrl?: string
  presence: 'online' | 'away' | 'dnd' | 'offline'
  statusText?: string
  statusEmoji?: string
  statusExpiresAt?: number
}

interface HydratedUserRecord {
  id: string
  username: string
  display_name?: string | null
  email?: string | null
  nickname?: string | null
  first_name?: string | null
  last_name?: string | null
  position?: string | null
  avatar_url?: string | null
  presence?: string | null
  status_text?: string | null
  status_emoji?: string | null
  status_expires_at?: string | number | null
  fully_hydrated?: boolean
}

interface LiveUserStatusUpdate {
  userId: string
  presence?: string | null
  statusText?: string | null
  statusEmoji?: string | null
  statusExpiresAt?: string | number | null
}

const hydratedUsers = ref<Record<string, HydratedUserRecord>>({})
const pendingUsers = ref<Record<string, boolean>>({})
const userErrors = ref<Record<string, string | undefined>>({})
const expiryTimers = new Map<string, ReturnType<typeof setTimeout>>()

function toExpiryMs(value: unknown): number | undefined {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value > 1_000_000_000_000 ? value : value * 1000
  }

  if (typeof value === 'string' && value.length > 0) {
    const numeric = Number(value)
    if (Number.isFinite(numeric) && numeric > 0) {
      return numeric > 1_000_000_000_000 ? numeric : numeric * 1000
    }

    const parsed = Date.parse(value)
    if (Number.isFinite(parsed)) {
      return parsed
    }
  }

  return undefined
}

function clearExpiryTimer(userId: string) {
  const timer = expiryTimers.get(userId)
  if (timer) {
    clearTimeout(timer)
    expiryTimers.delete(userId)
  }
}

function clearExpiredCustomStatus(userId: string) {
  const existing = hydratedUsers.value[userId]
  if (!existing) {
    return
  }

  hydratedUsers.value = {
    ...hydratedUsers.value,
    [userId]: {
      ...existing,
      status_text: null,
      status_emoji: null,
      status_expires_at: null,
    },
  }
}

function scheduleExpiry(userId: string, expiresAt: unknown) {
  clearExpiryTimer(userId)

  const expiresAtMs = toExpiryMs(expiresAt)
  if (!expiresAtMs) {
    return
  }

  const remainingMs = expiresAtMs - Date.now()
  if (remainingMs <= 0) {
    clearExpiredCustomStatus(userId)
    return
  }

  expiryTimers.set(
    userId,
    setTimeout(() => {
      clearExpiredCustomStatus(userId)
      expiryTimers.delete(userId)
    }, remainingMs),
  )
}

function upsertHydratedUser(userId: string, partial: Partial<HydratedUserRecord>) {
  const existing = hydratedUsers.value[userId]
  const next: HydratedUserRecord = {
    id: existing?.id || userId,
    username: existing?.username || '',
    ...existing,
    ...partial,
  }

  hydratedUsers.value = {
    ...hydratedUsers.value,
    [userId]: next,
  }

  scheduleExpiry(userId, next.status_expires_at)
}

function mergeUserSummary(userId: string, teamStore = useTeamStore(), presenceStore = usePresenceStore()): UserSummary | null {
  const teamMember = teamStore.members.find((member) => member.user_id === userId)
  const hydrated = hydratedUsers.value[userId]
  const livePresence = presenceStore.getUserPresence(userId).value?.presence
  const presence = normalizePresenceStatus(livePresence || hydrated?.presence || teamMember?.presence)
  const statusExpiresAt = toExpiryMs(hydrated?.status_expires_at)
  const statusExpired = Boolean(statusExpiresAt && statusExpiresAt <= Date.now())

  if (!teamMember && !hydrated) {
    return null
  }

  return {
    id: userId,
    username: hydrated?.username || teamMember?.username || '',
    displayName: hydrated?.display_name || teamMember?.display_name || undefined,
    email: hydrated?.email || undefined,
    nickname: hydrated?.nickname || undefined,
    firstName: hydrated?.first_name || undefined,
    lastName: hydrated?.last_name || undefined,
    position: hydrated?.position || undefined,
    avatarUrl: hydrated?.avatar_url || teamMember?.avatar_url || undefined,
    presence,
    statusText: statusExpired ? undefined : hydrated?.status_text || undefined,
    statusEmoji: statusExpired ? undefined : hydrated?.status_emoji || undefined,
    statusExpiresAt: statusExpired ? undefined : statusExpiresAt,
  }
}

async function ensureUserSummary(userId: string) {
  const existing = hydratedUsers.value[userId]
  if (!userId || pendingUsers.value[userId] || existing?.fully_hydrated) {
    return
  }

  pendingUsers.value = { ...pendingUsers.value, [userId]: true }

  try {
    const { data } = await usersApi.get(userId)
    upsertHydratedUser(userId, {
      id: data.id,
      username: data.username,
      display_name: data.display_name,
      email: data.email,
      nickname: (data as any).nickname,
      first_name: (data as any).first_name,
      last_name: (data as any).last_name,
      position: (data as any).position,
      avatar_url: data.avatar_url,
      presence: data.presence,
      status_text: (data as any).status_text,
      status_emoji: (data as any).status_emoji,
      status_expires_at: (data as any).status_expires_at,
      fully_hydrated: true,
    })
    userErrors.value = { ...userErrors.value, [userId]: undefined }
  } catch (err: any) {
    userErrors.value = {
      ...userErrors.value,
      [userId]: err?.response?.data?.message || 'Failed to load user profile',
    }
  } finally {
    pendingUsers.value = { ...pendingUsers.value, [userId]: false }
  }
}

export function getUserSummarySnapshot(userId: string) {
  return mergeUserSummary(userId)
}

export function prefetchUserSummaries(userIds: string[]) {
  const uniqueIds = [...new Set(userIds.filter(Boolean))]
  uniqueIds.forEach((userId) => {
    void ensureUserSummary(userId)
  })
}

export function applyUserStatusSnapshot(update: LiveUserStatusUpdate) {
  const partial: Partial<HydratedUserRecord> = {}

  if (update.presence !== undefined) {
    partial.presence = update.presence
  }
  if ('statusText' in update) {
    partial.status_text = update.statusText ?? null
  }
  if ('statusEmoji' in update) {
    partial.status_emoji = update.statusEmoji ?? null
  }
  if ('statusExpiresAt' in update) {
    partial.status_expires_at = update.statusExpiresAt ?? null
  }

  upsertHydratedUser(update.userId, partial)
}

export function clearUserSummaryCache() {
  hydratedUsers.value = {}
  pendingUsers.value = {}
  userErrors.value = {}
  expiryTimers.forEach((timer) => clearTimeout(timer))
  expiryTimers.clear()
}

export function useUserSummary(userIdSource: () => string | null | undefined) {
  const teamStore = useTeamStore()
  const presenceStore = usePresenceStore()

  const userSummary = computed(() => {
    const userId = userIdSource()
    if (!userId) {
      return null
    }
    return mergeUserSummary(userId, teamStore, presenceStore)
  })

  const isLoading = computed(() => {
    const userId = userIdSource()
    return !!(userId && pendingUsers.value[userId])
  })

  const error = computed(() => {
    const userId = userIdSource()
    return (userId && userErrors.value[userId]) || ''
  })

  watchEffect(() => {
    const userId = userIdSource()
    if (!userId) {
      return
    }
    void ensureUserSummary(userId)
  })

  return {
    userSummary,
    isLoading,
    error,
  }
}
