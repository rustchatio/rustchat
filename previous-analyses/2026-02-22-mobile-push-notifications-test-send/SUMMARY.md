# SUMMARY

- Topic: Mobile push notifications and in-app test notification send failure
- Date: 2026-02-22
- Scope: Push notifications (mobile client + Rustchat backend integration), server API behavior, push test flow
- Compatibility contract:
  - Mobile sends test notification via `POST /api/v4/notifications/test` and expects server failures to surface as API errors (Mattermost behavior).
  - Mobile registers push device metadata via `PUT /api/v4/users/sessions/device` with `device_id`, `device_notification_disabled`, `mobile_version`.
  - Mobile pings `/api/v4/system/ping?device_id=...` and Mattermost returns `CanReceiveNotifications` for push capability verification.
  - Backend must have an actual push delivery path configured (push-proxy with FCM/APNS and/or direct FCM fallback) to deliver test and real notifications.
- Open questions:
  - What platform/device are you testing on (Android physical device, iPhone physical device, simulator/emulator)?
  - Are there backend/push-proxy runtime logs showing `No devices found`, `FCM not configured`, `APNS not configured`, or token errors during the test send?

## Observed Root Causes (Likely, ordered)

1. Local push delivery pipeline is not configured.
   - Evidence: `.env` only sets `RUSTCHAT_PUSH_PROXY_URL` and does not define Firebase/APNS credentials.
   - `docker-compose.yml` passes empty Firebase/APNS env vars by default when not supplied.
   - `push-proxy` disables FCM/APNS clients when env vars are missing.
   - Rustchat backend direct FCM fallback also returns `NotConfigured` when FCM config is absent.

2. Rustchat hides test notification failures from the mobile client.
   - Rustchat `/notifications/test` returns HTTP 200 + `{status:"OK", error:"..."}` even when sending fails.
   - `../rustchat-mobile` test UI only treats thrown/request errors as failure, so it can show "Test notification sent" while no push was delivered.

3. Rustchat misses Mattermost ping push-verification compatibility (`CanReceiveNotifications`).
   - This does not directly prevent delivery, but it weakens mobile-side diagnostics and push capability verification behavior.

## Why this explains your symptoms

- Normal push notifications fail because there is no configured FCM/APNS path in the current environment.
- The mobile “Send a test notification” action appears non-functional because the backend swallows the actual send error and returns HTTP 200, so the app cannot distinguish success from a backend-side push failure.
