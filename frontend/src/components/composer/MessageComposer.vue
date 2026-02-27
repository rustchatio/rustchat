<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { Smile, Paperclip, Send, X, File as FileIcon, Phone, Type } from 'lucide-vue-next'
import { useToast } from '../../composables/useToast'
import { filesApi, type FileUploadResponse } from '../../api/files'
import FileUploader from '../atomic/FileUploader.vue'
import EmojiPicker from '../atomic/EmojiPicker.vue'
import FormattingToolbar from './FormattingToolbar.vue'
import MarkdownPreview from './MarkdownPreview.vue'
import MentionAutocomplete from './MentionAutocomplete.vue'
import { useTeamStore } from '../../stores/teams'
import { useCallsStore } from '../../stores/calls'
import { useChannelStore } from '../../stores/channels'
import { usePreferencesStore } from '../../stores/preferences'

const emit = defineEmits(['send', 'typing', 'startAudioCall'])

const toast = useToast()
const teamStore = useTeamStore()
const callsStore = useCallsStore()
const channelStore = useChannelStore()
const preferencesStore = usePreferencesStore()

const content = ref('')
const showEmojiPicker = ref(false)
const showPreview = ref(false)
const showMentionMenu = ref(false)
const showFormatting = ref(true)
const mentionQuery = ref('')
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const fileUploaderRef = ref<InstanceType<typeof FileUploader> | null>(null)
const isMac = ref(false)

const attachedFiles = ref<{
    file: File
    uploading: boolean
    progress: number
    uploaded?: FileUploadResponse
}[]>([])

let lastTypingEmit = 0
const TYPING_ACTIVITY_INTERVAL_MS = 2000
const DRAFT_STORAGE_PREFIX = 'rustchat_draft:'
const MAX_TEXTAREA_HEIGHT = 320

const placeholderText = computed(() => {
    const channelName = channelStore.currentChannel?.display_name || channelStore.currentChannel?.name
    return channelName ? `Message #${channelName}` : 'Write a message'
})

const sendOnCtrlEnter = computed(() => preferencesStore.preferences?.send_on_ctrl_enter ?? false)
const formattingAllowed = computed(() => preferencesStore.preferences?.enable_post_formatting !== false)
const showToolbar = computed(() => formattingAllowed.value && showFormatting.value)

const canSend = computed(() => {
    const hasContent = content.value.trim().length > 0
    const hasUploadedFiles = attachedFiles.value.some((file) => file.uploaded)
    const hasUploadInProgress = attachedFiles.value.some((file) => file.uploading)
    return (hasContent || hasUploadedFiles) && !hasUploadInProgress
})

const uploadInProgressCount = computed(
    () => attachedFiles.value.filter((attachment) => attachment.uploading).length
)

const sendShortcutLabel = computed(() => {
    if (!sendOnCtrlEnter.value) return 'Enter'
    return isMac.value ? 'Cmd+Enter' : 'Ctrl+Enter'
})

function getDraftKey(channelId?: string): string | null {
    if (!channelId) return null
    return `${DRAFT_STORAGE_PREFIX}${channelId}`
}

function loadDraft(channelId?: string): string {
    const key = getDraftKey(channelId)
    if (!key) return ''

    try {
        const raw = localStorage.getItem(key)
        if (!raw) return ''
        const parsed = JSON.parse(raw) as { content?: string; timestamp?: number }

        if (!parsed.timestamp || Date.now() - parsed.timestamp > 7 * 24 * 60 * 60 * 1000) {
            localStorage.removeItem(key)
            return ''
        }

        return parsed.content || ''
    } catch {
        return ''
    }
}

function saveDraft() {
    const key = getDraftKey(channelStore.currentChannelId ?? undefined)
    if (!key) return

    if (!content.value.trim()) {
        localStorage.removeItem(key)
        return
    }

    localStorage.setItem(
        key,
        JSON.stringify({
            content: content.value,
            timestamp: Date.now(),
        })
    )
}

