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
}

interface HydratedUserRecord {
  id: string
  username: string
  display_name?: string
  email?: string
  nickname?: string
  first_name?: string
  last_name?: string
  position?: string
  avatar_url?: string
  presence?: string
  status_text?: string
  status_emoji?: string
}

const hydratedUsers = ref<Record<string, HydratedUserRecord>>({})
const pendingUsers = ref<Record<string, boolean>>({})
const userErrors = ref<Record<string, string | undefined>>({})

function mergeUserSummary(userId: string, teamStore = useTeamStore(), presenceStore = usePresenceStore()): UserSummary | null {
  const teamMember = teamStore.members.find((member) => member.user_id === userId)
  const hydrated = hydratedUsers.value[userId]
  const livePresence = presenceStore.getUserPresence(userId).value?.presence
  const presence = normalizePresenceStatus(livePresence || hydrated?.presence || teamMember?.presence)

  if (!teamMember && !hydrated) {
    return null
  }

  return {
    id: userId,
    username: hydrated?.username || teamMember?.username || '',
    displayName: hydrated?.display_name || teamMember?.display_name,
    email: hydrated?.email,
    nickname: hydrated?.nickname,
    firstName: hydrated?.first_name,
    lastName: hydrated?.last_name,
    position: hydrated?.position,
    avatarUrl: hydrated?.avatar_url || teamMember?.avatar_url,
    presence,
    statusText: hydrated?.status_text,
    statusEmoji: hydrated?.status_emoji,
  }
}

export function clearUserSummaryCache() {
  hydratedUsers.value = {}
  pendingUsers.value = {}
  userErrors.value = {}
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
    if (!userId || hydratedUsers.value[userId] || pendingUsers.value[userId]) {
      return
    }

    pendingUsers.value = { ...pendingUsers.value, [userId]: true }
    void usersApi.get(userId)
      .then(({ data }) => {
        hydratedUsers.value = {
          ...hydratedUsers.value,
          [userId]: {
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
          },
        }
        userErrors.value = { ...userErrors.value, [userId]: undefined }
      })
      .catch((err: any) => {
        userErrors.value = {
          ...userErrors.value,
          [userId]: err?.response?.data?.message || 'Failed to load user profile',
        }
      })
      .finally(() => {
        pendingUsers.value = { ...pendingUsers.value, [userId]: false }
      })
  })

  return {
    userSummary,
    isLoading,
    error,
  }
}
