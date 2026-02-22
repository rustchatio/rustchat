<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Smile, Paperclip, Send, X, File as FileIcon, Video, Phone, Type } from 'lucide-vue-next';
import { useToast } from '../../composables/useToast';
import { filesApi, type FileUploadResponse } from '../../api/files';
import FileUploader from '../atomic/FileUploader.vue';
import EmojiPicker from '../atomic/EmojiPicker.vue';
import FormattingToolbar from './FormattingToolbar.vue';
import MarkdownPreview from './MarkdownPreview.vue';
import MentionAutocomplete from './MentionAutocomplete.vue';
import { useTeamStore } from '../../stores/teams';
import { useConfigStore } from '../../stores/config';
import { useCallsStore } from '../../stores/calls';
import { useChannelStore } from '../../stores/channels';

const emit = defineEmits(['send', 'typing', 'startCall', 'startAudioCall'])
const content = ref('')
const showEmojiPicker = ref(false)
const showPreview = ref(false)
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const toast = useToast()
const teamStore = useTeamStore()
const configStore = useConfigStore()
const callsStore = useCallsStore()
const channelStore = useChannelStore()

const showMentionMenu = ref(false)
const showFormatting = ref(true) // Formatting toolbar visible by default
const attachedFiles = ref<{ 
    file: File; 
    uploading: boolean; 
    progress: number;
    uploaded?: FileUploadResponse 
}[]>([])
const mentionQuery = ref('')

let lastTypingEmit = 0
const TYPING_ACTIVITY_INTERVAL_MS = 2000

// Toggle formatting toolbar
function toggleFormatting() {
  showFormatting.value = !showFormatting.value
}

// Keyboard shortcut handler for formatting toggle
function handleGlobalKeydown(event: KeyboardEvent) {
  // Ctrl+Alt+T or Cmd+Opt+T
  if ((event.ctrlKey || event.metaKey) && event.altKey && event.key === 't') {
    event.preventDefault()
    toggleFormatting()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
})

// Slash command handling
async function handleSlashCommand(command: string, args: string[]): Promise<boolean> {
    const channelId = channelStore.currentChannelId
    if (!channelId) return false

    switch (command) {
        case '/call':
        case '/call start': {
            const subCommand = args[0]
            
            if (subCommand === 'start' || !subCommand) {
                // Check if call already exists
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

function handleSend() {
  if (!content.value.trim() && attachedFiles.value.length === 0) return
  
  // Check for slash commands
  const text = content.value.trim()
  if (text.startsWith('/')) {
    const parts = text.split(' ')
    const command = parts[0]
    const args = parts.slice(1)
    
    // Handle /call start, /call join, etc.
    if (command === '/call' && args.length > 0) {
      const fullCommand = `${command} ${args[0]}`
      const remainingArgs = args.slice(1)
      handleSlashCommand(fullCommand, remainingArgs).then(handled => {
        if (handled) {
          content.value = ''
          attachedFiles.value = []
          showPreview.value = false
        }
      })
      return
    }
    
    if (command) {
      handleSlashCommand(command, args).then(handled => {
        if (handled) {
          content.value = ''
          attachedFiles.value = []
          showPreview.value = false
        }
      })
    }
    return
  }
  
  // Only send files that have finished uploading
  const fileIds = attachedFiles.value
    .filter(a => !a.uploading && a.uploaded)
    .map(a => a.uploaded!.id)

  const messageContent = content.value
  emit('send', { 
    content: messageContent, 
    file_ids: fileIds 
  })
  
  content.value = ''
  attachedFiles.value = []
  showPreview.value = false
}

function handleKeydown(e: KeyboardEvent) {
    // Ctrl/Cmd + B = Bold
    if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault()
        applyFormat('bold')
        return
    }
    // Ctrl/Cmd + I = Italic
    if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
        e.preventDefault()
        applyFormat('italic')
        return
    }
    // Send on Enter
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        handleSend()
    }
}

function handleInput() {
    const now = Date.now()
    if (content.value.trim().length > 0 && now - lastTypingEmit > TYPING_ACTIVITY_INTERVAL_MS) {
        lastTypingEmit = now
        emit('typing')
    }
    
    // Check for @ mention trigger
    const textarea = textareaRef.value
    if (textarea) {
        const cursorPos = textarea.selectionStart
        const textBefore = content.value.substring(0, cursorPos)
        const lastAt = textBefore.lastIndexOf('@')
        
        if (lastAt !== -1 && (lastAt === 0 || /\s/.test(textBefore.charAt(lastAt - 1)))) {
            const query = textBefore.substring(lastAt + 1)
            if (!/\s/.test(query)) {
                mentionQuery.value = query
                showMentionMenu.value = true
                
                // Fetch members if not loaded
                if (teamStore.members.length === 0 && teamStore.currentTeamId) {
                    teamStore.fetchMembers(teamStore.currentTeamId as string)
                }
                return
            }
        }
        showMentionMenu.value = false
    }
}

function handleMentionSelect(username: string) {
    const textarea = textareaRef.value
    if (textarea) {
        const cursorPos = textarea.selectionStart
        const textBefore = content.value.substring(0, cursorPos)
        const lastAt = textBefore.lastIndexOf('@')
        
        const newContent = content.value.substring(0, lastAt) + '@' + username + ' ' + content.value.substring(cursorPos)
        content.value = newContent
        
        // Refocus and move cursor
        textarea.focus()
        const newPos = lastAt + username.length + 2 // +2 for @ and space
        setTimeout(() => {
            textarea.setSelectionRange(newPos, newPos)
        }, 0)
    }
    showMentionMenu.value = false
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
            before = '**'; after = '**'; break
        case 'italic':
            before = '*'; after = '*'; break
        case 'code':
            before = '`'; after = '`'; break
        case 'link':
            before = '['; after = '](url)'; break
        case 'quote':
            prefix = '> '; break
        case 'bullet':
            prefix = '- '; break
        case 'numbered':
            prefix = '1. '; break
    }
    
    if (prefix) {
        // Line-based formatting
        const lineStart = content.value.lastIndexOf('\n', start - 1) + 1
        content.value = content.value.substring(0, lineStart) + prefix + content.value.substring(lineStart)
    } else {
        // Wrap selection
        content.value = content.value.substring(0, start) + before + selectedText + after + content.value.substring(end)
        // Move cursor
        textarea.focus()
        setTimeout(() => {
            textarea.setSelectionRange(start + before.length, end + before.length)
        }, 0)
    }
}

