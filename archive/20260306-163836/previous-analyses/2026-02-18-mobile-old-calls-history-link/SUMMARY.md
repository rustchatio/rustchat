- Topic: Mattermost mobile old call posts must become history (non-joinable) after call end.
- Date: 2026-02-18
- Scope: Calls websocket + call post payload compatibility for mobile post rendering.

- Compatibility contract:
  1. Calls posts use type `custom_calls` (Mattermost web constant): `/Users/scolak/Projects/mattermost/webapp/channels/src/utils/constants.tsx:725`.
  2. Mobile calls post container treats a post as ended/inactive when `post.props.end_at` is present and skips active-call observers in that case: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/components/calls_custom_message/index.ts:33`.
  3. Mobile UI renders ended history card only when `start_at > 0 && end_at > 0`, otherwise it renders Join/Leave actions: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/components/calls_custom_message/calls_custom_message.tsx:176` and `/Users/scolak/Projects/mattermost-mobile/app/products/calls/components/calls_custom_message/calls_custom_message.tsx:219`.
  4. Therefore server must transition the call thread post from `end_at: 0` to `end_at: <timestamp>` at call end and emit a post update event so existing timeline items update in place.

- Open questions:
  1. Mattermost calls plugin server source is not present in `../mattermost` checkout, so exact plugin-side SQL/update sequencing is inferred from mobile behavior contract.
