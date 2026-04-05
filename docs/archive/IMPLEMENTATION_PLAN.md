# RustChat Implementation Plan: TODOs, Stubs, and Missing Features

## Executive Summary

This document catalogs all TODOs, stubs, and missing features in the RustChat codebase and provides a prioritized implementation roadmap.

**Last Updated:** 2026-03-17

---

## Priority Legend

- **P0 (Critical)**: Blocks core functionality, required for basic operation
- **P1 (High)**: Important for production readiness, major user-facing features
- **P2 (Medium)**: Enhances usability, nice-to-have features
- **P3 (Low)**: Future enhancements, can be deferred

---

## Category 1: Backend API Stubs & Missing Implementations

### 1.1 Email Service Providers (P2)

**Location:** `backend/src/services/email_provider.rs:467,473`

**Current State:**
```rust
// TODO: Implement SES provider
// TODO: Implement SendGrid provider
```

**Implementation Plan:**
1. Add AWS SES provider with credential management
2. Add SendGrid provider with API key support
3. Update email configuration to support provider selection
4. Add provider-specific error handling

**Estimated Effort:** 2-3 days

---

### 1.2 MJML Template Compilation (P2)

**Location:** `backend/src/services/template_renderer.rs:325`

**Current State:**
```rust
warn!("MJML compilation requested but not implemented");
```

**Implementation Plan:**
1. Research MJML compilation options:
   - Option A: Use `mjml-rs` crate (Rust native)
   - Option B: Call external MJML service/API
2. Implement MJML to HTML compilation
3. Add caching for compiled templates
4. Add MJML validation

**Estimated Effort:** 1-2 days

---

### 1.3 Custom Profile Attributes Management (P1 - Partial)

**Location:** `backend/src/api/v4/custom_profile.rs:77-121`

**Current State:**
- Routes exist but return 501 Not Implemented
- GET /fields works
- POST/PATCH/DELETE return stubs

**Implementation Plan:**
1. Database schema for custom profile fields:
   ```sql
   CREATE TABLE custom_profile_fields (
       id UUID PRIMARY KEY,
       name VARCHAR(255) NOT NULL,
       field_type VARCHAR(50) NOT NULL,
       options JSONB,
       team_id UUID REFERENCES teams(id),
       created_at TIMESTAMPTZ DEFAULT NOW()
   );
   
   CREATE TABLE custom_profile_values (
       user_id UUID REFERENCES users(id),
       field_id UUID REFERENCES custom_profile_fields(id),
       value TEXT,
       PRIMARY KEY (user_id, field_id)
   );
   ```
2. Implement CRUD operations for field management
3. Implement user value storage/retrieval
4. Add permission checks (manage_system for fields)

**Estimated Effort:** 3-4 days

---

### 1.4 Commands API - Full Implementation (P1)

**Location:** `backend/src/api/v4/commands.rs`

**Current State:**
- List commands returns hardcoded data
- GET/PUT/DELETE return empty stubs
- Custom command HTTP execution not implemented

**Implementation Plan:**
1. Database schema for custom commands:
   ```sql
   CREATE TABLE commands (
       id UUID PRIMARY KEY,
       team_id UUID REFERENCES teams(id),
       user_id UUID REFERENCES users(id),
       trigger VARCHAR(255) NOT NULL,
       method VARCHAR(10) DEFAULT 'GET',
       url TEXT NOT NULL,
       headers JSONB,
       auto_complete BOOLEAN DEFAULT false,
       auto_complete_hint TEXT
   );
   ```
2. Implement full CRUD operations
3. Implement command execution with HTTP client
4. Add timeout and retry logic
5. Implement autocomplete/lookup endpoints

**Estimated Effort:** 4-5 days

---

### 1.5 Scheduled Posts (P1)

**Location:** `backend/compat/inventory_endpoints.csv:162-165`

**Current State:**
- All scheduled post endpoints return 501

