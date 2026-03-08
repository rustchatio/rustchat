# Server Findings

## Evidence

### Screenshot 1: User account status menu
- Endpoint or component: User account menu composition and order
- Source path: `../mattermost/webapp/channels/src/components/user_account_menu/user_account_menu.tsx`
- Source lines: `79-133`
- Observed behavior: Menu renders user header, optional custom-status row, then Online/Away/DND/Offline, then Profile, then Log out.
- Notes: This order matches the screenshot and is not a free-form list.

- Endpoint or component: Set custom status label
- Source path: `../mattermost/webapp/channels/src/components/user_account_menu/user_account_set_custom_status_menuitem.tsx`
- Source lines: `26-29`
- Observed behavior: Label is exactly `Set custom status`.
- Notes: Rustchat currently shows `Set a custom status`, which is text mismatch.

- Endpoint or component: DND item semantics
- Source path: `../mattermost/webapp/channels/src/components/user_account_menu/user_account_dnd_menuitem.tsx`
- Source lines: `131-136`, `177-210`
- Observed behavior: DND row includes secondary text (`Disables all notifications`) and opens a submenu with durations.
- Notes: This is richer than a one-click status toggle.

- Endpoint or component: Profile and logout labels
- Source path: `../mattermost/webapp/channels/src/components/user_account_menu/user_account_profile_menuitem.tsx`, `../mattermost/webapp/channels/src/components/user_account_menu/user_account_logout_menuitem.tsx`
- Source lines: `70-73`, `27-30`
- Observed behavior: Labels are `Profile` and `Log out`.
- Notes: Wording and placement are contract-significant for parity.

### Screenshot 2/7: Settings modal and Display settings
- Endpoint or component: Core user settings tabs
- Source path: `../mattermost/webapp/channels/src/components/user_settings/modal/user_settings_modal.tsx`
- Source lines: `260-287`
- Observed behavior: Tabs are exactly `Notifications`, `Display`, `Sidebar`, `Advanced`.
- Notes: These are the primary tab ids that drive modal content.

- Endpoint or component: Plugin preferences sidebar section
- Source path: `../mattermost/webapp/channels/src/components/settings_sidebar/settings_sidebar.tsx`
- Source lines: `159-183`
- Observed behavior: Plugin tabs render under a separate header titled `PLUGIN PREFERENCES`.
- Notes: Calls appears under this section in screenshots.

- Endpoint or component: Display settings section structure
- Source path: `../mattermost/webapp/channels/src/components/user_settings/display/user_settings_display.tsx`
- Source lines: `1187-1217`
- Observed behavior: Display tab renders ordered settings rows including Theme, thread options, clock/timezone, previews, message/channel display, reactions, emoticons, language.
- Notes: Uses editable collapsed rows, not one large form.

- Endpoint or component: Theme expanded editor behavior
- Source path: `../mattermost/webapp/channels/src/components/user_settings/display/user_settings_theme/user_settings_theme.tsx`
- Source lines: `185-239`, `263-291`, `295-307`
- Observed behavior: Theme row has collapsed summary, expanded premade/custom mode radios, premade chooser, `See other themes` link, and save/cancel via `SettingItemMax`.
- Notes: Screenshot with premade tiles and save/cancel maps directly.

### Screenshot 6: Sidebar settings
- Endpoint or component: Sidebar settings panel
- Source path: `../mattermost/webapp/channels/src/components/user_settings/sidebar/user_settings_sidebar.tsx`
- Source lines: `55-73`
- Observed behavior: Sidebar settings screen contains two configurable rows.
- Notes: No extra preferences in this screen.

- Endpoint or component: Group unread channels row
- Source path: `../mattermost/webapp/channels/src/components/user_settings/sidebar/show_unreads_category/show_unreads_category.tsx`
- Source lines: `125-127`
- Observed behavior: Row title is `Group unread channels separately`.
- Notes: Collapsed description shows `On`/`Off`.

- Endpoint or component: DM limit row
- Source path: `../mattermost/webapp/channels/src/components/user_settings/sidebar/limit_visible_gms_dms/limit_visible_gms_dms.tsx`
- Source lines: `127-129`
- Observed behavior: Row title is `Number of direct messages to show`.
- Notes: Uses select control in expanded mode.

### Screenshot 5: Advanced settings
- Endpoint or component: Advanced settings section ordering
- Source path: `../mattermost/webapp/channels/src/components/user_settings/advanced/user_settings_advanced.tsx`
- Source lines: `766-789`
- Observed behavior: Includes send-on-ctrl-enter, formatting, join/leave, performance debugging, unread-scroll-position, sync-drafts sections in that order.
- Notes: Matches screenshot row inventory.

- Endpoint or component: Specific advanced labels
- Source path: `../mattermost/webapp/channels/src/components/user_settings/advanced/user_settings_advanced.tsx`, `../mattermost/webapp/channels/src/components/user_settings/advanced/join_leave_section/join_leave_section.tsx`, `../mattermost/webapp/channels/src/components/user_settings/advanced/performance_debugging_section/performance_debugging_section.tsx`
- Source lines: `176-180`, `265-267`, `351-353`, `437-439`, `91-93`, `74-77`, `92-95`
- Observed behavior: Labels include `Send Messages on CTRL+ENTER` (or Mac variant), `Enable Post Formatting`, `Enable Join/Leave Messages`, `Performance Debugging`, `Scroll position when viewing an unread channel`, `Allow message drafts to sync with the server`, and collapsed `No settings enabled`.
- Notes: Text matches screenshots.

