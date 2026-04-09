# WebSocket Disconnection UX Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement progressive disconnection UI states to prevent users from acting on stale chat data when WebSocket connection is lost.

**Architecture:** Extend existing `useWebSocket` composable to track connection state with timestamps and retry countdown. Add three UI components: a status banner for brief disconnections, a full-screen modal for critical state, and a header indicator. Disable composer immediately on disconnect with clear messaging.

**Tech Stack:** Vue 3, TypeScript, Tailwind CSS, existing WebSocket infrastructure

---

## Prerequisites

- Review design doc: `docs/plans/2026-04-08-websocket-disconnection-ux-design.md`
- Understand current WebSocket implementation in `frontend/src/composables/useWebSocket.ts`
- Check existing `App.vue` inactive tab overlay pattern for modal styling reference

---

## Task 1: Extend useWebSocket Composable with State Management

**Files:**
- Modify: `frontend/src/composables/useWebSocket.ts:42-60` (add new refs)
- Modify: `frontend/src/composables/useWebSocket.ts:335-436` (connect/disconnect functions)
- Modify: `frontend/src/composables/useWebSocket.ts:763-775` (return values)

**Step 1: Add new reactive state refs (after line 48)**

```typescript
// Connection state management
const connectionStatus = ref<'connected' | 'reconnecting' | 'disconnected' | 'failed'>('connected')
const disconnectedAt = ref<Date | null>(null)
const reconnectAttempt = ref(0)
const nextRetryIn = ref(0)
const connectionError = ref<string | null>(null)

// Constants
const MAX_RECONNECT_ATTEMPTS = 10
const RECONNECT_DELAY_BASE = 1000
const RECONNECT_DELAY_MAX = 10000
const STATE_TRANSITION_MS = {
  TO_DISCONNECTED: 5000,   // 5 seconds
  TO_FAILED: 30000         // 30 seconds
}

// Countdown timer
let countdownTimer: ReturnType<typeof setInterval> | null = null
```

**Step 2: Add helper functions (after new refs)**

```typescript
function startCountdown() {
  stopCountdown()
  nextRetryIn.value = Math.min(
    RECONNECT_DELAY_BASE * Math.pow(1.5, reconnectAttempt.value),
    RECONNECT_DELAY_MAX
  ) / 1000
  
  countdownTimer = setInterval(() => {
    if (nextRetryIn.value > 0) {
      nextRetryIn.value--
    } else {
      stopCountdown()
    }
  }, 1000)
}

function stopCountdown() {
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
}

function updateConnectionStatus() {
  if (connected.value) {
    connectionStatus.value = 'connected'
    disconnectedAt.value = null
    connectionError.value = null
    stopCountdown()
    return
  }
  
  if (!disconnectedAt.value) {
    connectionStatus.value = 'reconnecting'
    return
  }
  
  const disconnectedMs = Date.now() - disconnectedAt.value.getTime()
  
  if (disconnectedMs >= STATE_TRANSITION_MS.TO_FAILED || reconnectAttempt.value >= MAX_RECONNECT_ATTEMPTS) {
    connectionStatus.value = 'failed'
    stopCountdown()
  } else if (disconnectedMs >= STATE_TRANSITION_MS.TO_DISCONNECTED) {
    connectionStatus.value = 'disconnected'
  } else {
    connectionStatus.value = 'reconnecting'
  }
}
```

**Step 3: Modify connect() - reset state on open (around line 354)**

Add inside `socket.onopen`:
```typescript
socket.onopen = () => {
    const openedAfterReconnect = reconnectAttempts.value > 0
    console.log('WebSocket connected')
    connected.value = true
    reconnectAttempts.value = 0
    reconnectAttempt.value = 0
    clearReconnectTimer()
    stopCountdown()
    updateConnectionStatus() // ADD THIS
    
    // ... rest of existing onopen logic ...
}
```

**Step 4: Modify onclose handler - track state (around line 385)**

