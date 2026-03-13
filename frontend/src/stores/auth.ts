import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { useStorage } from '@vueuse/core'
import client from '../api/client'
import { useThemeStore } from './theme'
import { useMessageStore } from './messages'
import { useChannelStore } from './channels'
import { useUnreadStore } from './unreads'
import { usePresenceStore } from './presence'
import { useTeamStore } from './teams'
import { useChannelPreferencesStore } from './channelPreferences'
import { useUIStore } from './ui'

type LogoutReason = 'manual' | 'expired' | 'unauthorized'

function parseJwtExpiryMs(tokenValue: string): number | null {
    if (!tokenValue) {
        return null
    }

    const parts = tokenValue.split('.')
    if (parts.length < 2) {
        return null
    }

    const payloadPart = parts[1]
    if (!payloadPart) {
        return null
    }

    const normalized = payloadPart.replace(/-/g, '+').replace(/_/g, '/')
    const paddingLength = (4 - (normalized.length % 4)) % 4
    const padded = normalized + '='.repeat(paddingLength)

    try {
        const payload = JSON.parse(atob(padded)) as { exp?: unknown }
        const expSeconds = Number(payload.exp)
        if (!Number.isFinite(expSeconds) || expSeconds <= 0) {
            return null
        }
        return expSeconds * 1000
    } catch {
        return null
    }
}

export const useAuthStore = defineStore('auth', () => {
    const token = useStorage('auth_token', '')
    const user = ref<any>(null)
    let tokenExpiryTimer: ReturnType<typeof setTimeout> | null = null
    let isLoggingOut = false

    // Set MMAUTHTOKEN cookie for img/video tags that can't send headers
    function setAuthCookie(tokenValue: string) {
        document.cookie = `MMAUTHTOKEN=${tokenValue}; path=/; SameSite=Strict`
    }

    function clearAuthCookie() {
        document.cookie = 'MMAUTHTOKEN=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT'
    }

    function clearTokenExpiryTimer() {
        if (tokenExpiryTimer) {
            clearTimeout(tokenExpiryTimer)
            tokenExpiryTimer = null
        }
    }

    async function clearSessionState() {
        useMessageStore().resetSessionState()
        useChannelStore().clearChannels()
        useUnreadStore().clearAllState()
        usePresenceStore().clear()
        useTeamStore().clear()
        useChannelPreferencesStore().clearState()

        const uiStore = useUIStore()
        uiStore.closeVideoCall()
        uiStore.closeRhs()
        uiStore.closeSettings()
        uiStore.closeLhs()

        // Avoid import cycles with the calls store by loading it lazily.
        try {
            const callsModule = await import('./calls')
            callsModule.useCallsStore().resetSessionState()
        } catch (error) {
            console.debug('Failed to reset calls state during logout', error)
        }
    }

    function scheduleTokenExpiryLogout() {
        clearTokenExpiryTimer()

        const expiryMs = parseJwtExpiryMs(token.value)
        if (!expiryMs) {
            return
        }

        const remainingMs = expiryMs - Date.now()
        if (remainingMs <= 0) {
            void logout('expired')
            return
        }

        tokenExpiryTimer = setTimeout(() => {
            void logout('expired')
        }, remainingMs)
    }

    const isAuthenticated = computed(() => !!token.value)

    async function login(credentials: any) {
        const { data } = await client.post('/auth/login', credentials)
        token.value = data.token
        setAuthCookie(data.token)
        user.value = data.user
        // Fetch full profile
        await fetchMe()
    }

    async function fetchMe() {
        if (!token.value) return
        // Sync cookie on page reload (token may be in localStorage but cookie cleared)
        setAuthCookie(token.value)
        try {
            const { data } = await client.get('/auth/me')
            // Map custom_status fields for easier access
            if (data.custom_status) {
                data.status_text = data.custom_status.text
                data.status_emoji = data.custom_status.emoji
                data.status_expires_at = data.custom_status.expires_at
            }
            user.value = data
            const themeStore = useThemeStore()
            await themeStore.syncFromServer()
        } catch (e) {
            await logout('unauthorized')
        }
    }

    async function logout(_reason: LogoutReason = 'manual') {
        if (isLoggingOut) {
            return
        }
        isLoggingOut = true
        clearTokenExpiryTimer()
        try {
            token.value = ''
            localStorage.setItem('auth_token', '')
            clearAuthCookie()
            user.value = null
            await clearSessionState()

            if (window.location.pathname !== '/login') {
                window.location.replace('/login')
            }
        } finally {
            isLoggingOut = false
        }
    }

    async function updateStatus(status: { status?: string, presence?: string, text?: string, emoji?: string, duration?: string, duration_minutes?: number, dnd_end_time?: number }) {
        if (!token.value) return
        try {
            const payload = { ...status }
            if (payload.presence && !payload.status) {
                payload.status = payload.presence
            }
            delete payload.presence

            const { data } = await client.put('/users/me/status', payload)
            if (user.value) {
                if (data.status) user.value.presence = data.status
                user.value.status_text = data.text
                user.value.status_emoji = data.emoji
                user.value.status_expires_at = data.expires_at

                // Also update the nested object to stay in sync
                if (data.text !== undefined || data.emoji !== undefined) {
                    user.value.custom_status = {
                        text: data.text,
                        emoji: data.emoji,
                        expires_at: data.expires_at
                    }
                }
            }
        } catch (e) {
            console.error('Failed to update status', e)
        }
    }

    const authPolicy = ref<any>(null)

    async function getAuthPolicy() {
        try {
            const { data } = await client.get('/auth/policy')
            authPolicy.value = data
            return data
        } catch (e) {
            console.error('Failed to fetch auth policy', e)
        }
    }

    watch(
        () => token.value,
        () => {
            if (!token.value) {
                clearTokenExpiryTimer()
                return
            }
            scheduleTokenExpiryLogout()
        },
        { immediate: true }
    )

    return { token, user, isAuthenticated, login, logout, fetchMe, updateStatus, authPolicy, getAuthPolicy }
})
