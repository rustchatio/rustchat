# Summary

- Topic: A2 - Implement real v4 group syncables behavior (link/patch/unlink + membership effects)
- Date: 2026-02-28
- Scope:
  - `POST/PUT/DELETE/GET /api/v4/groups/{group_id}/{syncable_type}/{syncable_id}*`
  - Group syncable persistence and membership reconciliation side effects
  - Group-syncable response contract fields (`team_id` / `channel_id`, `type`, `auto_add`, `scheme_admin`)
- Compatibility contract:
  - Path `syncable_type` must be `teams|channels` (plural path segment) and map to logical syncable types Team/Channel.
  - Link endpoint accepts JSON patch with `auto_add` and `scheme_admin`; creates or upserts syncable and returns `201` with syncable payload.
  - Patch endpoint updates existing syncable and returns `200` with updated payload.
  - Unlink endpoint deletes syncable and returns status OK body.
  - Get single syncable returns serialized syncable with type-specific identifiers (`team_id` or `channel_id`), plus `type` and flags.
  - Get syncables returns array of serialized syncables for group + syncable type.
  - Linking/patching/unlinking triggers async membership convergence/cleanup rather than blocking endpoint completion.
- Open questions:
  - Rustchat parity target for permission granularity is currently simplified versus Mattermost system-console permissions; implementation will enforce admin-level checks until fine-grained permissions exist.