Replace the onclose handler with:
```typescript
socket.onclose = (event) => {
    console.log('WebSocket disconnected', event.code, event.reason)
    connected.value = false
    ws.value = null
    
    if (isAuthExpiryCloseEvent(event)) {
        clearReconnectTimer()
        reconnectAttempts.value = 0
        reconnectAttempt.value = 0
        stopCountdown()
        updateConnectionStatus()
        void authStore.logout('expired')
        return
    }
    
    if (!authStore.token) {
        clearReconnectTimer()
        reconnectAttempts.value = 0
        reconnectAttempt.value = 0
        stopCountdown()
        updateConnectionStatus()
        return
    }
    
    // Track disconnect time on first disconnect
    if (!disconnectedAt.value) {
        disconnectedAt.value = new Date()
    }
    reconnectAttempt.value++
    updateConnectionStatus()
    
    // Start countdown for UI
    startCountdown()
    
    // Attempt to reconnect
    if (reconnectAttempt.value < MAX_RECONNECT_ATTEMPTS) {
        const baseDelay = Math.min(
            RECONNECT_DELAY_BASE * Math.pow(1.5, reconnectAttempt.value),
            RECONNECT_DELAY_MAX
        )
        const jitter = Math.random() * 1000
        const delay = baseDelay + jitter
        
        console.log(`Reconnecting in ${Math.round(delay)}ms... (attempt ${reconnectAttempt.value})`)
        clearReconnectTimer()
        reconnectTimer = setTimeout(() => {
            if (!connected.value) connect()
        }, delay)
    } else {
        console.log('Max reconnection attempts reached')
        updateConnectionStatus()
    }
}
```

**Step 5: Update return object (around line 763)**

Add to return:
```typescript
return {
    connected,
    connectionStatus,        // ADD
    disconnectedAt,          // ADD
    reconnectAttempt,        // ADD
    nextRetryIn,             // ADD
    connectionError,         // ADD
    connect,
    disconnect,
    // ... rest
}
```

**Step 6: Clean up on disconnect()**

Add to `disconnect()` function:
```typescript
function disconnect() {
    clearReconnectTimer()
    stopCountdown()  // ADD
    if (ws.value) {
        ws.value.close()
        ws.value = null
    }
    connected.value = false
    reconnectAttempts.value = 0
    reconnectAttempt.value = 0  // ADD
    disconnectedAt.value = null  // ADD
    updateConnectionStatus()  // ADD
    subscriptions.value.clear()
}
```

**Step 7: Test the changes**

Run: `cd frontend && npm run build`  
Expected: No TypeScript errors

**Step 8: Commit**

```bash
git add frontend/src/composables/useWebSocket.ts
git commit -m "feat(websocket): add connection state tracking with timestamps and countdown"
```

---

## Task 2: Create ConnectionStatusBar Component

**Files:**
- Create: `frontend/src/components/ui/ConnectionStatusBar.vue`

**Step 1: Create component file**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { WifiOff, Loader2, AlertCircle } from 'lucide-vue-next'

const props = defineProps<{
  status: 'reconnecting' | 'disconnected' | 'failed'
  nextRetryIn: number
}>()

const emit = defineEmits<{
  retry: []
}>()

const config = computed(() => {
  switch (props.status) {
    case 'reconnecting':
      return {
        bg: 'bg-amber-500/95',
        icon: Loader2,
        iconClass: 'animate-spin',
        message: 'Reconnecting...',
        showRetry: false
      }
    case 'disconnected':
      return {
        bg: 'bg-orange-500/95',
        icon: AlertCircle,
        iconClass: '',
        message: `Connection lost. Retrying in ${props.nextRetryIn}s...`,
        showRetry: true
      }
    case 'failed':
      return {
        bg: 'bg-red-500/95',
        icon: WifiOff,
        iconClass: '',
        message: 'Connection failed. Please reconnect.',
        showRetry: true
      }
    default:
      return null
  }
})

const cfg = computed(() => config.value)
</script>

