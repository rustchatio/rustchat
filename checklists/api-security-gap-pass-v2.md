# API + Security Requirements Gap Pass (v2)

**Date**: 2026-03-08
**Checklist**: /Users/scolak/Projects/rustchat/checklists/api-security.md
**Spec Evaluated**: /Users/scolak/Projects/rustchat/SPEC.md
**Previous Baseline**: /Users/scolak/Projects/rustchat/checklists/api-security-gap-pass.md

## Result Summary

- Total checklist items evaluated: 30
- Pass: 15
- Partial: 8
- Gap/Fail: 7
- Delta vs previous pass: +13 pass, -13 gap/fail

## Remaining P0/P1 Gaps

### P0

1. Deny-path error contract still lacks exact wire values.
- Checklist: CHK004, CHK012
- Remaining gap: spec requires documented status/error envelope but does not yet state concrete expected values.

2. Websocket payload field contract remains underspecified.
- Checklist: CHK007
- Remaining gap: event names are specified, but required/optional payload fields are not enumerated.

### P1

3. Alternate scenario requirements are not explicit.
- Checklist: CHK017
- Remaining gap: partial support/unsupported-route boundaries are not codified as normative requirements.

4. Concurrency edge cases for reaction toggles are not specified.
- Checklist: CHK021
- Remaining gap: no normative behavior for simultaneous toggles by multiple users.

5. Performance NFR thresholds are still missing.
- Checklist: CHK025
- Remaining gap: no measurable responsiveness or throughput target for realtime update flows.

6. Verification environment prerequisites are not formalized as requirements.
- Checklist: CHK027, CHK030
- Remaining gap: release decision ownership and gating behavior are still described in task status rather than normative requirement language.

## Partial Items To Tighten

- CHK003: add explicit disclosure-boundary language for config payloads.
- CHK008: add explicit expected output conditions for manual commands.
- CHK011: align spec language with known verification failures and acceptance gating.
- CHK013/CHK014: move measurable reaction/unread outcomes from verification bullets to normative FR/AC text.
- CHK016/CHK018/CHK029: expand success/exception scope language and resolve non-goal boundary ambiguity.

## Recommended Next Patch Blocks

- `## Websocket Event Payload Contract (Normative)`
- `## Alternate and Unsupported Scenario Requirements`
- `## Realtime Performance Targets (Normative)`
- `## Verification Prerequisites and Release Decision Rule`

