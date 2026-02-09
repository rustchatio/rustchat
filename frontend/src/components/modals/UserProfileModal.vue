<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { X, Mail, MessageCircle, Briefcase } from 'lucide-vue-next';
import RcAvatar from '../ui/RcAvatar.vue';
import BaseButton from '../atomic/BaseButton.vue';
import { usePresenceStore } from '../../stores/presence';
import { useChannelStore } from '../../stores/channels';
import { useRouter } from 'vue-router';
import client from '../../api/client';

interface UserProfile {
  id: string;
  username: string;
  email: string;
  display_name?: string;
  first_name?: string;
  last_name?: string;
  nickname?: string;
  position?: string;
  avatar_url?: string;
  presence?: string;
  status_text?: string;
  status_emoji?: string;
}

const props = defineProps<{
  show: boolean;
  userId: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const router = useRouter();
const presenceStore = usePresenceStore();
const channelStore = useChannelStore();

const loading = ref(false);
const error = ref('');
const user = ref<UserProfile | null>(null);

const fullName = computed(() => {
  if (!user.value) return '';
  const first = user.value.first_name || '';
  const last = user.value.last_name || '';
  if (first || last) return `${first} ${last}`.trim();
  return user.value.display_name || user.value.nickname || user.value.username;
});

const userPresence = computed(() => {
  if (!user.value) return 'offline';
  return presenceStore.getUserPresence(user.value.id).value?.presence || 'offline';
});

const presenceLabel = computed(() => {
  switch (userPresence.value) {
    case 'online': return 'Active';
    case 'away': return 'Away';
    case 'dnd': return 'Do Not Disturb';
    default: return 'Offline';
  }
});

const presenceColor = computed(() => {
  switch (userPresence.value) {
    case 'online': return 'bg-green-500';
    case 'away': return 'bg-amber-500';
    case 'dnd': return 'bg-red-500';
    default: return 'bg-gray-400';
  }
});

async function fetchUser() {
  if (!props.userId) return;
  loading.value = true;
  error.value = '';
  try {
    const { data } = await client.get(`/users/${props.userId}`);
    user.value = data;
  } catch (e: any) {
    error.value = e.response?.data?.message || 'Failed to load user profile';
  } finally {
    loading.value = false;
  }
}

async function startDirectMessage() {
  if (!user.value) return;
  try {
    // Create DM channel via API
    const { data: channel } = await client.post('/channels/direct', {
      user_ids: [user.value.id]
    });
    if (channel) {
      channelStore.selectChannel(channel.id);
      channelStore.addChannel(channel);
      emit('close');
      router.push('/');
    }
  } catch (e) {
    console.error('Failed to start DM', e);
  }
}

watch(() => props.show, (isOpen) => {
  if (isOpen && props.userId) {
    fetchUser();
  }
});

function handleClose() {
  user.value = null;
  error.value = '';
  emit('close');
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center">
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/50" @click="handleClose"></div>
      
      <!-- Modal -->
      <div class="relative bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-sm mx-4 overflow-hidden">
        <!-- Header -->
        <div class="flex items-center justify-end px-4 py-3">
          <button @click="handleClose" class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
            <X class="w-5 h-5 text-gray-500" />
          </button>
        </div>

        <!-- Loading State -->
        <div v-if="loading" class="p-8 flex items-center justify-center">
          <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full"></div>
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="p-8 text-center text-red-500">
          {{ error }}
        </div>

        <!-- Profile Content -->
        <div v-else-if="user" class="pb-6">
          <!-- Avatar & Name Section -->
          <div class="flex flex-col items-center px-6 -mt-4">
            <RcAvatar 
              :userId="user.id"
              :username="user.username"
              :src="user.avatar_url"
              :size="96"
              :showPresence="false"
              class="ring-4 ring-white dark:ring-gray-800 shadow-lg"
            />
            
            <h2 class="mt-4 text-xl font-bold text-gray-900 dark:text-white text-center">
              {{ fullName }}
            </h2>
            
            <p v-if="user.nickname && user.nickname !== fullName" class="text-sm text-gray-500 dark:text-gray-400">
              {{ user.nickname }}
            </p>
            
            <!-- Presence Badge -->
            <div class="mt-2 flex items-center space-x-2">
              <span :class="presenceColor" class="w-2 h-2 rounded-full"></span>
              <span class="text-sm text-gray-600 dark:text-gray-400">{{ presenceLabel }}</span>
            </div>

            <!-- Custom Status -->
            <div v-if="user.status_text" class="mt-2 flex items-center space-x-1 text-sm text-gray-600 dark:text-gray-400">
              <span v-if="user.status_emoji">{{ user.status_emoji }}</span>
              <span>{{ user.status_text }}</span>
            </div>
          </div>

          <!-- Details Section -->
          <div class="mt-6 px-6 space-y-4">
            <!-- Username -->
            <div class="flex items-center space-x-3 text-sm">
              <span class="text-gray-400">@</span>
              <span class="text-gray-700 dark:text-gray-300">{{ user.username }}</span>
            </div>

            <!-- Email -->
            <div class="flex items-center space-x-3 text-sm">
              <Mail class="w-4 h-4 text-gray-400" />
              <span class="text-gray-700 dark:text-gray-300">{{ user.email }}</span>
            </div>

            <!-- Position -->
            <div v-if="user.position" class="flex items-center space-x-3 text-sm">
              <Briefcase class="w-4 h-4 text-gray-400" />
              <span class="text-gray-700 dark:text-gray-300">{{ user.position }}</span>
            </div>
          </div>

          <!-- Message Button -->
          <div class="mt-6 px-6">
            <BaseButton class="w-full" @click="startDirectMessage">
              <MessageCircle class="w-4 h-4 mr-2" />
              Message
            </BaseButton>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