**Implementation Plan:**
1. Database schema:
   ```sql
   CREATE TABLE scheduled_posts (
       id UUID PRIMARY KEY,
       user_id UUID REFERENCES users(id),
       channel_id UUID REFERENCES channels(id),
       content TEXT NOT NULL,
       scheduled_at TIMESTAMPTZ NOT NULL,
       file_ids UUID[],
       processed BOOLEAN DEFAULT false,
       created_at TIMESTAMPTZ DEFAULT NOW()
   );
   ```
2. Background worker for processing scheduled posts
3. CRUD endpoints for management
4. WebSocket events for updates

**Estimated Effort:** 3-4 days

---

### 1.6 Plugin System (P2)

**Location:** `backend/src/api/v4/plugins.rs:90,104,143,158,173,243`

**Current State:**
- Upload, install, remove, enable/disable all return 501
- Marketplace endpoints stubbed

**Implementation Plan:**
1. Plugin storage system (S3/local filesystem)
2. Plugin manifest parsing
3. Plugin lifecycle management
4. Plugin isolation (WASM or subprocess)
5. Hook registration system
6. Marketplace integration

**Estimated Effort:** 10-14 days (complex feature)

---

### 1.7 Interactive Dialogs (P2)

**Location:** `backend/src/api/v4/dialogs.rs:18,29,40`

**Current State:**
- All dialog endpoints return 501

**Implementation Plan:**
1. Dialog definition schema
2. Dialog submission handling
3. Integration with commands/webhooks
4. Frontend dialog rendering support

**Estimated Effort:** 3-4 days

---

### 1.8 LDAP Integration (P1)

**Location:** `backend/src/api/v4/ldap.rs:11-21`

**Current State:**
- All LDAP endpoints return 501 stubs

**Implementation Plan:**
1. LDAP client configuration
2. LDAP authentication flow
3. Group synchronization
4. User attribute mapping
5. Background sync worker

**Estimated Effort:** 5-7 days

---

### 1.9 SAML Integration (P1)

**Location:** `backend/src/api/v4/saml.rs:12-19`

**Current State:**
- All SAML endpoints return 501 stubs

**Implementation Plan:**
1. SAML configuration storage
2. Identity Provider metadata parsing
3. SAML authentication flow
4. Assertion parsing and validation
5. User provisioning

**Estimated Effort:** 5-7 days

---

### 1.10 Compliance & Reporting (P3)

**Location:** `backend/src/api/v4/compliance.rs:38`, `backend/src/api/v4/reports.rs`

**Current State:**
- Compliance endpoints stubbed
- Report generation not implemented

**Implementation Plan:**
1. Data export functionality
2. Message/compliance export
3. Audit log querying
4. Report scheduling

**Estimated Effort:** 4-6 days

---

### 1.11 Shared Channels (P2)

**Location:** `backend/src/api/v4/shared_channels.rs`

**Current State:**
- Routes likely stubbed

**Implementation Plan:**
1. Cross-server channel sharing protocol
2. Federation support
3. Message synchronization
4. Permission mapping

**Estimated Effort:** 7-10 days

---

## Category 2: Frontend Missing Features

### 2.1 Scheduled Messages UI (P1)

**Location:** `frontend/src/components/composer/MattermostComposer.vue:803`

**Current State:**
```vue
Send later (coming soon)
```

**Implementation Plan:**
1. Date/time picker component
2. Scheduled messages list view
3. Edit/cancel scheduled messages
4. Backend integration with scheduled posts API

**Estimated Effort:** 2-3 days

---

### 2.2 Team Settings - Permission Tab (P2)

**Location:** `frontend/src/components/modals/TeamSettingsModal.vue:449`

**Current State:**
```vue
<p>Permission settings coming soon</p>
```

**Implementation Plan:**
1. Permission scheme editor
2. Role management UI
3. Channel moderation settings
4. Integration with roles/schemes API

**Estimated Effort:** 3-4 days

---

### 2.3 Policy Preview - Dry Run (P2)

**Location:** `frontend/src/components/admin/PolicyPreviewModal.vue:172`

**Current State:**
```vue
Select users above to simulate the policy application (dry-run feature coming soon)
```

