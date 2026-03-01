<script setup lang="ts">
import { ref, watch } from 'vue'
import { X, LogOut, Bell, Monitor, Layout, Settings, Phone } from 'lucide-vue-next'
import DisplayTab from './display/DisplayTab.vue'
import SidebarTab from './sidebar/SidebarTab.vue'
import AdvancedTab from './advanced/AdvancedTab.vue'
import CallsTab from './calls/CallsTab.vue'
import NotificationsTab from './notifications/NotificationsTab.vue'
import ProfileTab from './profile/ProfileTab.vue'
import { useAuthStore } from '../../stores/auth'
import { useUIStore, type SettingsTab } from '../../stores/ui'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits(['close'])

const auth = useAuthStore()
const ui = useUIStore()
const activeTab = ref<SettingsTab>('notifications')
const error = ref('')
const success = ref('')

const tabs: Array<{ id: SettingsTab; label: string; icon: unknown }> = [
  { id: 'notifications', label: 'Notifications', icon: Bell },
  { id: 'display', label: 'Display', icon: Monitor },
  { id: 'sidebar', label: 'Sidebar', icon: Layout },
  { id: 'advanced', label: 'Advanced', icon: Settings },
]

const pluginTabs: Array<{ id: SettingsTab; label: string; icon: unknown }> = [
  { id: 'calls', label: 'Calls', icon: Phone },
]

const allTabs = [...tabs, ...pluginTabs, { id: 'profile' as SettingsTab, label: 'Profile', icon: Bell }]

// Reset state when modal opens
watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    error.value = ''
    success.value = ''
    activeTab.value = allTabs.some((tab) => tab.id === ui.settingsTab) ? ui.settingsTab : 'notifications'
  }
})

function setTab(tab: SettingsTab) {
  activeTab.value = tab
  ui.settingsTab = tab
}

function handleLogout() {
  emit('close')
  auth.logout()
}
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-4" role="dialog">
     <!-- Backdrop -->
    <div class="fixed inset-0 bg-gray-500/75 dark:bg-black/70" @click="$emit('close')"></div>
    
    <!-- Modal Panel -->
    <div class="relative bg-white dark:bg-gray-800 rounded-xl shadow-2xl ring-1 ring-black/5 w-full max-w-3xl max-h-[90vh] flex flex-col sm:flex-row overflow-hidden">
        
        <!-- Sidebar -->
        <div class="w-full sm:w-56 bg-gray-50 dark:bg-gray-900 border-b sm:border-b-0 sm:border-r border-gray-200 dark:border-gray-700 flex flex-col shrink-0">
            <div class="p-4 sm:p-6 font-bold text-lg dark:text-white">Settings</div>
            <nav class="flex sm:flex-col gap-1 px-2 sm:px-3 pb-2 sm:pb-0 overflow-x-auto sm:overflow-x-visible">
                <button
                    v-for="tab in tabs"
                    :key="tab.id"
                    @click="setTab(tab.id)"
                    class="flex items-center px-3 py-2 text-sm font-medium rounded-md whitespace-nowrap"
                    :class="activeTab === tab.id 
                        ? 'bg-white dark:bg-gray-800 text-primary shadow-sm border border-gray-200 dark:border-gray-700' 
                        : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'"
                >
                    <component :is="tab.icon" class="mr-2 flex-shrink-0 h-4 w-4 sm:h-5 sm:w-5" />
                    {{ tab.label }}
                </button>
            </nav>

            <div class="px-3 pt-2 pb-1 text-xs font-bold tracking-wide text-gray-500 dark:text-gray-400 border-t border-gray-200 dark:border-gray-700 mt-1">
              PLUGIN PREFERENCES
            </div>
            <nav class="flex sm:flex-col gap-1 px-2 sm:px-3 pb-2 overflow-x-auto sm:overflow-x-visible">
              <button
                v-for="tab in pluginTabs"
                :key="tab.id"
                @click="setTab(tab.id)"
                class="flex items-center px-3 py-2 text-sm font-medium rounded-md whitespace-nowrap"
                :class="activeTab === tab.id
                  ? 'bg-white dark:bg-gray-800 text-primary shadow-sm border border-gray-200 dark:border-gray-700'
                  : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'"
              >
                <component :is="tab.icon" class="mr-2 flex-shrink-0 h-4 w-4 sm:h-5 sm:w-5" />
                {{ tab.label }}
              </button>
            </nav>
            
            <div class="hidden sm:block mt-auto p-3 border-t border-gray-200 dark:border-gray-700">
              <button
                @click="handleLogout"
                class="flex items-center w-full px-3 py-2 text-sm font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-md transition-colors"
              >
                <LogOut class="mr-2 h-5 w-5" />
                Log out
              </button>
            </div>
        </div>

        <!-- Content -->
        <div class="flex-1 flex flex-col min-w-0 min-h-0">
            <div class="flex items-center justify-between px-4 sm:px-6 py-3 sm:py-4 border-b border-gray-200 dark:border-gray-700 shrink-0">
                <h3 class="text-base sm:text-lg font-medium leading-6 text-gray-900 dark:text-white">
                    {{ allTabs.find(t => t.id === activeTab)?.label || 'Settings' }}
                </h3>
                <button @click="$emit('close')" class="rounded-md bg-white dark:bg-gray-800 text-gray-400 hover:text-gray-500 focus:outline-none p-1">
                    <X class="h-5 w-5 sm:h-6 sm:w-6" />
                </button>
            </div>
            
            <div class="flex-1 overflow-y-auto p-4 sm:p-6">
                <!-- Messages -->
                <div v-if="error" class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-600 dark:text-red-400 text-sm">
                  {{ error }}
                </div>
                <div v-if="success" class="mb-4 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg text-green-600 dark:text-green-400 text-sm">
                  {{ success }}
                </div>

                <!-- Profile Tab -->
                <div v-if="activeTab === 'profile'">
                    <ProfileTab />
                </div>

                <!-- Notifications Tab -->
                <div v-else-if="activeTab === 'notifications'">
                    <NotificationsTab />
                </div>

                <!-- Display Tab -->
                <div v-else-if="activeTab === 'display'">
                    <div class="mb-4 text-sm text-gray-600 dark:text-gray-400">
                        Customize your display preferences.
                    </div>
                    <DisplayTab />
                </div>

                <!-- Sidebar Tab -->
                <div v-else-if="activeTab === 'sidebar'">
                    <div class="mb-4 text-sm text-gray-600 dark:text-gray-400">
                        Configure your sidebar preferences.
                    </div>
                    <SidebarTab />
                </div>

                <!-- Advanced Tab -->
                <div v-else-if="activeTab === 'advanced'">
                    <AdvancedTab />
                </div>

                <!-- Calls Tab -->
                <div v-else-if="activeTab === 'calls'">
                    <div class="text-sm text-gray-600 dark:text-gray-400 mb-4">
                        Configure your audio and video devices for calls.
                    </div>
                    <CallsTab />
                </div>

            </div>
        </div>
    </div>
  </div>
</template>
