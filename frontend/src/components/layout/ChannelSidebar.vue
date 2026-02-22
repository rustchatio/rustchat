<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import {
  Hash, Lock, ChevronDown, ChevronRight, Plus, MessageCircle, Settings, Compass, Shield, Check, LogOut, MoreVertical
} from 'lucide-vue-next';
import { useTeamStore } from '../../stores/teams';
import { useChannelStore } from '../../stores/channels';
import { useAuthStore } from '../../stores/auth';
import { usePresenceStore } from '../../features/presence';
import { useUnreadStore } from '../../stores/unreads';
import { useChannelPreferencesStore } from '../../stores/channelPreferences';
import CreateChannelModal from '../modals/CreateChannelModal.vue';
import DirectMessageModal from '../modals/DirectMessageModal.vue';
import TeamSettingsModal from '../modals/TeamSettingsModal.vue';
import BrowseTeamsModal from '../modals/BrowseTeamsModal.vue';
import BrowseChannelsModal from '../modals/BrowseChannelsModal.vue';
import ChannelContextMenu from '../channels/ChannelContextMenu.vue';
import AddChannelMembersModal from '../modals/AddChannelMembersModal.vue';
import ChannelSettingsModal from '../modals/ChannelSettingsModal.vue';
import type { SidebarCategory } from '../../api/channels';
import { channelRepository } from '../../features/channels/repositories/channelRepository';

const teamStore = useTeamStore();
const channelStore = useChannelStore();
const authStore = useAuthStore();
const presenceStore = usePresenceStore();
const unreadStore = useUnreadStore();
const channelPrefsStore = useChannelPreferencesStore();

const showCreateModal = ref(false);
const showDirectMessageModal = ref(false);
const showTeamSettings = ref(false);
const showTeamMenu = ref(false);
const showBrowseTeams = ref(false);
const showBrowseChannels = ref(false);
const showAddMembersModal = ref(false);
const addMembersChannelId = ref('');
const addMembersChannelName = ref('');

// Context menu state
const contextMenuChannel = ref<{
    id: string;
    name: string;
    type: 'public' | 'private' | 'dm' | 'group';
    unread: number;
    isOwner: boolean;
} | null>(null);
const contextMenuTrigger = ref<HTMLElement | null>(null);
const showMoveToModal = ref(false);
const moveToCategories = ref<SidebarCategory[]>([]);
const moveToChannelId = ref('');

// Channel settings modal state
const showChannelSettings = ref(false);

// Reload channels when team changes
watch(() => teamStore.currentTeamId, (teamId) => {
    if (teamId) {
        channelStore.fetchChannels(teamId);
        teamStore.fetchMembers(teamId);
    } else {
        channelStore.clearChannels();
    }
}, { immediate: true });

// Helper to deduplicate channels by ID
function dedupeChannels(channels: any[]) {
    const unique = new Map();
    channels.forEach(c => {
        if (!unique.has(c.id)) {
            unique.set(c.id, c);
        }
    });
    return Array.from(unique.values());
}

const categories = computed(() => {
    const allChannels = dedupeChannels([
        ...channelStore.publicChannels,
        ...channelStore.privateChannels,
        ...channelStore.directMessages
    ]);

    // Separate favorites
    const favoriteChannels = allChannels.filter(c => channelPrefsStore.isFavorite(c.id));
    const nonFavoritePublic = channelStore.publicChannels.filter(c => !channelPrefsStore.isFavorite(c.id));
    const nonFavoritePrivate = channelStore.privateChannels.filter(c => !channelPrefsStore.isFavorite(c.id));
    const nonFavoriteDMs = channelStore.directMessages.filter(c => !channelPrefsStore.isFavorite(c.id));

    const result = [];

    // Favorites category (if any favorites)
    if (favoriteChannels.length > 0) {
        result.push({
            id: 'favorites',
            name: 'Favorites',
            collapsed: false,
            channels: favoriteChannels.map(c => normalizeChannelForDisplay(c)),
        });
    }

    // Regular categories
    result.push(
        {
            id: 'channels',
            name: 'Channels',
            collapsed: false,
            channels: dedupeChannels(nonFavoritePublic).map(c => normalizeChannelForDisplay(c)),
        },
        {
            id: 'private',
            name: 'Private Channels',
            collapsed: false,
            channels: dedupeChannels(nonFavoritePrivate).map(c => normalizeChannelForDisplay(c)),
        },
        {
            id: 'dms',
            name: 'Direct Messages',
            collapsed: false,
            channels: dedupeChannels(nonFavoriteDMs).map(c => normalizeChannelForDisplay(c)),
        }
    );

    return result;
});

function normalizeChannelForDisplay(c: any) {
    let displayName = c.display_name || c.name;
    let otherId = '';
    let status = 'offline';
    
    // Handle DM channels
    if (c.channel_type === 'direct' || c.name?.startsWith('dm_')) {
        const parts = c.name.split('_');
        if (parts.length === 3) {
            otherId = parts[1] === authStore.user?.id ? parts[2] : parts[1];
            const member = teamStore.members.find(m => m.user_id === otherId);
            if (member) {
                displayName = member.display_name || member.username;
            }
        }
        
        // Get presence status
        const memberPresence = otherId
            ? teamStore.members.find(m => m.user_id === otherId)?.presence
            : undefined;
        status = otherId
            ? (presenceStore.presenceMap.get(otherId)?.presence || memberPresence || 'offline')
            : 'offline';
    }
    
    const channelType = c.channel_type === 'direct' ? 'dm' : 
                        c.channel_type === 'group' ? 'group' :
                        c.channel_type === 'private' ? 'private' : 'public';
    
    return {
        id: c.id,
        name: displayName,
        type: channelType,
        status: status,
        unread: unreadStore.getChannelUnreadCount(c.id),
        mention: (unreadStore.channelMentions[c.id] || 0) > 0,
        creator_id: c.creator_id,
    };
}

