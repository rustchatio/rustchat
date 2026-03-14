<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
    show: boolean
    anchorEl?: HTMLElement | null
}>()

const emit = defineEmits<{
    (e: 'select', emoji: string): void
    (e: 'close'): void
}>()

const categories = [
    { id: 'frequent', name: '👍', emojis: ['👍', '❤️', '😂', '🎉', '🤔', '👀', '🙌', '💯'] },
    { id: 'smileys', name: '😀', emojis: ['😀', '😃', '😄', '😁', '😆', '😅', '🤣', '😂', '🙂', '😊', '😇', '🥰', '😍', '🤩', '😘', '😗', '😚', '😋', '😛', '😜', '🤪', '😝', '🤑', '🤗', '🤭', '🤫', '🤔', '🤐', '🤨', '😐', '😑', '😶', '😏', '😒', '🙄', '😬', '🤥', '😌', '😔', '😪', '🤤', '😴', '😷'] },
    { id: 'gestures', name: '👋', emojis: ['👋', '🤚', '🖐️', '✋', '🖖', '👌', '🤌', '🤏', '✌️', '🤞', '🤟', '🤘', '🤙', '👈', '👉', '👆', '🖕', '👇', '☝️', '👍', '👎', '✊', '👊', '🤛', '🤜', '👏', '🙌', '👐', '🤲', '🤝', '🙏'] },
    { id: 'hearts', name: '❤️', emojis: ['❤️', '🧡', '💛', '💚', '💙', '💜', '🖤', '🤍', '🤎', '💔', '❤️‍🔥', '❤️‍🩹', '❣️', '💕', '💞', '💓', '💗', '💖', '💘', '💝'] },
    { id: 'objects', name: '💡', emojis: ['⭐', '🌟', '✨', '⚡', '🔥', '💫', '🎯', '🎪', '🎨', '🎬', '🎤', '🎧', '🎵', '🎶', '🎹', '🥁', '🎸', '🎺', '🎻', '🎲', '🎮', '🕹️', '🎰', '🧩'] },
    { id: 'symbols', name: '✅', emojis: ['✅', '❌', '❓', '❗', '💯', '🔴', '🟠', '🟡', '🟢', '🔵', '🟣', '⚫', '⚪', '🟤', '🔶', '🔷', '🔸', '🔹', '▪️', '▫️', '◾', '◽', '◼️', '◻️', '⬛', '⬜'] },
]

const activeCategory = ref('frequent')
const searchQuery = ref('')
const pickerRef = ref<HTMLElement | null>(null)
const pickerStyle = ref<Record<string, string>>({})

const filteredEmojis = computed(() => {
    const cat = categories.find(c => c.id === activeCategory.value)
    if (!cat) return []
    
    if (searchQuery.value) {
        return cat.emojis.filter(e => e.includes(searchQuery.value))
    }
    return cat.emojis
})

function selectEmoji(emoji: string) {
    emit('select', emoji)
    emit('close')
}

function updatePosition() {
    if (!props.show || !props.anchorEl || !pickerRef.value) return

    const anchorRect = props.anchorEl.getBoundingClientRect()
    const panelRect = pickerRef.value.getBoundingClientRect()
    const viewportPadding = 8
    const gap = 10

    let left = anchorRect.right - panelRect.width
    left = Math.max(viewportPadding, Math.min(left, window.innerWidth - panelRect.width - viewportPadding))

    let top = anchorRect.top - panelRect.height - gap
    if (top < viewportPadding) {
        top = anchorRect.bottom + gap
    }
    top = Math.max(viewportPadding, Math.min(top, window.innerHeight - panelRect.height - viewportPadding))

    pickerStyle.value = {
        left: `${Math.round(left)}px`,
        top: `${Math.round(top)}px`,
    }
}

function handlePointerDown(event: MouseEvent) {
    if (!props.show) return
    const target = event.target as Node | null
    if (!target) return
    if (pickerRef.value?.contains(target)) return
    if (props.anchorEl?.contains(target)) return
    emit('close')
}

function handleKeyDown(event: KeyboardEvent) {
    if (props.show && event.key === 'Escape') {
        emit('close')
    }
}

watch(
    () => props.show,
    async (visible) => {
        if (!visible) return
        await nextTick()
        updatePosition()
    }
)

watch(
    () => props.anchorEl,
    () => {
        if (!props.show) return
        void nextTick(updatePosition)
    }
)

onMounted(() => {
    window.addEventListener('resize', updatePosition)
    window.addEventListener('scroll', updatePosition, true)
    document.addEventListener('mousedown', handlePointerDown)
    document.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
    window.removeEventListener('resize', updatePosition)
    window.removeEventListener('scroll', updatePosition, true)
    document.removeEventListener('mousedown', handlePointerDown)
    document.removeEventListener('keydown', handleKeyDown)
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="show"
      ref="pickerRef"
      :style="pickerStyle"
      class="fixed z-[9999] w-[22rem] max-w-[calc(100vw-1rem)] bg-white dark:bg-gray-800 rounded-2xl shadow-2xl border border-black/5 dark:border-white/10 overflow-hidden animate-fade-in"
    >
      <!-- Header -->
      <div class="p-2 border-b border-gray-200 dark:border-gray-700">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search emoji..."
          class="w-full px-3 py-1.5 text-sm bg-gray-100 dark:bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary text-gray-900 dark:text-white"
        />
      </div>

      <!-- Categories -->
      <div class="flex items-center px-2 py-1 border-b border-gray-100 dark:border-gray-700 space-x-1">
        <button
          v-for="cat in categories"
          :key="cat.id"
          @click="activeCategory = cat.id"
          class="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          :class="activeCategory === cat.id ? 'bg-gray-200 dark:bg-gray-600' : ''"
        >
          {{ cat.name }}
        </button>
      </div>

      <!-- Emojis Grid -->
      <div class="p-2 grid grid-cols-8 gap-1 max-h-56 overflow-y-auto">
        <button
          v-for="emoji in filteredEmojis"
          :key="emoji"
          @click="selectEmoji(emoji)"
          class="p-1.5 text-xl hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
        >
          {{ emoji }}
        </button>
      </div>

      <!-- Empty State -->
      <div v-if="filteredEmojis.length === 0" class="p-4 text-center text-gray-500 text-sm">
        No emojis found
      </div>
    </div>
  </Teleport>
</template>
