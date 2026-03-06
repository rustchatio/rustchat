<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { format, formatDistanceToNow } from 'date-fns'
import { Smile, MessageSquare, MoreHorizontal, Pencil, Trash2, Pin, X, Check, Bookmark, ArrowRight, Video, Phone, PhoneOff } from 'lucide-vue-next'
import type { Message } from '../../stores/messages'
import { useMessageStore } from '../../stores/messages'
import { useAuthStore } from '../../stores/auth'
import { useUnreadStore } from '../../stores/unreads'
import { useUIStore } from '../../stores/ui'
import { useConfigStore } from '../../stores/config'
import { postsApi } from '../../api/posts'
import EmojiPicker from '../atomic/EmojiPicker.vue'
import FilePreview from '../atomic/FilePreview.vue'
import RcAvatar from '../ui/RcAvatar.vue'
import ImageGallery from '../atomic/ImageGallery.vue'
import { getEmojiChar } from '../../utils/emoji'
import type { FileUploadResponse } from '../../api/files'

import { renderMarkdown } from '../../utils/markdown'

const props = defineProps<{
  message: Message
}>()

const emit = defineEmits<{
  (e: 'edit', id: string): void
  (e: 'delete', id: string): void
  (e: 'reply', id: string): void
  (e: 'update', id: string, content: string): void
  (e: 'openProfile', userId: string): void
}>()

function openUserProfile() {
  emit('openProfile', props.message.userId)
}

const authStore = useAuthStore()
const messageStore = useMessageStore()
const unreadStore = useUnreadStore()
const uiStore = useUIStore()
const configStore = useConfigStore()

const showActions = ref(false)
const showMenu = ref(false)
const showEmojiPicker = ref(false)
const deleting = ref(false)
const isEditing = ref(false)
const editContent = ref('')
const editInputRef = ref<HTMLTextAreaElement | null>(null)
const saving = ref(false)

const isOwnMessage = computed(() => authStore.user?.id === props.message.userId)
const isEdited = computed(() => Boolean(props.message.editedAt))
const canEditMessage = computed(() => {
    if (!isOwnMessage.value) {
        return false
    }

    const limit = Number(configStore.siteConfig.post_edit_time_limit_seconds ?? -1)
    if (Number.isNaN(limit) || limit < 0) {
        return true
    }
    if (limit === 0) {
        return false
    }

    const createdAt = new Date(props.message.timestamp).getTime()
    if (!Number.isFinite(createdAt)) {
        return false
    }
    return Date.now() - createdAt < limit * 1000
})
const isSystemMessage = computed(() => {
    const type = props.message.props?.type
    return type === 'system_join_leave' || type === 'system_purpose' || type === 'system_header'
})
const isVideoCall = computed(() => props.message.props?.type === 'video_call')
const isCallsProtocol = computed(() => props.message.props?.type === 'custom_com.mattermost.calls')

function joinCall() {
    if (!props.message.props) return
    const { meeting_url, mode } = props.message.props

    if (mode === 'embed_iframe') {
        uiStore.openVideoCall(meeting_url)
    } else {
        window.open(meeting_url, '_blank', 'noopener,noreferrer')
    }
}

async function handleSave() {
    try {
        if (props.message.isSaved) {
            await messageStore.unsaveMessage(props.message.id, props.message.channelId)
        } else {
            await messageStore.saveMessage(props.message.id, props.message.channelId)
        }
        showMenu.value = false
    } catch (e) {
        console.error('Failed to toggle save', e)
    }
}

async function handlePin() {
    try {
        if (props.message.isPinned) {
            await messageStore.unpinMessage(props.message.id, props.message.channelId)
        } else {
            await messageStore.pinMessage(props.message.id, props.message.channelId)
        }
        showMenu.value = false
    } catch (e) {
        console.error('Failed to toggle pin', e)
    }
}

async function handleMarkAsUnread() {
    try {
        await unreadStore.markAsUnreadFromPost(props.message.id);
        showMenu.value = false;
    } catch (e) {
        console.error('Failed to mark as unread', e)
    }
}

async function handleDelete() {
  if (!confirm('Delete this message?')) return
  
  deleting.value = true
  try {
    await postsApi.delete(props.message.id)
    emit('delete', props.message.id)
  } catch (e) {
    console.error('Failed to delete message', e)
  } finally {
    deleting.value = false
    showMenu.value = false
  }
}

