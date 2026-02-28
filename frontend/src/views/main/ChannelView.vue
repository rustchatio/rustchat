<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useStorage } from '@vueuse/core';
import { useChannelStore } from '../../stores/channels';
import { useMessageStore } from '../../stores/messages';
import { useUnreadStore } from '../../stores/unreads';
import { useCallsStore } from '../../stores/calls';
import { useWebSocket } from '../../composables/useWebSocket';
import AppShell from '../../components/layout/AppShell.vue';
import ChannelHeader from '../../components/channel/ChannelHeader.vue';
import MessageList from '../../components/channel/MessageList.vue';
import MessageComposer from '../../components/composer/MessageComposer.vue';
import SavedMessagesPanel from '../../components/channel/SavedMessagesPanel.vue';
import PinnedMessagesPanel from '../../components/channel/PinnedMessagesPanel.vue';
import SearchPanel from '../../components/channel/SearchPanel.vue';
import ChannelMembersPanel from '../../components/channel/ChannelMembersPanel.vue';
import ChannelInfoPanel from '../../components/channel/ChannelInfoPanel.vue';
import ChannelSettingsModal from '../../components/modals/ChannelSettingsModal.vue';
import VideoCallModal from '../../components/modals/VideoCallModal.vue';
import UserProfileModal from '../../components/modals/UserProfileModal.vue';
import TypingIndicator from '../../components/channel/TypingIndicator.vue';
import ActiveCall from '../../components/calls/ActiveCall.vue';
import IncomingCallModal from '../../components/calls/IncomingCallModal.vue';
import { useUIStore, type RhsView } from '../../stores/ui';

const channelStore = useChannelStore();
const messageStore = useMessageStore();
const unreadStore = useUnreadStore();
const callsStore = useCallsStore();
const uiStore = useUIStore();
const { sendTyping, sendMessage, subscribe, unsubscribe } = useWebSocket();

// Persist RHS state per channel
const rhsStateByChannel = useStorage<Record<string, { view: RhsView; contextId?: string }>>('rhs_state_by_channel', {});

// Load active calls on mount
onMounted(async () => {
    await callsStore.loadConfig()
    await callsStore.loadCalls()
    
    // Add keyboard shortcuts
    document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
})

const currentChannel = computed(() => channelStore.currentChannel);
const channelId = computed(() => channelStore.currentChannelId);

const messageListRef = ref<any>(null);

// Channel settings modal
const showChannelSettings = ref(false);

// User profile modal
const showUserProfile = ref(false);
const profileUserId = ref<string | null>(null);

function handleOpenProfile(userId: string) {
  profileUserId.value = userId;
  showUserProfile.value = true;
}

// Save RHS state when channel changes or panel closes
watch(() => uiStore.rhsView, (view) => {
    if (channelId.value && view) {
        rhsStateByChannel.value[channelId.value] = {
            view,
            contextId: uiStore.rhsContextId || undefined
        }
    }
})

// Restore RHS state when channel changes
watch(channelId, (newId, oldId) => {
    if (oldId) {
        unsubscribe(oldId);
        // Save state for old channel
        if (uiStore.rhsView) {
            rhsStateByChannel.value[oldId] = {
                view: uiStore.rhsView,
                contextId: uiStore.rhsContextId || undefined
            }
        }
    }
    if (newId) {
        messageStore.fetchMessages(newId);
        subscribe(newId);
        
        // Restore RHS state for this channel
        const savedState = rhsStateByChannel.value[newId]
        if (savedState?.view) {
            // Small delay to ensure UI is ready
            setTimeout(() => {
                uiStore.openRhs(savedState.view, savedState.contextId)
            }, 50)
        }
    }
    showChannelSettings.value = false;
}, { immediate: true });

// Mark as read when channel changes
watch(channelId, (newId) => {
    if (newId) {
        unreadStore.markAsRead(newId);
    }
});

async function onSendMessage(data: { content: string, file_ids: string[] }) {
    if (channelId.value) {
        // Optimistic send via WebSocket
        await sendMessage(channelId.value, data.content, undefined, data.file_ids);
    }
}

function onTyping() {
    if (channelId.value) {
        sendTyping(channelId.value);
    }
}

function handleMessageReply(messageId: string) {
    uiStore.openRhs('thread', messageId);
}

function handleMessageDelete(messageId: string) {
    // Remove from local state - the API call is made in MessageItem
    if (channelId.value) {
        const messages = messageStore.messagesByChannel[channelId.value];
        if (messages) {
            const index = messages.findIndex(m => m.id === messageId);
            if (index !== -1) {
                messages.splice(index, 1);
            }
        }
    }
}

function handleMessageJump(messageId: string) {
    if (messageListRef.value) {
        messageListRef.value.scrollToMessage(messageId);
    }
}

function handleChannelDeleted() {
    channelStore.removeChannel(currentChannel.value?.id || '');
}

