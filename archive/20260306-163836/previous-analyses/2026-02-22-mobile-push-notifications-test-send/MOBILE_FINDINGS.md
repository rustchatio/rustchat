# MOBILE_FINDINGS

- Screen, store, or service: Test notification UI trigger and error handling
- Source path: `../rustchat-mobile/app/screens/settings/notifications/send_test_notification_notice/send_test_notification_notice.tsx:58`
- Source lines: 58-76
- Observed behavior: The UI marks the action as success (`sent`) whenever the request returns without a thrown transport error. It only shows error state when `result.error` exists (network/API exception path), not when the server returns `{status:"OK", error:"..."}` in the JSON body.
- Notes: This means a server-side logical failure can be presented as a successful "Test notification sent" button state.

- Screen, store, or service: REST client path for test notifications
- Source path: `../rustchat-mobile/app/client/rest/posts.ts:234`
- Source lines: 234-239
- Observed behavior: Mobile calls `POST /api/v4/notifications/test`.
- Notes: This matches Mattermost server API route expectations.

- Screen, store, or service: Remote action wrapper for test notifications
- Source path: `../rustchat-mobile/app/actions/remote/notifications.ts:274`
- Source lines: 274-282
- Observed behavior: Wrapper returns server JSON directly and only sets `{error}` on thrown exceptions.
- Notes: Coupled with the UI logic above, HTTP 200 with an embedded error field looks like success.

- Screen, store, or service: Push token registration format
- Source path: `../rustchat-mobile/app/init/push_notifications.ts:283`
- Source lines: 283-305
- Observed behavior: Mobile stores device token as `<platform-prefix>-v2:<native-token>` (e.g. `android_rn-v2:...`, `apple_rn-v2:...`).
- Notes: This matches Rustchat's `parse_mobile_device_id` expectations.

- Screen, store, or service: Session/device registration call
- Source path: `../rustchat-mobile/app/actions/remote/entry/common.ts:294`
- Source lines: 294-314
- Observed behavior: Mobile sends device metadata via `client.setExtraSessionProps(...)` after login, passing the stored device token (`device_id`) to the server.
- Notes: The call is not awaited (`client.setExtraSessionProps(...)` fire-and-forget), so failures may be easy to miss without server logs.

- Screen, store, or service: Device registration endpoint path used by mobile
- Source path: `../rustchat-mobile/app/client/rest/users.ts:384`
- Source lines: 384-397
- Observed behavior: Mobile uses `PUT /api/v4/users/sessions/device` with body fields `device_id`, `device_notification_disabled`, and `mobile_version`.
- Notes: Rustchat must support this exact endpoint/body to capture tokens for push delivery.

- Screen, store, or service: Ping push capability verification request
- Source path: `../rustchat-mobile/app/client/rest/general.ts:32`
- Source lines: 32-41
- Observed behavior: Mobile includes `device_id` on `/api/v4/system/ping` when verifying push capability.
- Notes: Mattermost server uses this to return `CanReceiveNotifications` in ping response.

- Screen, store, or service: Upstream mobile baseline availability
- Source path: `../mattermost-mobile`
- Source lines: N/A
- Observed behavior: The local `../mattermost-mobile` directory is not a usable checkout in this environment (only `.adopt-convert-release` exists).
- Notes: Baseline mobile behavior was inferred from `../rustchat-mobile` (clone) and Mattermost server sources.
