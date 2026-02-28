<script setup lang="ts">
import { Bold, Italic, Strikethrough, Heading, Code, Link2, List, ListOrdered, Quote, Eye, EyeOff } from 'lucide-vue-next'

const emit = defineEmits<{
  (e: 'format', type: string): void
  (e: 'togglePreview'): void
}>()

defineProps<{
  showPreview: boolean
}>()

const formatActions = [
  { icon: Bold, type: 'bold', title: 'Bold (Ctrl+B)', label: 'Bold' },
  { icon: Italic, type: 'italic', title: 'Italic (Ctrl+I)', label: 'Italic' },
  { icon: Strikethrough, type: 'strike', title: 'Strikethrough (Ctrl+Shift+X)', label: 'Strikethrough' },
  { icon: Heading, type: 'heading', title: 'Heading', label: 'Heading' },
  { icon: Code, type: 'code', title: 'Inline code', label: 'Inline code' },
  { icon: Code, type: 'codeblock', title: 'Code block', label: 'Code block' },
  { icon: Link2, type: 'link', title: 'Link', label: 'Link' },
  { icon: Quote, type: 'quote', title: 'Quote', label: 'Quote' },
  { icon: List, type: 'bullet', title: 'Bullet list', label: 'Bullet list' },
  { icon: ListOrdered, type: 'numbered', title: 'Numbered list', label: 'Numbered list' },
]
</script>

<template>
  <div class="flex items-center gap-0.5 overflow-x-auto border-b border-border-1 bg-bg-surface-2/50 px-1.5 py-1 whitespace-nowrap">
    <!-- Formatting buttons -->
    <button
      v-for="action in formatActions"
      :key="action.type"
      @click="$emit('format', action.type)"
      :title="action.title"
      :aria-label="action.label"
      class="rounded p-1.5 text-text-3 transition-standard hover:bg-bg-surface-1 hover:text-text-1 focus-ring"
    >
      <component :is="action.icon" class="w-4 h-4" />
    </button>
    
    <!-- Divider -->
    <div class="mx-1 h-5 w-px bg-border-1"></div>
    
    <!-- Preview toggle -->
    <button
      @click="$emit('togglePreview')"
      :title="showPreview ? 'Hide preview' : 'Show preview'"
      aria-label="Toggle markdown preview"
      class="rounded p-1.5 transition-standard focus-ring"
      :class="showPreview 
        ? 'bg-brand/10 text-brand'
        : 'text-text-3 hover:bg-bg-surface-1 hover:text-text-1'"
    >
      <component :is="showPreview ? EyeOff : Eye" class="w-4 h-4" />
    </button>
  </div>
</template>
