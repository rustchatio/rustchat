<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Pencil } from 'lucide-vue-next'
import SettingItemMax from '../SettingItemMax.vue'
import { useCallsStore } from '../../../stores/calls'

const callsStore = useCallsStore()
const expandedRow = ref<string | null>(null)
const saving = ref(false)
const permissionError = ref<string | null>(null)

const audioInputDevices = ref<MediaDeviceInfo[]>([])
const audioOutputDevices = ref<MediaDeviceInfo[]>([])
const videoDevices = ref<MediaDeviceInfo[]>([])

const selectedAudioInput = ref('')
const selectedAudioOutput = ref('')
const selectedVideoDevice = ref('')

const audioInputLabel = computed(() => {
  const device = audioInputDevices.value.find((d) => d.deviceId === selectedAudioInput.value)
  return device?.label || 'Default microphone'
})

const audioOutputLabel = computed(() => {
  const device = audioOutputDevices.value.find((d) => d.deviceId === selectedAudioOutput.value)
  return device?.label || 'Default speaker'
})

const videoDeviceLabel = computed(() => {
  const device = videoDevices.value.find((d) => d.deviceId === selectedVideoDevice.value)
  return device?.label || 'Default camera'
})

onMounted(async () => {
  await enumerateDevices()
  syncLocalState()
})

function syncLocalState() {
  selectedAudioInput.value = callsStore.preferredAudioInput || audioInputDevices.value[0]?.deviceId || ''
  selectedAudioOutput.value = callsStore.preferredAudioOutput || audioOutputDevices.value[0]?.deviceId || ''
  selectedVideoDevice.value = callsStore.preferredVideoDevice || videoDevices.value[0]?.deviceId || ''
}

async function enumerateDevices() {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: true })
    stream.getTracks().forEach((track) => track.stop())

    const devices = await navigator.mediaDevices.enumerateDevices()
    audioInputDevices.value = devices.filter((d) => d.kind === 'audioinput')
    audioOutputDevices.value = devices.filter((d) => d.kind === 'audiooutput')
    videoDevices.value = devices.filter((d) => d.kind === 'videoinput')

    permissionError.value = null
  } catch (error: any) {
    console.error('Failed to enumerate devices', error)
    permissionError.value = error?.message || 'Permission denied'
  }
}

function expandRow(rowId: string) {
  if (expandedRow.value === rowId) {
    return
  }
  syncLocalState()
  expandedRow.value = rowId
}

function cancelEdit() {
  syncLocalState()
  expandedRow.value = null
}

async function saveAudioDevices() {
  saving.value = true
  try {
    await callsStore.setPreferredAudioInput(selectedAudioInput.value)
    await callsStore.setPreferredAudioOutput(selectedAudioOutput.value)
    expandedRow.value = null
  } finally {
    saving.value = false
  }
}

