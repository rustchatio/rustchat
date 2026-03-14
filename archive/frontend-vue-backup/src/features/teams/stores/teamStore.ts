// Team Store - Pure state management for teams

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import type { Team, TeamId, TeamMember } from '../../../core/entities/Team'

export const useTeamStore = defineStore('teamStore', () => {
  // State
  const teams = ref<Map<TeamId, Team>>(new Map())
  const publicTeams = ref<Team[]>([])
  const members = ref<TeamMember[]>([])
  const currentTeamId = ref<TeamId | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const allTeams = computed(() => Array.from(teams.value.values()))

  const currentTeam = computed(() => {
    if (!currentTeamId.value) return null
    return teams.value.get(currentTeamId.value) || null
  })

  // Actions
  function setTeams(items: Team[]) {
    teams.value.clear()
    for (const team of items) {
      teams.value.set(team.id, team)
    }
  }

  function addTeam(team: Team) {
    teams.value.set(team.id, team)
  }

  function updateTeam(team: Team) {
    const existing = teams.value.get(team.id)
    if (existing) {
      teams.value.set(team.id, { ...existing, ...team })
    }
  }

  function removeTeam(teamId: TeamId) {
    teams.value.delete(teamId)
    
    // If we removed the current team, select another
    if (currentTeamId.value === teamId) {
      const remaining = allTeams.value
      currentTeamId.value = remaining[0]?.id || null
    }
  }

  function setPublicTeams(items: Team[]) {
    publicTeams.value = items
  }

  function setCurrentTeamId(teamId: TeamId | null) {
    currentTeamId.value = teamId
  }

  function setMembers(items: TeamMember[]) {
    members.value = items
  }

  function setLoading(value: boolean) {
    loading.value = value
  }

  function setError(err: string | null) {
    error.value = err
  }

  function clearError() {
    error.value = null
  }

  function clear() {
    teams.value.clear()
    publicTeams.value = []
    members.value = []
    currentTeamId.value = null
  }

  return {
    // State (readonly)
    teams: readonly(teams),
    publicTeams: readonly(publicTeams),
    members: readonly(members),
    currentTeamId: readonly(currentTeamId),
    loading: readonly(loading),
    error: readonly(error),

    // Getters
    allTeams,
    currentTeam,

    // Actions
    setTeams,
    addTeam,
    updateTeam,
    removeTeam,
    setPublicTeams,
    setCurrentTeamId,
    setMembers,
    setLoading,
    setError,
    clearError,
    clear
  }
})
