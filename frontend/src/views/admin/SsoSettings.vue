<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useToast } from '../../composables/useToast'
import { adminApi, type SsoConfig, type CreateSsoConfigRequest, type AuthConfig } from '../../api/admin'
import { Shield, Plus, Edit2, Trash2, TestTube, AlertCircle, CheckCircle, ExternalLink, HelpCircle } from 'lucide-vue-next'

const toast = useToast()

// State
const ssoConfigs = ref<SsoConfig[]>([])
const authConfig = ref<AuthConfig | null>(null)
const loading = ref(false)
const showAddModal = ref(false)
const showEditModal = ref(false)
const showTestModal = ref(false)
const testingConfig = ref<SsoConfig | null>(null)
const testResult = ref<{ success: boolean; message: string; details: any } | null>(null)
const editingConfig = ref<SsoConfig | null>(null)
const siteUrl = ref('')

// Form state for creating/editing
const form = ref<CreateSsoConfigRequest & { id?: string }>({
  provider_type: 'oidc',
  provider_key: '',
  display_name: '',
  issuer_url: '',
  client_id: '',
  client_secret: '',
  scopes: [],
  is_active: true,
  auto_provision: true,
  default_role: 'member',
  allow_domains: [],
  github_org: '',
  github_team: '',
  groups_claim: 'groups',
  role_mappings: {},
})

const defaultScopes = {
  github: ['read:user', 'user:email'],
  google: ['openid', 'profile', 'email'],
  oidc: ['openid', 'profile', 'email'],
}

// Computed
const callbackUrl = computed(() => {
  return `${siteUrl.value}/api/v1/oauth2/${form.value.provider_key}/callback`
})

const isOidc = computed(() => form.value.provider_type === 'oidc' || form.value.provider_type === 'google')
const isGithub = computed(() => form.value.provider_type === 'github')

// Load data
onMounted(async () => {
  await Promise.all([loadSsoConfigs(), loadAuthConfig(), loadSiteUrl()])
})

async function loadSsoConfigs() {
  try {
    const response = await adminApi.getSsoConfigs()
    ssoConfigs.value = response.data
  } catch (error) {
    toast.error('Failed to load SSO configurations')
  }
}

async function loadAuthConfig() {
  try {
    const response = await adminApi.getConfig()
    authConfig.value = response.data.authentication
  } catch (error) {
    console.error('Failed to load auth config', error)
  }
}

async function loadSiteUrl() {
  try {
    const response = await adminApi.getConfig()
    siteUrl.value = response.data.site.site_url || window.location.origin
  } catch {
    siteUrl.value = window.location.origin
  }
}

async function updateAuthSettings() {
  if (!authConfig.value) return
  
  try {
    await adminApi.updateConfig('authentication', authConfig.value)
    toast.success('Authentication settings updated')
  } catch (error) {
    toast.error('Failed to update authentication settings')
  }
}

// Modal handlers
function openAddModal() {
  resetForm()
  showAddModal.value = true
}

function openEditModal(config: SsoConfig) {
  editingConfig.value = config
  form.value = {
    id: config.id,
    provider_type: config.provider_type,
    provider_key: config.provider_key,
    display_name: config.display_name || '',
    issuer_url: config.issuer_url || '',
    client_id: config.client_id || '',
    client_secret: '', // Don't populate secret
    scopes: config.scopes?.length ? config.scopes : defaultScopes[config.provider_type as keyof typeof defaultScopes],
    is_active: config.is_active,
    auto_provision: config.auto_provision,
    default_role: config.default_role || 'member',
    allow_domains: config.allow_domains || [],
    github_org: config.github_org || '',
    github_team: config.github_team || '',
    groups_claim: config.groups_claim || 'groups',
    role_mappings: config.role_mappings || {},
  }
  showEditModal.value = true
}

function resetForm() {
  form.value = {
    provider_type: 'oidc',
    provider_key: '',
    display_name: '',
    issuer_url: '',
    client_id: '',
    client_secret: '',
    scopes: [...defaultScopes.oidc],
    is_active: true,
    auto_provision: true,
    default_role: 'member',
    allow_domains: [],
    github_org: '',
    github_team: '',
    groups_claim: 'groups',
    role_mappings: {},
  }
  editingConfig.value = null
}

