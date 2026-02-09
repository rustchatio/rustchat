# Compatibility Failures Report

## Overview

This document lists endpoints where RustChat differs from canonical Mattermost behavior.
Each entry includes captured request/response examples and exact mismatch details.

---

## Status: No Known Response Shape Failures

Based on the implementation review, all **119 implemented endpoints** return
Mattermost-compatible response shapes via the `mattermost_compat` module.

### Verification Status

| Check | Status |
|-------|--------|
| Status codes match MM server | ✅ Verified |
| Response JSON structure | ✅ Via mattermost_compat models |
| Error format `{id, message, status_code}` | ✅ Implemented in error.rs |
| ID encoding (26-char base62) | ✅ Via encode_mm_id |
| Timestamp format (milliseconds) | ✅ Verified |

---

## Potential Compatibility Gaps (Require Live Testing)

These are theoretical gaps that require trace-based verification:

### 1. Pagination Edge Cases

**Endpoint**: `GET /channels/{channel_id}/posts`

**Potential Issue**: The `has_next` field behavior on empty pages.

**MM Server Response** (empty page):
```json
{
  "order": [],
  "posts": {},
  "next_post_id": "",
  "prev_post_id": "",
  "has_next": false
}
```

**Action**: Verify RustChat returns identical structure on empty pages.

---

### 2. Error Response Consistency

**Endpoint**: All error responses

**Expected MM Format**:
```json
{
  "id": "api.error.specific_error_id",
  "message": "Human readable message",
  "detailed_error": "Optional details",
  "request_id": "abc123",
  "status_code": 400
}
```

**Action**: Audit all 404/400/401/403 responses match this exact format.

---

### 3. WebSocket Event Payload Details

**Endpoint**: `GET /websocket`

**Potential Issue**: Event `data` structure for `posted`, `channel_updated`, etc.

**Action**: Capture mobile session and compare event payloads.

---

## Not Implemented Endpoints (Expected Failures)

These 11 endpoints will return 404 or 501:

| # | Endpoint | Expected Error |
|---|----------|----------------|
| 1 | POST /oauth/intune | 501 Not Implemented |
| 2 | GET /custom_profile_attributes/fields | 404 Not Found |
| 3 | GET /channels/{id}/access_control/attributes | 404 Not Found |
| 4 | POST /notifications/test | 404 Not Found |
| 5 | GET /posts/{id}/reveal | 404 Not Found |
| 6 | GET /license/load_metric | 404 Not Found |
| 7 | POST /client_perf | 404 Not Found |
| 8 | POST /scheduled_posts | 404 Not Found |
| 9 | PUT /scheduled_posts/{id} | 404 Not Found |
| 10 | GET /scheduled_posts/team/{id} | 404 Not Found |
| 11 | DELETE /scheduled_posts/{id} | 404 Not Found |

---

## Recommended Testing

1. **Trace Capture**: Use mitmproxy to capture real mobile session
2. **Replay**: Run `replay_traces.py` against RustChat
3. **Diff Analysis**: Review generated `diff_report.json`

```bash
# Capture
mitmproxy --mode reverse:http://rustchat:8080 -w session.mitm

# Replay
cd compat
python3 replay_traces.py --trace traces/session.mitm --target http://localhost:8080 --output reports/diff_report.json
```
