// Config Store - Pure state management for site config

import { defineStore } from 'pinia'
import { ref, readonly } from 'vue'
import type { PublicConfig } from '../../../api/site'

const DEFAULT_CONFIG: PublicConfig = {
  site_name: 'RustChat',
  logo_url: undefined,
  mirotalk_enabled: false,
  enable_sso: false,
  require_sso: false
}

export const useConfigStore = defineStore('configStore', () => {
  // State
  const siteConfig = ref<PublicConfig>(DEFAULT_CONFIG)

  // Actions
  function setConfig(config: Partial<PublicConfig>) {
    siteConfig.value = { ...siteConfig.value, ...config }
  }

  return {
    // State (readonly)
    siteConfig: readonly(siteConfig),

    // Actions
    setConfig
  }
})
