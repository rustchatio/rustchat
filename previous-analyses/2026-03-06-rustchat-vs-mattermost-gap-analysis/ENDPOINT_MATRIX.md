# Endpoint Matrix

## Method

- Upstream baseline extracted from Mattermost OpenAPI (`570` method+path entries).
- RustChat v4 routes extracted from `backend/src/api/v4/**/*.rs` route declarations (`555` method+path entries).
- Paths normalized for placeholder comparison (`{id}` family).

## Global coverage

| Metric | Count |
| :--- | ---: |
| Upstream baseline | 570 |
| RustChat v4 extracted | 555 |
| Exact method+path matches | 438 |
| Missing from RustChat | 132 |
| RustChat-only extras | 117 |
| Coverage | 76.8% |

## Top resources by gap volume

| Resource | Baseline | Matched | Missing | Coverage |
| :--- | ---: | ---: | ---: | ---: |
| users | 114 | 102 | 12 | 89.5% |
| plugins | 56 | 10 | 46 | 17.9% |
| teams | 47 | 39 | 8 | 83.0% |
| channels | 39 | 38 | 1 | 97.4% |
| posts | 23 | 20 | 3 | 87.0% |
| groups | 21 | 11 | 10 | 52.4% |
| access_control_policies | 15 | 9 | 6 | 60.0% |
| data_retention | 15 | 10 | 5 | 66.7% |

## Sampled core mobile route set (74 routes)

- Covered: 73
- Missing: 1
- Missing endpoint: `PUT /api/v4/posts/{post_id}`

Note: `/api/v4/users/me/channel_members` is served by RustChat via parameterized route `/users/{user_id}/channel_members` with `me` resolution.

## High-priority mismatches

| ID | Upstream | RustChat | Impact |
| :--- | :--- | :--- | :--- |
| G-001 | `PUT /api/v4/posts/{post_id}` | not present | post update contract gap |
| G-002 | `GET /api/v4/posts/{post_id}/reveal` | `POST /api/v4/posts/{post_id}/reveal` | method mismatch |
| G-003 | `DELETE /api/v4/posts/{post_id}/burn` | `POST /api/v4/posts/{post_id}/burn` | method mismatch |
| G-004 | `GET /api/v4/channels` | not present | admin/system listing gap |
