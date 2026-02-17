<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import {
  Hash, Lock, ChevronDown, ChevronRight, Plus, MessageCircle, Settings, Compass, Shield, Check, LogOut
} from 'lucide-vue-next';import { useTeamStore } from '../../stores/teams';
import { useChannelStore } from '../../stores/channels';
import { useAuthStore } from '../../stores/auth';
import { usePresenceStore } from '../../stores/presence';
import { useUnreadStore } from '../../stores/unreads';
import CreateChannelModal from '../modals/CreateChannelModal.vue';
import DirectMessageModal from '../modals/DirectMessageModal.vue';
import TeamSettingsModal from '../modals/TeamSettingsModal.vue';
import BrowseTeamsModal from '../modals/BrowseTeamsModal.vue';
import BrowseChannelsModal from '../modals/BrowseChannelsModal.vue';

const teamStore = useTeamStore();
const channelStore = useChannelStore();
const authStore = useAuthStore();
const presenceStore = usePresenceStore();
const unreadStore = useUnreadStore();

const showCreateModal = ref(false);
const showDirectMessageModal = ref(false);
const showTeamSettings = ref(false);
const showTeamMenu = ref(false);
const showBrowseTeams = ref(false);
const showBrowseChannels = ref(false);

// Reload channels when team changes
watch(() => teamStore.currentTeamId, (teamId) => {
    if (teamId) {
        channelStore.fetchChannels(teamId);
        teamStore.fetchMembers(teamId);
    } else {
        channelStore.clearChannels();
    }
}, { immediate: true });

const categories = computed(() => {
    // Helper to deduplicate channels by ID
    const dedupe = (channels: any[]) => {
        const unique = new Map();
        channels.forEach(c => {
            if (!unique.has(c.id)) {
                unique.set(c.id, c);
            }
        });
        const result = Array.from(unique.values());
        console.log('Deduped DMs:', result.map(c => ({ id: c.id, name: c.name })));
        return result;
    };

    return [
        {
            id: 'channels',
            name: 'Channels',
            collapsed: false,
            channels: dedupe(channelStore.publicChannels).map(c => ({
                id: c.id,
                name: c.display_name || c.name,
                type: 'public',
                unread: unreadStore.getChannelUnreadCount(c.id),
                mention: (unreadStore.channelMentions[c.id] || 0) > 0,
            })),
        },
        {
            id: 'private',
            name: 'Private Channels',
            collapsed: false,
            channels: dedupe(channelStore.privateChannels).map(c => ({
                id: c.id,
                name: c.display_name || c.name,
                type: 'private',
                unread: unreadStore.getChannelUnreadCount(c.id),
                mention: (unreadStore.channelMentions[c.id] || 0) > 0,
            })),
        },
        {
            id: 'dms',
            name: 'Direct Messages',
            collapsed: false,
            channels: dedupe(channelStore.directMessages).map(c => {
                let DisplayName = c.display_name || c.name;
                let otherId = '';
                
                // If it's a DM channel with a generated name, try to resolve the other user's name
                if (c.name.startsWith('dm_')) {
                    const parts = c.name.split('_');
                    if (parts.length === 3) {
                        otherId = parts[1] === authStore.user?.id ? parts[2] : parts[1];
                        const member = teamStore.members.find(m => m.user_id === otherId);
                        if (member) {
                            DisplayName = member.display_name || member.username;
                        }
                    }
                }
                
                // Prefer live WS presence, then fallback to team member snapshot from API.
                const memberPresence = otherId
                    ? teamStore.members.find(m => m.user_id === otherId)?.presence
                    : undefined;
                const status = otherId
                    ? (presenceStore.presenceMap.get(otherId)?.presence || memberPresence || 'offline')
                    : 'offline';
                
                return {
                    id: c.id,
                    name: DisplayName,
                    type: 'dm',
                    status: status,
                    unread: unreadStore.getChannelUnreadCount(c.id),
                    mention: (unreadStore.channelMentions[c.id] || 0) > 0,
                } as any;
            }),
        }
    ];
});

function selectChannel(channelId: string) {
    channelStore.selectChannel(channelId);
    // Mark as read when selecting
    unreadStore.markAsRead(channelId);
}

const collapsedCategories = ref(new Set<string>());

function toggleCategory(catId: string) {
    if (collapsedCategories.value.has(catId)) {
        collapsedCategories.value.delete(catId);
    } else {
        collapsedCategories.value.add(catId);
    }
}

function isCategoryCollapsed(catId: string) {
    return collapsedCategories.value.has(catId);
}

function handleTeamDeleted() {
    teamStore.removeTeam(teamStore.currentTeamId || '');
}

function handleAddCategory(catId: string) {
    if (catId === 'dms') {
        showDirectMessageModal.value = true;
    } else {
        showCreateModal.value = true;
    }
}

async function handleLeaveTeam() {
    if (!teamStore.currentTeam) return;
    if (!confirm(`Are you sure you want to leave ${teamStore.currentTeam.display_name || teamStore.currentTeam.name}?`)) return;
    
    try {
        await teamStore.leaveTeam(teamStore.currentTeam.id);
        showTeamMenu.value = false;
    } catch (e) {
        console.error('Failed to leave team', e);
    }
}
</script>

