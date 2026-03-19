<template>
  <div
    class="flex-shrink-0 flex items-center justify-center w-9 h-9 rounded-full"
    :class="bgClass"
  >
    <component :is="iconComponent" class="w-4 h-4" :class="colorClass" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { AtSign, MessageCircle, Heart, Mail, MessageSquare } from 'lucide-vue-next'
import { ActivityType } from '../../features/activity/types'

const props = defineProps<{
  type: ActivityType
}>()

type IconConfig = {
  icon: unknown
  bg: string
  color: string
}

const iconConfigs: Record<ActivityType, IconConfig> = {
  [ActivityType.MENTION]: {
    icon: AtSign,
    bg: 'bg-blue-100 dark:bg-blue-900/40',
    color: 'text-blue-600 dark:text-blue-400'
  },
  [ActivityType.REPLY]: {
    icon: MessageCircle,
    bg: 'bg-green-100 dark:bg-green-900/40',
    color: 'text-green-600 dark:text-green-400'
  },
  [ActivityType.REACTION]: {
    icon: Heart,
    bg: 'bg-pink-100 dark:bg-pink-900/40',
    color: 'text-pink-600 dark:text-pink-400'
  },
  [ActivityType.DM]: {
    icon: Mail,
    bg: 'bg-purple-100 dark:bg-purple-900/40',
    color: 'text-purple-600 dark:text-purple-400'
  },
  [ActivityType.THREAD_REPLY]: {
    icon: MessageSquare,
    bg: 'bg-orange-100 dark:bg-orange-900/40',
    color: 'text-orange-600 dark:text-orange-400'
  }
}

const config = computed(() => iconConfigs[props.type] ?? iconConfigs[ActivityType.MENTION])
const iconComponent = computed(() => config.value.icon)
const bgClass = computed(() => config.value.bg)
const colorClass = computed(() => config.value.color)
</script>