function clearDraft() {
    const key = getDraftKey(channelStore.currentChannelId ?? undefined)
    if (!key) return
    localStorage.removeItem(key)
}

function autoResize() {
    const textarea = textareaRef.value
    if (!textarea) return

    textarea.style.height = 'auto'
    const nextHeight = Math.min(textarea.scrollHeight, MAX_TEXTAREA_HEIGHT)
    textarea.style.height = `${nextHeight}px`
    textarea.style.overflowY = textarea.scrollHeight > MAX_TEXTAREA_HEIGHT ? 'auto' : 'hidden'
}

function resetComposer() {
    content.value = ''
    attachedFiles.value = []
    showPreview.value = false
    showMentionMenu.value = false
    showEmojiPicker.value = false

    nextTick(() => {
        if (textareaRef.value) {
            textareaRef.value.style.height = 'auto'
            textareaRef.value.focus()
        }
    })
}

function resetForChannelChange(channelId?: string) {
    content.value = loadDraft(channelId)
    attachedFiles.value = []
    showPreview.value = false
    showMentionMenu.value = false
    showEmojiPicker.value = false
    mentionQuery.value = ''

    nextTick(() => {
        autoResize()
    })
}

function toggleFormatting() {
    if (!formattingAllowed.value) return
    showFormatting.value = !showFormatting.value
}

function handleGlobalKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.altKey && event.key.toLowerCase() === 't') {
        event.preventDefault()
        toggleFormatting()
    }
}

async function handleSlashCommand(command: string, args: string[]): Promise<boolean> {
    const channelId = channelStore.currentChannelId
    if (!channelId) return false

    switch (command) {
        case '/call':
        case '/call start': {
            const subCommand = args[0]

            if (subCommand === 'start' || !subCommand) {
                const existingCall = callsStore.currentChannelCall(channelId)
                if (existingCall) {
                    toast.error('A call is already in progress', 'Join the existing call instead')
                    return true
                }

                try {
                    await callsStore.startCall(channelId)
                    toast.success('Call started', 'You are now in a call')
                } catch (error: any) {
                    toast.error('Failed to start call', error.message || 'Unknown error')
                }
                return true
            }

            if (subCommand === 'join') {
                try {
                    await callsStore.joinCall(channelId)
                } catch (error: any) {
                    toast.error('Failed to join call', error.message || 'Unknown error')
                }
                return true
            }

            if (subCommand === 'leave') {
                if (callsStore.isInCall) {
                    await callsStore.leaveCall()
                } else {
                    toast.error('Not in a call', 'You are not currently in a call')
                }
                return true
            }

            if (subCommand === 'end') {
                if (callsStore.isInCall) {
                    await callsStore.endCall()
                } else {
                    toast.error('Not in a call', 'You are not currently in a call')
                }
                return true
            }

            return false
        }
        default:
            return false
    }
}

async function handleSend() {
    if (!canSend.value) return

    const trimmedText = content.value.trim()
    if (trimmedText.startsWith('/')) {
        const parts = trimmedText.split(' ')
        const command = parts[0] ?? ''
        const args = parts.slice(1)
        const firstArg = args[0] ?? ''

        let handled = false
        if (command === '/call' && firstArg) {
            handled = await handleSlashCommand(`${command} ${firstArg}`, args.slice(1))
        } else {
            handled = await handleSlashCommand(command, args)
        }

        if (handled) {
            clearDraft()
            resetComposer()
            return
        }
    }

    const fileIds = attachedFiles.value
        .filter((attachment) => !attachment.uploading && attachment.uploaded)
        .map((attachment) => attachment.uploaded!.id)

    emit('send', {
        content: content.value,
        file_ids: fileIds,
    })

    clearDraft()
    resetComposer()
}

