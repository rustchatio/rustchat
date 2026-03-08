- Rustchat target path: `frontend/src/utils/idCompat.ts` (new)
- Required behavior: Provide shared Mattermost-ID decode and recursive payload normalization for `id`, `*_id`, `*_ids` fields.
- Current gap: Normalization is duplicated/partial in websocket-only path.
- Planned change: Added centralized helper functions and wired at client boundaries.
- Verification test: `npm --prefix frontend run build` passed after integration.
- Status: Completed

- Rustchat target path: `frontend/src/api/client.ts`
- Required behavior: Normalize incoming API payload IDs before stores consume them; normalize outgoing params/body IDs to internal UUID where possible.
- Current gap: No ID normalization in axios interceptors.
- Planned change: Added request/response normalization interceptors using shared helper.
- Verification test: `npm --prefix frontend run build` passed.
- Status: Completed

- Rustchat target path: `frontend/src/composables/useWebSocket.ts`
- Required behavior: Normalize IDs for all websocket events/payload shapes (not only posted path), including envelope channel_id and nested post payload.
- Current gap: Partial handling only for posted payload.
- Planned change: Switched to shared helper for envelope/data normalization; retained posted-specific parse normalization for wrapped `post` payload.
- Verification test: `npm --prefix frontend run build` passed.
- Status: Completed

- Rustchat target path: `frontend/src/composables/useWebSocket.ts`
- Required behavior: Use Mattermost-compatible websocket endpoint (`/api/v4/websocket`) by default for parity with mattermost-mobile realtime protocol path.
- Current gap: WebUI defaulted to `/api/v1/ws`.
- Planned change: Switched default websocket URL construction to `/api/v4/websocket?token=...`.
- Verification test: `npm --prefix frontend run build` passed.
- Status: Completed

- Rustchat target path: `frontend/src/composables/useWebSocket.ts`
- Required behavior: Parse Mattermost websocket reaction events where payload is wrapped as `data.reaction` JSON string/object.
- Current gap: WebUI expected flat `{post_id,user_id,emoji_name}` and ignored wrapped reaction payload.
- Planned change: Added `normalizeWsReactionPayload` to parse/unwrap, normalize IDs, and feed store handlers.
- Verification test: `npm --prefix frontend run build` passed.
- Status: Completed

Compatibility checklist snapshot:
- API Contract: Completed for ID-format compatibility layer at WebUI boundary
- Realtime Contract: Completed for websocket ID normalization path and endpoint parity (`/api/v4/websocket`)
- Data Semantics: Completed for ID key normalization (`id`, `*_id`, `*_ids`)
- Auth and Permissions: N/A for this patch
- Client Expectations: Completed for mixed-format ID tolerance
- Verification: Build evidence captured; manual smoke still recommended for full regression confidence
