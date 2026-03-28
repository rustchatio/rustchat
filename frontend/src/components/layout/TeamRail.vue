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
  <aside class="w-[var(--team-rail-width)] bg-bg-surface-2 flex flex-col items-center py-3 z-20 shrink-0 border-r border-border-1">
    <!-- Teams List -->
    <div class="flex-1 flex flex-col items-center gap-2 w-full px-2">
      <div 
        v-for="team in teamStore.teams" 
        :key="team.id"
        class="group relative w-full flex justify-center"
      >
        <!-- Active Indicator -->
        <div 
          v-if="teamStore.currentTeamId === team.id"
          class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-8 bg-brand rounded-r-full transition-standard"
        ></div>

        <!-- Team Button -->
        <button
          @click="selectTeam(team.id)"
          class="w-10 h-10 rounded-r-2 font-bold text-sm transition-standard relative overflow-hidden flex items-center justify-center"
          :class="{ 
            'bg-brand text-brand-foreground shadow-1': teamStore.currentTeamId === team.id,
            'bg-bg-surface-1 text-text-1 hover:bg-bg-surface-1 hover:text-brand border border-border-1': teamStore.currentTeamId !== team.id
          }"
          :title="team.display_name || team.name"
        >
          {{ getInitials(team.display_name || team.name) }}
        </button>

        <!-- Unread Indicator -->
        <div 
          v-if="unreadStore.getTeamUnreadCount(team.id) > 0 && teamStore.currentTeamId !== team.id"
          class="absolute -top-0.5 right-1 w-3 h-3 bg-danger rounded-full border-2 border-bg-surface-2 shadow-1"
        ></div>
      </div>

      <!-- Empty State -->
      <div v-if="teamStore.teams.length === 0 && !teamStore.loading" class="text-text-3 text-[10px] text-center px-2 uppercase font-bold tracking-tight mt-4">
        No teams
      </div>
    </div>

    <!-- Add Team Button -->
    <div class="pt-3 border-t border-border-1 mt-auto w-full px-2">
      <button 
        @click="showCreateModal = true"
        class="w-10 h-10 mx-auto rounded-r-2 bg-bg-surface-1 hover:bg-success hover:text-white transition-standard flex items-center justify-center text-text-3 border border-dashed border-border-2 hover:border-success"
        title="Create Team"
      >
        <Plus class="w-5 h-5" />
      </button>
    </div>

    <!-- Create Team Modal -->
    <CreateTeamModal :show="showCreateModal" @close="showCreateModal = false" />
  </aside>
</template>