function handleKeydown(event: KeyboardEvent) {
    if (showMentionMenu.value) {
        if (event.key === 'ArrowUp' || event.key === 'ArrowDown' || event.key === 'Enter') {
            event.preventDefault()
            return
        }

        if (event.key === 'Escape') {
            showMentionMenu.value = false
            event.preventDefault()
            return
        }
    }

    if (event.key === 'Escape' && showEmojiPicker.value) {
        showEmojiPicker.value = false
        event.preventDefault()
        return
    }

    if ((event.ctrlKey || event.metaKey) && !event.altKey) {
        if (event.key.toLowerCase() === 'b') {
            event.preventDefault()
            applyFormat('bold')
            return
        }

        if (event.key.toLowerCase() === 'i') {
            event.preventDefault()
            applyFormat('italic')
            return
        }
    }

    if (event.key !== 'Enter' || event.shiftKey) return

    if (sendOnCtrlEnter.value) {
        if (event.ctrlKey || event.metaKey) {
            event.preventDefault()
            void handleSend()
        }
        return
    }

    if (!event.ctrlKey && !event.metaKey) {
        event.preventDefault()
        void handleSend()
    }
}

function handleInput() {
    autoResize()
    saveDraft()

    const now = Date.now()
    if (content.value.trim().length > 0 && now - lastTypingEmit > TYPING_ACTIVITY_INTERVAL_MS) {
        lastTypingEmit = now
        emit('typing')
    }

    const textarea = textareaRef.value
    if (!textarea) return

    const cursorPos = textarea.selectionStart
    const textBefore = content.value.substring(0, cursorPos)
    const lastAt = textBefore.lastIndexOf('@')

    if (lastAt !== -1 && (lastAt === 0 || /\s/.test(textBefore.charAt(lastAt - 1)))) {
        const query = textBefore.substring(lastAt + 1)
        if (!/\s/.test(query)) {
            mentionQuery.value = query
            showMentionMenu.value = true

            if (teamStore.members.length === 0 && teamStore.currentTeamId) {
                teamStore.fetchMembers(teamStore.currentTeamId as string)
            }
            return
        }
    }

    showMentionMenu.value = false
}

function handleMentionSelect(username: string) {
    const textarea = textareaRef.value
    if (!textarea) return

    const cursorPos = textarea.selectionStart
    const textBefore = content.value.substring(0, cursorPos)
    const lastAt = textBefore.lastIndexOf('@')

    content.value = `${content.value.substring(0, lastAt)}@${username} ${content.value.substring(cursorPos)}`
    showMentionMenu.value = false
    saveDraft()

    textarea.focus()
    const newPos = lastAt + username.length + 2
    setTimeout(() => {
        textarea.setSelectionRange(newPos, newPos)
        autoResize()
    }, 0)
}

function applyFormat(type: string) {
    const textarea = textareaRef.value
    if (!textarea) return

    const start = textarea.selectionStart
    const end = textarea.selectionEnd
    const selectedText = content.value.substring(start, end)

    let before = ''
    let after = ''
    let prefix = ''

    switch (type) {
        case 'bold':
            before = '**'
            after = '**'
            break
        case 'italic':
            before = '*'
            after = '*'
            break
        case 'code':
            before = '`'
            after = '`'
            break
        case 'link':
            before = '['
            after = '](url)'
            break
        case 'quote':
            prefix = '> '
            break
        case 'bullet':
            prefix = '- '
            break
        case 'numbered':
            prefix = '1. '
            break
    }

    if (prefix) {
        const lineStart = content.value.lastIndexOf('\n', start - 1) + 1
        content.value = content.value.substring(0, lineStart) + prefix + content.value.substring(lineStart)
    } else {
        content.value =
            content.value.substring(0, start) + before + selectedText + after + content.value.substring(end)
        textarea.focus()
        setTimeout(() => {
            textarea.setSelectionRange(start + before.length, end + before.length)
        }, 0)
    }

    saveDraft()
    autoResize()
}

function openFilePicker() {
    fileUploaderRef.value?.openFilePicker()
}

