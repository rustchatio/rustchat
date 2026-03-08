# Mobile Findings

## Mattermost mobile (upstream consumer contract)

- Screen, store, or service: Initial channel selection on app entry.
- Source path: `../mattermost-mobile/app/actions/remote/entry/common.ts`
- Source lines: `278-291`
- Observed behavior: Entry flow checks whether the user is a member of the default channel for the initial team; if yes, it uses that channel ID; otherwise it falls back to the first available open team channel.
- Notes: Default channel membership is a first-class navigation signal.

- Screen, store, or service: Default-channel lookup for team.
- Source path: `../mattermost-mobile/app/queries/servers/channel.ts`
- Source lines: `355-390`
- Observed behavior: Query returns default channel for team if available, otherwise first channel fallback.
- Notes: This is used in multiple team/channel resolution paths.

- Screen, store, or service: Default channel identity.
- Source path: `../mattermost-mobile/app/utils/channel/index.ts`
- Source lines: `162-164`
- Observed behavior: `isDefaultChannel` is name-based (`channel?.name === General.DEFAULT_CHANNEL`).
- Notes: Semantics are contract-bound to Mattermost default channel naming.

- Screen, store, or service: Selection behavior tests.
- Source path: `../mattermost-mobile/app/utils/channel/index.test.ts`
- Source lines: `76-125`
- Observed behavior: Tests explicitly validate default-channel preference vs fallback behavior.
- Notes: Confirms expected priority order for default channel usage.

- Screen, store, or service: Entry behavior tests.
- Source path: `../mattermost-mobile/app/actions/remote/entry/common.test.ts`
- Source lines: `177-194`
- Observed behavior: Tests verify default channel is chosen when available in memberships.
- Notes: This is a compatibility-sensitive UX invariant.

## rustchat compatibility implications

- Screen, store, or service: Team creation and membership bootstrap (server-side impact on mobile).
- Source path: `backend/src/api/teams.rs`
- Source lines: `50-113`, `167-217`, `305-364`
- Observed behavior: Team creation does not create canonical default channels; membership flows use all-public-channel auto-add logic.
- Notes: This can diverge from mobile assumptions that a stable default channel identity exists.

- Screen, store, or service: Mattermost v4 compatibility route for LDAP user group sync memberships.
- Source path: `backend/src/api/v4/ldap.rs`
- Source lines: `42-44`, `137-144`
- Observed behavior: Exposed as GET + 501 placeholder.
- Notes: Enterprise/mobile/admin flows expecting POST action semantics cannot rely on this endpoint.
