<script setup lang="ts">
import { X } from 'lucide-vue-next'
import BaseButton from '../atomic/BaseButton.vue'

defineProps<{
  label: string
  description?: string
  loading?: boolean
  saveDisabled?: boolean
}>()

defineEmits<{
  save: []
  cancel: []
}>()
</script>

<template>
  <div class="bg-gray-50 dark:bg-gray-800/50 border border-gray-200 dark:border-gray-700 rounded-lg">
    <!-- Header -->
    <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
      <div>
        <h4 class="text-sm font-semibold text-gray-900 dark:text-white">{{ label }}</h4>
        <p v-if="description" class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{{ description }}</p>
      </div>
      <button
        type="button"
        @click="$emit('cancel')"
        class="p-1 rounded-md text-gray-400 hover:text-gray-500 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
    
    <!-- Content slot -->
    <div class="p-4">
      <slot />
    </div>
    
    <!-- Actions -->
    <div class="px-4 py-3 border-t border-gray-200 dark:border-gray-700 flex items-center justify-end gap-2">
      <BaseButton 
        size="sm" 
        variant="secondary" 
        @click="$emit('cancel')"
        :disabled="loading"
      >
        Cancel
      </BaseButton>
      <BaseButton 
        size="sm" 
        @click="$emit('save')"
        :loading="loading"
        :disabled="saveDisabled"
      >
        Save
      </BaseButton>
    </div>
  </div>
</template>