async function saveVideoDevices() {
  saving.value = true
  try {
    await callsStore.setPreferredVideoDevice(selectedVideoDevice.value)
    expandedRow.value = null
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="space-y-1">
    <h3 class="mb-1 px-2 text-3xl sm:text-[2rem] font-semibold tracking-tight text-gray-900 dark:text-white">Calls Settings</h3>

    <div class="rounded-lg border border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-900">
      <div v-if="expandedRow !== 'audio_devices'">
        <button
          type="button"
          class="flex w-full items-start justify-between gap-4 border-b border-gray-200 px-4 py-4 text-left hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800/50"
          @click="expandRow('audio_devices')"
        >
          <div>
            <div class="text-xl sm:text-2xl font-medium leading-tight text-gray-900 dark:text-white">Audio devices</div>
            <div class="mt-1 text-sm text-gray-500 dark:text-gray-400">Set up audio devices to be used for Mattermost calls</div>
          </div>
          <span class="mt-0.5 inline-flex items-center gap-1 text-sm font-medium text-primary">
            <Pencil class="h-4 w-4" />
            Edit
          </span>
        </button>
      </div>

      <SettingItemMax
        v-else
        label="Audio devices"
        description="Set up audio devices to be used for Mattermost calls"
        :loading="saving"
        @save="saveAudioDevices"
        @cancel="cancelEdit"
      >
        <div class="space-y-4">
          <div v-if="permissionError" class="rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800 dark:border-amber-700 dark:bg-amber-900/20 dark:text-amber-300">
            Permission required for device listing: {{ permissionError }}
          </div>

          <div>
            <div class="mb-2 text-sm font-semibold text-gray-900 dark:text-white">Microphone</div>
            <div class="mb-2 text-xs text-gray-500 dark:text-gray-400">Current: {{ audioInputLabel }}</div>
            <div class="space-y-2">
              <label
                v-for="device in audioInputDevices"
                :key="device.deviceId"
                class="flex items-start gap-3 rounded-md border border-gray-200 p-3 text-sm hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800/50"
              >
                <input v-model="selectedAudioInput" type="radio" :value="device.deviceId" class="mt-0.5 h-4 w-4" />
                <span>{{ device.label || 'Default microphone' }}</span>
              </label>
            </div>
          </div>

          <div>
            <div class="mb-2 text-sm font-semibold text-gray-900 dark:text-white">Speaker</div>
            <div class="mb-2 text-xs text-gray-500 dark:text-gray-400">Current: {{ audioOutputLabel }}</div>
            <div v-if="audioOutputDevices.length > 0" class="space-y-2">
              <label
                v-for="device in audioOutputDevices"
                :key="device.deviceId"
                class="flex items-start gap-3 rounded-md border border-gray-200 p-3 text-sm hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800/50"
              >
                <input v-model="selectedAudioOutput" type="radio" :value="device.deviceId" class="mt-0.5 h-4 w-4" />
                <span>{{ device.label || 'Default speaker' }}</span>
              </label>
            </div>
            <div v-else class="rounded-md border border-gray-200 p-3 text-sm text-gray-500 dark:border-gray-700 dark:text-gray-400">
              No selectable output devices were detected in this browser.
            </div>
          </div>
        </div>
      </SettingItemMax>

      <div v-if="expandedRow !== 'video_devices'">
        <button
          type="button"
          class="flex w-full items-start justify-between gap-4 px-4 py-4 text-left hover:bg-gray-50 dark:hover:bg-gray-800/50"
          @click="expandRow('video_devices')"
        >
          <div>
            <div class="text-xl sm:text-2xl font-medium leading-tight text-gray-900 dark:text-white">Video devices</div>
            <div class="mt-1 text-sm text-gray-500 dark:text-gray-400">Set up video devices to be used for Mattermost calls</div>
          </div>
          <span class="mt-0.5 inline-flex items-center gap-1 text-sm font-medium text-primary">
            <Pencil class="h-4 w-4" />
            Edit
          </span>
        </button>
      </div>

      <SettingItemMax
        v-else
        label="Video devices"
        description="Set up video devices to be used for Mattermost calls"
        :loading="saving"
        @save="saveVideoDevices"
        @cancel="cancelEdit"
      >
        <div class="space-y-4">
          <div class="mb-2 text-sm font-semibold text-gray-900 dark:text-white">Camera</div>
          <div class="mb-2 text-xs text-gray-500 dark:text-gray-400">Current: {{ videoDeviceLabel }}</div>
          <div class="space-y-2">
            <label
              v-for="device in videoDevices"
              :key="device.deviceId"
              class="flex items-start gap-3 rounded-md border border-gray-200 p-3 text-sm hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800/50"
            >
              <input v-model="selectedVideoDevice" type="radio" :value="device.deviceId" class="mt-0.5 h-4 w-4" />
              <span>{{ device.label || 'Default camera' }}</span>
            </label>
          </div>
          <div v-if="videoDevices.length === 0" class="rounded-md border border-gray-200 p-3 text-sm text-gray-500 dark:border-gray-700 dark:text-gray-400">
            No camera devices found.
          </div>
        </div>
      </SettingItemMax>
    </div>
  </div>
</template>
