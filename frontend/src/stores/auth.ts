import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useStorage } from '@vueuse/core'
import client from '../api/client'
import { useRouter } from 'vue-router'
import { useThemeStore } from './theme'

export const useAuthStore = defineStore('auth', () => {
    const token = useStorage('auth_token', '')
    const user = ref<any>(null)
    const router = useRouter()

    // Set MMAUTHTOKEN cookie for img/video tags that can't send headers
    function setAuthCookie(tokenValue: string) {
        document.cookie = `MMAUTHTOKEN=${tokenValue}; path=/; SameSite=Strict`
    }

    function clearAuthCookie() {
        document.cookie = 'MMAUTHTOKEN=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT'
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
            logout()
        }
    }

    async function logout() {
        token.value = ''
        clearAuthCookie()
        user.value = null
        router.push('/login')
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

    return { token, user, isAuthenticated, login, logout, fetchMe, updateStatus, authPolicy, getAuthPolicy }
})