const fileUploaderRef = ref<InstanceType<typeof FileUploader> | null>(null)

function openFilePicker() {
    fileUploaderRef.value?.openFilePicker()
}

async function handleFiles(files: File[]) {
    for (const file of files) {
        const attachment = { 
            file, 
            uploading: true, 
            progress: 0,
            uploaded: undefined as FileUploadResponse | undefined 
        }
        attachedFiles.value.push(attachment)
        
        try {
            const response = await filesApi.upload(file, undefined, (progressEvent) => {
                if (progressEvent.total) {
                    attachment.progress = Math.round((progressEvent.loaded * 100) / progressEvent.total)
                }
            })
            attachment.uploaded = response.data
            toast.success('File uploaded', file.name)
        } catch (e: any) {
            toast.error('Upload failed', e.message || 'Unknown error')
            attachedFiles.value = attachedFiles.value.filter(a => a !== attachment)
        } finally {
            attachment.uploading = false
        }
    }
}

function removeAttachment(index: number) {
    attachedFiles.value.splice(index, 1)
}

function insertEmoji(emoji: string) {
    content.value += emoji
    showEmojiPicker.value = false
}

function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}
</script>

<template>
  <FileUploader 
    ref="fileUploaderRef"
    class="p-2 pt-1 shrink-0 z-20"
    @files-selected="handleFiles"
  >

    <div class="relative rounded-2xl bg-bg-surface-1 border border-border-1 shadow-lg ring-1 ring-black/5 overflow-hidden transition-slow focus-within:ring-2 focus-within:ring-primary/5 focus-within:border-primary/50">
        <!-- Formatting Toolbar -->
        <FormattingToolbar 
          v-if="showFormatting"
          :showPreview="showPreview"
          @format="applyFormat"
          @togglePreview="showPreview = !showPreview"
          class="border-b border-border-1 bg-bg-surface-2/50"
        />
        
        <!-- Markdown Preview -->
        <div v-if="showPreview" class="p-3">
          <MarkdownPreview :content="content" />
        </div>
        
        <!-- Attached Files Preview -->
        <div v-if="attachedFiles.length > 0" class="flex flex-wrap gap-2 p-2 border-b border-border-1 bg-surface/50">
          <div 
            v-for="(attachment, index) in attachedFiles"
            :key="index"
            class="relative flex flex-col space-y-1.5 bg-bg-surface-2 border border-border-1 rounded-xl p-2.5 min-w-[180px] overflow-hidden group/file shadow-sm"
          >
            <div class="flex items-center space-x-3">
                <div class="p-2 bg-primary/10 rounded-lg">
                    <FileIcon class="w-5 h-5 text-primary" />
                </div>
                <div class="flex-1 min-w-0">
                    <div class="text-[13px] font-semibold text-text-1 truncate pr-4">
                        {{ attachment.file.name }}
                    </div>
                    <div class="text-[11px] text-text-3">
                        {{ formatFileSize(attachment.file.size) }}
                    </div>
                </div>
                <button 
                  @click="removeAttachment(index)"
                  class="absolute top-2 right-2 p-1.5 hover:bg-danger/10 hover:text-danger rounded-lg text-text-3 transition-colors opacity-0 group-hover/file:opacity-100"
                >
                  <X class="w-4 h-4" />
                </button>
            </div>

            <!-- Upload Progress -->
            <div v-if="attachment.uploading" class="space-y-1.5">
                <div class="flex justify-between text-[10px] uppercase tracking-wider font-bold">
                    <span class="text-primary">Uploading...</span>
                    <span class="text-text-3">{{ attachment.progress }}%</span>
                </div>
                <div class="h-1.5 w-full bg-border-1 rounded-full overflow-hidden">
                    <div 
                        class="h-full bg-primary transition-all duration-300 ease-out"
                        :style="{ width: `${attachment.progress}%` }"
                    ></div>
                </div>
            </div>
            
            <div v-else-if="attachment.uploaded" class="text-[10px] text-success flex items-center space-x-1 font-bold uppercase tracking-wider">
                <div class="w-1 h-1 rounded-full bg-success"></div>
                <span>Ready</span>
            </div>
          </div>
        </div>

        <!-- Input -->
        <div class="relative">
          <textarea 
              ref="textareaRef"
              v-model="content"
              @keydown="handleKeydown"
              @input="handleInput"
              rows="1" 
              class="w-full bg-transparent border-0 focus:ring-0 resize-none max-h-36 py-2.5 px-3 min-h-[42px] text-text-1 placeholder-text-3 placeholder:font-light" 
              placeholder="Message... (Use @ to mention)"
              aria-label="Message composer"
          ></textarea>
          
          <!-- Mention Menu -->
          <MentionAutocomplete 
            :show="showMentionMenu"
            :query="mentionQuery"
            @select="handleMentionSelect"
            @close="showMentionMenu = false"
          />
        </div>

        <!-- Footer Actions -->
        <div class="flex justify-between items-center px-1.5 pb-1.5">
            <div class="flex space-x-1 text-text-3 relative">
                <button 
                  @click="openFilePicker"
                  class="p-2 hover:bg-primary/10 hover:text-primary rounded-lg transition-colors" 
                  title="Attach file"
                >
                    <Paperclip class="w-4.5 h-4.5" />
                </button>
                <div class="relative">
                  <button 
                    @click="showEmojiPicker = !showEmojiPicker"
                    class="p-2 hover:bg-amber-500/10 hover:text-amber-500 rounded-lg transition-colors" 
                    title="Emoji"
                  >
                      <Smile class="w-4.5 h-4.5" />
                  </button>
                  <EmojiPicker 
                    :show="showEmojiPicker" 
                    @select="insertEmoji" 
                    @close="showEmojiPicker = false" 
                  />
                </div>

                <!-- Formatting Toggle -->
                <button
                  @click="toggleFormatting"
                  class="p-2 hover:bg-primary/10 hover:text-primary rounded-lg transition-colors"
                  :title="showFormatting ? 'Hide formatting (Ctrl+Alt+T)' : 'Show formatting (Ctrl+Alt+T)'"
                  :class="{ 'text-primary bg-primary/10': showFormatting }"
                >
                    <Type class="w-4.5 h-4.5" />
                </button>

                <button
                  v-if="configStore.siteConfig.mirotalk_enabled"
                  @click="$emit('startCall')"
                  class="p-2 hover:bg-green-500/10 hover:text-green-500 rounded-lg transition-colors"
                  title="Start video call"
                >
                    <Video class="w-4.5 h-4.5" />
                </button>
                
                <!-- Native Audio Call Button -->
                <button
                  v-if="!callsStore.isInCall"
                  @click="$emit('startAudioCall')"
                  class="p-2 hover:bg-blue-500/10 hover:text-blue-500 rounded-lg transition-colors"
                  title="Start audio call"
                >
                    <Phone class="w-4.5 h-4.5" />
                </button>
                <button
                  v-else
                  @click="callsStore.toggleExpanded()"
                  class="p-2 bg-green-500/20 text-green-500 rounded-lg transition-colors animate-pulse"
                  title="Show active call"
                >
                    <Phone class="w-4.5 h-4.5" />
                </button>
            </div>
            
            <div class="flex items-center space-x-4">
                 <div class="text-[10px] text-text-3 hidden sm:block font-medium tracking-wide">
                    <span class="bg-surface/50 px-1 py-0.5 rounded text-xs mr-1">⌘B</span> 
                    <span class="bg-surface/50 px-1 py-0.5 rounded text-xs mr-1">⌘I</span>
                    <span class="bg-surface/50 px-1 py-0.5 rounded text-xs">⌘⌥T</span>
                </div>
                <button 
                    @click="handleSend"
                    :disabled="!content.trim() && attachedFiles.length === 0"
                    class="p-2 bg-primary hover:bg-primary-hover text-white rounded-lg disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-lg shadow-primary/20 active:scale-95"
                >
                    <Send class="w-4 h-4" />
                </button>
            </div>
        </div>
    </div>
  </FileUploader>
</template>
