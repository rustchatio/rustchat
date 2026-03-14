<script setup lang="ts">
import { ref, watch, computed, nextTick } from 'vue'
import { format, isSameYear, isToday, isYesterday } from 'date-fns'
import { ArrowDown } from 'lucide-vue-next'
import { useMessageStore } from '../../stores/messages'
import { useUnreadStore } from '../../stores/unreads'
import { usePresence, extractUserIds } from '../../composables/usePresence'
import MessageItem from './MessageItem.vue'

const props = defineProps<{
  channelId: string
}>()

const emit = defineEmits<{
  (e: 'reply', messageId: string): void
  (e: 'delete', messageId: string): void
  (e: 'edit', messageId: string): void
  (e: 'openProfile', userId: string): void
}>()

const messageStore = useMessageStore()
const unreadStore = useUnreadStore()
const presence = usePresence() // For batch fetching
const containerRef = ref<HTMLElement | null>(null)
const shouldAutoScroll = ref(true)
const showNewMessagesBtn = ref(false)

const messages = computed(() => messageStore.messagesByChannel[props.channelId] || [])
const readState = computed(() => unreadStore.getChannelReadState(props.channelId))

type TimelineItem =
  | { kind: 'date'; key: string; label: string }
  | { kind: 'message'; key: string; message: (typeof messages.value)[number] }

function formatDateSeparator(date: Date): string {
    if (!Number.isFinite(date.getTime())) {
        return ''
    }
    if (isToday(date)) {
        return 'Today'
    }
    if (isYesterday(date)) {
        return 'Yesterday'
    }
    if (isSameYear(date, new Date())) {
        return format(date, 'EEEE, MMM d')
    }
    return format(date, 'PP')
}

const timelineItems = computed<TimelineItem[]>(() => {
    const items: TimelineItem[] = []
    let lastDayKey: string | null = null

    for (const message of messages.value) {
        const date = new Date(message.timestamp)
        const dayKey = Number.isFinite(date.getTime()) ? format(date, 'yyyy-MM-dd') : message.id

        if (dayKey !== lastDayKey) {
            items.push({
                kind: 'date',
                key: `date-${dayKey}`,
                label: formatDateSeparator(date),
            })
            lastDayKey = dayKey
        }

        items.push({
            kind: 'message',
            key: `msg-${message.id}`,
            message,
        })
    }

    return items
})

// Batch fetch user statuses when messages change
watch(() => messages.value, (newMessages) => {
    if (newMessages.length > 0) {
        const userIds = extractUserIds(newMessages)
        if (userIds.length > 0) {
            presence.fetchMissing(userIds)
        }
    }
}, { immediate: true })

// Handle scroll events to detect if user is at bottom or top (infinite scroll)
async function handleScroll() {
  if (!containerRef.value || messageStore.loading) return
  
  const { scrollTop, scrollHeight, clientHeight } = containerRef.value
  const distanceToBottom = scrollHeight - scrollTop - clientHeight
  
  // 1. Auto-scroll logic
  const atBottom = distanceToBottom < 50
  shouldAutoScroll.value = atBottom
  
  if (atBottom) {
    showNewMessagesBtn.value = false
    
    // Mark as read if there are unreads
    if (unreadStore.getChannelUnreadCount(props.channelId) > 0) {
        unreadStore.markAsRead(props.channelId)
    }
  }

  // 2. Reverse infinite scroll (load older)
  if (scrollTop < 100 && messageStore.hasMoreOlder(props.channelId)) {
    const oldScrollHeight = scrollHeight
    await messageStore.fetchOlderMessages(props.channelId)
    
    // Preserve scroll position after prepending messages
    nextTick(() => {
      if (containerRef.value) {
        const newScrollHeight = containerRef.value.scrollHeight
        containerRef.value.scrollTop = newScrollHeight - oldScrollHeight
      }
    })
  }
}

// Scroll to bottom
function scrollToBottom(behavior: ScrollBehavior = 'auto') {
  if (!containerRef.value) return
  containerRef.value.scrollTo({
    top: containerRef.value.scrollHeight,
    behavior
  })
}

// Watch for NEW messages (not full fetch)
watch(() => messages.value.length, (newLen, oldLen) => {
  if (newLen > oldLen) {
    if (shouldAutoScroll.value) {
      nextTick(() => scrollToBottom('smooth'))
    } else {
      showNewMessagesBtn.value = true
    }
  }
})

// Watch for loading state change (e.g. refresh via WebSocket)
watch(() => messageStore.loading, (loading) => {
    if (!loading && !messageStore.isLoadingOlder) {
        nextTick(() => scrollToBottom('auto'))
    }
})

