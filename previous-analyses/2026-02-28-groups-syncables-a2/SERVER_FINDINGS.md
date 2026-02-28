# Server Findings

- Endpoint or component: Group syncable route contract and methods.
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `13-62`
- Observed behavior: Uses `syncable_type: teams|channels`; link is POST, unlink is DELETE, get single/list are GET, patch is PUT.
- Notes: Route shape is strict and uses plural path segment names.

- Endpoint or component: Link group syncable behavior.
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `319-405`
- Observed behavior: Parses `GroupSyncablePatch`, upserts syncable, returns `201`, then schedules async `SyncRolesAndMembership`.
- Notes: Endpoint completion is not blocked on full reconciliation.

- Endpoint or component: Patch group syncable behavior.
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `501-589`
- Observed behavior: Loads existing syncable, applies patch (`auto_add`, `scheme_admin`), updates syncable, returns payload, schedules async sync.
- Notes: Async model mirrors link path.

- Endpoint or component: Unlink group syncable behavior.
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `590-645`
- Observed behavior: Deletes syncable, returns status OK, schedules async cleanup (`RemoveMembershipsFromUnlinkedSyncable`).
- Notes: Membership cleanup semantics are eventual.

- Endpoint or component: Syncable payload serialization contract.
- Source path: `../mattermost/server/public/model/group_syncable.go`
- Source lines: `23-142`
- Observed behavior: Syncable marshals with type-specific id key (`team_id` or `channel_id`), shared fields (`group_id`, `auto_add`, `scheme_admin`, timestamps), and `type` (`Team` or `Channel`).
- Notes: `GroupSyncablePatch` fields are optional booleans.

- Endpoint or component: Membership side effects for link/patch/unlink.
- Source path: `../mattermost/server/channels/app/syncables.go`
- Source lines: `273-310`, `312-322`
- Observed behavior: link/patch sync path updates roles + creates missing memberships; unlink path removes constrained memberships.
- Notes: Team membership must be ensured before channel membership.

- Endpoint or component: Current Rustchat behavior.
- Source path: `backend/src/api/v4/groups.rs`
- Source lines: `1-200`
- Observed behavior: All groups/syncables endpoints are placeholder stubs.
- Notes: No persistence, no membership effects, no retrievable syncable state.
