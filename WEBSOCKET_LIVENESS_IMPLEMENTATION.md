# Mattermost-Compatible WebSocket Liveness Implementation

## Overview

This implementation adds Mattermost-compatible WebSocket liveness and session resumption to RustChat, eliminating the 2-3 second reconnection loops experienced by mobile clients.

## Key Features

### 1. WebSocket Protocol-Level Ping/Pong
- **Ping Interval**: 60 seconds (`PING_INTERVAL`)
- **Pong Timeout**: 100 seconds (`PONG_WAIT`)
- **Write Deadline**: 30 seconds (`WRITE_WAIT`)
- Uses WebSocket control frames (opcode 0x09/0x0A), NOT JSON messages

### 2. Connection ID & Session Resumption
- Mobile clients reconnect with query params: `?connection_id=<uuid>&sequence_number=<i64>`
- In-memory `ConnectionStore` retains state for 5 minutes after disconnect
- On resume: replays all messages with `sequence_number > provided_seq_num`
- New connections receive `connection_id` in the `hello` event

### 3. Message Sequencing & Buffering
- Buffer size: 128 messages per connection (Mattermost default)
- FIFO eviction when buffer is full
- Monotonic sequence numbers for reliable delivery

### 4. TCP Keepalive for Mobile Networks
- Enabled TCP keepalive to prevent carrier drops
- Configured for 15-second intervals (under 30-60s carrier timeouts)

### 5. Graceful Shutdown
- Proper WebSocket close codes:
  - `1000`: Normal closure
  - `1001`: Going away (server restart)
  - `1008`: Policy violation (auth failure)
  - `1011`: Internal server error

## File Structure

```
backend/src/realtime/
├── mod.rs                    # Updated to export new modules
├── connection_store.rs       # Session management and resumption
├── websocket_actor.rs        # WebSocket I/O with ping/pong
├── hub.rs                    # Existing broadcast hub
└── events.rs                 # Existing event types

backend/src/api/v4/websocket.rs  # Updated to use new system
backend/src/api/mod.rs           # Updated AppState with ConnectionStore
```

## Constants

```rust
// From backend/src/realtime/websocket_actor.rs
const WRITE_WAIT: Duration = Duration::from_secs(30);
const PONG_WAIT: Duration = Duration::from_secs(100);
const PING_INTERVAL: Duration = Duration::from_secs(60);

// From backend/src/realtime/connection_store.rs
const CONNECTION_TTL: Duration = Duration::from_secs(300); // 5 minutes
const MESSAGE_BUFFER_SIZE: usize = 128;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);
```

## WebSocket Protocol

### Initial Handshake
Server sends `hello` event immediately on connection:
```json
{
  "event": "hello",
  "data": {
    "connection_id": "uuid-generated-by-server",
    "server_version": "rustchat-0.1.0",
    "protocol_version": "1.0"
  },
  "seq": 0
}
```

### Session Resumption
On reconnect with `?connection_id=abc&sequence_number=45`:
1. Server looks up `abc` in connection store
2. If found: Sends missed messages with `seq > 45`
3. If expired: Treats as new connection, sends `hello` with new ID

### Ping/Pong Frames
- Server sends: `WebSocket::Message::Ping([])` every 60s
- Client responds: `WebSocket::Message::Pong([])` (automatic or explicit)
- No JSON involved in heartbeat

## API Changes

### Query Parameters
The WebSocket endpoint accepts:
- `token`: Authentication token (or use Authorization header)
- `connection_id`: Existing connection ID for resumption
- `sequence_number`: Last sequence number received by client

Example:
```
ws://localhost:8080/api/v4/websocket?connection_id=550e8400-e29b-41d4-a716-446655440000&sequence_number=42
```

### Dependencies Added
- `dashmap = "6.1"`: Thread-safe concurrent hash map
- `libc = "0.2"`: For TCP keepalive socket options

## Backward Compatibility

- Web clients that don't send `connection_id` continue to work
- New sessions are created with generated `connection_id`
- Old WebSocket endpoint at `/ws` is preserved but deprecated

## Monitoring

The `ConnectionStore` provides statistics:
```rust
pub struct ConnectionStoreStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub inactive_connections: usize,
    pub unique_users: usize,
}
```

## Testing

### Manual Testing Checklist
1. **Flapping Elimination**: Mobile client maintains connection for >10 minutes
2. **Resume Functionality**: Disconnect WiFi for 30s, reconnect without "Loading..." screen
3. **Protocol Compliance**: Wireshark shows Ping frames every 60s
4. **Memory Safety**: ConnectionState purged after 5min TTL

### Expected Behavior
- Mobile app should show stable connection status
- No full data reloads on brief disconnections
- Smooth message delivery resumption

## Performance Considerations

- Connection state is held in memory (DashMap)
- 128 messages buffered per connection (~ few KB)
- 5-minute TTL prevents memory leaks
- Cleanup runs every 60 seconds

## Future Enhancements

- Persistent session storage (Redis) for multi-server deployments
- Configurable buffer sizes and TTL
- Metrics export for monitoring
- Rate limiting per connection

## References

- Mattermost `web_conn.go` constants: `writeWaitTime=30s`, `pongWaitTime=100s`, `pingInterval=60s`
- Mobile client reconnect logic: Exponential backoff starting at 1s
