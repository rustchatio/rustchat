# Mobile Findings

- Screen, store, or service: Users REST client status bulk and custom status
- Source path: `../rustchat-mobile/app/client/rest/users.ts`
- Source lines: 406, 429, 436, 443
- Observed behavior:
  - Status bulk request posts raw `userIds` array to `/users/status/ids`.
  - Custom status routes call `/users/me/status/custom` and `/users/me/status/custom/recent/delete`.
- Notes: Backend must preserve exact route and payload compatibility.

- Screen, store, or service: Remote status reducer expectations
- Source path: `../rustchat-mobile/app/actions/remote/user.ts`
- Source lines: 370-377
- Observed behavior: Expects statuses payload to be an array and reduces by `status.user_id`.
- Notes: Map/object payloads break the reducer flow.

- Screen, store, or service: Categories REST client
- Source path: `../rustchat-mobile/app/client/rest/categories.ts`
- Source lines: 20, 33
- Observed behavior:
  - Reads category order using `GET /categories/order`.
  - Writes category updates with raw category array body.
- Notes: Rustchat must support both calls with Mattermost-compatible shapes.
