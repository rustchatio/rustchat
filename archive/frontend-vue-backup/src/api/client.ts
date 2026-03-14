import axios from 'axios'
import { useAuthStore } from '../stores/auth'
import { normalizeIdsDeep, shouldNormalizeHttpPayload } from '../utils/idCompat'

const client = axios.create({
    baseURL: import.meta.env.VITE_API_URL || '/api/v1',
})

client.interceptors.request.use(config => {
    const authStore = useAuthStore()
    if (authStore.token) {
        config.headers.Authorization = `Bearer ${authStore.token}`
    }

    if (shouldNormalizeHttpPayload(config.params)) {
        config.params = normalizeIdsDeep(config.params)
    }
    if (shouldNormalizeHttpPayload(config.data)) {
        config.data = normalizeIdsDeep(config.data)
    }

    return config
})

client.interceptors.response.use(
    response => {
        if (shouldNormalizeHttpPayload(response.data)) {
            response.data = normalizeIdsDeep(response.data)
        }
        return response
    },
    error => {
        if (error.response?.data && shouldNormalizeHttpPayload(error.response.data)) {
            error.response.data = normalizeIdsDeep(error.response.data)
        }
        if (error.response?.status === 401) {
            const authStore = useAuthStore()
            authStore.logout()
        }
        return Promise.reject(error)
    }
)

export default client