<template>
  <Transition name="slide-down">
    <div
      v-if="cfg"
      :class="cfg.bg"
      class="fixed top-0 left-0 right-0 z-[100] flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium text-white shadow-lg"
    >
      <component :is="cfg.icon" class="h-4 w-4" :class="cfg.iconClass" />
      <span>{{ cfg.message }}</span>
      <button
        v-if="cfg.showRetry"
        @click="emit('retry')"
        class="ml-2 rounded bg-white/20 px-2 py-0.5 text-xs font-semibold hover:bg-white/30 transition-colors"
      >
        Retry now
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.slide-down-enter-active,
.slide-down-leave-active {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.slide-down-enter-from,
.slide-down-leave-to {
  transform: translateY(-100%);
  opacity: 0;
}
</style>
```

**Step 2: Test component compiles**

Run: `cd frontend && npm run build`  
Expected: No errors

**Step 3: Commit**

```bash
git add frontend/src/components/ui/ConnectionStatusBar.vue
git commit -m "feat(ui): add ConnectionStatusBar component for disconnect states"
```

---

## Task 3: Create ConnectionLostModal Component

**Files:**
- Create: `frontend/src/components/ui/ConnectionLostModal.vue`

**Step 1: Create component file**

```vue
<script setup lang="ts">
import { WifiOff, RefreshCw, RotateCcw } from 'lucide-vue-next'

const emit = defineEmits<{
  reconnect: []
  refresh: []
}>()
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div class="fixed inset-0 z-[110]">
        <!-- Backdrop with blur -->
        <div 
          class="absolute inset-0 bg-black/40 backdrop-blur-md"
          aria-hidden="true"
        />
        
        <!-- Modal -->
        <div class="relative flex h-full items-center justify-center p-4">
          <div 
            class="w-full max-w-md rounded-xl border border-border-1 bg-bg-surface-1 p-8 text-center shadow-2xl"
            role="dialog"
            aria-modal="true"
            aria-labelledby="disconnect-title"
          >
            <!-- Icon -->
            <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-500/10">
              <WifiOff class="h-8 w-8 text-red-500" />
            </div>
            
            <!-- Title -->
            <h2 
              id="disconnect-title"
              class="mt-4 text-lg font-semibold text-text-1"
            >
              Disconnected
            </h2>
            
            <!-- Message -->
            <p class="mt-2 text-sm text-text-2">
              You've been disconnected from the server. 
              Your conversation may be out of date.
            </p>
            
            <!-- Actions -->
            <div class="mt-6 flex flex-col gap-3">
              <button
                @click="emit('reconnect')"
                class="inline-flex items-center justify-center gap-2 rounded-lg bg-brand px-4 py-2.5 text-sm font-medium text-brand-foreground transition-standard hover:bg-brand-hover"
              >
                <RotateCcw class="h-4 w-4" />
                Reconnect
              </button>
              
              <button
                @click="emit('refresh')"
                class="inline-flex items-center justify-center gap-2 rounded-lg bg-bg-surface-2 px-4 py-2.5 text-sm font-medium text-text-2 transition-standard hover:bg-bg-surface-3 hover:text-text-1"
              >
                <RefreshCw class="h-4 w-4" />
                Refresh page
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
```

**Step 2: Test component compiles**

Run: `cd frontend && npm run build`  
Expected: No errors

**Step 3: Commit**

```bash
git add frontend/src/components/ui/ConnectionLostModal.vue
git commit -m "feat(ui): add ConnectionLostModal for critical disconnect state"
```

---

## Task 4: Update MessageComposer to Disable on Disconnect

**Files:**
- Modify: `frontend/src/components/composer/MessageComposer.vue:1-50` (add imports and props)
- Modify: `frontend/src/components/composer/MessageComposer.vue:85-100` (canSend computed)
- Modify: `frontend/src/components/composer/MessageComposer.vue:990-1010` (send button template)

**Step 1: Add connection status import and prop**

Add to imports (around line 1-30):
```typescript
import { useWebSocket } from '../../composables/useWebSocket'

// Inside setup:
const { connectionStatus } = useWebSocket()

const isConnected = computed(() => connectionStatus.value === 'connected')
```

**Step 2: Update canSend computed to check connection (around line 89)**

```typescript
const canSend = computed(() => {
  if (!isConnected.value) return false
  const hasContent = content.value.trim().length > 0
  const hasUploadedFiles = attachedFiles.value.some((file) => file.uploaded)
  const hasUploadInProgress = attachedFiles.value.some((file) => file.uploading)
  return (hasContent || hasUploadedFiles) && !hasUploadInProgress
})
```

**Step 3: Add disabled reason computed**

```typescript
const sendDisabledReason = computed(() => {
  if (!isConnected.value) return 'Reconnecting...'
  if (attachedFiles.value.some(f => f.uploading)) return 'Uploading...'
  if (!content.value.trim() && !attachedFiles.value.some(f => f.uploaded)) return 'Type a message'
  return ''
})
```

**Step 4: Update send button template (around line 991)**

```vue
<button
  class="flex h-9 min-w-9 items-center justify-center gap-1.5 rounded-r-1 bg-brand px-3 text-brand-foreground shadow-1 transition-standard hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-50 sm:px-4"
  :disabled="!canSend"
  :title="sendDisabledReason"
  aria-label="Send message"
  @click="handleSend"
>
  <Send class="h-4 w-4" />
  <span class="hidden sm:inline text-xs font-medium">Send</span>
</button>
```

**Step 5: Test build**

Run: `cd frontend && npm run build`  
Expected: No errors

**Step 6: Commit**

```bash
git add frontend/src/components/composer/MessageComposer.vue
git commit -m "feat(composer): disable sending when disconnected with tooltip reason"
```

---

## Task 5: Integrate Components into App.vue

**Files:**
- Modify: `frontend/src/App.vue:1-25` (add imports)
- Modify: `frontend/src/App.vue:50-78` (add components to template)

**Step 1: Add imports**

```typescript
import ConnectionStatusBar from './components/ui/ConnectionStatusBar.vue'
import ConnectionLostModal from './components/ui/ConnectionLostModal.vue'

// Add to useWebSocket destructuring:
const { 
  connect, 
  disconnect, 
  connectionStatus,      // ADD
  nextRetryIn,           // ADD
  reconnectAttempt       // ADD
} = useWebSocket()
```

**Step 2: Add handler functions**

```typescript
// Connection action handlers
function handleRetryConnection() {
  // Force immediate reconnect attempt
  disconnect()
  setTimeout(() => connect(), 100)
}

function handleRefreshPage() {
  window.location.reload()
}
```

**Step 3: Update template**

Add after `<ToastManager />`:
```vue
<template>
  <ToastManager ref="toastManagerRef" />
  
  <!-- Connection status banner (States 1 & 2) -->
  <ConnectionStatusBar
    v-if="connectionStatus !== 'connected' && connectionStatus !== 'failed'"
    :status="connectionStatus"
    :next-retry-in="nextRetryIn"
    @retry="handleRetryConnection"
  />
  
  <!-- Connection lost modal (State 3) -->
  <ConnectionLostModal
    v-if="connectionStatus === 'failed'"
    @reconnect="handleRetryConnection"
    @refresh="handleRefreshPage"
  />
  
  <CommandPalette v-if="isActiveTab" />
  <!-- ... rest of template ... -->
</template>
```

**Step 4: Add content dimming when disconnected**

Wrap router-view with dimming container:
```vue
<!-- Main content with dimming when disconnected -->
<div 
  :class="{ 
    'opacity-60': connectionStatus === 'disconnected',
    'opacity-80': connectionStatus === 'reconnecting'
  }"
  class="transition-opacity duration-500"