**Implementation Plan:**
1. Dry-run API endpoint
2. Preview results display
3. Affected users/channels list
4. Impact analysis visualization

**Estimated Effort:** 2-3 days

---

### 2.4 Command Palette Full Implementation (P2)

**Location:** `frontend/src/components/ui/CommandPalette.vue:12`

**Current State:**
```typescript
// Placeholder data - replace with store/API later
```

**Implementation Plan:**
1. Integrate with actual stores
2. Add all navigation shortcuts
3. Implement user/channel search
4. Recent items tracking
5. Keyboard navigation

**Estimated Effort:** 2-3 days

---

### 2.5 Advanced Settings - Full Implementation (P2)

**Location:** `frontend/src/components/settings/advanced/AdvancedTab.vue`

**Current State:**
- Likely incomplete or using placeholder data

**Implementation Plan:**
1. Add all advanced configuration options
2. Import/export settings
3. Developer mode toggle
4. Performance settings

**Estimated Effort:** 1-2 days

---

## Category 3: WebSocket & Real-time Features

### 3.1 Missing WebSocket Events (P1)

**Location:** `archive/20260306-163836/previous-analyses/2026-02-07-mobile-client-requirements/websocket_events.md`

**Missing Events:**
- `status_change` - User status changed
- `channel_viewed` - User viewed channel  
- `reaction_added` - Reaction added
- `reaction_removed` - Reaction removed
- `post_edited` - Post edited
- `post_deleted` - Post deleted
- `user_added` - User added to channel/team
- `user_removed` - User removed from channel/team

**Implementation Plan:**
1. Add event emission in respective handlers
2. Ensure proper broadcast scope
3. Mobile-compatible payload format
4. Event ordering guarantees

**Estimated Effort:** 2-3 days

---

### 3.2 WebSocket Auth Expiry Enforcement (P0 - Active Spec)

**Location:** `SPEC.md:1-94`

**Current State:**
- WebSocket connections stay open after JWT expiry
- No forced logout on token expiry

**Implementation Plan:**
1. Capture JWT `exp` claim during WebSocket handshake
2. Add expiry deadline check in WebSocket loop
3. Close connection with auth violation code on expiry
4. Frontend: Add JWT expiry timer
5. Frontend: Centralized logout on expiry
6. Prevent reconnect with expired token

**Estimated Effort:** 2-3 days

---

## Category 4: Mobile Compatibility Gaps

### 4.1 Device Session Management (P1)

**Location:** `archive/20260306-163836/previous-analyses/2026-02-07-mobile-client-requirements/mobile_client_matrix.md:17-18`

**Missing:**
- `POST /api/v4/users/sessions/device` - attachDevice
- `DELETE /api/v4/users/sessions/device` - detachDevice

**Implementation Plan:**
1. Device token storage
2. Push notification device registration
3. Device management endpoints
4. Multi-device session tracking

**Estimated Effort:** 2-3 days

---

### 4.2 Mobile-Required User Endpoints (P1)

**Location:** `mobile_client_matrix.md:19-35`

**Missing/Stubbed:**
- User preferences endpoints
- Batch status endpoints
- File thumbnail generation

**Implementation Plan:**
1. Review all mobile-required endpoints
2. Implement missing handlers
3. Add mobile-specific response formats
4. Test with mobile smoke scripts

**Estimated Effort:** 3-5 days

---

## Category 5: Security & Enterprise Features

### 5.1 Access Control Attributes (P2)

**Location:** `backend/compat/inventory_endpoints.csv:73`

**Current State:**
- `GET /channels/{channel_id}/access_control/attributes` not implemented

**Implementation Plan:**
1. Access control schema design
2. Attribute-based access control (ABAC)
3. Policy evaluation engine
4. Channel-level attribute management

**Estimated Effort:** 4-6 days

---

### 5.2 Data Retention Policies (P2)

**Location:** `backend/src/api/v4/data_retention.rs`

**Current State:**
- NPS endpoint stubbed
- Retention policies likely incomplete

**Implementation Plan:**
1. Retention policy configuration
2. Background cleanup jobs
3. Message archival
4. Compliance reporting