function onProviderTypeChange() {
  const type = form.value.provider_type
  form.value.scopes = [...defaultScopes[type as keyof typeof defaultScopes]]
  
  // Set default display name
  if (!form.value.display_name) {
    const defaults: Record<string, string> = {
      github: 'GitHub',
      google: 'Google',
      oidc: 'SSO',
    }
    form.value.display_name = defaults[type] || 'SSO'
  }
}

async function saveConfig() {
  loading.value = true
  try {
    const payload: CreateSsoConfigRequest = {
      ...form.value,
      // Only include non-empty arrays
      allow_domains: form.value.allow_domains?.filter(Boolean) || undefined,
    }

    // Remove empty strings for optional fields
    if (!payload.github_org) delete payload.github_org
    if (!payload.github_team) delete payload.github_team
    if (!payload.groups_claim) delete payload.groups_claim
    if (!payload.issuer_url) delete payload.issuer_url

    if (editingConfig.value) {
      // For updates, we use update API
      const updatePayload: any = { ...payload }
      delete updatePayload.provider_type // Can't change type on edit
      delete updatePayload.id
      
      // Only include client_secret if provided
      if (!updatePayload.client_secret) {
        delete updatePayload.client_secret
      }
      
      await adminApi.updateSsoConfig(editingConfig.value.id, updatePayload)
      toast.success('SSO configuration updated')
      showEditModal.value = false
    } else {
      await adminApi.createSsoConfig(payload)
      toast.success('SSO configuration created')
      showAddModal.value = false
    }
    
    await loadSsoConfigs()
  } catch (error: any) {
    const message = error.response?.data?.error || 'Failed to save SSO configuration'
    toast.error(message)
  } finally {
    loading.value = false
  }
}

async function deleteConfig(config: SsoConfig) {
  if (!confirm(`Are you sure you want to delete the "${config.display_name || config.provider_key}" configuration?`)) {
    return
  }
  
  try {
    await adminApi.deleteSsoConfig(config.id)
    toast.success('SSO configuration deleted')
    await loadSsoConfigs()
  } catch (error) {
    toast.error('Failed to delete SSO configuration')
  }
}

async function testConfig(config: SsoConfig) {
  testingConfig.value = config
  testResult.value = null
  showTestModal.value = true
  
  try {
    const response = await adminApi.testSsoConfig(config.id)
    testResult.value = response.data
  } catch (error: any) {
    testResult.value = {
      success: false,
      message: error.response?.data?.error || 'Test failed',
      details: null,
    }
  }
}

function getProviderIcon(type: string) {
  const icons: Record<string, string> = {
    github: 'GitHub',
    google: 'Google',
    oidc: 'SSO',
    saml: 'SAML',
  }
  return icons[type] || type
}

