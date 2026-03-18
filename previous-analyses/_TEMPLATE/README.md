# Analysis Template

Use this template when analyzing upstream Mattermost behavior for compatibility work.

## Directory Structure

Create a new analysis folder: `previous-analyses/YYYY-MM-DD-<topic>/`

```
previous-analyses/YYYY-MM-DD-<topic>/
├── README.md           # Main analysis document (this template)
├── server/             # Mattermost server code references
│   ├── findings.md
│   └── snippets/
├── mobile/             # Mattermost mobile code references (if applicable)
│   ├── findings.md
│   └── snippets/
├── contracts/          # API contracts discovered
│   ├── endpoints.md
│   ├── websocket-events.md
│   └── response-schemas/
└── gaps.md             # Documented gaps between upstream and implementation
```

## Analysis Template

```markdown
# Analysis: <Topic>

**Date**: YYYY-MM-DD
**Analyst**: <Name>
**Upstream Version**: Mattermost Server X.X.X, Mobile X.X.X
**Scope**: <What endpoints/events/behaviors are being analyzed>

## Executive Summary

<2-3 sentence summary of findings>

## Server-Side Analysis

### Endpoints Analyzed

| Endpoint | Method | Location | Key Finding |
|----------|--------|----------|-------------|
| /api/v4/... | GET | server/... | ... |

### WebSocket Events

| Event | Source | Payload Fields | When Triggered |
|-------|--------|----------------|----------------|
| ... | ... | ... | ... |

### Critical Code Paths

```go
// Mattermost server reference code
// File: server/...
```

## Mobile Client Analysis

### Journey Points

| Screen | API Call | Event Handler |
|--------|----------|---------------|
| ... | ... | ... |

## Contract Documentation

### Request/Response Schemas

```json
{
  "type": "object",
  "properties": { ... }
}
```

### Status Code Behavior

| Scenario | Status Code | Response Body |
|----------|-------------|---------------|
| ... | ... | ... |

## Gaps Identified

| Gap | Severity | Notes |
|-----|----------|-------|
| ... | High/Med/Low | ... |

## Recommendations

1. ...
2. ...

## Evidence Links

- Mattermost Server: `server/...`
- Mattermost Mobile: `mobile/...`