**Estimated Effort:** 3-5 days

---

### 5.3 Content Flagging (P2)

**Location:** `backend/src/api/v4/content_flagging.rs`

**Current State:**
- Basic structure exists
- Full implementation needed

**Implementation Plan:**
1. Content reporting system
2. Moderation queue
3. Automated content scanning
4. User sanctions

**Estimated Effort:** 4-6 days

---

## Category 6: Performance & Monitoring

### 6.1 Performance Metrics Endpoint (P3)

**Location:** `backend/compat/inventory_endpoints.csv:150`

**Current State:**
- `POST /client_perf` not implemented

**Implementation Plan:**
1. Client performance data collection
2. Metrics aggregation
3. Performance dashboard
4. Alerting on degradation

**Estimated Effort:** 2-3 days

---

### 6.2 License Load Metrics (P3)

**Location:** `backend/compat/inventory_endpoints.csv:143`

**Current State:**
- `GET /license/load_metric` not implemented

**Implementation Plan:**
1. User count tracking
2. Message rate metrics
3. Storage usage
4. Feature usage analytics

**Estimated Effort:** 1-2 days

---

## Implementation Roadmap

### Phase 1: Critical (P0) - Week 1
1. **WebSocket Auth Expiry Enforcement** (SPEC.md)
   - Backend JWT expiry enforcement
   - Frontend forced logout

### Phase 2: High Priority (P1) - Weeks 2-4
1. **LDAP Integration** (Week 2)
2. **SAML Integration** (Week 2-3)
3. **Commands API** (Week 3)
4. **Scheduled Posts** (Week 4)
5. **Custom Profile Attributes** (Week 4)
6. **WebSocket Events** (parallel)
7. **Mobile Device Sessions** (parallel)

### Phase 3: Medium Priority (P2) - Weeks 5-8
1. **Email Providers** (SES, SendGrid)
2. **MJML Compilation**
3. **Interactive Dialogs**
4. **Plugin System** (core functionality)
5. **Shared Channels**
6. **Team Settings - Permissions**
7. **Policy Dry-Run**
8. **Command Palette**

### Phase 4: Lower Priority (P3) - Weeks 9-12
1. **Compliance & Reporting**
2. **Data Retention**
3. **Content Flagging**
4. **Performance Metrics**
5. **Advanced Settings**
6. **Scheduled Messages UI**

---

## Resource Estimates

| Category | Features | Est. Days | Priority |
|----------|----------|-----------|----------|
| Backend API | 11 | 45-60 | P0-P2 |
| Frontend UI | 6 | 12-18 | P1-P2 |
| WebSocket | 2 | 4-6 | P0-P1 |
| Mobile Compat | 2 | 5-8 | P1 |
| Security/Enterprise | 3 | 11-17 | P2 |
| Performance | 2 | 3-5 | P3 |
| **Total** | **26** | **80-114** | - |

---

## Testing Requirements

For each implemented feature:

1. **Unit Tests:**
   - `cargo test --no-fail-fast -- --nocapture`
   - `npm run test` (frontend)

2. **Integration Tests:**
   - `BASE=http://127.0.0.1:3000 ./scripts/mm_compat_smoke.sh`
   - `BASE=http://127.0.0.1:3000 ./scripts/mm_mobile_smoke.sh`

3. **Manual Verification:**
   - Feature-specific test scenarios
   - Mobile compatibility checks
   - Browser compatibility

---

## Notes

- **Enterprise vs OSS:** Some features (LDAP, SAML, Compliance) may be marked as enterprise-only
- **Compatibility:** All implementations must maintain Mattermost API v4 compatibility
- **Documentation:** Update API documentation and AGENTS.md with implementation details
- **Migration:** Some stubs may require database migrations when fully implemented

---

## Related Documents

- `SPEC.md` - Active specification for WebSocket auth expiry
- `backend/compat/inventory_endpoints.csv` - API endpoint inventory
- `previous-analyses/` - Detailed analysis artifacts for each feature area
- `AGENTS.md` - Development guidelines
