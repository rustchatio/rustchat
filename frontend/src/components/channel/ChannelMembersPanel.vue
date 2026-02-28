<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { Search, UserPlus, Shield, User } from 'lucide-vue-next';
import RcAvatar from '../ui/RcAvatar.vue';
import api from '../../api/client';

const props = defineProps<{
    channelId: string;
}>();

const emit = defineEmits(['close']);

const members = ref<any[]>([]);
const loading = ref(false);
const searchQuery = ref('');

async function fetchMembers() {
    if (!props.channelId) return;
    loading.value = true;
    try {
        const response = await api.get(`/channels/${props.channelId}/members`);
        members.value = response.data;
    } catch (e) {
        console.error('Failed to fetch channel members:', e);
    } finally {
        loading.value = false;
    }
}

const filteredMembers = computed(() => {
    if (!searchQuery.value) return members.value;
    const q = searchQuery.value.toLowerCase();
    return members.value.filter(m => 
        m.username?.toLowerCase().includes(q) || 
        m.display_name?.toLowerCase().includes(q)
    );
});

const onlineMembers = computed(() => filteredMembers.value.filter(m => m.presence === 'online' || m.presence === 'dnd'));
const offlineMembers = computed(() => filteredMembers.value.filter(m => m.presence !== 'online' && m.presence !== 'dnd'));

onMounted(fetchMembers);
watch(() => props.channelId, fetchMembers);

</script>

<template>
    <div class="h-full bg-surface dark:bg-surface-dim flex flex-col">
        <!-- Toolbar -->
        <div class="p-4 border-b border-border-dim dark:border-white/5 space-y-4 bg-surface-dim/30">
            <div class="relative group">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 group-focus-within:text-primary transition-colors" />
                <input 
                    v-model="searchQuery"
                    type="text" 
                    placeholder="Find members" 
                    class="w-full bg-surface dark:bg-surface-dim border border-border-dim dark:border-white/5 rounded-xl pl-10 pr-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary/50 transition-all shadow-sm"
                />
            </div>
            <button class="w-full flex items-center justify-center space-x-2 py-2 px-3 bg-primary/5 text-primary rounded-xl text-sm font-bold hover:bg-primary/10 transition-all active:scale-[0.98]">
                <UserPlus class="w-4 h-4" />
                <span>Invite People</span>
            </button>
        </div>

        <!-- Members List -->
        <div class="flex-1 overflow-y-auto custom-scrollbar p-3 space-y-6">
            <!-- Online Members -->
            <div v-if="onlineMembers.length > 0">
                <div class="px-3 pb-2 text-[11px] font-bold text-gray-500 uppercase tracking-widest">Online — {{ onlineMembers.length }}</div>
                <div class="space-y-1">
                    <div 
                        v-for="member in onlineMembers" 
                        :key="member.user_id"
                        class="flex items-center space-x-3 p-2 rounded-xl hover:bg-surface-dim dark:hover:bg-gray-800/40 cursor-pointer group transition-all"
                    >
                        <RcAvatar 
                            :userId="member.user_id"
                            :username="member.username"
                            :src="member.avatar_url"
                            size="sm"
                            class="rounded-lg"
                        />
                        <div class="flex-1 min-w-0">
                            <div class="flex items-center justify-between">
                                <span class="text-[14px] font-semibold text-gray-900 dark:text-gray-100 truncate group-hover:text-primary transition-colors">{{ member.display_name || member.username }}</span>
                                <Shield v-if="member.role === 'admin'" class="w-3.5 h-3.5 text-amber-500" title="Admin" />
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Offline Members -->
            <div v-if="offlineMembers.length > 0">
                <div class="px-3 pb-2 text-[11px] font-bold text-gray-500 uppercase tracking-widest">Offline — {{ offlineMembers.length }}</div>
                <div class="space-y-1">
                    <div 
                        v-for="member in offlineMembers" 
                        :key="member.user_id"
                        class="flex items-center space-x-3 p-2 rounded-xl hover:bg-surface-dim dark:hover:bg-gray-800/40 cursor-pointer group transition-all opacity-70 grayscale-[0.5] hover:opacity-100 hover:grayscale-0"
                    >
                        <RcAvatar 
                            :userId="member.user_id"
                            :username="member.username"
                            :src="member.avatar_url"
                            size="sm"
                            :showPresence="false"
                            class="rounded-lg"
                        />
                        <div class="flex-1 min-w-0">
                            <div class="flex items-center justify-between">
                                <span class="text-[14px] font-semibold text-gray-900 dark:text-gray-100 truncate group-hover:text-primary transition-colors">{{ member.display_name || member.username }}</span>
                                <Shield v-if="member.role === 'admin'" class="w-3.5 h-3.5 text-amber-500" title="Admin" />
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Loading Indicator -->
            <div v-if="loading" class="py-10 flex flex-col items-center space-y-3">
                <div class="animate-spin w-6 h-6 border-2 border-primary border-t-transparent rounded-full font-medium uppercase tracking-widest"></div>
                <p class="text-[10px] text-gray-400 font-bold uppercase tracking-widest">Fetching Members</p>
            </div>

            <!-- No results -->
            <div v-if="!loading && filteredMembers.length === 0" class="py-16 text-center space-y-4 px-6">
                <div class="w-16 h-16 bg-surface-dim dark:bg-slate-800/50 rounded-full flex items-center justify-center mx-auto border border-border-dim dark:border-white/5 opacity-50">
                    <User class="w-8 h-8 text-gray-400" />
                </div>
                <div>
                   <p class="text-[15px] font-semibold text-gray-700 dark:text-gray-200">No members found</p>
                   <p class="text-xs text-gray-500 mt-1">Try a different search term or check for typos.</p>
                </div>
            </div>
        </div>
    </div>

</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #E2E8F0;
  border-radius: 4px;
}
.dark .custom-scrollbar::-webkit-scrollbar-thumb {
  background: #334155;
}
</style>
