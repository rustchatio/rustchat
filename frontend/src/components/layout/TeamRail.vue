<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Plus } from 'lucide-vue-next';
import { useTeamStore } from '../../stores/teams';
import { useUnreadStore } from '../../stores/unreads';
import CreateTeamModal from '../modals/CreateTeamModal.vue';

const teamStore = useTeamStore();
const unreadStore = useUnreadStore();
const showCreateModal = ref(false);

onMounted(() => {
    teamStore.fetchTeams();
});

function selectTeam(teamId: string) {
    teamStore.selectTeam(teamId);
}

function getInitials(name: string): string {
    return name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
}
</script>

<template>
  <aside class="w-[68px] bg-bg-surface-2 flex flex-col items-center py-sp-4 space-y-sp-4 z-20 shrink-0 transition-standard">
    <div 
      v-for="team in teamStore.teams" 
      :key="team.id"
      class="group relative"
    >
      <!-- Active Indicator -->
      <div 
        v-if="teamStore.currentTeamId === team.id"
        class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-8 bg-brand rounded-r-full transition-standard"
      ></div>

      <!-- Team Icon -->
      <button
        @click="selectTeam(team.id)"
        class="w-12 h-12 rounded-r-2 bg-bg-surface-1 hover:bg-brand transition-standard cursor-pointer flex items-center justify-center text-text-1 hover:text-white font-bold text-sm overflow-hidden border border-border-1 group-hover:shadow-1"
        :class="{ 'border-brand ring-2 ring-brand/20 bg-brand text-white': teamStore.currentTeamId === team.id }"
        :title="team.display_name || team.name"
      >
        {{ getInitials(team.display_name || team.name) }}
      </button>

      <!-- Unread Indicator (Dot) -->
      <div 
        v-if="unreadStore.getTeamUnreadCount(team.id) > 0 && teamStore.currentTeamId !== team.id"
        class="absolute -top-1 -right-1 w-3.5 h-3.5 bg-brand rounded-full border-2 border-bg-surface-2 flex items-center justify-center shadow-1 pointer-events-none"
      >
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="teamStore.teams.length === 0 && !teamStore.loading" class="text-text-3 text-[10px] text-center px-1 uppercase font-bold tracking-tight">
        No teams
    </div>

    <!-- Add Team Button -->
    <button 
      @click="showCreateModal = true"
      class="w-12 h-12 rounded-r-2 bg-bg-surface-1 hover:bg-success transition-standard cursor-pointer flex items-center justify-center text-text-3 hover:text-white group border border-dashed border-border-2"
      title="Create Team"
    >
      <Plus class="w-5 h-5 group-hover:scale-110 transition-transform" />
    </button>

    <!-- Create Team Modal -->
    <CreateTeamModal :show="showCreateModal" @close="showCreateModal = false" />
  </aside>
</template>
