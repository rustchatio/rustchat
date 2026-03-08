---
name: mattermost-api-parity
description: Rules for strict Mattermost API contract parity, including JSON error IDs, localized message semantics, and parameter compatibility boundaries.
license: MIT
---

# Mattermost API Parity

Use this skill when implementing or reviewing compatibility-sensitive HTTP behavior.

## Objectives

- Match upstream API contract semantics exactly for status codes, response shape, and errors.
- Enforce canonical error IDs (for example `REQUEST_VARIABLE_MISSING`) and expected localized `message`/`msg` behavior.
- Prevent ad-hoc request/response schema drift.

## Contract Rules

1. Error envelope parity:
   - Preserve upstream-compatible error shape and field names for the target endpoint.
   - Use canonical error IDs from upstream behavior; do not invent IDs for compatibility surfaces.
2. Message semantics:
   - Preserve upstream-compatible user-facing message behavior, including i18n expectations where applicable.
   - Do not hardcode incompatible wording when upstream uses translatable keys or structured message patterns.
3. Parameter and route semantics:
   - Follow established parameter naming, defaults, and validation behavior from upstream API documentation and sources.
   - Do not introduce alternative parameter patterns on compatibility endpoints.

## Workflow

1. Collect upstream evidence from `../mattermost` docs/source for the exact endpoint behavior.
2. Capture expected contract for request, response, error, and edge cases.
3. Compare Rustchat behavior against that contract.
4. Record explicit evidence paths and line references in analysis artifacts.
5. Reject parity claims that do not include concrete evidence.

## Boundary

- Treat upstream API docs/source as the authority for compatibility behavior.
- If behavior is ambiguous, mark it as unresolved and gather more evidence before implementation.

## Output Expectations

- Every parity claim includes upstream evidence, Rustchat implementation evidence, and verification evidence.
- No compatibility change is marked complete while schema, error ID, or parameter behavior is only "close enough".
