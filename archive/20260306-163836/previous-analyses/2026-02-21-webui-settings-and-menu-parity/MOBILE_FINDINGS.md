# Mobile Findings

## Scope note

- Mobile evidence source used in this pass: `../rustchat-mobile` (clone/rebrand of Mattermost Mobile in this workspace).

## Evidence by screenshot area

- Screen, store, or service: Screenshot 1 account menu (status + custom status)
- Source path: `../rustchat-mobile/app/screens/home/account/components/options/index.tsx`, `../rustchat-mobile/app/screens/home/account/components/options/user_presence/index.tsx`, `../rustchat-mobile/app/screens/home/account/components/options/custom_status/index.tsx`
- Source lines: `50-73`, `87-156`, `77-83`
- Observed behavior: Mobile account options include status chooser and custom status, but status is a separate bottom sheet with Online/Away/Do Not Disturb/Offline only; DND duration submenu from web menu is not present.
- Notes: Mobile text uses `Set a custom status` and opens screen/modal `Screens.CUSTOM_STATUS`.

- Screen, store, or service: Screenshot 2/7 display settings and theme editor
- Source path: `../rustchat-mobile/app/screens/settings/settings.tsx`, `../rustchat-mobile/app/screens/settings/display/display.tsx`, `../rustchat-mobile/app/screens/settings/display_theme/display_theme.tsx`
- Source lines: `63-113`, `97-127`, `34-45`
- Observed behavior: Mobile settings root has `Notifications`, `Display`, `Advanced Settings` (no `Sidebar` tab); display page contains Theme/Clock/Timezone/CRT rows; theme selection saves immediately on tap.
- Notes: Mobile does not use web modal-style row expansion with Save/Cancel for theme.

- Screen, store, or service: Screenshot 3 channel context menu
- Source path: `../rustchat-mobile/app/screens/channel/header/quick_actions/index.tsx`, `../rustchat-mobile/app/components/channel_actions/channel_actions.tsx`, `../rustchat-mobile/app/components/channel_actions/favorite_box/favorite_box.tsx`, `../rustchat-mobile/app/components/channel_actions/mute_box/mute_box.tsx`, `../rustchat-mobile/app/components/channel_actions/leave_channel_label/leave_channel_label.tsx`
- Source lines: `57-96`, `61-109`, `26-43`, `26-43`, `117-168`
- Observed behavior: Mobile quick actions include Favorite, Mute, Copy Link, Add Members, Leave/Close channel; no `Move to...` submenu item; no channel-level mark-as-read item in this action menu.
- Notes: Mark-as-unread exists for posts/threads, not as a channel menu entry (`../rustchat-mobile/app/screens/post_options/options/mark_unread_option/mark_unread_option.tsx:35-42`, `../rustchat-mobile/app/screens/thread_options/options/mark_as_unread_option.tsx:35-41`).

- Screen, store, or service: Screenshot 4 calls settings (plugin preferences)
- Source path: `../rustchat-mobile/app/screens/settings/notifications/notifications.tsx`, `../rustchat-mobile/app/screens/settings/notification_call/notification_call.tsx`
- Source lines: `179-188`, `113-185`
- Observed behavior: Mobile has call notification settings (sound toggle and ringtone selection), but no plugin-preferences section with `Audio devices` / `Video devices` edit rows.
- Notes: Search for `Audio devices` and `Video devices` labels returned no mobile hits.

- Screen, store, or service: Screenshot 5 advanced settings rows
- Source path: `../rustchat-mobile/app/screens/settings/advanced/advanced.tsx`, `../rustchat-mobile/app/screens/channel/channel_post_list/index.ts`
- Source lines: `91-121`, `41-45`
- Observed behavior: Mobile advanced screen is device-focused (`Delete local files`, optional `Component library`) and does not expose web advanced rows. However, mobile still reads advanced preference `join_leave` to filter join/leave posts.
- Notes: Backend still must support advanced preference categories/names even if mobile UI differs.

- Screen, store, or service: Screenshot 6 sidebar settings
- Source path: `../rustchat-mobile/app/constants/preferences.ts`, `../rustchat-mobile/app/screens/home/channel_list/categories_list/categories/helpers/observe_flattened_categories.ts`
- Source lines: `56-59`, `73-80`, `210-214`
- Observed behavior: Mobile does not expose a `Sidebar Settings` screen, but it consumes sidebar preferences `limit_visible_dms_gms` and `show_unread_section` when building channel list.
- Notes: Web changes to sidebar preference keys directly affect mobile channel list behavior.

- Screen, store, or service: Screenshot 8 notifications
- Source path: `../rustchat-mobile/app/screens/settings/notifications/notifications.tsx`, `../rustchat-mobile/app/screens/settings/notifications/notifications_disabled_notice/index.tsx`, `../rustchat-mobile/app/screens/settings/notifications/send_test_notification_notice/send_test_notification_notice.tsx`
- Source lines: `159-208`, `46-55`, `124-136`
- Observed behavior: Mobile notifications screen includes Mentions/Push/Call/Email/Automatic replies and troubleshooting notice (`Send a test notification`, `Troubleshooting docs`), plus a disabled-notifications danger notice.
- Notes: Mobile uses a notice card rather than web `Permission required` title tag.