// Watch for channel change to refetch and reset scroll
watch(() => props.channelId, async (newId) => {
    if (newId) {
        showNewMessagesBtn.value = false
        shouldAutoScroll.value = true
        await messageStore.fetchMessages(newId)
        nextTick(() => scrollToBottom())
    }
}, { immediate: true })

const highlightedMessageId = ref<string | null>(null)

function scrollToMessage(messageId: string) {
  const element = containerRef.value?.querySelector(`[data-message-id="${messageId}"]`)
  if (element) {
    element.scrollIntoView({ behavior: 'smooth', block: 'center' })
    highlightedMessageId.value = messageId
    setTimeout(() => {
      highlightedMessageId.value = null
    }, 2000)
    shouldAutoScroll.value = false
  }
}

defineExpose({ scrollToMessage })

function handleReply(id: string) {
  emit('reply', id)
}

function handleDelete(id: string) {
  emit('delete', id)
}

function handleEdit(id: string) {
  emit('edit', id)
}

function handleOpenProfile(userId: string) {
  emit('openProfile', userId)
}
</script>

<template>
  <div 
    class="flex-1 overflow-y-auto custom-scrollbar relative bg-bg-surface-1" 
    ref="containerRef"
    @scroll="handleScroll"
  >
    <div class="max-w-[var(--msg-max-width)] mx-auto px-[var(--msg-gutter)] py-2 space-y-3">
        <!-- New Messages Floating Button -->
        <transition
          enter-active-class="transition-standard duration-200"
          enter-from-class="transform translate-y-4 opacity-0"
          enter-to-class="transform translate-y-0 opacity-100"
          leave-active-class="transition-standard duration-150"
          leave-from-class="transform translate-y-0 opacity-100"
          leave-to-class="transform translate-y-4 opacity-0"
        >
          <div 
            v-if="showNewMessagesBtn"
            @click="scrollToBottom('smooth')"
            class="fixed bottom-sp-7 left-1/2 -translate-x-1/2 z-10 bg-brand hover:bg-brand-hover text-white px-sp-4 py-sp-2 rounded-full shadow-2 cursor-pointer flex items-center space-x-sp-2 text-sm font-medium animate-bounce"
          >
            <ArrowDown class="w-4 h-4" />
            <span>New messages</span>
          </div>
        </transition>

        <!-- Loading State -->
        <div v-if="messageStore.loading" class="text-center text-text-3 py-6">
            <div class="animate-spin w-8 h-8 border-2 border-brand border-t-transparent rounded-full mx-auto mb-sp-2"></div>
            <p>Loading messages...</p>
        </div>

        <!-- Empty State -->
        <div v-else-if="messages.length === 0" class="text-center text-text-3 py-6">
            <p class="text-lg font-medium text-text-1">This is the start of the channel.</p>
            <p class="text-sm mt-sp-2">Send a message to start the conversation.</p>
        </div>

        <!-- Message List -->
        <div v-else class="space-y-[1px]">
            <template v-for="item in timelineItems" :key="item.key">
                <div
                    v-if="item.kind === 'date'"
                    class="flex items-center my-3 select-none"
                >
                    <div class="flex-1 h-px bg-border-1"></div>
                    <span class="px-3 text-[11px] font-semibold uppercase tracking-wide text-text-3">
                        {{ item.label }}
                    </span>
                    <div class="flex-1 h-px bg-border-1"></div>
                </div>

                <template v-else>
                    <!-- New Messages Divider -->
                    <div 
                        v-if="readState?.first_unread_message_id && Number(item.message.seq) === Number(readState.first_unread_message_id)" 
                        class="flex items-center my-4 py-1.5"
                    >
                        <div class="flex-1 h-px bg-rose-500/30"></div>
                        <div class="px-sp-3 flex items-center space-x-sp-2">
                            <span class="text-[11px] font-bold text-rose-500 uppercase tracking-widest">New Messages</span>
                        </div>
                        <div class="flex-1 h-px bg-rose-500/30"></div>
                    </div>

                    <MessageItem 
                        :message="item.message" 
                        :data-message-id="item.message.id"
                        :class="{ 'bg-brand/5 ring-1 ring-brand/20': highlightedMessageId === item.message.id }"
                        class="transition-standard rounded-r-1"
                        @reply="handleReply"
                        @delete="handleDelete"
                        @edit="handleEdit"
                        @openProfile="handleOpenProfile"
                    />
                </template>
            </template>
        </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #CBD5E1;
  border-radius: 6px;
}
.dark .custom-scrollbar::-webkit-scrollbar-thumb {
  background: #374151;
}
</style>
