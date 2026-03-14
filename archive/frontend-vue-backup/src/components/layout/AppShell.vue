<script setup lang="ts">
import { watch } from 'vue';
import GlobalHeader from './GlobalHeader.vue';
import TeamRail from './TeamRail.vue';
import ChannelSidebar from './ChannelSidebar.vue';
import RightSidebar from './RightSidebar.vue';
import { useUIStore } from '../../stores/ui';
import { useChannelStore } from '../../stores/channels';
import { useBreakpoints } from '../../composables/useBreakpoints';

const emit = defineEmits<{
  (e: 'rhsJump', messageId: string): void;
  (e: 'openChannelSettings'): void;
}>();

const ui = useUIStore();
const channelStore = useChannelStore();
const { isMobile, isMobileOrTablet } = useBreakpoints();

watch(isMobile, (mobile) => {
  if (!mobile && ui.isLhsOpen) {
    ui.closeLhs();
  }
});

watch(() => channelStore.currentChannelId, () => {
  if (isMobile.value && ui.isLhsOpen) {
    ui.closeLhs();
  }
});
</script>

<template>
  <div class="h-screen flex flex-col overflow-hidden bg-bg-app text-text-1 transition-standard">
    <!-- Top Header -->
    <GlobalHeader class="z-30" />

    <!-- Mobile Left Sidebar Drawer -->
    <transition
      enter-active-class="transition-standard duration-300 transform"
      enter-from-class="-translate-x-full opacity-0"
      enter-to-class="translate-x-0 opacity-100"
      leave-active-class="transition-standard duration-200 transform"
      leave-from-class="translate-x-0 opacity-100"
      leave-to-class="-translate-x-full opacity-0"
    >
      <div v-if="ui.isLhsOpen && isMobile" class="fixed top-[64px] bottom-0 left-0 z-50 flex shadow-2xl">
        <TeamRail class="h-full border-r border-border-1" />
        <ChannelSidebar class="h-full border-r border-border-1" />
      </div>
    </transition>
    <div
      v-if="ui.isLhsOpen && isMobile"
      class="fixed inset-0 z-40 bg-black/40 backdrop-blur-sm"
      @click="ui.closeLhs()"
    ></div>

    <div class="flex flex-1 overflow-hidden relative">
        <!-- Team Rail (Leftmost) -->
        <TeamRail v-if="!isMobile" class="border-r border-border-1" />

        <!-- Channel Sidebar (LHS) -->
        <ChannelSidebar v-if="!isMobile" class="border-r border-border-1" />

        <!-- Main Content (Center) -->
        <main 
          class="flex-1 flex flex-col min-w-0 bg-bg-surface-1 relative transition-standard overflow-hidden"
          :class="{ 'shadow-2': ui.isRhsOpen && isMobileOrTablet }"
        >
            <slot />
            
            <!-- Mobile Overlay for Sidebar/RHS -->
            <div 
              v-if="ui.isRhsOpen && isMobileOrTablet" 
              class="absolute inset-0 bg-black/40 backdrop-blur-sm z-30 lg:hidden transition-standard"
              @click="ui.closeRhs()"
            ></div>
        </main>

        <!-- Right Sidebar (RHS) -->
        <transition
          enter-active-class="transition-standard duration-300 transform"
          enter-from-class="translate-x-full opacity-0"
          enter-to-class="translate-x-0 opacity-100"
          leave-active-class="transition-standard duration-200 transform"
          leave-from-class="translate-x-0 opacity-100"
          leave-to-class="translate-x-full opacity-0"
        >
          <RightSidebar 
            v-if="ui.isRhsOpen" 
            class="fixed lg:relative top-0 right-0 h-full z-40 lg:z-10 shadow-2 lg:shadow-none bg-bg-surface-1 border-l border-border-1"
            :class="[isMobileOrTablet ? 'w-[85%] sm:w-[360px]' : 'w-[360px]']"
            @jump="emit('rhsJump', $event)"
            @openSettings="emit('openChannelSettings')"
          />
        </transition>
    </div>
  </div>
</template>
