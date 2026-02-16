<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { Bell, Search, HelpCircle, LogOut, Settings, Smile, Shield } from 'lucide-vue-next';
import { useAuthStore } from '../../stores/auth';
import { useUIStore } from '../../stores/ui';
import SearchModal from '../modals/SearchModal.vue';
import SetStatusModal from '../modals/SetStatusModal.vue';
import RcAvatar from '../ui/RcAvatar.vue';
import NotificationsDropdown from './NotificationsDropdown.vue';
import { useConfigStore } from '../../stores/config';
import { usePresenceStore } from '../../stores/presence';
import { useUnreadStore } from '../../stores/unreads';

const auth = useAuthStore();
const ui = useUIStore();
const configStore = useConfigStore();
const presenceStore = usePresenceStore();
const unreadStore = useUnreadStore();

const showSearch = ref(false);
const showUserMenu = ref(false);
const showSetStatus = ref(false);
const showNotifications = ref(false);

// Initialize self presence
if (auth.user) {
  presenceStore.setSelfPresence({
    userId: auth.user.id,
    username: auth.user.username,
    presence: (auth.user.presence as any) || 'online'
  });
}

// Keyboard shortcut for search
function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        showSearch.value = true;
    }
    if (e.key === 'Escape') {
        showSearch.value = false;
        showUserMenu.value = false;
    }
}

onMounted(() => {
    document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown);
});

async function setPresence(status: 'online' | 'away' | 'dnd' | 'offline') {
    await auth.updateStatus({ presence: status });
    presenceStore.updatePresenceFromEvent(auth.user?.id || '', status);
    showUserMenu.value = false;
}

const userPresence = computed(() => {
    return presenceStore.self?.presence || 'online';
});

const statusColor = computed(() => {
    switch (userPresence.value) {
        case 'online': return 'bg-green-500';
        case 'away': return 'bg-amber-500';
        case 'dnd': return 'bg-red-500';
        case 'offline': return 'bg-gray-400';
        default: return 'bg-green-500';
    }
});

const statusLabel = computed(() => {
     switch (userPresence.value) {
        case 'online': return 'Online';
        case 'away': return 'Away';
        case 'dnd': return 'Do not disturb';
        case 'offline': return 'Offline';
        default: return 'Online';
    }
});
</script>