// Check if user is channel owner
function isChannelOwner(channel: any): boolean {
    return channel.creator_id === authStore.user?.id
}

// Check if user is admin
function isUserAdmin(): boolean {
    return ['system_admin', 'org_admin', 'admin'].includes(authStore.user?.role || '')
}

// Open context menu for a channel
function openContextMenu(channel: any, event: MouseEvent) {
    event.stopPropagation()
    const channelType = channel.type === 'dm' || channel.type === 'group' || channel.type === 'public' || channel.type === 'private'
        ? channel.type 
        : 'public'
    contextMenuChannel.value = {
        id: channel.id,
        name: channel.name,
        type: channelType,
        unread: channel.unread,
        isOwner: isChannelOwner(channel)
    }
    contextMenuTrigger.value = event.currentTarget as HTMLElement
}

// Close context menu
function closeContextMenu() {
    contextMenuChannel.value = null
    contextMenuTrigger.value = null
}

// Handle context menu action
function handleContextMenuAction(action: string) {
    console.log('Context menu action:', action)
    if (action === 'leave' || action === 'delete') {
        // Channel will be removed from store, refresh
        if (teamStore.currentTeamId) {
            channelStore.fetchChannels(teamStore.currentTeamId)
        }
    }
}

// Handle open add members modal
function handleOpenAddMembers() {
    if (contextMenuChannel.value) {
        addMembersChannelId.value = contextMenuChannel.value.id
        addMembersChannelName.value = contextMenuChannel.value.name
        showAddMembersModal.value = true
    }
}

// Handle open move to modal
function handleOpenMoveTo(cats: SidebarCategory[]) {
    moveToCategories.value = cats
    if (contextMenuChannel.value) {
        moveToChannelId.value = contextMenuChannel.value.id
        showMoveToModal.value = true
    }
}

// Handle open channel details
function handleOpenChannelDetails() {
    if (contextMenuChannel.value) {
        // Select the channel first so it's loaded in the store
        channelStore.selectChannel(contextMenuChannel.value.id)
        showChannelSettings.value = true
    }
}

// Handle move to category
async function handleMoveToCategory(cat: SidebarCategory) {
    if (!authStore.user?.id || !teamStore.currentTeamId) return
    try {
        await channelRepository.updateCategories(authStore.user.id, teamStore.currentTeamId, [cat])
        showMoveToModal.value = false
    } catch (e) {
        console.error('Failed to move channel:', e)
    }
}

// Fetch preferences on mount
onMounted(() => {
    channelPrefsStore.fetchPreferences()
})

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
                        :class="{ 'bg-green-500': channel.status === 'online', 'bg-transparent border-2 border-border-2': channel.status === 'offline', 'bg-yellow-500': channel.status === 'away', 'bg-red-500': channel.status === 'dnd' }"
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
                    :class="{ 'bg-green-500': channel.status === 'online', 'border border-border-2': channel.status === 'offline', 'bg-yellow-500': channel.status === 'away', 'bg-red-500': channel.status === 'dnd' }"
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

               <!-- Context Menu Trigger (3-dot) -->
               <button
                 @click.stop="openContextMenu(channel, $event)"
                 class="opacity-0 group-hover/item:opacity-100 flex items-center justify-center w-6 h-6 hover:bg-bg-surface-1 rounded transition-opacity"
                 title="More actions"
               >
                 <MoreVertical class="w-4 h-4 text-text-3" />
               </button>

               <!-- Context Menu (teleported to body to overlay above all panels) -->
               <Teleport to="body" v-if="contextMenuChannel?.id === channel.id">
                 <ChannelContextMenu
                   :channel-id="contextMenuChannel!.id"
                   :channel-name="contextMenuChannel!.name"
                   :channel-type="contextMenuChannel!.type"
                   :is-owner="contextMenuChannel!.isOwner"
                   :is-admin="isUserAdmin()"
                   :unread-count="channel.unread"
                   :trigger-element="contextMenuTrigger"
                   @close="closeContextMenu"
                   @action="handleContextMenuAction"
                   @open-add-members="handleOpenAddMembers"
                   @open-move-to="handleOpenMoveTo"
                   @open-details="handleOpenChannelDetails"
                 />
               </Teleport>
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

    <!-- Add Members Modal -->
    <AddChannelMembersModal
      :show="showAddMembersModal"
      :channel-id="addMembersChannelId"
      :channel-name="addMembersChannelName"
      @close="showAddMembersModal = false"
    />

    <!-- Move to Category Modal -->
    <Teleport to="body" v-if="showMoveToModal">
      <div class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="showMoveToModal = false"></div>
        <div class="relative bg-bg-surface-1 rounded-r-2 shadow-2xl w-64 py-2 border border-border-1">
          <div class="px-3 py-2 text-xs font-bold text-text-3 uppercase tracking-wider border-b border-border-1 mb-1">
            Move to...
          </div>
          <button
            v-for="cat in moveToCategories"
            :key="cat.id"
            @click="handleMoveToCategory(cat)"
            class="w-full px-4 py-2 text-left text-sm text-text-2 hover:bg-bg-surface-2 hover:text-text-1 transition-standard"
          >
            {{ cat.display_name }}
          </button>
          <div v-if="moveToCategories.length === 0" class="px-4 py-3 text-sm text-text-3 text-center">
            No categories available
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Channel Settings Modal -->
    <ChannelSettingsModal
      :isOpen="showChannelSettings"
      :channel="channelStore.currentChannel"
      @close="showChannelSettings = false"
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
