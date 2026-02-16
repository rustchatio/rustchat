<script setup lang="ts">
import { ref, watch } from 'vue'
import { Search, ExternalLink } from 'lucide-vue-next'
import { format } from 'date-fns'
import type { Message } from '../../stores/messages'
import { useMessageStore } from '../../stores/messages'


const props = defineProps<{
    channelId: string
}>()

const emit = defineEmits<{
    (e: 'close'): void
    (e: 'jump', messageId: string): void
}>()

const messageStore = useMessageStore()

const searchQuery = ref('')
const searchResults = ref<Message[]>([])
const loading = ref(false)

async function handleSearch() {
    if (!searchQuery.value.trim() || !props.channelId) {
        searchResults.value = []
        return
    }
    
    loading.value = true
    try {
        searchResults.value = await messageStore.searchMessages(
            props.channelId, 
            searchQuery.value
        )
    } catch (e) {
        console.error('Search failed', e)
    } finally {
        loading.value = false
    }
}

// Debounce search
let timeout: any
watch(searchQuery, () => {
    clearTimeout(timeout)
    timeout = setTimeout(() => {
        handleSearch()
    }, 300)
})

function jumpToMessage(message: Message) {
    emit('jump', message.id)
}
</script>

<template>
  <div 
    class="flex-1 flex flex-col min-h-0 bg-surface dark:bg-surface-dim"
  >
    <!-- Search Input Area -->
    <div class="p-4 border-b border-border-dim dark:border-white/5 bg-surface-dim/30">
        <div class="relative group">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 group-focus-within:text-primary transition-colors" />
            <input 
                v-model="searchQuery"
                type="text"
                placeholder="Search messages..."
                class="w-full pl-10 pr-4 py-2 bg-surface dark:bg-surface-dim border border-border-dim dark:border-white/5 rounded-xl text-sm focus-ring shadow-sm"
                autofocus
                aria-label="Search messages in this channel"
            />
        </div>
    </div>

    <!-- Results List -->
    <div class="flex-1 overflow-y-auto custom-scrollbar">
      <div v-if="loading" class="text-center py-12 text-gray-500">
        <div class="animate-spin w-6 h-6 border-2 border-primary border-t-transparent rounded-full mx-auto mb-3"></div>
        <p class="text-xs font-medium uppercase tracking-widest">Searching...</p>
      </div>
      
      <div v-else-if="searchQuery && searchResults.length === 0" class="text-center py-12 text-gray-500 px-6">
        <div class="w-16 h-16 bg-surface-dim dark:bg-slate-800/50 rounded-full flex items-center justify-center mx-auto mb-4 border border-border-dim dark:border-white/5">
             <Search class="w-8 h-8 text-gray-300" />
        </div>
        <p class="text-[15px] font-semibold text-gray-700 dark:text-gray-200">No results found</p>
        <p class="text-xs text-gray-500 mt-1">We couldn't find anything matching "{{ searchQuery }}"</p>
      </div>

      <div v-else-if="!searchQuery" class="text-center py-16 text-gray-400 px-6">
        <Search class="w-16 h-16 mx-auto mb-4 opacity-10" />
        <p class="text-sm font-medium">Search for messages and files in this channel</p>
      </div>

      <div v-else class="divide-y divide-border-dim dark:divide-white/5">
        <div 
            v-for="message in searchResults" 
            :key="message.id"
            class="px-5 py-5 hover:bg-surface-dim dark:hover:bg-gray-800/40 transition-all group relative cursor-pointer border-l-2 border-transparent hover:border-primary/50"
            @click="jumpToMessage(message)"
        >
            <div class="flex items-start justify-between mb-1.5">
                <div class="flex items-center space-x-2">
                    <span class="font-bold text-sm text-gray-900 dark:text-gray-100 hover:text-primary transition-colors">{{ message.username }}</span>
                    <span class="text-[10px] text-gray-500 font-medium tracking-tight">{{ format(new Date(message.timestamp), 'MMM d, h:mm a') }}</span>
                </div>
                <div class="opacity-0 group-hover:opacity-100 transition-all transform translate-x-1 group-hover:translate-x-0">
                    <button class="text-gray-400 hover:text-primary p-1 rounded-md hover:bg-primary/5 transition-all" title="Jump to message">
                        <ExternalLink class="w-4 h-4" />
                    </button>
                </div>
            </div>
            <div class="text-[14px] text-gray-700 dark:text-gray-300 line-clamp-3 mt-1 leading-relaxed">
                {{ message.content }}
            </div>
        </div>
      </div>
    </div>
  </div>

</template>