### Screenshot 8: Notifications settings
- Endpoint or component: Notifications section assembly
- Source path: `../mattermost/webapp/channels/src/components/user_settings/notifications/user_settings_notifications.tsx`
- Source lines: `1008-1011`, `1033-1112`
- Observed behavior: Renders row order with desktop/mobile notifications, desktop sounds, email, keywords, auto-responder, and troubleshooting notice block.
- Notes: `Learn more about notifications` is also present in header.

- Endpoint or component: Permission required tag
- Source path: `../mattermost/webapp/channels/src/components/user_settings/notifications/desktop_and_mobile_notification_setting/index.tsx`, `../mattermost/webapp/channels/src/components/user_settings/notifications/desktop_and_mobile_notification_setting/notification_permission_title_tag/index.tsx`
- Source lines: `335-341`, `46-49`
- Observed behavior: Desktop/mobile row title can append danger-style tag with text `Permission required`.
- Notes: Screenshot shows this exact tag.

- Endpoint or component: Troubleshooting notice and buttons
- Source path: `../mattermost/webapp/channels/src/components/user_settings/notifications/send_test_notification_notice.tsx`
- Source lines: `85`, `110`, `126-131`
- Observed behavior: Renders hint card titled `Troubleshooting notifications` with `Send a test notification` and `Troubleshooting docs` actions.
- Notes: Matches screenshot footer notice.

### Screenshot 3: Sidebar channel context menu
- Endpoint or component: Menu item labels and ordering
- Source path: `../mattermost/webapp/channels/src/components/sidebar/sidebar_channel/sidebar_channel_menu/sidebar_channel_menu.tsx`
- Source lines: `60-71`, `120-129`, `165-168`, `205-208`, `231-234`, `244-246`, `301-310`
- Observed behavior: Menu ordering is Mark as Read/Unread, Favorite/Unfavorite, Mute/Unmute, separator, Move to..., separator, Copy Link/Add Members, separator, Leave Channel.
- Notes: This exact sequence appears in screenshot.

- Endpoint or component: Move-to submenu
- Source path: `../mattermost/webapp/channels/src/components/channel_move_to_sub_menu/index.tsx`
- Source lines: `177-188`
- Observed behavior: `Move to...` is a submenu entry with right-chevron and explicit submenu aria label.
- Notes: Needed for category reassignment UX.

### Screenshot 9: Composer formatting toggle
- Endpoint or component: Bottom action composition
- Source path: `../mattermost/webapp/channels/src/components/advanced_text_editor/advanced_text_editor.tsx`
- Source lines: `842-851`
- Observed behavior: Bottom action bar contains formatting toggle first, then file upload, emoji, send button.
- Notes: This differs from always-visible top toolbar layouts.

- Endpoint or component: Toggle formatting button and tooltip shortcut
- Source path: `../mattermost/webapp/channels/src/components/advanced_text_editor/toggle_formatting_bar.tsx`, `../mattermost/webapp/channels/src/components/keyboard_shortcuts/keyboard_shortcuts_sequence/keyboard_shortcuts.ts`
- Source lines: `26-38`, `53-61`, `376-394`
- Observed behavior: Toggle button uses `FormatLetterCase` icon with chevron and tooltip text switching between `Show Formatting` and `Hide Formatting` with Ctrl|Alt|T / Cmd|Opt|T shortcuts.
- Notes: Screenshot tooltip content aligns.

## API and websocket contract evidence (Mattermost server)

- Endpoint or component: Preferences routes
- Source path: `../mattermost/server/channels/api4/preference.go`
- Source lines: `17-22`, `104-112`, `149`, `166-183`
- Observed behavior: `/users/{user_id}/preferences` supports GET/PUT, category and name filters, delete via `/delete`; PUT/POST delete require non-empty preference arrays with max 100; success returns status OK.
- Notes: Payload is preference array objects, not arbitrary map.

- Endpoint or component: Status routes and validation
- Source path: `../mattermost/server/channels/api4/status.go`
- Source lines: `15-24`, `92-101`, `119-134`, `149-167`, `199-201`, `205-220`
- Observed behavior: Status supports GET/PUT and custom-status endpoints; URL and payload user id must match; status accepts online/offline/away/dnd; custom status can return `501 Not Implemented` when disabled.
- Notes: Includes legacy POST `/status/custom/recent/delete` for mobile compatibility.

- Endpoint or component: Channel member notify props
- Source path: `../mattermost/server/channels/api4/channel.go`
- Source lines: `80`, `1816-1846`
- Observed behavior: `PUT /channels/{channel_id}/members/{user_id}/notify_props` parses a JSON map and updates channel-member notification props after permission checks.
- Notes: Used by mute-channel behavior.

- Endpoint or component: Mark post unread
- Source path: `../mattermost/server/channels/api4/post.go`
- Source lines: `43`, `1189-1213`
- Observed behavior: `POST /users/{user_id}/posts/{post_id}/set_unread` accepts body map with `collapsed_threads_supported` and returns unread state object from `MarkChannelAsUnreadFromPost`.
- Notes: This endpoint powers `Mark as unread` logic.

- Endpoint or component: Sidebar categories routes
- Source path: `../mattermost/server/channels/api4/channel.go`, `../mattermost/server/channels/api4/api.go`, `../mattermost/server/channels/api4/channel_category.go`
- Source lines: `42-49`, `216`, `97-167`, `203-257`, `317-395`
- Observed behavior: Category APIs are team-scoped under `/users/{user_id}/teams/{team_id}/channels/categories` with create/update/list/order/get/delete operations and permission checks.
- Notes: Move-to behavior depends on these endpoints.
