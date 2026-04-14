# Mobile Compatibility Matrix

This page tracks headline compatibility for Mattermost clients.

For implementation details, protected paths, and contract testing workflow:
- See [Compatibility Scope](../compatibility-scope.md)
- See [Development Compatibility Guide](../development/compatibility.md)

## Current Headline Coverage

| Scope | Coverage | Last Audited |
|---|---:|---|
| Mobile-critical endpoints | 39/41 (95.1%) | 2026-03-17 |

## Known Gaps

| Endpoint | Status | Impact | Planned |
|---|---|---|---|
| `POST /api/v4/emoji` | Not implemented | Custom emoji upload unavailable | Phase 2 |
| `POST /api/v4/posts/search` | Not implemented | Advanced post search unavailable | Phase 2 |

## Notes

- The mobile-critical metric is not the same as full Mattermost API v4 coverage.
- Full v4 surface analysis remains lower because RustChat intentionally focuses on mobile and desktop client viability first.
