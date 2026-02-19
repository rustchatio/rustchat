import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { siteApi, type PublicConfig } from '../api/site'
import { adminApi, type AuthConfig } from '../api/admin'
import { useWebSocket } from '../composables/useWebSocket'

export interface FullConfig {
    site: PublicConfig
    authentication: AuthConfig
}

export const useConfigStore = defineStore('config', () => {
    const siteConfig = ref<PublicConfig>({
        site_name: 'RustChat',
        logo_url: undefined,
        mirotalk_enabled: false
    })
    
    const authConfig = ref<AuthConfig | null>(null)
    const configLoaded = ref(false)

    // Computed full config
    const config = computed<FullConfig | null>(() => {
        if (!authConfig.value) return null
        return {
            site: siteConfig.value,
            authentication: authConfig.value
        }
    })

    async function fetchPublicConfig() {
        try {
            const { data } = await siteApi.getInfo()
            siteConfig.value = data
        } catch (e) {
            console.error('Failed to fetch site config', e)
        }
    }

    async function loadConfig() {
        if (configLoaded.value) return
        
        await fetchPublicConfig()
        
        // Try to load auth config - this might fail if not authenticated
        // but that's okay, we just won't have SSO info
        try {
            const { data } = await adminApi.getConfig()
            authConfig.value = data.authentication
        } catch (e) {
            // Not authenticated or endpoint not available
            // Set default auth config
            authConfig.value = {
                enable_email_password: true,
                enable_sso: false,
                require_sso: false,
                allow_registration: true,
                enable_sign_in_with_email: true,
                enable_sign_in_with_username: true,
                enable_sign_up_with_email: true,
                enable_sign_up_with_gitlab: false,
                enable_sign_up_with_google: false,
                enable_sign_up_with_office365: false,
                enable_sign_up_with_openid: false,
                enable_user_creation: true,
                enable_open_server: false,
                enable_guest_accounts: false,
                enable_multifactor_authentication: false,
                enforce_multifactor_authentication: false,
                enable_saml: false,
                enable_ldap: false,
                password_min_length: 8,
                password_require_lowercase: true,
                password_require_uppercase: true,
                password_require_number: true,
                password_require_symbol: false,
                password_enable_forgot_link: true,
                session_length_hours: 24,
            }
        }
        
        configLoaded.value = true
    }

    function setAuthConfig(newConfig: AuthConfig) {
        authConfig.value = newConfig
    }

    // Initialize WebSocket listener for live updates
    function initSync() {
        const { onEvent } = useWebSocket()

        onEvent('config_updated', (data: any) => {
            if (data.category === 'site') {
                // Preserve other fields like mirotalk_enabled which are not in the 'site' category event yet
                siteConfig.value = {
                    ...siteConfig.value,
                    site_name: data.config.site_name,
                    logo_url: data.config.logo_url
                }
            } else if (data.category === 'authentication') {
                authConfig.value = { ...authConfig.value, ...data.config }
            }
        })
    }

    return { 
        siteConfig, 
        authConfig, 
        config,
        configLoaded,
        fetchPublicConfig, 
        loadConfig,
        setAuthConfig,
        initSync 
    }
})
