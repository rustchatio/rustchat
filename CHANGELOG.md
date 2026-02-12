# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-02-12

### Added
- Mobile compatibility analysis and verification artifacts for calls and messaging attachment flows.
- Release version bump across backend and frontend metadata to `0.3.1`.

### Fixed
- Desktop call screen sharing flow stabilization so screen-on/screen-off control paths are functional end-to-end.
- Mattermost mobile calls now start working reliably with improved call signaling/state sync behavior.
- Mobile ringing/notification lifecycle alignment (including dismissal persistence and state refresh behavior).
- Mobile message history attachment visibility after re-login by preserving file metadata in post-list responses.

## [0.3.0] - 2026-02-07

### Added
- CI quality gates for backend and frontend build/test workflows.
- Expanded Mattermost API v4 compatibility coverage and status reporting.
- Calls plugin architecture improvements (state handling, signaling path hardening).
- Stronger deployment documentation and operational guidance.

### Changed
- WebSocket stack rationalization and cleanup for more predictable runtime behavior.
- Release metadata and project versioning updated to `0.3.0`.
- Documentation updated to reflect current implementation status and compatibility scope.

### Fixed
- Multiple test suite and integration issues that blocked reliable validation.
- Semantic compatibility gaps where endpoints existed but behavior was incomplete.
- Configuration and environment drift between docs, compose, and runtime behavior.
- Various reliability and maintainability issues across API and realtime layers.

### Security
- Tighter production posture for default settings and deployment guidance.
- Better separation between development-friendly and production-safe defaults.

### Deployment
- This release is considered deployment-ready for managed environments with proper production configuration (TLS, secrets, database backups, and monitoring).

## [0.0.1] - 2026-01-24

### Added
- Initial working version of RustChat.
- Real-time messaging via WebSockets.
- Thread support.
- Unread messages system.
- S3/MinIO file uploads.
- User presence and status.
- Organization and Team structures.

### Fixed
- Disappearing messages issue (schema mismatch).
- Thread reply UI duplication.