function getProviderBadgeClass(type: string) {
  const classes: Record<string, string> = {
    github: 'bg-gray-800 text-white',
    google: 'bg-blue-600 text-white',
    oidc: 'bg-purple-600 text-white',
    saml: 'bg-orange-600 text-white',
  }
  return classes[type] || 'bg-gray-600 text-white'
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
          <Shield class="w-6 h-6 text-indigo-600" />
          Single Sign-On (SSO)
        </h1>
        <p class="text-gray-600 dark:text-gray-400 mt-1">
          Configure OAuth2 and OIDC authentication providers
        </p>
      </div>
      <button
        @click="openAddModal"
        class="inline-flex items-center px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors"
      >
        <Plus class="w-4 h-4 mr-2" />
        Add Provider
      </button>
    </div>

    <!-- Global SSO Settings -->
    <div v-if="authConfig" class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Global SSO Settings</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <label class="flex items-center justify-between p-4 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700/50">
          <div>
            <div class="font-medium text-gray-900 dark:text-white">Enable SSO</div>
            <div class="text-sm text-gray-500 dark:text-gray-400">Allow users to sign in with configured providers</div>
          </div>
          <input
            v-model="authConfig.enable_sso"
            @change="updateAuthSettings"
            type="checkbox"
            class="w-5 h-5 text-indigo-600 rounded focus:ring-indigo-500"
          />
        </label>

        <label class="flex items-center justify-between p-4 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700/50">
          <div>
            <div class="font-medium text-gray-900 dark:text-white">Require SSO</div>
            <div class="text-sm text-gray-500 dark:text-gray-400">Disable password login, SSO only</div>
          </div>
          <input
            v-model="authConfig.require_sso"
            @change="updateAuthSettings"
            type="checkbox"
            class="w-5 h-5 text-indigo-600 rounded focus:ring-indigo-500"
          />
        </label>
      </div>
    </div>

    <!-- Provider List -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
      <div class="p-6 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Configured Providers</h2>
      </div>

      <div v-if="ssoConfigs.length === 0" class="p-8 text-center text-gray-500 dark:text-gray-400">
        <Shield class="w-12 h-12 mx-auto mb-4 opacity-50" />
        <p>No SSO providers configured yet.</p>
        <button
          @click="openAddModal"
          class="mt-4 text-indigo-600 hover:text-indigo-700 font-medium"
        >
          Add your first provider
        </button>
      </div>

      <div v-else class="divide-y divide-gray-200 dark:divide-gray-700">
        <div
          v-for="config in ssoConfigs"
          :key="config.id"
          class="p-6 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
              <span
                class="px-2 py-1 text-xs font-semibold rounded"
                :class="getProviderBadgeClass(config.provider_type)"
              >
                {{ getProviderIcon(config.provider_type) }}
              </span>
              <div>
                <h3 class="font-semibold text-gray-900 dark:text-white">
                  {{ config.display_name || config.provider_key }}
                </h3>
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  Key: {{ config.provider_key }}
                  <span v-if="config.issuer_url" class="ml-2">• {{ config.issuer_url }}</span>
                </p>
                <div class="flex items-center gap-3 mt-1 text-sm">
                  <span
                    :class="config.is_active ? 'text-green-600' : 'text-gray-500'"
                    class="flex items-center gap-1"
                  >
                    <span
                      class="w-2 h-2 rounded-full"
                      :class="config.is_active ? 'bg-green-500' : 'bg-gray-400'"
                    />
                    {{ config.is_active ? 'Active' : 'Inactive' }}
                  </span>
                  <span class="text-gray-400">|</span>
                  <span class="text-gray-600 dark:text-gray-400">
                    Auto-provision: {{ config.auto_provision ? 'On' : 'Off' }}
                  </span>
                  <span v-if="config.allow_domains?.length" class="text-gray-400">|</span>
                  <span v-if="config.allow_domains?.length" class="text-gray-600 dark:text-gray-400">
                    Domains: {{ config.allow_domains.join(', ') }}
                  </span>
                  <span v-if="config.github_org" class="text-gray-400">|</span>
                  <span v-if="config.github_org" class="text-gray-600 dark:text-gray-400">
                    Org: {{ config.github_org }}
                    <span v-if="config.github_team">/{{ config.github_team }}</span>
                  </span>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-2">
              <button
                @click="testConfig(config)"
                class="p-2 text-gray-600 hover:text-indigo-600 hover:bg-indigo-50 dark:text-gray-400 dark:hover:text-indigo-400 dark:hover:bg-indigo-900/20 rounded-lg transition-colors"
                title="Test Configuration"
              >
                <TestTube class="w-5 h-5" />
              </button>
              <button
                @click="openEditModal(config)"
                class="p-2 text-gray-600 hover:text-blue-600 hover:bg-blue-50 dark:text-gray-400 dark:hover:text-blue-400 dark:hover:bg-blue-900/20 rounded-lg transition-colors"
                title="Edit"
              >
                <Edit2 class="w-5 h-5" />
              </button>
              <button
                @click="deleteConfig(config)"
                class="p-2 text-gray-600 hover:text-red-600 hover:bg-red-50 dark:text-gray-400 dark:hover:text-red-400 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                title="Delete"
              >
                <Trash2 class="w-5 h-5" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <div
      v-if="showAddModal || showEditModal"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      @click.self="showAddModal = false; showEditModal = false"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
        <div class="p-6 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
            {{ editingConfig ? 'Edit Provider' : 'Add Provider' }}
          </h2>
        </div>

        <div class="p-6 space-y-6">
          <!-- Provider Type -->
          <div v-if="!editingConfig">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Provider Type *
            </label>
            <select
              v-model="form.provider_type"
              @change="onProviderTypeChange"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            >
              <option value="github">GitHub (OAuth2)</option>
              <option value="google">Google (OIDC)</option>
              <option value="oidc">Generic OIDC (Keycloak, ZITADEL, Authentik, etc.)</option>
            </select>
          </div>

          <!-- Provider Key -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Provider Key *
              <span class="text-gray-400 font-normal ml-1">(used in URLs, e.g., "github", "oidc-keycloak")</span>
            </label>
            <input
              v-model="form.provider_key"
              :disabled="!!editingConfig"
              type="text"
              placeholder="e.g., github"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white disabled:opacity-50"
            />
            <p class="text-xs text-gray-500 mt-1">
              Lowercase letters, numbers, and hyphens only. Cannot be changed after creation.
            </p>
          </div>

          <!-- Display Name -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Display Name
            </label>
            <input
              v-model="form.display_name"
              type="text"
              :placeholder="form.provider_type === 'github' ? 'GitHub' : form.provider_type === 'google' ? 'Google' : 'Single Sign-On'"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>

          <!-- OIDC-specific: Issuer URL -->
          <div v-if="isOidc">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Issuer URL *
              <span class="text-gray-400 font-normal ml-1">(e.g., https://accounts.google.com)</span>
            </label>
            <input
              v-model="form.issuer_url"
              type="url"
              placeholder="https://"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>

          <!-- Client ID -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Client ID *
            </label>
            <input
              v-model="form.client_id"
              type="text"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>

          <!-- Client Secret -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Client Secret {{ editingConfig ? '(leave blank to keep current)' : '*' }}
            </label>
            <input
              v-model="form.client_secret"
              type="password"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>

          <!-- Scopes -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Scopes
            </label>
            <div class="flex flex-wrap gap-2 mb-2">
              <span
                v-for="(scope, index) in form.scopes"
                :key="index"
                class="inline-flex items-center px-2 py-1 bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300 rounded text-sm"
              >
                {{ scope }}
                <button
                  @click="form.scopes.splice(index, 1)"
                  class="ml-1 hover:text-indigo-900"
                >
                  ×
                </button>
              </span>
            </div>
            <input
              type="text"
              placeholder="Add scope and press Enter"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              @keydown.enter.prevent="($event.target as HTMLInputElement).value && (form.scopes || []).push(($event.target as HTMLInputElement).value); ($event.target as HTMLInputElement).value = ''"
            />
          </div>

          <!-- Google-specific: Allowed Domains -->
          <div v-if="form.provider_type === 'google'">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Allowed Domains
              <span class="text-gray-400 font-normal ml-1">(optional, restrict to specific email domains)</span>
            </label>
            <div class="flex flex-wrap gap-2 mb-2">
              <span
                v-for="(domain, index) in (form.allow_domains || [])"
                :key="index"
                class="inline-flex items-center px-2 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded text-sm"
              >
                {{ domain }}
                <button
                  @click="(form.allow_domains || []).splice(index, 1)"
                  class="ml-1 hover:text-green-900"
                >
                  ×
                </button>
              </span>
            </div>
            <input
              type="text"
              placeholder="e.g., company.com"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              @keydown.enter.prevent="($event.target as HTMLInputElement).value && ((form.allow_domains || (form.allow_domains = [])).push(($event.target as HTMLInputElement).value)); ($event.target as HTMLInputElement).value = ''"
            />
          </div>

          <!-- GitHub-specific: Org/Team restrictions -->
          <div v-if="isGithub" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Required Organization
                <span class="text-gray-400 font-normal ml-1">(optional)</span>
              </label>
              <input
                v-model="form.github_org"
                type="text"
                placeholder="e.g., myorg"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Required Team
                <span class="text-gray-400 font-normal ml-1">(optional, within the organization)</span>
              </label>
              <input
                v-model="form.github_team"
                type="text"
                placeholder="e.g., developers"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>
          </div>

          <!-- OIDC-specific: Groups claim and role mappings -->
          <div v-if="isOidc" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Groups Claim
                <span class="text-gray-400 font-normal ml-1">(claim name in ID token containing user groups)</span>
              </label>
              <input
                v-model="form.groups_claim"
                type="text"
                placeholder="groups"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>
          </div>

          <!-- Settings -->
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <label class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer">
              <input
                v-model="form.is_active"
                type="checkbox"
                class="w-5 h-5 text-indigo-600 rounded"
              />
              <div>
                <div class="font-medium text-gray-900 dark:text-white">Active</div>
                <div class="text-xs text-gray-500">Show on login page</div>
              </div>
            </label>

            <label class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer">
              <input
                v-model="form.auto_provision"
                type="checkbox"
                class="w-5 h-5 text-indigo-600 rounded"
              />
              <div>
                <div class="font-medium text-gray-900 dark:text-white">Auto-Provision</div>
                <div class="text-xs text-gray-500">Create new users automatically</div>
              </div>
            </label>

            <div class="p-3 border border-gray-200 dark:border-gray-700 rounded-lg">
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Default Role
              </label>
              <select
                v-model="form.default_role"
                class="w-full px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
              >
                <option value="member">Member</option>
                <option value="team_admin">Team Admin</option>
                <option value="org_admin">Org Admin</option>
              </select>
            </div>
          </div>

          <!-- Callback URL Info -->
          <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
            <div class="flex items-start gap-3">
              <HelpCircle class="w-5 h-5 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5" />
              <div>
                <h4 class="font-medium text-blue-900 dark:text-blue-100">Callback URL</h4>
                <p class="text-sm text-blue-700 dark:text-blue-300 mt-1">
                  Configure this redirect URL in your {{ form.provider_type === 'github' ? 'GitHub OAuth app' : form.provider_type === 'google' ? 'Google Cloud Console' : 'OIDC provider' }}:
                </p>
                <code class="block mt-2 px-3 py-2 bg-blue-100 dark:bg-blue-900/40 rounded text-sm text-blue-800 dark:text-blue-200 break-all">
                  {{ callbackUrl }}
                </code>
              </div>
            </div>
          </div>
        </div>

        <div class="p-6 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-3">
          <button
            @click="showAddModal = false; showEditModal = false"
            class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            @click="saveConfig"
            :disabled="loading"
            class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50 transition-colors"
          >
            {{ loading ? 'Saving...' : (editingConfig ? 'Update' : 'Create') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Test Result Modal -->
    <div
      v-if="showTestModal"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      @click.self="showTestModal = false"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full">
        <div class="p-6 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
            Test {{ testingConfig?.display_name || testingConfig?.provider_key }}
          </h2>
        </div>

        <div class="p-6">
          <div v-if="!testResult" class="flex items-center justify-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
            <span class="ml-3 text-gray-600 dark:text-gray-400">Testing configuration...</span>
          </div>

          <div v-else>
            <div
              class="flex items-center gap-3 p-4 rounded-lg mb-4"
              :class="testResult.success ? 'bg-green-50 dark:bg-green-900/20 text-green-800 dark:text-green-200' : 'bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-200'"
            >
              <CheckCircle v-if="testResult.success" class="w-6 h-6" />
              <AlertCircle v-else class="w-6 h-6" />
              <span class="font-medium">{{ testResult.message }}</span>
            </div>

            <div v-if="testResult.details" class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 overflow-auto max-h-64">
              <pre class="text-sm text-gray-700 dark:text-gray-300">{{ JSON.stringify(testResult.details, null, 2) }}</pre>
            </div>
          </div>
        </div>

        <div class="p-6 border-t border-gray-200 dark:border-gray-700 flex justify-end">
          <button
            @click="showTestModal = false"
            class="px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
