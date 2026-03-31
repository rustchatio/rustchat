import { HttpClient } from './http/HttpClient'
import { useAuthStore } from '../stores/auth'
import { normalizeIdsDeep, shouldNormalizeHttpPayload } from '../utils/idCompat'

/**
 * Main API client for v1 endpoints
 * Replaces Axios with native Fetch-based HttpClient
 */
const client = new HttpClient({
    baseURL: import.meta.env.VITE_API_URL || '/api/v1',
    requestInterceptor: (config) => {
        const authStore = useAuthStore()

        // Add auth header if token exists
        if (authStore.token) {
            config.headers = {
                ...config.headers,
                Authorization: `Bearer ${authStore.token}`,
            }
        }

        // Normalize IDs in params and body
        if (shouldNormalizeHttpPayload(config.params)) {
            config.params = normalizeIdsDeep(config.params)
        }
        if (shouldNormalizeHttpPayload(config.data)) {
            config.data = normalizeIdsDeep(config.data)
        }

        return config
    },
    responseInterceptor: (response) => {
        // Normalize IDs in response data
        if (shouldNormalizeHttpPayload(response.data)) {
            response.data = normalizeIdsDeep(response.data)
        }

        // Handle 401 - logout
        if (response.status === 401) {
            const authStore = useAuthStore()
            authStore.logout()
        }

        return response
    },
})

export default client