<template>
  <header class="h-[64px] bg-bg-surface-1 border-b border-border-1 flex items-center justify-between px-4 text-text-1 shrink-0 z-30 relative transition-standard">
    <!-- Left: Logo & Team -->
    <div class="flex items-center min-w-[200px]">
      <div class="font-bold text-lg tracking-tight mr-4 flex items-center">
        <img v-if="configStore.siteConfig.logo_url" :src="configStore.siteConfig.logo_url" class="w-[50px] h-[50px] rounded mr-2 object-cover" alt="Logo" />
        <div v-else class="w-8 h-8 bg-primary rounded mr-2 flex items-center justify-center text-sm font-bold">{{ configStore.siteConfig.site_name.charAt(0).toUpperCase() }}</div>
        {{ configStore.siteConfig.site_name }}
      </div>
    </div>

    <!-- Center: Search -->
    <div class="flex-1 max-w-2xl px-4 hidden sm:block">
        <div 
          @click="showSearch = true"
          class="flex items-center w-full bg-bg-surface-2 hover:bg-bg-surface-1 border border-border-1 rounded-r-2 px-4 py-2 cursor-pointer transition-standard group focus-ring shadow-1"
        >
          <Search class="w-4 h-4 text-text-3 group-hover:text-text-2 transition-colors mr-3" />
          <span class="text-sm text-text-3 group-hover:text-text-2 transition-colors flex-1">
            Search {{ configStore.siteConfig.site_name }}
          </span>
          <div class="flex items-center space-x-1 opacity-50 group-hover:opacity-100 transition-opacity">
            <kbd class="px-1.5 py-0.5 bg-bg-app border border-border-1 rounded text-[10px] font-bold text-text-2">⌘</kbd>
            <kbd class="px-1.5 py-0.5 bg-bg-app border border-border-1 rounded text-[10px] font-bold text-text-2">K</kbd>
          </div>
        </div>
    </div>

    <!-- Right: Actions -->
    <div class="flex items-center space-x-3">
      <button class="text-text-3 hover:text-text-1 transition-colors p-1.5 rounded-r-1 hover:bg-bg-surface-2">
        <HelpCircle class="w-5 h-5" />
      </button>
      
      <div class="relative">
        <button 
          @click="showNotifications = !showNotifications"
          class="relative text-text-3 hover:text-text-1 transition-standard p-1.5 rounded-r-1 hover:bg-bg-surface-2 focus-ring"
          aria-label="Toggle notifications"
          title="Notifications"
        >
          <Bell class="w-5 h-5" />
          <span 
            v-if="unreadStore.totalUnreadCount > 0"
            class="absolute top-1 right-1 block h-2.5 w-2.5 rounded-full ring-2 ring-bg-surface-1 bg-danger animate-pulse"
          ></span>
        </button>
        <NotificationsDropdown v-if="showNotifications" @close="showNotifications = false" />
        <div v-if="showNotifications" class="fixed inset-0 z-40" @click="showNotifications = false"></div>
      </div>
      

      <!-- User Menu -->
      <div class="ml-2 relative">
        <div class="cursor-pointer relative" @click="showUserMenu = !showUserMenu">
           <RcAvatar 
             :userId="auth.user?.id"
             :src="auth.user?.avatar_url" 
             :username="auth.user?.username" 
             size="md"
           />
        </div>

        <!-- Dropdown -->
        <div v-if="showUserMenu" class="absolute top-full right-0 mt-3 w-72 glass-panel rounded-2xl shadow-2xl py-2 z-50 origin-top-right focus:outline-none animate-fade-in ring-1 ring-black/5">
            <!-- User Info Section -->
            <div class="px-5 py-4 border-b border-black/5 dark:border-white/5 bg-black/5 dark:bg-white/5">
                <div class="flex items-center space-x-3">
                    <RcAvatar 
                      :userId="auth.user?.id"
                      :src="auth.user?.avatar_url" 
                      :username="auth.user?.username" 
                      size="lg"
                      class="ring-2 ring-primary/20"
                    />
                    <div class="flex-1 min-w-0">
                        <p class="text-[15px] font-bold text-gray-900 dark:text-white truncate">{{ auth.user?.display_name || auth.user?.username }}</p>
                        <div class="flex items-center mt-1">
                            <div class="h-2 w-2 rounded-full mr-2" :class="statusColor"></div>
                            <p class="text-xs font-medium text-gray-500 dark:text-gray-400 capitalize">{{ statusLabel }}</p>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Custom Status Button -->
            <div class="p-2">
                <button 
                    @click="showSetStatus = true; showUserMenu = false"
                    class="w-full text-left px-3 py-2.5 text-sm font-medium text-gray-700 dark:text-gray-200 hover:bg-primary/10 hover:text-primary dark:hover:text-primary-hover rounded-xl flex items-center transition-standard group focus-ring bg-white/50 dark:bg-black/20 border border-black/5 dark:border-white/5"
                >
                    <span v-if="auth.user?.status_emoji" class="mr-2 text-lg">{{ auth.user.status_emoji }}</span>
                    <Smile v-else class="w-4.5 h-4.5 mr-2 opacity-60 group-hover:opacity-100" />
                    <span class="truncate">{{ auth.user?.status_text || 'Set a custom status' }}</span>
                </button>
            </div>

            <div class="px-2 pb-2">
                <div class="text-[10px] font-bold text-gray-400 dark:text-gray-500 uppercase tracking-widest px-3 py-2">
                    Availability
                </div>
                <!-- Presence Options Grid -->
                <div class="grid grid-cols-2 gap-1 px-1">
                     <button @click="setPresence('online')" class="flex items-center space-x-2 px-3 py-2 text-xs font-semibold rounded-lg transition-standard focus-ring group" :class="userPresence === 'online' ? 'bg-primary/10 text-primary' : 'text-gray-600 dark:text-gray-400 hover:bg-black/5 dark:hover:bg-white/5'">
                        <div class="h-2 w-2 rounded-full bg-green-500 group-hover:scale-125 transition-transform"></div>
                        <span>Online</span>
                    </button>
                    <button @click="setPresence('away')" class="flex items-center space-x-2 px-3 py-2 text-xs font-semibold rounded-lg transition-standard focus-ring group" :class="userPresence === 'away' ? 'bg-amber-100/50 dark:bg-amber-900/20 text-amber-600 dark:text-amber-400' : 'text-gray-600 dark:text-gray-400 hover:bg-black/5 dark:hover:bg-white/5'">
                        <div class="h-2 w-2 rounded-full bg-amber-500 group-hover:scale-125 transition-transform"></div>
                        <span>Away</span>
                    </button>
                     <button @click="setPresence('dnd')" class="flex items-center space-x-2 px-3 py-2 text-xs font-semibold rounded-lg transition-standard focus-ring group" :class="userPresence === 'dnd' ? 'bg-red-100/50 dark:bg-red-900/20 text-red-600 dark:text-red-400' : 'text-gray-600 dark:text-gray-400 hover:bg-black/5 dark:hover:bg-white/5'">
                        <div class="h-2 w-2 rounded-full bg-red-500 group-hover:scale-125 transition-transform"></div>
                        <span>Busy</span>
                    </button>
                     <button @click="setPresence('offline')" class="flex items-center space-x-2 px-3 py-2 text-xs font-semibold rounded-lg transition-standard focus-ring group" :class="userPresence === 'offline' ? 'bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300' : 'text-gray-600 dark:text-gray-400 hover:bg-black/5 dark:hover:bg-white/5'">
                        <div class="h-2 w-2 rounded-full bg-gray-400 group-hover:scale-125 transition-transform"></div>
                        <span>Invisible</span>
                    </button>
                </div>
            </div>

            <div class="border-t border-black/5 dark:border-white/10 my-1"></div>

            <!-- Settings & Account -->
            <div class="p-2 space-y-0.5">
                <button
                  v-if="auth.user?.role === 'system_admin' || auth.user?.role === 'org_admin'"
                  @click="$router.push('/admin'); showUserMenu = false"
                  class="w-full text-left px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-primary/10 hover:text-primary rounded-xl flex items-center transition-standard focus-ring"
                >
                    <Shield class="w-4.5 h-4.5 mr-2.5 opacity-60" />
                    Admin Console
                </button>
                <button 
                  @click="ui.openSettings(); showUserMenu = false"
                  class="w-full text-left px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-primary/10 hover:text-primary rounded-xl flex items-center transition-standard focus-ring"
                >
                    <Settings class="w-4.5 h-4.5 mr-2.5 opacity-60" />
                    Preferences
                </button>
                <button 
                    @click="auth.logout()"
                     class="w-full text-left px-3 py-2 text-sm font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-xl flex items-center transition-standard focus-ring"
                >
                    <LogOut class="w-4.5 h-4.5 mr-2.5 opacity-60" />
                    Sign Out
                </button>
            </div>
        </div>
        
        <!-- Click outside -->
        <div v-if="showUserMenu" class="fixed inset-0 z-40" @click="showUserMenu = false"></div>
      </div>
    </div>

    <!-- Search Modal -->
    <SearchModal :show="showSearch" @close="showSearch = false" />
    <!-- Set Status Modal -->
    <SetStatusModal :show="showSetStatus" @close="showSetStatus = false" />
  </header>
</template>
