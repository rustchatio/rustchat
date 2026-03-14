<script lang="ts">
import type { Theme } from '../../../stores/theme'

// Theme options - defined outside script setup
export const THEME_EDITOR_OPTIONS: Array<{
  id: Theme | 'custom'
  label: string
  swatches: { primary: string; accent: string; background: string; sidebarBg?: string }
  colors?: {
    sidebarBg: string
    sidebarText: string
    centerChannelBg: string
    centerChannelColor: string
    linkColor: string
    buttonBg: string
    buttonColor: string
  }
}> = [
  { 
    id: 'light', 
    label: 'Light', 
    swatches: { primary: '#2563eb', accent: '#0ea5e9', background: '#f6f8fb', sidebarBg: '#1e325c' },
    colors: {
      sidebarBg: '#1e325c',
      sidebarText: '#ffffff',
      centerChannelBg: '#f6f8fb',
      centerChannelColor: '#3d3c40',
      linkColor: '#2563eb',
      buttonBg: '#2563eb',
      buttonColor: '#ffffff',
    }
  },
  { 
    id: 'dark', 
    label: 'Dark', 
    swatches: { primary: '#38bdf8', accent: '#22d3ee', background: '#0b1220', sidebarBg: '#1f222a' },
    colors: {
      sidebarBg: '#1f222a',
      sidebarText: '#ffffff',
      centerChannelBg: '#0b1220',
      centerChannelColor: '#dddddd',
      linkColor: '#38bdf8',
      buttonBg: '#38bdf8',
      buttonColor: '#ffffff',
    }
  },
  { 
    id: 'modern', 
    label: 'Modern', 
    swatches: { primary: '#0f766e', accent: '#14b8a6', background: '#f3f7f6', sidebarBg: '#1a1d24' },
    colors: {
      sidebarBg: '#1a1d24',
      sidebarText: '#e2e8f0',
      centerChannelBg: '#f3f7f6',
      centerChannelColor: '#0f172a',
      linkColor: '#0f766e',
      buttonBg: '#0f766e',
      buttonColor: '#ffffff',
    }
  },
  { 
    id: 'metallic', 
    label: 'Metallic', 
    swatches: { primary: '#475569', accent: '#d97706', background: '#e7eaee', sidebarBg: '#334155' },
    colors: {
      sidebarBg: '#334155',
      sidebarText: '#f1f5f9',
      centerChannelBg: '#e7eaee',
      centerChannelColor: '#1e293b',
      linkColor: '#d97706',
      buttonBg: '#475569',
      buttonColor: '#ffffff',
    }
  },
  { 
    id: 'futuristic', 
    label: 'Futuristic', 
    swatches: { primary: '#06b6d4', accent: '#22c55e', background: '#030712', sidebarBg: '#0f172a' },
    colors: {
      sidebarBg: '#0f172a',
      sidebarText: '#22c55e',
      centerChannelBg: '#030712',
      centerChannelColor: '#06b6d4',
      linkColor: '#22c55e',
      buttonBg: '#06b6d4',
      buttonColor: '#000000',
    }
  },
  { 
    id: 'high-contrast', 
    label: 'High Contrast', 
    swatches: { primary: '#00e5ff', accent: '#ffd400', background: '#000000', sidebarBg: '#000000' },
    colors: {
      sidebarBg: '#000000',
      sidebarText: '#ffffff',
      centerChannelBg: '#000000',
      centerChannelColor: '#ffffff',
      linkColor: '#00e5ff',
      buttonBg: '#00e5ff',
      buttonColor: '#000000',
    }
  },
  { 
    id: 'simple', 
    label: 'Simple', 
    swatches: { primary: '#0369a1', accent: '#16a34a', background: '#fafaf9', sidebarBg: '#44403c' },
    colors: {
      sidebarBg: '#44403c',
      sidebarText: '#fafaf9',
      centerChannelBg: '#fafaf9',
      centerChannelColor: '#292524',
      linkColor: '#0369a1',
      buttonBg: '#16a34a',
      buttonColor: '#ffffff',
    }
  },
  { 
    id: 'dynamic', 
    label: 'Dynamic', 
    swatches: { primary: '#e11d48', accent: '#f59e0b', background: '#111827', sidebarBg: '#1f2937' },
    colors: {
      sidebarBg: '#1f2937',
      sidebarText: '#f9fafb',
      centerChannelBg: '#111827',
      centerChannelColor: '#e5e7eb',
      linkColor: '#e11d48',
      buttonBg: '#f59e0b',
      buttonColor: '#000000',
    }
  },
  {
    id: 'custom',
    label: 'Custom',
    swatches: { primary: '#6366f1', accent: '#8b5cf6', background: '#ffffff' },
  }
]
</script>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Check } from 'lucide-vue-next'

