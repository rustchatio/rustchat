<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { Bell, Search, HelpCircle, LogOut, Smile, Shield, User, Check } from 'lucide-vue-next';
import { useAuthStore } from '../../stores/auth';
import { useUIStore } from '../../stores/ui';
import SearchModal from '../modals/SearchModal.vue';
import SetStatusModal from '../modals/SetStatusModal.vue';
import RcAvatar from '../ui/RcAvatar.vue';
import NotificationsDropdown from './NotificationsDropdown.vue';
import { useConfigStore } from '../../stores/config';
import { usePresenceStore } from '../../features/presence';
import { useUnreadStore } from '../../stores/unreads';

const auth = useAuthStore();
const ui = useUIStore();
const configStore = useConfigStore();
const presenceStore = usePresenceStore();
const unreadStore = useUnreadStore();
const router = useRouter();

const showSearch = ref(false);
const showUserMenu = ref(false);
const showSetStatus = ref(false);
const showNotifications = ref(false);
const showDndSubmenu = ref(false);

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
        showDndSubmenu.value = false;
    }
}

onMounted(() => {
    document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown);
});

async function setPresence(status: 'online' | 'away' | 'dnd' | 'offline') {
    await auth.updateStatus({ status });
    presenceStore.updatePresenceFromEvent(auth.user?.id || '', status);
    showUserMenu.value = false;
    showDndSubmenu.value = false;
}

function calculateDndEndTime(duration: string): number | undefined {
    const now = new Date();
    switch (duration) {
        case 'thirty_minutes':
            return now.getTime() + 30 * 60 * 1000;
        case 'one_hour':
            return now.getTime() + 60 * 60 * 1000;
        case 'four_hours':
            return now.getTime() + 4 * 60 * 60 * 1000;
        case 'tomorrow': {
            const midnight = new Date(now);
            midnight.setHours(24, 0, 0, 0);
            return midnight.getTime();
        }
        case 'this_week': {
            const endOfWeek = new Date(now);
            const daysUntilSunday = (7 - endOfWeek.getDay()) % 7;
            endOfWeek.setDate(endOfWeek.getDate() + daysUntilSunday + 1);
            endOfWeek.setHours(0, 0, 0, 0);
            return endOfWeek.getTime();
        }
        default:
            return undefined;
    }
}

async function setDndWithDuration(duration: string) {
    const dndEndTime = calculateDndEndTime(duration);
    await auth.updateStatus({ 
        status: 'dnd',
        dnd_end_time: dndEndTime
    });
    presenceStore.updatePresenceFromEvent(auth.user?.id || '', 'dnd');
    showUserMenu.value = false;
    showDndSubmenu.value = false;
}

function openCustomStatus() {
    showSetStatus.value = true;
    showUserMenu.value = false;
}

function openProfile() {
    ui.openSettings('profile');
    showUserMenu.value = false;
}

const userPresence = computed(() => {
    return presenceStore.self?.presence || 'online';
});



const dndDurations = [
    { label: '30 minutes', value: 'thirty_minutes' },
    { label: '1 hour', value: 'one_hour' },
    { label: '4 hours', value: 'four_hours' },
    { label: 'Tomorrow', value: 'tomorrow' },
    { label: 'This week', value: 'this_week' },
];
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

        <!-- Mattermost-style Dropdown Menu -->
        <div v-if="showUserMenu" class="absolute top-full right-0 mt-2 w-64 bg-white dark:bg-gray-800 rounded-lg shadow-xl py-1 z-50 origin-top-right focus:outline-none ring-1 ring-black/5 dark:ring-white/10">
            
            <!-- Set Custom Status -->
            <button 
                @click="openCustomStatus"
                class="w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center transition-colors"
            >
                <span v-if="auth.user?.status_emoji" class="mr-3 text-lg">{{ auth.user.status_emoji }}</span>
                <Smile v-else class="w-4 h-4 mr-3 text-gray-400" />
                <span class="truncate">{{ auth.user?.status_text || 'Set custom status' }}</span>
            </button>

            <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>

            <!-- Online -->
            <button 
                @click="setPresence('online')"
                class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center justify-between transition-colors"
            >
                <div class="flex items-center">
                    <div class="w-2 h-2 rounded-full bg-green-500 mr-3"></div>
                    <span>Online</span>
                </div>
                <Check v-if="userPresence === 'online'" class="w-4 h-4 text-primary" />
            </button>

            <!-- Away -->
            <button 
                @click="setPresence('away')"
                class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center justify-between transition-colors"
            >
                <div class="flex items-center">
                    <div class="w-2 h-2 rounded-full bg-amber-500 mr-3"></div>
                    <span>Away</span>
                </div>
                <Check v-if="userPresence === 'away'" class="w-4 h-4 text-primary" />
            </button>

            <!-- Do Not Disturb (with submenu) -->
            <button 
                @click="showDndSubmenu = !showDndSubmenu"
                class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center justify-between transition-colors"
            >
                <div class="flex items-center">
                    <div class="w-2 h-2 rounded-full bg-red-500 mr-3"></div>
                    <div class="flex flex-col">
                        <span>Do not disturb</span>
                        <span class="text-xs text-gray-400">Pause notifications</span>
                    </div>
                </div>
                <div class="flex items-center">
                    <Check v-if="userPresence === 'dnd' && !showDndSubmenu" class="w-4 h-4 text-primary mr-2" />
                    <svg v-if="!showDndSubmenu" class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                    </svg>
                    <svg v-else class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                </div>
            </button>

            <!-- DND Duration Submenu -->
            <div v-if="showDndSubmenu" class="bg-gray-50 dark:bg-gray-900 py-1">
                <button 
                    v-for="duration in dndDurations" 
                    :key="duration.value"
                    @click="setDndWithDuration(duration.value)"
                    class="w-full text-left pl-11 pr-4 py-1.5 text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                >
                    {{ duration.label }}
                </button>
            </div>

            <!-- Offline -->
            <button 
                @click="setPresence('offline')"
                class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center justify-between transition-colors"
            >
                <div class="flex items-center">
                    <div class="w-2 h-2 rounded-full bg-gray-400 mr-3"></div>
                    <span>Offline</span>
                </div>
                <Check v-if="userPresence === 'offline'" class="w-4 h-4 text-primary" />
            </button>

            <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>

            <!-- Profile -->
            <button 
                @click="openProfile"
                class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center transition-colors"
            >
                <User class="w-4 h-4 mr-3 text-gray-400" />
                <span>Profile</span>
            </button>

            <!-- Admin Console (if admin) -->
            <button
                v-if="['system_admin', 'org_admin', 'admin', 'administrator'].includes(auth.user?.role)"
                @click="router.push('/admin'); showUserMenu = false"
                class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center transition-colors"
            >
                <Shield class="w-4 h-4 mr-3 text-gray-400" />
                <span>Admin Console</span>
            </button>

            <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>

            <!-- Log out -->
            <button 
                @click="auth.logout(); showUserMenu = false"
                class="w-full text-left px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center transition-colors"
            >
                <LogOut class="w-4 h-4 mr-3" />
                <span>Log out</span>
            </button>
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