>
  <router-view />
</div>
```

**Step 5: Test build**

Run: `cd frontend && npm run build`  
Expected: No errors

**Step 6: Commit**

```bash
git add frontend/src/App.vue
git commit -m "feat(app): integrate connection status components with global overlay"
```

---

## Task 6: Add Header Connection Indicator (Optional Enhancement)

**Files:**
- Modify: `frontend/src/components/layout/GlobalHeader.vue` (add indicator)

**Step 1: Add connection status to header**

Add import:
```typescript
import { useWebSocket } from '../../composables/useWebSocket'

// In setup:
const { connectionStatus } = useWebSocket()

const connectionDotClass = computed(() => {
  switch (connectionStatus.value) {
    case 'connected': return 'bg-green-500'
    case 'reconnecting': return 'bg-amber-500 animate-pulse'
    case 'disconnected': return 'bg-orange-500 animate-pulse'
    case 'failed': return 'bg-red-500'
    default: return 'bg-gray-400'
  }
})
```

**Step 2: Add to template**

Add next to user avatar or team name:
```vue
<div class="flex items-center gap-2">
  <!-- Connection indicator -->
  <div 
    :class="connectionDotClass"
    class="h-2 w-2 rounded-full"
    :title="`Connection: ${connectionStatus}`"
  />
  <!-- ... existing header content ... -->
</div>
```

**Step 3: Commit**

```bash
git add frontend/src/components/layout/GlobalHeader.vue
git commit -m "feat(header): add connection status indicator dot"
```

---

## Task 7: Manual Testing Checklist

**Test scenarios:**

1. **Brief disconnect:**
   - Turn off WiFi for 2 seconds
   - Expect: Yellow banner appears, content dims to 80%
   - Turn WiFi back on
   - Expect: Banner disappears, content restores

2. **Extended disconnect:**
   - Turn off WiFi for 10 seconds
   - Expect: Orange banner with countdown
   - Click "Retry now"
   - Expect: Immediate reconnect attempt

3. **Critical disconnect:**
   - Turn off WiFi for 35 seconds
   - Expect: Full-screen modal appears
   - Click "Reconnect"
   - Expect: Attempts reconnection

4. **Composer behavior:**
   - Disconnect
   - Expect: Send button disabled with "Reconnecting..." tooltip
   - Type message
   - Reconnect
   - Expect: Send button enables, message can be sent

5. **Page refresh:**
   - Disconnect for 35+ seconds
   - Click "Refresh page"
   - Expect: Page reloads

---

## Summary of Files Changed

| File | Change |
|------|--------|
| `frontend/src/composables/useWebSocket.ts` | Add state tracking |
| `frontend/src/components/ui/ConnectionStatusBar.vue` | New component |
| `frontend/src/components/ui/ConnectionLostModal.vue` | New component |
| `frontend/src/components/composer/MessageComposer.vue` | Disable on disconnect |
| `frontend/src/App.vue` | Integrate components |
| `frontend/src/components/layout/GlobalHeader.vue` | Status indicator (optional) |

---

## Post-Implementation Notes

- Consider adding Sentry logging for disconnect events
- Monitor reconnection success rates
- Consider queueing messages typed during brief disconnections
