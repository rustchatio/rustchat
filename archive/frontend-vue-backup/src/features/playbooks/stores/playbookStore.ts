// Playbook Store - Pure state management for playbooks

import { defineStore } from 'pinia'
import { ref, readonly } from 'vue'
import type { Playbook, PlaybookFull, PlaybookRun, RunWithTasks } from '../../../api/playbooks'

export const usePlaybookStore = defineStore('playbookStore', () => {
  // State
  const playbooks = ref<Playbook[]>([])
  const currentPlaybook = ref<PlaybookFull | null>(null)
  const runs = ref<PlaybookRun[]>([])
  const currentRun = ref<RunWithTasks | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Actions
  function setPlaybooks(value: Playbook[]) {
    playbooks.value = value
  }

  function addPlaybook(playbook: Playbook) {
    playbooks.value.push(playbook)
  }

  function updatePlaybook(playbook: Playbook) {
    const index = playbooks.value.findIndex(p => p.id === playbook.id)
    if (index !== -1) {
      playbooks.value[index] = playbook
    }
  }

  function removePlaybook(id: string) {
    playbooks.value = playbooks.value.filter(p => p.id !== id)
  }

  function setCurrentPlaybook(playbook: PlaybookFull | null) {
    currentPlaybook.value = playbook
  }

  function setRuns(value: PlaybookRun[]) {
    runs.value = value
  }

  function addRun(run: PlaybookRun) {
    runs.value.unshift(run)
  }

  function setCurrentRun(run: RunWithTasks | null) {
    currentRun.value = run
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

  return {
    // State (readonly)
    playbooks: readonly(playbooks),
    currentPlaybook: readonly(currentPlaybook),
    runs: readonly(runs),
    currentRun: readonly(currentRun),
    loading: readonly(loading),
    error: readonly(error),

    // Actions
    setPlaybooks,
    addPlaybook,
    updatePlaybook,
    removePlaybook,
    setCurrentPlaybook,
    setRuns,
    addRun,
    setCurrentRun,
    setLoading,
    setError,
    clearError
  }
})