async function handleFiles(files: File[]) {
    for (const file of files) {
        const attachment = {
            file,
            uploading: true,
            progress: 0,
            uploaded: undefined as FileUploadResponse | undefined,
        }
        attachedFiles.value.push(attachment)

        try {
            const response = await filesApi.upload(file, channelStore.currentChannelId || undefined, (progressEvent) => {
                if (progressEvent.total) {
                    attachment.progress = Math.round((progressEvent.loaded * 100) / progressEvent.total)
                }
            })

            attachment.uploaded = response.data
            toast.success('File uploaded', file.name)
        } catch (error: any) {
            toast.error('Upload failed', error.message || 'Unknown error')
            attachedFiles.value = attachedFiles.value.filter((fileItem) => fileItem !== attachment)
        } finally {
            attachment.uploading = false
        }
    }
}

function removeAttachment(index: number) {
    attachedFiles.value.splice(index, 1)
}

function insertEmoji(emoji: string) {
    const textarea = textareaRef.value
    if (!textarea) {
        content.value += emoji
        saveDraft()
        showEmojiPicker.value = false
        return
    }

    const start = textarea.selectionStart
    const end = textarea.selectionEnd
    content.value = `${content.value.substring(0, start)}${emoji}${content.value.substring(end)}`
    saveDraft()
    showEmojiPicker.value = false

    nextTick(() => {
        textarea.focus()
        const newCursorPos = start + emoji.length
        textarea.setSelectionRange(newCursorPos, newCursorPos)
        autoResize()
    })
}

function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

watch(
    () => channelStore.currentChannelId,
    (channelId) => {
        resetForChannelChange(channelId || undefined)
    },
    { immediate: true }
)

watch(
    () => formattingAllowed.value,
    (allowed) => {
        if (!allowed) {
            showFormatting.value = false
            showPreview.value = false
        }
    },
    { immediate: true }
)

onMounted(() => {
    isMac.value = navigator.platform.toUpperCase().includes('MAC')
    window.addEventListener('keydown', handleGlobalKeydown)

    if (!preferencesStore.preferences) {
        void preferencesStore.fetchPreferences()
    }

    if (textareaRef.value) {
        autoResize()
    }
})

onUnmounted(() => {
    window.removeEventListener('keydown', handleGlobalKeydown)
})
</script>

