import api from './client'

export interface PublicConfig {
    site_name: string
    logo_url?: string
    mirotalk_enabled?: boolean
    enable_sso: boolean
    require_sso: boolean
}

export const siteApi = {
    getInfo: () => api.get<PublicConfig>('/site/info'),
}
