# Endpoint Matrix

| Surface | Upstream Expectation | Rustchat Current | Gap |
|---|---|---|---|
| `GET /api/v4/system/ping` | Exists, returns status payload | Missing (empty router file) | Yes (P0) |
| `GET /api/v4/system/version` | Exists, used for bootstrap/version checks | Missing (empty router file) | Yes (P0) |
| `GET /api/v4/config/client?format=old` | Exists | Present in `config_client.rs` | No |
| `GET /api/v4/license/client?format=old` | Exists | Present in `config_client.rs` | No |
| `GET /api/v4/websocket` | Exists and used by mobile | Route still present in `api/v4/mod.rs` | No direct gap |
| `GET/POST /api/v4/plugins*` | Exists for compatibility surfaces | Missing (empty plugins router) | Yes (P1) |
| `GET/POST /api/v4/groups*` | Exists in compatibility scope | Missing (empty groups router) | Yes (P1) |
| `GET/POST /api/v4/custom_profile*` | Exists in compatibility scope | Missing (empty custom profile router) | Yes (P2) |
| `GET/POST /api/v4/reports*` | Exists in compatibility scope | Missing (empty reports router) | Yes (P2) |
| `GET/POST /api/v4/access_control*` | Exists in compatibility scope | Missing (empty access_control router) | Yes (P2) |
