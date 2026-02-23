# Password Reset Security Implementation

## 1. Checklist: Best Practices for Password Setup via Email

### A. Token & Link Security
- [x] Token is cryptographically random (256 bits entropy via `rand::thread_rng()`)
- [x] Store **only SHA-256 hash** of token server-side (raw tokens never stored)
- [x] Token bound to: `user_id`, `purpose`, `expires_at`, `used_at`, `created_at`, `created_ip`, `user_agent`
- [x] Token is **single-use**: `used_at` timestamp prevents replay
- [x] Token has **short TTL**: 60 minutes (configurable via `TOKEN_VALIDITY_MINUTES`)
- [x] Validation uses constant-time comparison via `constant_time_compare()`
- [x] Link/code does **not** reveal whether email exists (anti-enumeration)

### B. API/UX Safety
- [x] "Request password reset" returns **same response** for existing/non-existing emails
- [x] Rate limiting:
  - [x] per IP: 10 requests/hour (`RATE_LIMIT_IP_HOURLY`)
  - [x] per email: 3 requests/hour (`RATE_LIMIT_EMAIL_HOURLY`)
- [x] No sensitive info in logs; tokens hashed before logging
- [x] Password policy enforced:
  - [x] Minimum length: 12 characters
  - [x] Requires uppercase, lowercase, digit, special character
  - [x] Rejects common passwords (password, 123456, qwerty, admin, letmein)
- [x] Password stored with Argon2id (modern hash with salt)

### C. Email System Reliability
- [x] Email sent via workflow purpose `password_reset` (system-required, cannot be disabled)
- [x] Template rendering is strict with variable schema validation
- [x] Outbox worker with idempotent retry logic
- [x] Email events recorded: queued → sending → delivered/failed

### D. Multi-tenant Correctness
- [x] Tokens include `user_id` in database (cannot cross tenant boundaries)
- [x] User lookup includes `is_active=true` and `deleted_at IS NULL` checks

### E. Database Security
- [x] Row-level locking (`FOR UPDATE`) prevents race conditions on token consumption
- [x] Transaction ensures token marked used AND password updated atomically

---

## 2. Gap Analysis

### Implemented ✅
| Feature | Implementation |
|---------|---------------|
| Secure token generation | `generate_secure_token()` - 32 bytes random |
| Token hashing | SHA-256 with hex encoding |
| Constant-time comparison | `constant_time_compare()` mitigates timing attacks |
| Rate limiting | Per-email and per-IP with hourly windows |
| Password policy | Length, complexity, common password checks |
| Anti-enumeration | Same response for all emails, token created even if user not found |
| Single-use tokens | `used_at` timestamp prevents replay |
| Short TTL | 60 minutes expiry |
| Row locking | `FOR UPDATE` prevents double-spend |
| Transaction safety | Token consumption + password update in one transaction |

### Gaps / Future Improvements
| Gap | Priority | Notes |
|-----|----------|-------|
| Redis-based rate limiting | Medium | Currently using DB; Redis would be more scalable |
| Breached password API check | Low | Could integrate HaveIBeenPwned API |
| TOTP for password reset | Low | High-security environments may want 2FA before reset |
| Device fingerprinting | Low | Could track device consistency for security |

---

## 3. Test Plan Matrix

| ID | Test Case | Type | Status | File:Line |
|----|-----------|------|--------|-----------|
| T01 | Token generation creates 32-byte random string | Unit | ✅ | `password_reset.rs:unit tests` |
| T02 | Multiple tokens are all unique | Unit | ✅ | `password_reset.rs:unit tests` |
| T03 | Token hashing is consistent and produces 64-char hex | Unit | ✅ | `password_reset.rs:unit tests` |
| T04 | Constant-time compare works correctly | Unit | ✅ | `password_reset.rs:unit tests` |
| T05 | Password policy rejects short passwords | Unit | ✅ | `password_reset.rs:unit tests` |
| T06 | Password policy rejects missing uppercase | Unit | ✅ | `password_reset.rs:unit tests` |
| T07 | Password policy rejects missing lowercase | Unit | ✅ | `password_reset.rs:unit tests` |
| T08 | Password policy rejects missing digit | Unit | ✅ | `password_reset.rs:unit tests` |
| T09 | Password policy rejects missing special char | Unit | ✅ | `password_reset.rs:unit tests` |
| T10 | Password policy rejects common passwords | Unit | ✅ | `password_reset.rs:unit tests` |
| T11 | Password policy accepts valid passwords | Unit | ✅ | `password_reset.rs:unit tests` |
| T12 | Request reset creates token in database | Integration | ✅ | `api_password_reset.rs:36` |
| T13 | Anti-enumeration: token created for non-existent email | Integration | ✅ | `api_password_reset.rs:58` |
| T14 | Full flow: request → validate → reset | Integration | ✅ | `api_password_reset.rs:80` |
| T15 | Token replay protection | Integration | ✅ | `api_password_reset.rs:147` |
| T16 | Expired token rejection | Integration | ✅ | `api_password_reset.rs:189` |
| T17 | Invalid token rejection | Integration | ✅ | `api_password_reset.rs:215` |
| T18 | Password policy enforced on reset | Integration | ✅ | `api_password_reset.rs:230` |
| T19 | Rate limit per email (3/hour) | Integration | ✅ | `api_password_reset.rs:280` |
| T20 | Rate limit per IP (10/hour) | Integration | ✅ | `api_password_reset.rs:312` |
| T21 | API forgot password endpoint | API | ✅ | `api_password_reset.rs:345` |
| T22 | API anti-enumeration response | API | ✅ | `api_password_reset.rs:368` |
| T23 | API reset password endpoint | API | ✅ | `api_password_reset.rs:390` |
| T24 | API validate token endpoint (valid) | API | ✅ | `api_password_reset.rs:425` |
| T25 | API validate token endpoint (invalid) | API | ✅ | `api_password_reset.rs:458` |
| T26 | API weak password rejection | API | ✅ | `api_password_reset.rs:480` |
| T27 | Password hash actually changes after reset | Security | ✅ | `api_password_reset.rs:506` |

