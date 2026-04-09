# WebSocket Disconnection UX Design

**Date:** 2026-04-08  
**Status:** Approved for Implementation  
**Related:** WebSocket connection management, real-time sync

---

## Problem Statement

When users leave the RustChat window open for extended periods, the WebSocket connection can be lost. Currently, the UI continues to display potentially stale content without clear indication, leading to:
- Users acting on outdated information (deleted/edited messages)
- Confusion about why new messages aren't appearing
- Sending messages that fail silently

## Design Goals

1. **Transparency**: User always knows the connection state
2. **Progressive urgency**: Visual severity increases with disconnection time
3. **Clear actions**: User knows exactly what to do
4. **Safety**: Prevent actions on potentially stale data

---

## UX States

### State 1: Brief Disconnection (< 5 seconds)

**Trigger:** WebSocket closes, auto-reconnect pending

**Visual Design:**
- Yellow/amber banner at top of viewport
- Message: "Reconnecting..."
- Content opacity: 80%
- Header indicator: 🟡 Yellow dot

**Behavior:**
- Auto-retry with exponential backoff
- Composer disabled with tooltip: "Reconnecting..."
- All content visible but dimmed

**Technical:**
- Timer starts on `onclose` event
- Reconnect attempt every 1-10s (exponential backoff)
- If reconnected within 5s → return to normal

---

### State 2: Extended Disconnection (5-30 seconds)

**Trigger:** Reconnection attempts continue to fail

**Visual Design:**
- Orange banner at top
- Message: "Connection lost. Retrying in X seconds..."
- Countdown timer showing next attempt
- Content opacity: 60%
- Header indicator: 🟠 Orange dot with pulse animation

**Behavior:**
- Composer completely disabled
- "Retry now" button in banner (immediate reconnect attempt)
- Content still visible for reference but clearly marked as stale

**Technical:**
- Backoff capped at 10 seconds between attempts
- Max 10 reconnection attempts before critical state
- User can force immediate retry

---

### State 3: Critical Disconnection (> 30 seconds)

**Trigger:** Maximum reconnection attempts exhausted or 30s elapsed

**Visual Design:**
- Full-screen overlay with backdrop blur
- Centered modal card
- Title: "🔌 Disconnected"
- Message: "You've been disconnected from the server. Your conversation may be out of date."
- Background: Previous content visible but heavily blurred (backdrop-blur-md)

**Actions:**
- Primary: "Reconnect" (attempts WebSocket reconnection)
- Secondary: "Refresh page" (full page reload)

**Behavior:**
- All interactive elements disabled
- User must take explicit action to continue
- On reconnect: sync missed messages automatically

---

## State Transitions

```
┌─────────────┐     onClose      ┌─────────────────┐
│  CONNECTED  │ ───────────────→ │  RECONNECTING   │
│   (green)   │                  │    (< 5s)       │
└─────────────┘                  │   (yellow)      │
     ↑                            └─────────────────┘
     │ onOpen                           │
     │                                  │ timeout 5s
     │                                  ↓
     │                            ┌─────────────────┐
     │                            │  DISCONNECTED   │
     │                            │   (5-30s)       │
     │                            │   (orange)      │
     │                            └─────────────────┘
     │                                  │
     │                                  │ timeout 30s
     │                                  │ or max retries
     │                                  ↓
     │                            ┌─────────────────┐
     └─────────────────────────── │     FAILED      │
         onOpen (successful)      │   (> 30s)       │
                                  │    (red)        │
                                  └─────────────────┘
```

---

## Component Architecture

### New Components

1. **ConnectionStatusBar** - Banner for States 1 & 2
   - Position: Fixed top
   - Props: status, countdown, onRetry
   
2. **ConnectionLostModal** - Full-screen overlay for State 3
   - Position: Fixed inset-0, z-50
   - Props: onReconnect, onRefresh
   - Reuses existing modal styling patterns

3. **ConnectionIndicator** - Header dot indicator
   - Position: GlobalHeader or ChannelSidebar
   - Props: status

### Modified Components

1. **MessageComposer** - Disable on disconnect
   - Watch connection status
   - Add disabled state with tooltip
   
2. **useWebSocket composable** - State management
   - Add connectionStatus ref
   - Add disconnectedAt timestamp
   - Track reconnection countdown
   
3. **App.vue** - Global overlay integration
   - Add ConnectionLostModal at root level
   - Pass connection state from useWebSocket

---

## Technical Specifications

### State Constants
```typescript
const STATE_THRESHOLDS = {
  RECONNECTING: 0,      // Immediate on disconnect
  DISCONNECTED: 5000,   // 5 seconds
  FAILED: 30000         // 30 seconds
} as const

const RETRY_CONFIG = {
  MAX_ATTEMPTS: 10,
  BASE_DELAY: 1000,
  MAX_DELAY: 10000,
  BACKOFF_MULTIPLIER: 1.5
} as const
```

### Reactive State
```typescript
interface ConnectionState {
  status: 'connected' | 'reconnecting' | 'disconnected' | 'failed'
  disconnectedAt: Date | null
  reconnectAttempt: number
  nextRetryIn: number  // seconds countdown
  lastError: string | null
}
```

### CSS Classes
```css
/* State 1: Brief */
.connection-banner--reconnecting {
  @apply bg-amber-500/90 text-white;
}

/* State 2: Extended */
.connection-banner--disconnected {
  @apply bg-orange-500/90 text-white;
}
.content--stale {
  @apply opacity-60 transition-opacity duration-500;
}

/* State 3: Critical */
.connection-overlay {
  @apply fixed inset-0 z-50 bg-black/40 backdrop-blur-md;
}
.connection-modal {
  @apply absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 
         bg-bg-surface-1 rounded-xl shadow-2xl border border-border-1
         p-8 max-w-md w-full text-center;
}
```

---

## Sync on Reconnect

When connection is restored:

1. **Immediate:** Restore full opacity, remove banners/modals
2. **Fetch missed messages:** 
   - Use `after` cursor with last known message seq
   - Or fetch recent messages with `?since=` timestamp
3. **Update unreads:** Call `unreadStore.fetchOverview()`
4. **Toast notification:** "Reconnected" (brief, auto-dismiss)

---

## Accessibility

- **Screen readers:** Announce state changes with `aria-live="polite"`
- **Focus management:** Trap focus in modal when in FAILED state
- **Keyboard:** Allow Escape to dismiss banner (but not modal)
- **Reduced motion:** Respect `prefers-reduced-motion`

---

## Edge Cases

1. **Tab inactive:** Use existing `useSingleActiveTab` logic, don't show disconnect UI if tab is just inactive
2. **Network offline:** Detect with `navigator.onLine`, show different message
3. **Auth expiry:** Existing logic handles token expiry, don't show disconnect UI
4. **Multiple disconnects:** Reset timers on each disconnect event

---

## Success Metrics

- User knows connection state within 1 second of disconnect
- No messages sent while disconnected (0% error rate)
- User can read old content during brief disconnections
- 100% of users take explicit action after 30s (no "stuck" states)

---

## Approved Implementation Details

**Timing Thresholds:** ✅ 5s (extended), 30s (critical)  
**Composer Behavior:** ✅ Disable immediately on disconnect  
**Background Content:** ✅ Blur in critical state (State 3)  

---

## References

- Similar patterns in Slack, Discord, Linear
- Reuses existing App.vue inactive tab overlay pattern
- Follows established design system (colors, spacing, typography)
