<script setup lang="ts">
import { X } from 'lucide-vue-next';
import { useUIStore } from '../../stores/ui';
import { useChannelStore } from '../../stores/channels';
import ThreadPanel from '../channel/ThreadPanel.vue';
import SearchPanel from '../channel/SearchPanel.vue';
import ChannelMembersPanel from '../channel/ChannelMembersPanel.vue';

const ui = useUIStore();
const channelStore = useChannelStore();
</script>

<template>
  <aside class="w-[400px] bg-surface dark:bg-surface-dim border-l border-border-dim dark:border-white/5 flex flex-col shadow-2xl z-20 shrink-0 transition-all duration-300">
    <!-- Header -->
    <div class="h-14 border-b border-border-dim dark:border-white/5 flex items-center justify-between px-4 shrink-0 bg-surface-dim dark:bg-surface-dim/50">
        <h3 class="font-bold text-[15px] text-gray-900 dark:text-gray-100 uppercase tracking-wider">
            <span v-if="ui.rhsView === 'thread'">Thread</span>
            <span v-else-if="ui.rhsView === 'search'">Search</span>
            <span v-else-if="ui.rhsView === 'info'">Channel Info</span>
            <span v-else-if="ui.rhsView === 'members'">Members</span>
        </h3>
        <button 
          @click="ui.closeRhs()" 
          class="p-1.5 hover:bg-gray-200 dark:hover:bg-gray-700/50 rounded-lg text-gray-500 transition-standard focus-ring"
          aria-label="Close sidebar"
          title="Close sidebar"
        >
            <X class="w-5 h-5" />
        </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto flex flex-col bg-surface dark:bg-surface-dim">
         <ThreadPanel v-if="ui.rhsView === 'thread'" />
         <SearchPanel v-if="ui.rhsView === 'search' && channelStore.currentChannelId" :channelId="channelStore.currentChannelId" @close="ui.closeRhs()" />
         <ChannelMembersPanel v-if="ui.rhsView === 'members' && channelStore.currentChannelId" :channelId="channelStore.currentChannelId" @close="ui.closeRhs()" />

         <div v-else-if="ui.rhsView === 'info'" class="flex-1 flex flex-col p-6 text-center text-gray-500">
             <p class="text-sm">Channel details and members will be refined soon.</p>
         </div>
         
         <div v-else-if="!ui.rhsView || !['thread', 'search', 'members'].includes(ui.rhsView)" class="flex-1 flex items-center justify-center text-gray-400">
             <p class="text-sm">No content</p>
         </div>
    </div>
  </aside>
</template>