- Screen, store, or service: Screenshot 9 composer formatting toggle
- Source path: `../rustchat-mobile/app/components/post_draft/quick_actions/quick_actions.tsx`
- Source lines: `95-133`
- Observed behavior: Mobile composer quick actions are attachment, mentions/slash, emoji, priority, burn-on-read; no web-style `Aa` formatting toggle or show/hide formatting toolbar affordance.
- Notes: This screenshot maps to web-only UX; mobile has a different composer control model.

- Screen, store, or service: Screenshots 10/11 desktop shell and left sidebar context
- Source path: `../rustchat-mobile/app/screens/home/channel_list/categories_list/categories/helpers/observe_flattened_categories.ts`, `../rustchat-mobile/app/screens/home/channel_list/categories_list/categories/categories.tsx`
- Source lines: `210-214`, `107-140`
- Observed behavior: Mobile channel list composition is driven by category/state observers and rendered as native list rows; desktop-like web shell geometry and right-click context interactions are not used.
- Notes: Functional contract overlap is via category/preferences data, not desktop UI composition.

## API and websocket contracts consumed by mobile

- Screen, store, or service: Preferences endpoints and payload shape
- Source path: `../rustchat-mobile/app/client/rest/base.ts`, `../rustchat-mobile/app/client/rest/preferences.ts`, `../rustchat-mobile/app/actions/remote/preference.ts`
- Source lines: `142-144`, `13-31`, `97-101`
- Observed behavior: Mobile uses `/users/{user_id}/preferences` and `/users/{user_id}/preferences/delete`; save chunks in groups of 100 preference objects.
- Notes: Server must accept array payloads and preference categories used by mobile (`display_settings`, `sidebar_settings`, `advanced_settings`, etc.).

- Screen, store, or service: Status and custom status endpoints
- Source path: `../rustchat-mobile/app/client/rest/users.ts`, `../rustchat-mobile/app/actions/remote/user.ts`
- Source lines: `420-445`, `706-751`
- Observed behavior: Mobile uses `PUT /users/{id}/status`, `PUT /users/me/status/custom`, `DELETE /users/me/status/custom`, and `POST /users/me/status/custom/recent/delete`.
- Notes: Custom status endpoint parity is required for account menu and custom status flows.

- Screen, store, or service: Channel mute notify props contract
- Source path: `../rustchat-mobile/app/client/rest/channels.ts`, `../rustchat-mobile/app/actions/remote/channel.ts`
- Source lines: `155-159`, `1319-1328`
- Observed behavior: Mobile mutes channels by toggling `mark_unread` via `PUT /channels/{channel_id}/members/{user_id}/notify_props`.
- Notes: Notify props endpoint semantics are shared between web and mobile.

- Screen, store, or service: Set unread endpoint contract
- Source path: `../rustchat-mobile/app/client/rest/posts.ts`, `../rustchat-mobile/app/actions/remote/post.ts`
- Source lines: `149-156`, `900-903`
- Observed behavior: Mobile sends `POST /users/{user_id}/posts/{post_id}/set_unread` with body `{\"collapsed_threads_supported\": true}`.
- Notes: Server must parse `collapsed_threads_supported` for mobile compatibility.

- Screen, store, or service: Team-scoped channel categories contract
- Source path: `../rustchat-mobile/app/client/rest/base.ts`, `../rustchat-mobile/app/client/rest/categories.ts`, `../rustchat-mobile/app/actions/remote/category.ts`
- Source lines: `78-88`, `14-37`, `88`
- Observed behavior: Mobile favorites/moves are category operations using `/users/{user_id}/teams/{team_id}/channels/categories`.
- Notes: Category routes and update semantics are critical for favorite/move parity.

- Screen, store, or service: Websocket events for status/preferences/categories
- Source path: `../rustchat-mobile/app/constants/websocket.ts`, `../rustchat-mobile/app/actions/websocket/event.ts`, `../rustchat-mobile/app/actions/websocket/users.ts`, `../rustchat-mobile/app/actions/websocket/preferences.ts`, `../rustchat-mobile/app/actions/websocket/category.ts`
- Source lines: `13-16`, `42`, `89-146`, `125-128`, `13-81`, `31-102`
- Observed behavior: Mobile consumes `status_change`, preference change events, and sidebar category events; handlers update local DB/state based on those events.
- Notes: Websocket event names and payload fields must remain Mattermost-compatible.

## Constraints and implications

- Mobile is not a pixel clone of the web screenshots; it uses native stacked screens and quick actions.
- Despite UI differences, backend parity is mandatory for shared contracts: preferences categories/names, status/custom-status APIs, notify_props semantics, set_unread body/response, categories APIs, and websocket event compatibility.