// Theme type already imported in module script

const props = defineProps<{
  modelValue: Theme
}>()

const emit = defineEmits<{
  'update:modelValue': [theme: Theme]
  save: [theme: Theme]
  cancel: []
}>()

// Local state for editing
const selectedOption = ref<Theme | 'custom'>(props.modelValue)
const isCustom = computed(() => selectedOption.value === 'custom')

// Custom color state
const customColors = ref({
  sidebarBg: '#1e325c',
  sidebarText: '#ffffff',
  centerChannelBg: '#ffffff',
  centerChannelColor: '#3d3c40',
  linkColor: '#166de0',
  buttonBg: '#166de0',
  buttonColor: '#ffffff',
})

// Initialize from current theme
watch(() => props.modelValue, (newTheme) => {
  const matched = THEME_EDITOR_OPTIONS.find(o => o.id === newTheme)
  if (matched) {
    selectedOption.value = matched.id
    if (matched.colors) {
      customColors.value = { ...matched.colors }
    }
  } else {
    selectedOption.value = 'custom'
  }
}, { immediate: true })

function selectTheme(themeId: Theme | 'custom') {
  selectedOption.value = themeId
  const option = THEME_EDITOR_OPTIONS.find(o => o.id === themeId)
  if (option?.colors) {
    customColors.value = { ...option.colors }
  }
}

function handleSave() {
  const themeToSave = selectedOption.value === 'custom' ? 'light' : selectedOption.value
  emit('save', themeToSave)
}

function handleCancel() {
  // Reset to original
  const matched = THEME_EDITOR_OPTIONS.find(o => o.id === props.modelValue)
  if (matched) {
    selectedOption.value = matched.id
    if (matched.colors) {
      customColors.value = { ...matched.colors }
    }
  }
  emit('cancel')
}

function updateCustomColor(key: keyof typeof customColors.value, value: string) {
  customColors.value[key] = value
  selectedOption.value = 'custom'
}
</script>

