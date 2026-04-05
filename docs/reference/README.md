# Reference Documentation

Quick reference materials for RustChat.

## Documentation Sections

### API Reference
- REST API endpoints (coming soon)
- WebSocket event types (coming soon)
- Error codes and meanings (coming soon)

### Compatibility
- [Mobile Compatibility Matrix](./compatibility-matrix.md) - Mattermost mobile client API coverage

### Configuration Reference
- [Environment Variables](./environment-variables.md) - Complete list of all configuration options

## Quick Links

### API Versions
- `/api/v1/*` - Native RustChat API
- `/api/v4/*` - Mattermost-compatible API

### WebSocket Endpoints
- `/api/v1/ws` - Native WebSocket (internal clients)
- `/api/v4/websocket` - Mattermost-compatible WebSocket

### Default Ports
| Port | Service |
|------|---------|
| 3000 | Backend API |
| 8080 | Nginx proxy (web UI + API) |
| 5432 | PostgreSQL |
| 6379 | Redis |
| 9000 | S3 API (RustFS) |

---

*For detailed configuration: See [Admin Guide](../admin/configuration.md).*