async function onStartAudioCall() {
    if (!channelId.value) return;
    
    try {
        // If there's an existing call in this channel, join it
        const existingCall = callsStore.currentChannelCall(channelId.value)
        if (existingCall) {
            await callsStore.joinCall(channelId.value)
        } else {
            // Start a new call
            await callsStore.startCall(channelId.value)
        }
    } catch (e) {
        console.error('Failed to start audio call', e);
        alert('Failed to start audio call');
    }
}

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
    // ESC to close RHS
    if (e.key === 'Escape' && uiStore.isRhsOpen) {
        uiStore.closeRhs()
        return
    }
    
    // Cmd/Ctrl + F to open search
    if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
        e.preventDefault()
        if (currentChannel.value) {
            uiStore.toggleRhs('search')
        }
        return
    }
    
    // Cmd/Ctrl + . to toggle thread panel (if there's a selected message)
    if ((e.metaKey || e.ctrlKey) && e.key === '.') {
        e.preventDefault()
        uiStore.toggleRhs('thread')
        return
    }
}
</script>

<template>
  <AppShell>
      <div class="flex h-full relative overflow-hidden">
          <!-- Background - uses theme surface color -->
          <div 
            class="absolute inset-0 pointer-events-none z-0"
            style="background-color: var(--bg-app);"
          ></div>

          <!-- Main Channel Area -->
          <div 
            class="relative flex flex-col flex-1 min-w-0 z-10 bg-transparent transition-all duration-300"
            :class="{ 'mr-0': !uiStore.isRhsOpen }"
          >
              <!-- No Channel Selected -->
              <div v-if="!currentChannel" class="flex-1 flex items-center justify-center text-slate-500">
                  <div class="text-center">
                      <p class="text-lg">Select a channel to start messaging</p>
                      <p class="text-sm mt-2">Choose a channel from the sidebar</p>
                  </div>
              </div>
              
              <!-- Channel View -->
              <template v-else>
                  <!-- Header -->
                  <ChannelHeader 
                      :name="currentChannel.display_name || currentChannel.name" 
                      :topic="currentChannel.purpose || currentChannel.header"
                      :channelType="currentChannel.channel_type"
                      :channelId="currentChannel.id"
                      @openSettings="showChannelSettings = true"
                  />
                  
                  <!-- Messages -->
                  <MessageList 
                    ref="messageListRef"
                    :channelId="currentChannel.id"
                    @reply="handleMessageReply"
                    @delete="handleMessageDelete"
                    @openProfile="handleOpenProfile"
                  />

                  <!-- Typing Indicator -->
                  <TypingIndicator :channelId="currentChannel.id" />

                  <!-- Composer -->
                  <MessageComposer 
                    @send="onSendMessage" 
                    @typing="onTyping" 
                    @startAudioCall="onStartAudioCall"
                  />
              </template>
          </div>

          <!-- RHS Panels Container -->
          <Transition
            enter-active-class="transition-transform duration-300 ease-out"
            enter-from-class="translate-x-full"
            enter-to-class="translate-x-0"
            leave-active-class="transition-transform duration-200 ease-in"
            leave-from-class="translate-x-0"
            leave-to-class="translate-x-full"
          >
            <div 
              v-if="uiStore.isRhsOpen"
              class="w-[400px] shrink-0 z-20 shadow-2xl border-l border-border-dim dark:border-white/5 overflow-hidden"
            >
              <ThreadPanel 
                v-if="uiStore.rhsView === 'thread'"
                @close="uiStore.closeRhs"
              />

              <SavedMessagesPanel 
                v-else-if="uiStore.rhsView === 'saved'"
                :show="true"
                @close="uiStore.closeRhs"
                @jump="handleMessageJump"
              />

              <PinnedMessagesPanel 
                v-else-if="uiStore.rhsView === 'pinned'"
                :show="true"
                @close="uiStore.closeRhs"
                @jump="handleMessageJump"
              />

              <SearchPanel 
                v-else-if="uiStore.rhsView === 'search' && currentChannel"
                :channelId="currentChannel.id"
                @close="uiStore.closeRhs"
                @jump="handleMessageJump"
              />

              <ChannelMembersPanel 
                v-else-if="uiStore.rhsView === 'members' && currentChannel"
                :channelId="currentChannel.id"
                @close="uiStore.closeRhs"
              />

              <ChannelInfoPanel 
                v-else-if="uiStore.rhsView === 'info' && currentChannel"
                :channelId="currentChannel.id"
                @close="uiStore.closeRhs"
                @openSettings="showChannelSettings = true"
              />
            </div>
          </Transition>
      </div>

      <!-- Channel Settings Modal -->
      <ChannelSettingsModal
        :isOpen="showChannelSettings"
        :channel="currentChannel"
        @close="showChannelSettings = false"
        @deleted="handleChannelDeleted"
      />

      <!-- Video Call Modal (Global for ChannelView context) -->
      <VideoCallModal
        :is-open="uiStore.isVideoCallOpen"
        :url="uiStore.videoCallUrl"
        @close="uiStore.closeVideoCall"
      />

      <!-- Active Call Widget -->
      <ActiveCall />

      <!-- Incoming Call Modal -->
      <IncomingCallModal />

      <!-- User Profile Modal -->
      <UserProfileModal
        :show="showUserProfile"
        :userId="profileUserId || ''"
        @close="showUserProfile = false"
      />
  </AppShell>
</template>