<template>
  <div class="space-y-6">
    <!-- Premade Themes -->
    <div>
      <h5 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">
        Premade Themes
      </h5>
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <button
          v-for="theme in THEME_EDITOR_OPTIONS.filter(t => t.id !== 'custom')"
          :key="theme.id"
          type="button"
          @click="selectTheme(theme.id as Theme)"
          class="relative flex flex-col items-center gap-2 p-3 rounded-lg border transition-all"
          :class="selectedOption === theme.id 
            ? 'border-primary bg-primary/5 dark:bg-primary/10 ring-2 ring-primary/30' 
            : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'"
        >
          <!-- Theme Swatch -->
          <div class="relative w-full h-16 rounded-md overflow-hidden border border-gray-200 dark:border-gray-700">
            <!-- Sidebar preview -->
            <div 
              class="absolute left-0 top-0 h-full w-1/3"
              :style="{ backgroundColor: theme.swatches.sidebarBg || theme.swatches.primary }"
            />
            <!-- Center channel preview -->
            <div 
              class="absolute right-0 top-0 h-full w-2/3"
              :style="{ backgroundColor: theme.swatches.background }"
            />
            <!-- Accent line -->
            <div 
              class="absolute left-1/3 top-0 h-full w-1"
              :style="{ backgroundColor: theme.swatches.accent }"
            />
          </div>
          
          <span class="text-xs font-medium text-gray-700 dark:text-gray-300 text-center">
            {{ theme.label }}
          </span>
          
          <!-- Selected checkmark -->
          <div 
            v-if="selectedOption === theme.id"
            class="absolute top-1 right-1 w-5 h-5 rounded-full bg-primary flex items-center justify-center"
          >
            <Check class="w-3 h-3 text-white" />
          </div>
        </button>
      </div>
    </div>

    <!-- Custom Theme Section -->
    <div v-if="isCustom" class="border-t border-gray-200 dark:border-gray-700 pt-6">
      <h5 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">
        Custom Theme Colors
      </h5>
      
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <!-- Sidebar Background -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.sidebarBg"
            @input="e => updateCustomColor('sidebarBg', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Sidebar BG</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.sidebarBg }}</div>
          </div>
        </div>

        <!-- Sidebar Text -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.sidebarText"
            @input="e => updateCustomColor('sidebarText', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Sidebar Text</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.sidebarText }}</div>
          </div>
        </div>

        <!-- Center Channel Background -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.centerChannelBg"
            @input="e => updateCustomColor('centerChannelBg', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Center BG</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.centerChannelBg }}</div>
          </div>
        </div>

        <!-- Center Channel Text -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.centerChannelColor"
            @input="e => updateCustomColor('centerChannelColor', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Center Text</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.centerChannelColor }}</div>
          </div>
        </div>

        <!-- Link Color -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.linkColor"
            @input="e => updateCustomColor('linkColor', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Link Color</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.linkColor }}</div>
          </div>
        </div>

        <!-- Button Background -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.buttonBg"
            @input="e => updateCustomColor('buttonBg', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Button BG</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.buttonBg }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Preview Section -->
    <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
      <h5 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">
        Preview
      </h5>
      <div 
        class="rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden h-24 flex"
        :style="{ 
          backgroundColor: isCustom ? customColors.centerChannelBg : THEME_EDITOR_OPTIONS.find(t => t.id === selectedOption)?.colors?.centerChannelBg || '#ffffff'
        }"
      >
        <!-- Sidebar preview -->
        <div 
          class="w-1/4 h-full flex flex-col p-2"
          :style="{ 
            backgroundColor: isCustom ? customColors.sidebarBg : THEME_EDITOR_OPTIONS.find(t => t.id === selectedOption)?.colors?.sidebarBg || '#1e325c'
          }"
        >
          <div 
            class="text-xs font-medium truncate"
            :style="{ 
              color: isCustom ? customColors.sidebarText : THEME_EDITOR_OPTIONS.find(t => t.id === selectedOption)?.colors?.sidebarText || '#ffffff'
            }"
          >
            Channels
          </div>
        </div>
        <!-- Center preview -->
        <div 
          class="flex-1 h-full p-2"
          :style="{ 
            color: isCustom ? customColors.centerChannelColor : THEME_EDITOR_OPTIONS.find(t => t.id === selectedOption)?.colors?.centerChannelColor || '#3d3c40'
          }"
        >
          <div class="text-xs mb-1">Welcome to our platform</div>
          <a 
            class="text-xs hover:underline"
            :style="{ 
              color: isCustom ? customColors.linkColor : THEME_EDITOR_OPTIONS.find(t => t.id === selectedOption)?.colors?.linkColor || '#166de0'
            }"
          >
            Click here to learn more
          </a>
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex items-center justify-end gap-2 pt-2">
      <button
        type="button"
        @click="handleCancel"
        class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
      >
        Cancel
      </button>
      <button
        type="button"
        @click="handleSave"
        class="px-4 py-2 text-sm font-medium text-white bg-primary hover:bg-primary/90 rounded-lg transition-colors"
      >
        Save
      </button>
    </div>
  </div>
</template>