function startEditing() {
  editContent.value = props.message.content
  isEditing.value = true
  showMenu.value = false
  nextTick(() => {
    editInputRef.value?.focus()
    editInputRef.value?.select()
  })
}

function cancelEditing() {
  isEditing.value = false
  editContent.value = ''
}

async function saveEdit() {
  if (!editContent.value.trim() || editContent.value === props.message.content) {
    cancelEditing()
    return
  }
  
  saving.value = true
  try {
    const { data: updatedPost } = await postsApi.update(props.message.id, editContent.value)
    messageStore.handleMessageUpdate(updatedPost)
    if (!updatedPost?.edited_at && !updatedPost?.edit_at) {
      messageStore.handleMessageUpdate({
        id: props.message.id,
        message: editContent.value,
        edited_at: new Date().toISOString(),
      })
    }
    emit('update', props.message.id, editContent.value)
    isEditing.value = false
  } catch (e) {
    console.error('Failed to update message', e)
  } finally {
    saving.value = false
  }
}

function handleEditKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    cancelEditing()
  } else if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    saveEdit()
  }
}

function handleReply() {
  emit('reply', props.message.id)
}

// Safe HTML escaping
// Safe HTML escaping

const formattedContent = computed(() => {
    return renderMarkdown(props.message.content, authStore.user?.username || undefined)
})

const isMentioned = computed(() => {
    const username = authStore.user?.username
    return username && props.message.content.includes(`@${username}`)
})

// Quick reactions
const quickEmojis = ['👍', '❤️', '😄']

// Gallery state
const showGallery = ref(false)
const galleryInitialIndex = ref(0)
const galleryImages = computed(() => 
  (props.message.files || []).filter(f => f.mime_type.startsWith('image/'))
)

function openGallery(file: FileUploadResponse) {
  const index = galleryImages.value.findIndex(f => f.id === file.id)
  if (index !== -1) {
    galleryInitialIndex.value = index
    showGallery.value = true
  }
}

async function handleEmojiSelect(emoji: string) {
    showEmojiPicker.value = false
    await toggleReaction(emoji)
}

async function toggleReaction(emoji: string) {
    const reaction = props.message.reactions?.find((r) => {
        const rendered = getEmojiChar(r.emoji)
        const selected = getEmojiChar(emoji)
        return r.emoji === emoji || rendered === emoji || rendered === selected
    })
    const me = authStore.user?.id
    if (!me) return

    const hasReacted = reaction?.users.includes(me)
    const emojiKey = reaction?.emoji || emoji

    try {
        if (hasReacted) {
            await postsApi.removeReaction(props.message.id, emojiKey)
            messageStore.handleReactionRemoved({
                post_id: props.message.id,
                user_id: me,
                emoji_name: emojiKey,
            })
        } else {
            await postsApi.addReaction(props.message.id, emojiKey)
            messageStore.handleReactionAdded({
                post_id: props.message.id,
                user_id: me,
                emoji_name: emojiKey,
            })
        }
    } catch (e) {
        console.error('Failed to toggle reaction', e)
    }
}
</script>