---

## 4. Code Changes Summary

### New Files
| File | Lines | Purpose |
|------|-------|---------|
| `src/services/password_reset.rs` | ~600 | Core password reset service with security features |
| `migrations/20260224000001_password_reset.sql` | ~130 | Database schema and templates |
| `tests/api_password_reset.rs` | ~550 | Comprehensive integration tests |

### Modified Files
| File | Changes |
|------|---------|
| `src/services/mod.rs` | Added `password_reset` module |
| `src/api/auth.rs` | Added `/password/forgot`, `/password/reset`, `/password/validate` endpoints |
| `src/error/mod.rs` | Added `TooManyRequests` error variant |

### API Endpoints
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/auth/password/forgot` | POST | Request password reset email |
| `/api/v1/auth/password/reset` | POST | Reset password with token |
| `/api/v1/auth/password/validate` | POST | Validate token without consuming |

---

## 5. Tests Added

### Unit Tests (in `password_reset.rs`)
- `test_generate_token_length` - Token length verification
- `test_generate_token_entropy` - Uniqueness verification
- `test_hash_token_consistency` - Hash consistency
- `test_hash_token_different_tokens` - Different inputs → different hashes
- `test_constant_time_compare` - Timing-safe comparison
- Password policy tests (6 tests)

### Integration Tests (in `tests/api_password_reset.rs`)
- Token creation and storage
- Anti-enumeration behavior
- Full reset flow
- Replay protection
- Expiry handling
- Rate limiting (per-email and per-IP)
- API endpoint tests
- Password hash verification

---

## 6. How to Run

### Prerequisites
```bash
# Ensure Docker is running for test database
# Ensure PostgreSQL is available
```

### Run Tests Locally
```bash
cd /Users/scolak/Projects/rustchat/backend

# Run all password reset tests
cargo test --test api_password_reset

# Run with output
cargo test --test api_password_reset -- --nocapture

# Run specific test
cargo test --test api_password_reset test_api_reset_password_endpoint -- --nocapture

# Run unit tests in the service module
cargo test password_reset::tests
```

### Run in CI
```bash
# Setup (already in GitHub Actions workflow)
docker-compose up -d postgres redis

# Run all tests
cargo test

# Generate coverage report
cargo tarpaulin --out xml
```

### Manual Testing with curl
```bash
# 1. Request password reset
curl -X POST http://localhost:3000/api/v1/auth/password/forgot \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com"}'

# 2. Validate token (UI preview)
curl -X POST http://localhost:3000/api/v1/auth/password/validate \
  -H "Content-Type: application/json" \
  -d '{"token": "TOKEN_FROM_EMAIL"}'

# 3. Reset password
curl -X POST http://localhost:3000/api/v1/auth/password/reset \
  -H "Content-Type: application/json" \
  -d '{"token": "TOKEN_FROM_EMAIL", "new_password": "NewStr0ng!Passw0rd"}'
```

---

## 7. Security Considerations

### Token Storage
- Raw tokens are **never** stored in the database
- Only SHA-256 hashes are stored
- Token entropy: 256 bits (32 bytes × 8)

### Rate Limiting
- Email-based: Prevents abuse of specific accounts
- IP-based: Prevents distributed attacks
- Both use sliding window (1 hour)

### Anti-Enumeration
- Same success response for all emails
- Token created even for non-existent emails
- Timing is not constant (DB queries differ) - this is a known limitation

### Token Lifecycle
1. Generated on request
2. Stored as hash
3. Sent via email (only time raw token exists)
4. Validated via hash comparison
5. Marked as used on first successful reset
6. Expires after 60 minutes

---

## 8. Database Schema

```sql
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash VARCHAR(64) NOT NULL,  -- SHA-256 hash only
    user_id UUID NULL REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    purpose VARCHAR(50) NOT NULL DEFAULT 'password_reset',
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    created_ip INET NULL,
    user_agent TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

*Document Version: 1.0*
*Last Updated: 2026-02-24*
