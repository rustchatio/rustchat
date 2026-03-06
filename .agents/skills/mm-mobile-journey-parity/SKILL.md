---
name: mm-mobile-journey-parity
description: Mobile-first compatibility validation workflow across login, channels, posts, notifications, files, and calls journeys.
license: MIT
---

# MM Mobile Journey Parity

Use this skill when validating user-visible behavior consumed by Mattermost mobile.

## Trigger Conditions

- Backend changes affecting mobile app flows.
- Parity claims for "works with Mattermost mobile".
- Regressions reported from mobile usage.

## Required Inputs

- Current analysis folder.
- Mobile sources:
  - `../mattermost-mobile/app/client/rest/*.ts`
  - `../mattermost-mobile/app/actions/remote/*.ts`
  - `../mattermost-mobile/app/actions/websocket/*.ts`
  - `../mattermost-mobile/app/products/calls/**`

## Workflow

1. Define journey checklist:
   - login/session
   - team/channel discovery and switching
   - posting/editing/reacting/threading
   - notifications/read state
   - file upload/preview/download
   - calls entry/join/leave/host controls
2. Map each journey to required endpoint/event contracts.
3. Verify route/method/status/schema support in RustChat.
4. Record journey-level outcome (`Pass`, `Partial`, `Fail`) with evidence.
5. Add missing/mismatched items to `GAP_REGISTER.md`.

## Expected Outputs

- `previous-analyses/<iteration>/UX_JOURNEYS.md`
- `previous-analyses/<iteration>/MOBILE_FINDINGS.md`
- `previous-analyses/<iteration>/GAP_REGISTER.md`

## Command Checklist

```bash
rg -n "login|logout|teams|channels|posts|reactions|thread|notifications|files|calls" \
  ../mattermost-mobile/app/client/rest ../mattermost-mobile/app/actions

./scripts/mm_compat_smoke.sh
./scripts/mm_mobile_smoke.sh
```

## Failure Handling

- If smoke tests point to wrong base URL, set `BASE` explicitly and rerun.
- If environment blocks execution, capture blocker and do static contract validation.

## Definition of Done

- Every core journey has a status and evidence.
- Any partial/fail journey is linked to one or more gap IDs.
- No "mobile compatible" claim is made without journey evidence.