<template>
  <!-- System Message -->
  <div v-if="isSystemMessage" class="flex items-center px-3 py-0.5 -mx-3 hover:bg-surface-dim dark:hover:bg-gray-800/40 transition-standard">
    <div class="flex items-center text-xs text-gray-500 dark:text-gray-400 italic w-full">
        <ArrowRight class="w-3.5 h-3.5 mr-2 text-gray-400" />
        <span v-html="formattedContent"></span>
        <span class="ml-2 text-[10px] text-gray-400">
            {{ format(new Date(message.timestamp), 'h:mm a') }}
        </span>
    </div>
  </div>

  <!-- Video Call Message -->
  <div v-else-if="isVideoCall" class="flex items-start group px-3 py-1.5 hover:bg-surface-dim dark:hover:bg-gray-800/40 -mx-3 transition-standard relative">
    <div class="shrink-0 select-none mr-3 mt-1">
      <RcAvatar
        :userId="message.userId"
        :src="message.avatarUrl"
        :username="message.username"
        size="md"
        class="w-8 h-8 rounded-md"
      />
    </div>
    <div class="flex-1 min-w-0">
        <div class="flex items-baseline mb-1">
            <span class="font-bold text-[15px] leading-tight text-gray-900 dark:text-gray-100 mr-2">{{ message.username }}</span>
            <span class="text-[11px] text-gray-500 font-medium tracking-tight">{{ format(new Date(message.timestamp), 'h:mm a') }}</span>
        </div>
        <div class="bg-surface-dim dark:bg-slate-800/50 border border-border-dim dark:border-slate-700/50 rounded-xl p-3 inline-flex flex-col max-w-sm shadow-sm transition-all hover:border-primary/30">
            <div class="flex items-center gap-3 mb-3">
                <div class="w-10 h-10 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center">
                    <Video class="w-5 h-5 text-green-600 dark:text-green-400" />
                </div>
        <div>
                    <div class="font-semibold text-gray-900 dark:text-gray-100">Video Call</div>
                    <div class="text-[13px] text-gray-500 dark:text-gray-400">
                        {{ message.props?.status === 'ended' ? 'Call ended' : 'Ongoing call' }}
                    </div>
                </div>
            </div>
            <button 
                v-if="message.props?.status !== 'ended'"
                @click="joinCall"
                class="w-full bg-green-600 hover:bg-green-700 text-white text-sm font-bold py-2 rounded-lg transition-colors flex items-center justify-center gap-2"
            >
                <Video class="w-4 h-4" />
                Join Call
            </button>
            <div v-else class="text-xs font-medium text-gray-400 dark:text-gray-500 flex items-center gap-1.5 px-1">
                <span>Call ended</span>
                <span>•</span>
                <span>{{ message.props?.duration_text || 'Just now' }}</span>
            </div>
        </div>
    </div>
  </div>

  <!-- mattermost calls plugin messages -->
  <div v-else-if="isCallsProtocol" class="flex items-center px-3 py-1 -mx-3 hover:bg-surface-dim dark:hover:bg-gray-800/40 transition-standard group">
    <div class="flex items-center text-[13px] text-gray-500 dark:text-gray-400 w-full">
        <div class="w-8 h-8 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center mr-3 shrink-0">
            <Phone class="w-4 h-4 text-blue-600 dark:text-blue-400" v-if="!message.props?.end_at" />
            <PhoneOff class="w-4 h-4 text-gray-400" v-else />
        </div>
        <div class="flex-1">
            <span class="font-semibold text-gray-700 dark:text-gray-200">{{ message.username }}</span>
            <span class="mx-1">{{ message.props?.end_at ? 'ended the call' : 'started a call' }}</span>
            <span v-if="message.props?.duration" class="opacity-70">({{ Math.floor(message.props.duration / 1000 / 60) }}m {{ Math.floor((message.props.duration / 1000) % 60) }}s)</span>
        </div>
        <span class="ml-2 text-[10px] text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity">
            {{ format(new Date(message.timestamp), 'h:mm a') }}
        </span>
    </div>
  </div>

  <!-- Regular Message -->
  <div 
    v-else
    class="flex items-start group transition-standard relative border-l-2 border-transparent"
    :class="[
        uiStore.density === 'compact' ? 'py-1' : 'py-1.5',
        isMentioned ? 'bg-brand/5 border-l-brand' : 'hover:bg-bg-app border-l-transparent',
        { 
            'opacity-70': message.status === 'sending',
            'bg-danger/5': message.status === 'failed'
        }
    ]"
    @mouseenter="showActions = true"
    @mouseleave="showActions = false; showMenu = false; showEmojiPicker = false"
  >
    <!-- Mention Indicator -->
    <div v-if="isMentioned" class="absolute left-0 top-0 bottom-0 w-1 bg-yellow-600"></div>

    <!-- Avatar -->
    <div class="shrink-0 select-none mr-2 mt-0.5 cursor-pointer" @click="openUserProfile">
      <RcAvatar 
        :userId="message.userId"
        :src="message.avatarUrl" 
        :username="message.username" 
        size="md"
        class="w-[var(--avatar-size)] h-[var(--avatar-size)] rounded-r-1 hover:shadow-2 transition-standard"
      />
    </div>

    <div class="flex-1 min-w-0">
      <!-- Header -->
      <div class="flex items-baseline mb-0.5 space-x-1.5">
        <span 
          class="font-semibold text-sm leading-tight text-text-1 hover:underline cursor-pointer transition-colors hover:text-brand"
          @click="openUserProfile"
        >
          {{ message.username }}
        </span>
        <span class="text-xs text-text-3 font-medium tracking-tight hover:underline cursor-pointer">
          {{ format(new Date(message.timestamp), 'h:mm a') }}
        </span>
        <span v-if="isEdited" class="text-[11px] text-text-3 italic">Edited</span>
        <!-- Status Indicators -->
        <span v-if="message.status === 'sending'" class="text-[10px] text-gray-400 ml-2 italic font-medium animate-pulse">Sending...</span>
        <span v-if="message.status === 'failed'" class="text-[10px] text-red-500 ml-2 font-bold uppercase tracking-wider">Failed</span>
        
        <div v-if="message.isPinned || message.isSaved" class="flex items-center space-x-1 ml-2">
            <span v-if="message.isPinned" class="bg-gray-200 dark:bg-gray-700 text-[10px] px-1 rounded text-gray-600 dark:text-gray-300 font-medium flex items-center h-4">
                <Pin class="w-2.5 h-2.5 mr-0.5" />
                Pinned
            </span>
            <span v-if="message.isSaved" class="bg-yellow-100 dark:bg-yellow-900/30 text-[10px] px-1 rounded text-yellow-700 dark:text-yellow-500 font-medium flex items-center h-4">
                <Bookmark class="w-2.5 h-2.5 mr-0.5 fill-current" />
                Saved
            </span>
        </div>
      </div>

      <!-- Body - Normal or Editing -->
      <div v-if="isEditing" class="mt-1 max-w-[70%]">
        <textarea
          ref="editInputRef"
          v-model="editContent"
          @keydown="handleEditKeydown"
          rows="2"
          class="w-full px-3 py-2 border border-blue-400 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white resize-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
        ></textarea>
        <div class="flex items-center space-x-2 mt-1">
          <button
            @click="saveEdit"
            :disabled="saving"
            class="px-2 py-1 bg-primary text-white text-xs rounded flex items-center space-x-1 hover:bg-blue-600 disabled:opacity-50"
          >
            <Check class="w-3 h-3" />
            <span>{{ saving ? 'Saving...' : 'Save' }}</span>
          </button>
          <button
            @click="cancelEditing"
            :disabled="saving"
            class="px-2 py-1 bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-200 text-xs rounded flex items-center space-x-1 hover:bg-gray-300"
          >
            <X class="w-3 h-3" />
            <span>Cancel</span>
          </button>
          <span class="text-xs text-gray-400">Esc to cancel • Enter to save</span>
        </div>
      </div>

      <div v-else class="relative group/content block">
        <div 
          class="text-text-1 text-chat-base mt-0.5 whitespace-pre-wrap leading-normal max-w-[95%] break-words selection:bg-brand/20"
          :class="{ 'bg-yellow-50/50 dark:bg-yellow-900/10 -mx-2 px-2 py-1 rounded': isMentioned }"
          v-html="formattedContent"
        ></div>

      </div>

      <div v-if="message.files && message.files.length > 0" class="mt-1.5 flex flex-wrap gap-1.5">
        <FilePreview
          v-for="file in message.files"
          :key="file.id"
          :file="file"
          @preview="openGallery"
        />
      </div>
      
      <!-- Thread Reply Count -->
      <div v-if="message.threadCount && message.threadCount > 0" class="mt-1.5">
        <button
          @click.stop="handleReply"
          class="flex items-center space-x-2 px-2 py-1 -ml-1 rounded-md hover:bg-blue-50 dark:hover:bg-blue-900/20 group/thread border border-transparent hover:border-blue-100 dark:hover:border-blue-800 transition-all"
        >
          <div class="flex -space-x-1.5 mr-1 pt-0.5">
             <MessageSquare class="w-3.5 h-3.5 text-blue-600 dark:text-blue-400" />
          </div>
          <span class="text-[13px] font-semibold text-blue-600 dark:text-blue-400">
            {{ message.threadCount }} {{ message.threadCount === 1 ? 'reply' : 'replies' }}
          </span>
          <span v-if="message.lastReplyAt" class="text-[11px] text-gray-400 group-hover/thread:text-gray-500">
            Last reply {{ formatDistanceToNow(new Date(message.lastReplyAt)) }} ago
          </span>
          <span class="text-[11px] text-blue-600 dark:text-blue-400 opacity-0 group-hover/thread:opacity-100 font-medium transition-opacity px-1">
            View thread
          </span>
        </button>
      </div>

      <!-- Reactions (Bottom Alignment) -->
      <div v-if="message.reactions && message.reactions.length > 0 && !isEditing" class="flex items-center mt-1.5 space-x-1.5 flex-wrap">
        <div 
          v-for="reaction in message.reactions" 
          :key="reaction.emoji"
          @click="toggleReaction(reaction.emoji)"
          class="bg-blue-50/80 dark:bg-blue-900/40 border border-blue-100/50 dark:border-blue-800/50 hover:border-blue-300 dark:hover:border-blue-600 rounded-lg px-2 py-0.5 text-[11px] cursor-pointer flex items-center space-x-1.5 transition-all select-none hover:scale-105 active:scale-95"
          :class="{ 'bg-brand/10 dark:bg-brand/20 border-brand/50 dark:border-brand/40 shadow-sm': reaction.users.includes(authStore.user?.id || '') }"
        >
          <span class="text-sm">{{ getEmojiChar(reaction.emoji) }}</span>
          <span class="font-bold text-blue-600 dark:text-blue-400" :class="{ 'text-brand dark:text-brand': reaction.users.includes(authStore.user?.id || '') }">{{ reaction.count }}</span>
        </div>
      </div>
    </div>

    <!-- Hover Actions -->
    <div 
      v-show="showActions && !isEditing"
      class="absolute left-12 bottom-0 translate-y-1/2 flex items-center bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-md z-10 px-1 py-0.5 transition-all duration-200 scale-95 origin-left hover:scale-100"
    >
      <!-- Quick Reactions -->
      <div class="flex items-center border-r border-gray-100 dark:border-gray-700 pr-1 mr-1">
        <button 
          v-for="emoji in quickEmojis" 
          :key="emoji"
          @click="toggleReaction(emoji)"
          class="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md transition-colors text-sm leading-none"
          :title="`React with ${emoji}`"
        >
          {{ emoji }}
        </button>
      </div>

      <button 
        @click="handleReply"
        class="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors rounded-md"
        title="Reply in thread"
      >
        <MessageSquare class="w-4 h-4" />
      </button>
      <div class="relative">
        <button 
          @click.stop="showEmojiPicker = !showEmojiPicker"
          class="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors rounded-md"
          :class="{ 'bg-gray-100 dark:bg-gray-700 text-indigo-500': showEmojiPicker }"
          title="Add reaction"
        >
          <Smile class="w-4 h-4" />
        </button>
        <EmojiPicker :show="showEmojiPicker" @select="handleEmojiSelect" @close="showEmojiPicker = false" />
      </div>
      <div class="relative">
        <button 
          @click.stop="showMenu = !showMenu"
          class="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors rounded-md"
          title="More actions"
        >
          <MoreHorizontal class="w-4 h-4" />
        </button>
        
        <!-- Dropdown Menu -->
        <div 
          v-if="showMenu"
          class="absolute right-0 top-full mt-1 w-40 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl z-10 py-1"
        >
          <button 
            v-if="isOwnMessage && canEditMessage"
            @click="startEditing"
            class="w-full px-3 py-1.5 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center"
          >
            <Pencil class="w-4 h-4 mr-2" />
            Edit message
          </button>
          <button 
            @click="handleSave"
            class="w-full px-3 py-1.5 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center"
          >
            <Bookmark class="w-4 h-4 mr-2" :class="{ 'fill-current': message.isSaved }" />
            {{ message.isSaved ? 'Unsave message' : 'Save message' }}
          </button>
          <button 
            @click="handlePin"
            class="w-full px-3 py-1.5 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center"
          >
            <Pin class="w-4 h-4 mr-2" :class="{ 'fill-current': message.isPinned }" />
            {{ message.isPinned ? 'Unpin from channel' : 'Pin to channel' }}
          </button>
          <button
            @click="handleMarkAsUnread"
            class="w-full px-3 py-1.5 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center"
          >
            <Check class="w-4 h-4 mr-2" />
            Mark as unread
          </button>
          <button 
            v-if="isOwnMessage"
            @click="handleDelete"
            :disabled="deleting"
            class="w-full px-3 py-1.5 text-left text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center"
          >
            <Trash2 class="w-4 h-4 mr-2" />
            {{ deleting ? 'Deleting...' : 'Delete message' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Image Gallery Lightbox -->
    <Teleport to="body">
      <ImageGallery 
        v-if="showGallery" 
        :images="galleryImages" 
        :initialIndex="galleryInitialIndex" 
        @close="showGallery = false" 
      />
    </Teleport>
  </div>
</template>
