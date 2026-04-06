# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Admin panel team members now load correctly (fixed missing `presence` column in SQL query)
- Thread view now displays replies properly (fixed API response format mismatch)
- Typing indicators now appear when other users are typing (fixed v1 WebSocket message format conversion)

## [0.3.5] - 2026-03-09

### Added
- VoIP Push Notification support for call ringing on mobile devices.
  - Push Proxy service with FCM (Android) and APNS (iOS) support.
  - Data-only FCM messages for Android call notifications (high priority, direct boot).
  - APNS VoIP push support for iOS CallKit integration (prepared, requires credentials).
  - Backend integration with `sub_type: "calls"` for mobile app call identification.
  - Call UUID generation for VoIP session tracking.
- Documentation for mobile push notification architecture and implementation requirements.

### Changed
- Docker Compose configuration to include push-proxy service on port 3001.
- Backend push notification service to route calls through push proxy.
- Version bump to 0.3.5 reflecting significant new features and maturity.

### Security
- Fixed protobuf vulnerability (RUSTSEC-2024-0437) by upgrading prometheus 0.13 -> 0.14.
- Fixed rustls-pemfile warning (RUSTSEC-2025-0134) by upgrading yup-oauth2 11 -> 12.
- Fixed dompurify XSS vulnerability (GHSA-v2wj-7wpq-c8vv).
- Fixed rollup path traversal vulnerability (GHSA-mw96-cpmx-2vgc).
- Updated AWS-LC to latest versions (aws-lc-rs 1.15.3 -> 1.16.1, aws-lc-sys 0.36.0 -> 0.38.0).

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
