# UX Journeys

## Journey 1: Reading older messages

1. User scrolls into prior-day history in a channel.
2. UI should show a date separator at day boundary (Today / Yesterday / date text).
3. Current Rustchat behavior shows only per-message time values.

## Journey 2: Editing own message with global policy

1. Admin sets edit policy globally (disabled, unlimited, or N-second limit e.g. 1800).
2. User edits own recent message.
3. Expected:
- edit allowed only when policy permits
- edited marker appears
- changed message appears immediately
4. Current Rustchat:
- no global limit enforcement in edit APIs
- no visible edited marker in message row
- local immediate update event is emitted but not handled upstream

## Journey 3: Toggling reaction emoji

1. User clicks emoji on a post -> add reaction.
2. Same user clicks same emoji again -> remove own reaction.
3. If other users reacted, total should decrement by 1 but remain visible.
4. Current state requires parity validation for immediate UI consistency.

## Journey 4: Clearing top notification dot

1. User opens unread channel/messages.
2. Channel is marked read/viewed.
3. Bell dot should clear once unread total reaches zero.
4. Current Rustchat uses mismatched stores for bell dot vs unread mutation path.
