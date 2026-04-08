# Connection Status

RustChat keeps you informed about your connection status to ensure you always know when your messages are being sent and received in real-time.

## Connection States

### 🟢 Connected

When you see a **green dot** in the header, you're fully connected and all messages are being sent and received in real-time.

---

### 🟡 Reconnecting

If your connection is interrupted, you'll see a **yellow banner** at the top of the screen:

> **Reconnecting...**

**What this means:**
- Your connection was temporarily lost (usually due to network changes or brief interruptions)
- RustChat is automatically trying to reconnect
- You can still read existing messages

**What happens:**
- The screen content is slightly dimmed (80% opacity)
- The message composer is disabled
- Reconnection happens automatically within a few seconds

**No action needed** - just wait a moment and you'll be reconnected automatically.

---

### 🟠 Connection Lost

If reconnection fails for more than 5 seconds, the banner turns **orange**:

> **Connection lost. Retrying in X seconds...**

**What this means:**
- The automatic reconnection is taking longer than expected
- Your conversation may be out of date
- New messages from others won't appear until reconnected

**What you can do:**
- Wait for the automatic retry
- Click **"Retry now"** to force an immediate reconnection attempt
- Continue reading existing messages (content is dimmed to 60%)

---

### 🔌 Disconnected

If your connection can't be restored after 30 seconds, a **full-screen modal** appears:

> **Disconnected**  
> You've been disconnected from the server. Your conversation may be out of date.

**You have two options:**

1. **Reconnect** - Attempts to restore your connection and sync any missed messages
2. **Refresh page** - Reloads the entire application (use this if reconnect doesn't work)

**Why this happens:**
- Your device went to sleep for an extended period
- Network connection was lost (WiFi turned off, switching networks)
- The server was restarted
- Browser tab was in the background for a long time

---

## Common Questions

### Will I lose my messages if I get disconnected?

**No.** Messages you sent before disconnection are already delivered. Messages others sent while you were disconnected will appear once you reconnect.

### What happens to messages I type while disconnected?

The message composer is disabled during disconnection to prevent messages from failing to send. Once reconnected, you can type and send normally.

### Why does the content get dimmed?

The dimming (80% → 60% → blur) is a visual cue that the content might be stale. Messages may have been edited or deleted while you were disconnected.

### How do I know if I missed any messages?

When you reconnect:
1. Missed messages will automatically appear in the conversation
2. Unread message badges will update
3. The connection indicator will turn green again

### Can I prevent disconnections?

Most disconnections are caused by:
- **Network changes** (switching WiFi, moving between cell towers)
- **Device sleep** (laptop lid closed, phone screen off)
- **Browser throttling** (tab in background for extended time)

These are normal and RustChat handles them gracefully. Just keep an eye on the connection indicator!

---

## Troubleshooting

### Stuck on "Reconnecting..." for a long time

1. Check your internet connection
2. Try clicking **"Retry now"** 
3. If still stuck, click **"Refresh page"** in the disconnect modal

### Keep getting disconnected frequently

1. Check if your WiFi is stable
2. Disable VPN temporarily to test
3. Check if browser extensions are interfering
4. Try a different browser

### Reconnect doesn't work but refresh does

This usually means your authentication session expired. Refreshing the page re-authenticates you. This is normal after very long periods of inactivity.

---

## Privacy Note

The connection status indicator only reflects your local network connection to the RustChat server. It doesn't track or report your online status to other users.
