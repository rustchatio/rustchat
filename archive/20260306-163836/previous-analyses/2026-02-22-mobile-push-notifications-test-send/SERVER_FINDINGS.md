# SERVER_FINDINGS

- Endpoint or component: Mattermost `/system/ping` push capability verification
- Source path: `../mattermost/server/channels/api4/system.go:216`
- Source lines: 216-218
- Observed behavior: When `device_id` is present on ping, Mattermost includes `CanReceiveNotifications` in the response via `SendTestPushNotification(...)`.
- Notes: Mobile uses this for push proxy verification UX/state.

- Endpoint or component: Mattermost `/notifications/test`
- Source path: `../mattermost/server/channels/api4/system.go:231`
- Source lines: 231-238
- Observed behavior: Mattermost calls `SendTestMessage(...)` and returns an API error if it fails; success returns `StatusOK`.
- Notes: This differs from Rustchat's current behavior, which always returns HTTP 200 with embedded error info.

- Endpoint or component: Mattermost session/device props attachment
- Source path: `../mattermost/server/channels/api4/user.go:2492`
- Source lines: 2492-2532
- Observed behavior: Mattermost handles device attachment through `/users/sessions/device`, parsing `device_id`, `device_notification_disabled`, `mobile_version`, and attaching device ID to the current session.
- Notes: Rustchat's mobile integration correctly targets this endpoint path.

- Endpoint or component: Mattermost device attach implementation
- Source path: `../mattermost/server/channels/api4/user.go:2534`
- Source lines: 2534-2582
- Observed behavior: Mattermost persists/associates the device ID with the session (`AttachDeviceId`) and rotates session cookie metadata for mobile sessions.
- Notes: Rustchat is not using Mattermost internals but needs behaviorally compatible outcomes for mobile push flows.
