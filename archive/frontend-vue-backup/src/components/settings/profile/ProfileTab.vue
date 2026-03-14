<script setup lang="ts">
import { ref, watch } from 'vue'
import { Camera } from 'lucide-vue-next'
import BaseButton from '../../atomic/BaseButton.vue'
import BaseInput from '../../atomic/BaseInput.vue'
import { useAuthStore } from '../../../stores/auth'
import { usersApi } from '../../../api/users'
import { filesApi } from '../../../api/files'

const auth = useAuthStore()
const loading = ref(false)
const error = ref('')
const success = ref('')
const fileInput = ref<HTMLInputElement | null>(null)

// Profile form fields
const username = ref('')
const displayName = ref('')
const avatarUrl = ref('')
const firstName = ref('')
const lastName = ref('')
const nickname = ref('')
const position = ref('')

// Status fields
const statusText = ref('')
const statusEmoji = ref('')

// Initialize from auth user
watch(() => auth.user, (user) => {
  if (user) {
    username.value = user.username || ''
    displayName.value = user.display_name || ''
    avatarUrl.value = user.avatar_url || ''
    firstName.value = user.first_name || ''
    lastName.value = user.last_name || ''
    nickname.value = user.nickname || ''
    position.value = user.position || ''
    statusText.value = user.status_text || ''
    statusEmoji.value = user.status_emoji || ''
  }
}, { immediate: true })

async function handleFileUpload(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files && input.files[0]) {
    const file = input.files[0]
    
    if (!file.type.startsWith('image/')) {
      error.value = 'Please select a valid image file'
      return
    }

    if (file.size > 5 * 1024 * 1024) {
      error.value = 'Image size must be less than 5MB'
      return
    }

    loading.value = true
    error.value = ''
    
    try {
      const response = await filesApi.upload(file)
      avatarUrl.value = response.data.url
      success.value = 'Avatar uploaded successfully! Click Save to apply.'
    } catch (e: any) {
      error.value = e.response?.data?.message || 'Failed to upload avatar'
    } finally {
      loading.value = false
      input.value = ''
    }
  }
}

async function handleSaveProfile() {
  if (!auth.user) return
  
  loading.value = true
  error.value = ''
  success.value = ''

  try {
    const response = await usersApi.update(auth.user.id, {
      username: username.value,
      display_name: displayName.value,
      avatar_url: avatarUrl.value,
      first_name: firstName.value,
      last_name: lastName.value,
      nickname: nickname.value,
      position: position.value,
    })
    
    // Update local auth state
    auth.user = {
      ...auth.user,
      username: response.data.username,
      display_name: response.data.display_name,
      avatar_url: response.data.avatar_url,
      first_name: firstName.value,
      last_name: lastName.value,
      nickname: nickname.value,
      position: position.value,
    }
    success.value = 'Profile updated successfully!'
    setTimeout(() => success.value = '', 3000)
  } catch (e: any) {
    error.value = e.response?.data?.message || 'Failed to update profile'
  } finally {
    loading.value = false
  }
}

async function handleSaveStatus() {
  loading.value = true
  error.value = ''
  success.value = ''
  
  try {
    await auth.updateStatus({
      text: statusText.value,
      emoji: statusEmoji.value
    })
    success.value = 'Status updated successfully!'
    setTimeout(() => success.value = '', 3000)
  } catch (e: any) {
    error.value = 'Failed to update status'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- Messages -->
    <div v-if="error" class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-600 dark:text-red-400 text-sm">
      {{ error }}
    </div>
    <div v-if="success" class="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg text-green-600 dark:text-green-400 text-sm">
      {{ success }}
    </div>

    <!-- Profile Section -->
    <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-4 space-y-4">
      <h4 class="text-sm font-semibold text-gray-900 dark:text-white border-b border-gray-200 dark:border-gray-700 pb-2">
        Profile Information
      </h4>
      
      <!-- Avatar -->
      <div class="flex items-center space-x-4">
        <div class="relative group">
          <div class="h-16 w-16 sm:h-20 sm:w-20 rounded-full bg-primary flex items-center justify-center text-xl sm:text-2xl text-white font-bold overflow-hidden ring-2 ring-transparent group-hover:ring-primary/50 transition-all">
            <img v-if="avatarUrl" :src="avatarUrl" alt="Avatar" class="w-full h-full object-cover" />
            <span v-else>{{ auth.user?.username?.charAt(0).toUpperCase() || 'U' }}</span>
          </div>
          <button 
            type="button"
            @click="fileInput?.click()"
            class="absolute bottom-0 right-0 w-6 h-6 sm:w-7 sm:h-7 bg-gray-800 dark:bg-gray-600 rounded-full flex items-center justify-center border-2 border-white dark:border-gray-800 hover:bg-gray-700 dark:hover:bg-gray-500 transition-colors"
          >
            <Camera class="w-3 h-3 sm:w-3.5 sm:h-3.5 text-white" />
          </button>
          <input ref="fileInput" type="file" accept="image/*" class="hidden" @change="handleFileUpload" />
        </div>
        <div>
          <p class="text-sm font-medium text-gray-900 dark:text-white">{{ auth.user?.username }}</p>
          <p class="text-xs text-gray-500">
            <button type="button" @click="fileInput?.click()" class="text-primary hover:underline">Click to upload</button>
          </p>
        </div>
      </div>

      <!-- Form Fields -->
      <div class="grid grid-cols-1 gap-4">
        <BaseInput label="Username" v-model="username" placeholder="your_username" :disabled="loading" />
        <BaseInput label="Display Name" v-model="displayName" placeholder="Your Name" :disabled="loading" />
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <BaseInput label="First Name" v-model="firstName" placeholder="John" :disabled="loading" />
          <BaseInput label="Last Name" v-model="lastName" placeholder="Doe" :disabled="loading" />
        </div>
        <BaseInput label="Nickname" v-model="nickname" placeholder="Johnny" :disabled="loading" />
        <BaseInput label="Position" v-model="position" placeholder="Software Engineer" :disabled="loading" />
        <BaseInput label="Avatar URL" v-model="avatarUrl" placeholder="https://example.com/avatar.jpg" :disabled="loading" />
        <div class="space-y-1">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">Email</label>
          <div class="px-3 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg text-gray-600 dark:text-gray-400 text-sm break-all">
            {{ auth.user?.email }}
          </div>
        </div>
      </div>
      
      <div class="flex justify-end">
        <BaseButton @click="handleSaveProfile" :loading="loading">Save Profile</BaseButton>
      </div>
    </div>

    <!-- Status Section -->
    <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-4 space-y-4">
      <h4 class="text-sm font-semibold text-gray-900 dark:text-white border-b border-gray-200 dark:border-gray-700 pb-2">
        Custom Status
      </h4>
      
      <div class="space-y-4">
        <div class="flex space-x-2">
          <div class="w-12">
            <BaseInput v-model="statusEmoji" placeholder="😀" class="text-center" :disabled="loading" />
          </div>
          <div class="flex-1">
            <BaseInput v-model="statusText" placeholder="What's your status?" :disabled="loading" />
          </div>
        </div>
        <p class="text-xs text-gray-500">Enter an emoji and a message to describe your status.</p>
      </div>

      <div class="flex justify-end">
        <BaseButton @click="handleSaveStatus" :loading="loading">Update Status</BaseButton>
      </div>
    </div>
  </div>
</template>
