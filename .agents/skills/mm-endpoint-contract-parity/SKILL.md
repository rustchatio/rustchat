---
name: mm-endpoint-contract-parity
description: Endpoint diff workflow for API v4 method/path/status/schema parity against upstream server and mobile usage.
license: MIT
---

# MM Endpoint Contract Parity

Use this skill when implementing or reviewing HTTP API compatibility.

## Trigger Conditions

- Any change under `backend/src/api/v4/**`.
- Any route/status/schema/error payload change.
- Any request that claims compatibility parity.

## Required Inputs

- Analysis folder: `previous-analyses/YYYY-MM-DD-<topic>/`.
- Upstream references:
  - `../mattermost/api/v4/source/*.yaml`
  - `../mattermost/server/channels/api4/*.go`
- RustChat route sources under `backend/src/api/v4/`.

## Workflow

1. Build upstream baseline from OpenAPI.
2. Extract RustChat method/path inventory from route declarations.
3. Normalize placeholders and diff method+path sets.
4. For each high-impact mismatch, capture status/schema/header semantics.
5. Update:
   - `ENDPOINT_MATRIX.md`
   - `GAP_REGISTER.md`
   - `GAP_PLAN.md`
6. Gate implementation on explicit gap entries.

## Expected Outputs

- `previous-analyses/<iteration>/ENDPOINT_MATRIX.md`
- `previous-analyses/<iteration>/GAP_REGISTER.md`
- `previous-analyses/<iteration>/GAP_PLAN.md`

## Command Checklist

```bash
# Upstream OpenAPI extraction (prefer temp copy to avoid mutating tracked output)
python3 tools/mm-compat/openapi/extract_paths.py

# Optional mobile static endpoint scan
python3 tools/mm-compat/static-scan/scan_repo.py --repo ../mattermost-mobile

# Merge/diff report
python3 tools/mm-compat/report/merge_and_diff.py
```

## Failure Handling

- If tooling mutates tracked files unexpectedly, rerun in a temporary copy under `/tmp`.
- If a route is ambiguous, resolve with direct upstream source evidence before coding.
- If behavior differs by edition/feature flag, tag gaps with deployment profile scope.

## Definition of Done

- Every compatibility-sensitive route change has a linked gap ID.
- Endpoint matrix includes coverage metrics and explicit method mismatches.
- No unreferenced compatibility claims remain.