<template>
  <FileUploader
    ref="fileUploaderRef"
    class="p-2 pt-1 shrink-0 z-20"
    @files-selected="handleFiles"
  >
    <div class="relative overflow-hidden rounded-r-2 border border-border-1 bg-bg-surface-1 shadow-1 transition-standard focus-within:border-brand/60 focus-within:ring-2 focus-within:ring-brand/15">
      <FormattingToolbar
        v-if="showToolbar"
        :showPreview="showPreview"
        @format="applyFormat"
        @togglePreview="showPreview = !showPreview"
      />

      <div v-if="showPreview" class="border-b border-border-1 px-3 py-2">
        <MarkdownPreview :content="content" />
      </div>

      <div v-if="attachedFiles.length > 0" class="flex flex-wrap gap-2 border-b border-border-1 bg-bg-surface-2/45 px-2.5 py-2">
        <div
          v-for="(attachment, index) in attachedFiles"
          :key="index"
          class="relative flex min-w-[220px] items-center gap-2 rounded-r-1 border border-border-1 bg-bg-surface-1 px-2 py-2"
        >
          <FileIcon class="h-4 w-4 shrink-0 text-text-3" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-xs font-medium text-text-1">{{ attachment.file.name }}</p>
            <p class="text-[11px] text-text-3">{{ formatFileSize(attachment.file.size) }}</p>
          </div>
          <button
            class="rounded p-1 text-text-3 transition-standard hover:bg-danger/10 hover:text-danger focus-ring"
            aria-label="Remove attachment"
            @click="removeAttachment(index)"
          >
            <X class="h-3.5 w-3.5" />
          </button>

          <div v-if="attachment.uploading" class="absolute inset-x-0 bottom-0 h-0.5 bg-border-1">
            <div class="h-full bg-brand transition-standard" :style="{ width: `${attachment.progress}%` }"></div>
          </div>
        </div>
      </div>

      <div class="relative">
        <textarea
          ref="textareaRef"
          v-model="content"
          rows="1"
          class="max-h-80 min-h-[50px] w-full resize-none border-0 bg-transparent px-3 py-3 text-sm leading-6 text-text-1 placeholder:text-text-3 focus:ring-0"
          :placeholder="placeholderText"
          aria-label="Message composer"
          @keydown="handleKeydown"
          @input="handleInput"
        ></textarea>

        <MentionAutocomplete
          :show="showMentionMenu"
          :query="mentionQuery"
          @select="handleMentionSelect"
          @close="showMentionMenu = false"
        />
      </div>

      <div class="flex items-center justify-between border-t border-border-1/60 px-2 py-1.5">
        <div class="flex items-center gap-1 text-text-3">
          <button
            class="rounded p-1.5 transition-standard hover:bg-brand/10 hover:text-brand focus-ring"
            title="Attach file"
            aria-label="Attach file"
            @click="openFilePicker"
          >
            <Paperclip class="h-4 w-4" />
          </button>

          <div class="relative">
            <button
              class="rounded p-1.5 transition-standard hover:bg-amber-500/10 hover:text-amber-500 focus-ring"
              title="Insert emoji"
              aria-label="Insert emoji"
              @click="showEmojiPicker = !showEmojiPicker"
            >
              <Smile class="h-4 w-4" />
            </button>
            <EmojiPicker
              :show="showEmojiPicker"
              @select="insertEmoji"
              @close="showEmojiPicker = false"
            />
          </div>

          <button
            class="rounded p-1.5 transition-standard focus-ring"
            :class="showToolbar ? 'bg-brand/10 text-brand' : 'hover:bg-brand/10 hover:text-brand'"
            :disabled="!formattingAllowed"
            :title="showToolbar ? 'Hide formatting toolbar (Ctrl+Alt+T)' : 'Show formatting toolbar (Ctrl+Alt+T)'"
            aria-label="Toggle formatting toolbar"
            @click="toggleFormatting"
          >
            <Type class="h-4 w-4" />
          </button>

          <button
            v-if="!callsStore.isInCall"
            class="rounded p-1.5 transition-standard hover:bg-blue-500/10 hover:text-blue-500 focus-ring"
            title="Start audio call"
            aria-label="Start audio call"
            @click="$emit('startAudioCall')"
          >
            <Phone class="h-4 w-4" />
          </button>
          <button
            v-else
            class="rounded bg-green-500/20 p-1.5 text-green-500 transition-standard focus-ring"
            title="Show active call"
            aria-label="Show active call"
            @click="callsStore.toggleExpanded()"
          >
            <Phone class="h-4 w-4" />
          </button>
        </div>

        <div class="flex items-center gap-3">
          <div class="hidden items-center gap-2 text-[11px] text-text-3 md:flex">
            <span><kbd class="rounded bg-bg-surface-2 px-1 py-0.5">{{ sendShortcutLabel }}</kbd> to send</span>
            <span><kbd class="rounded bg-bg-surface-2 px-1 py-0.5">Shift+Enter</kbd> newline</span>
          </div>

          <button
            class="flex items-center gap-1 rounded-r-1 bg-brand px-2.5 py-1.5 text-white shadow-1 transition-standard hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="!canSend"
            aria-label="Send message"
            @click="handleSend"
          >
            <Send class="h-4 w-4" />
            <span class="hidden text-sm font-medium sm:inline">Send</span>
          </button>
        </div>
      </div>

      <div v-if="uploadInProgressCount > 0" class="border-t border-border-1/50 px-3 py-1 text-[11px] text-text-3">
        Uploading {{ uploadInProgressCount }} file{{ uploadInProgressCount > 1 ? 's' : '' }}...
      </div>
    </div>
  </FileUploader>
</template>