<template>
  <aside class="w-[232px] flex flex-col shrink-0 select-none relative z-20 bg-bg-surface-2 transition-standard">
    <!-- Content Layer -->
    <div class="relative flex-1 flex flex-col text-text-2">
    <!-- Team Header -->
    <div 
      class="h-12 flex items-center justify-between px-3 hover:bg-bg-surface-1 cursor-pointer transition-standard group relative border-b border-border-1"
      @click="showTeamMenu = !showTeamMenu"
    >
      <h2 class="font-bold truncate text-text-1 text-sm tracking-tight">
        {{ teamStore.currentTeam?.display_name || teamStore.currentTeam?.name || 'Select Team' }}
      </h2>
      <ChevronDown class="w-4 h-4 text-text-3 group-hover:text-text-1 transition-standard" :class="{ 'rotate-180': showTeamMenu }" />
      
      <!-- Team Dropdown Menu -->
      <div 
        v-if="showTeamMenu"
        class="absolute top-full left-sp-2 right-sp-2 mt-sp-1 bg-bg-surface-1 rounded-r-2 shadow-2 border border-border-1 py-1 z-50 animate-fade-in"
        @click.stop
      >
        <button
          v-if="authStore.user?.role === 'system_admin' || authStore.user?.role === 'org_admin'"
          @click="$router.push('/admin')"
          class="w-full flex items-center px-4 py-2 text-sm text-text-2 hover:bg-bg-surface-2 transition-standard"
        >
          <Shield class="w-4 h-4 mr-3 text-brand" />
          System Console
        </button>
        <div class="h-px bg-border-1 my-1"></div>
        <button
          @click="showBrowseTeams = true; showTeamMenu = false"
          class="w-full flex items-center px-4 py-2 text-sm text-text-2 hover:bg-bg-surface-2 transition-standard"
        >
          <Compass class="w-4 h-4 mr-3" />
          Browse Teams
        </button>
        <button
          @click="showTeamSettings = true; showTeamMenu = false"
          class="w-full flex items-center px-4 py-2 text-sm text-text-2 hover:bg-bg-surface-2 transition-standard"
        >
          <Settings class="w-4 h-4 mr-3" />
          Team Settings
        </button>
        <div class="h-px bg-border-1 my-1"></div>
        <button
          @click="handleLeaveTeam"
          class="w-full flex items-center px-4 py-2 text-sm text-danger hover:bg-danger/5 transition-standard"
        >
          <LogOut class="w-4 h-4 mr-3" />
          Leave Team
        </button>
      </div>
    </div>

    <!-- Click outside to close menu -->
    <div v-if="showTeamMenu" class="fixed inset-0 z-40" @click="showTeamMenu = false"></div>

    <!-- Scrollable Content -->
    <div class="flex-1 overflow-y-auto p-1.5 space-y-2 custom-scrollbar">
      
      <!-- Loading State -->
      <div v-if="channelStore.loading" class="text-center py-4 text-gray-500 text-sm">
          Loading channels...
      </div>

      <!-- Categories -->
      <div v-for="cat in categories" :key="cat.id">
        <!-- Category Header -->
        <div 
          class="flex items-center justify-between px-2 py-1 text-text-3 hover:text-text-1 cursor-pointer group mb-0.5"
          @click="toggleCategory(cat.id)"
        >
          <div class="flex items-center text-[11px] font-bold uppercase tracking-widest">
            <component :is="isCategoryCollapsed(cat.id) ? ChevronRight : ChevronDown" class="w-3.5 h-3.5 mr-1 text-text-3" />
            {{ cat.name }}
          </div>
          <button 
            @click.stop="handleAddCategory(cat.id)"
            class="opacity-0 group-hover:opacity-100 p-0.5 hover:bg-bg-surface-1 rounded transition-standard"
          >
            <Plus class="w-3.5 h-3.5" />
          </button>
        </div>

        <!-- Channels List -->
        <div v-if="!isCategoryCollapsed(cat.id)" class="space-y-0.5 mt-0.5">
          <div 
            v-for="channel in cat.channels" 
            :key="channel.id"
            @click="selectChannel(channel.id)"
            class="group/item relative px-2 py-1.5 mx-1.5 rounded-r-1 flex items-center justify-between cursor-pointer transition-standard"
            :class="{ 
              'bg-brand text-white shadow-1': channelStore.currentChannelId === channel.id, 
              'text-text-2 hover:bg-bg-surface-1 hover:text-text-1': channelStore.currentChannelId !== channel.id 
            }"
          >
            <!-- Active Pill Indicator (Removing sidebar pill for Slack style active row) -->

            <div class="flex items-center min-w-0" :class="{ 'opacity-100': channelStore.currentChannelId === channel.id, 'opacity-70 group-hover/item:opacity-100': channelStore.currentChannelId !== channel.id }">
              <!-- Icon -->
              <span class="mr-1.5 shrink-0 transition-opacity" :class="channelStore.currentChannelId === channel.id ? 'text-white' : 'text-text-3 opacity-80 group-hover/item:opacity-100'">
                <Hash v-if="channel.type === 'public'" class="w-4 h-4" />
                <Lock v-else-if="channel.type === 'private'" class="w-3.5 h-3.5" />
                <div v-else-if="channel.type === 'dm'" class="relative flex items-center justify-center w-3.5 h-3.5">
                    <div 
                        class="w-2.5 h-2.5 rounded-full border border-border-2"
                        :class="{ 'bg-green-500': channel.status === 'online', 'bg-transparent border-2 border-border-2': channel.status === 'offline', 'bg-yellow-500': channel.status === 'away' }"
                    ></div>
                </div>
                <MessageCircle v-else class="w-4 h-4" />
              </span>
              
              <!-- Name -->
              <span class="truncate text-sm" :class="{ 'font-bold': channel.unread > 0, 'opacity-100': channelStore.currentChannelId === channel.id, 'opacity-90': channelStore.currentChannelId !== channel.id }">
                {{ channel.name }}
              </span>
            </div>

            <!-- Status Indicator for DM (Right Side) -->
            <div v-if="channel.type === 'dm'" class="ml-2 shrink-0">
                 <div 
                    class="w-2 h-2 rounded-full"
                    :class="{ 'bg-green-500': channel.status === 'online', 'border border-border-2': channel.status === 'offline', 'bg-yellow-500': channel.status === 'away' }"
                ></div>
            </div>

            <!-- Badges & Actions -->
            <div class="flex items-center ml-2 space-x-1.5 min-w-0">
               <!-- Mark as read on hover -->
               <button 
                 v-if="channel.unread > 0"
                 @click.stop="unreadStore.markAsRead(channel.id)"
                 class="opacity-0 group-hover/item:opacity-100 flex items-center justify-center w-5 h-5 hover:bg-slate-700/50 rounded transition-opacity"
                 title="Mark as read"
               >
                 <Check class="w-3.5 h-3.5 text-slate-300" />
               </button>

               <div v-if="channel.mention" class="shrink-0 w-2.5 h-2.5 rounded-full bg-danger ring-2 ring-bg-surface-2 shadow-[0_0_8px_rgba(239,68,68,0.4)]"></div>
               <div v-if="channel.unread > 0" class="shrink-0 px-2 h-5 flex items-center justify-center rounded-full bg-bg-surface-1 text-[10px] font-bold text-text-1 border border-border-1" :class="{ 'bg-white/20 border-none text-white': channelStore.currentChannelId === channel.id }">
                 {{ channel.unread > 99 ? '99+' : channel.unread }}
               </div>
            </div>
          </div>
          
          <!-- Empty Category State -->
          <div v-if="cat.channels.length === 0" class="px-8 py-2 text-xs text-gray-600 italic">
            No channels
          </div>
        </div>
      </div>
    </div>

    <div class="p-1.5 border-t border-border-1 space-y-0.5">
        <button 
          v-if="Object.values(unreadStore.channelUnreads).some(c => (c as number) > 0)"
          @click="unreadStore.markAllAsRead()"
          class="w-full flex items-center justify-start px-2 py-1.5 text-xs text-text-3 hover:bg-bg-surface-1 hover:text-text-1 rounded-r-1 transition-standard text-left group"
        >
            <Check class="w-4 h-4 mr-3 text-text-3 group-hover:text-success" />
            <span>Mark all as read</span>
        </button>
        <button 
          @click="showBrowseChannels = true"
          class="w-full flex items-center justify-start px-2 py-1.5 text-xs text-text-3 hover:bg-bg-surface-1 hover:text-text-1 rounded-r-1 transition-standard text-left"
        >
            <Compass class="w-4 h-4 mr-3" />
            <span>Browse channels</span>
        </button>
        <button 
          @click="showCreateModal = true"
          class="w-full flex items-center justify-start px-2 py-1.5 text-xs text-text-3 hover:bg-bg-surface-1 hover:text-text-1 rounded-r-1 transition-standard text-left"
        >
            <Plus class="w-4 h-4 mr-3" />
            <span>Create channel</span>
        </button>
    </div>

    <!-- Create Channel Modal -->
    <CreateChannelModal :show="showCreateModal" @close="showCreateModal = false" />
    
    <!-- Direct Message Modal -->
    <DirectMessageModal :show="showDirectMessageModal" @close="showDirectMessageModal = false" />
    
    <!-- Team Settings Modal -->
    <TeamSettingsModal 
      :isOpen="showTeamSettings" 
      :team="teamStore.currentTeam"
      @close="showTeamSettings = false"
      @deleted="handleTeamDeleted"
    />

    <BrowseTeamsModal 
      v-if="showBrowseTeams"
      :open="showBrowseTeams"
      @close="showBrowseTeams = false"
    />

    <BrowseChannelsModal 
      v-if="showBrowseChannels"
      :open="showBrowseChannels"
      @close="showBrowseChannels = false"
    />
    </div>
  </aside>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #374151;
  border-radius: 4px;
}
</style>
