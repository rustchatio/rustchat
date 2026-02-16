<script setup lang="ts">
import GlobalHeader from './GlobalHeader.vue';
import TeamRail from './TeamRail.vue';
import ChannelSidebar from './ChannelSidebar.vue';
import RightSidebar from './RightSidebar.vue';
import { useUIStore } from '../../stores/ui';
import { useBreakpoints } from '../../composables/useBreakpoints';

const ui = useUIStore();
const { isMobile, isMobileOrTablet } = useBreakpoints();
</script>

<template>
  <div class="h-screen flex flex-col overflow-hidden bg-surface text-gray-800 dark:text-gray-100 transition-colors duration-300">
    <!-- Top Header -->
    <GlobalHeader />

    <div class="flex flex-1 overflow-hidden relative">
        <!-- Team Rail (Leftmost) -->
        <TeamRail v-if="!isMobile" class="border-r border-sidebar-hover/30" />

        <!-- Channel Sidebar (LHS) -->
        <ChannelSidebar v-if="!isMobile" class="border-r border-border-dim dark:border-white/5" />

        <!-- Main Content (Center) -->
        <main 
          class="flex-1 flex flex-col min-w-0 bg-surface relative transition-all duration-300"
          :class="{ 'shadow-2xl': ui.isRhsOpen && isMobileOrTablet }"
        >
            <slot />
            
            <!-- Mobile Overlay for Sidebar/RHS -->
            <div 
              v-if="ui.isRhsOpen && isMobileOrTablet" 
              class="absolute inset-0 bg-black/20 dark:bg-black/40 backdrop-blur-sm z-30 lg:hidden"
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
            class="fixed lg:relative top-0 right-0 h-full z-40 lg:z-10 shadow-2xl lg:shadow-none bg-surface"
            :class="[isMobileOrTablet ? 'w-[85%] sm:w-[400px]' : 'w-[400px]']"
          />
        </transition>
    </div>
  </div>
</template>

