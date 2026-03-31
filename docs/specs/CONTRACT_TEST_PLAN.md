# Contract Test Plan: Fetch Migration

**Version:** 1.1 (Post-Eng-Review)  
**Updated:** Added E2E test requirements

---

## Overview

This document outlines the executable contract tests for the Axios-to-Fetch migration. These tests MUST pass before the migration is considered complete.

## Test Files

### 1. HttpClient.contract.test.ts
**Purpose:** Verify core HTTP client functionality

**Test Cases:**
- [ ] `GET request returns parsed JSON response`
- [ ] `POST request sends JSON body`
- [ ] `PUT request sends JSON body`
- [ ] `PATCH request sends partial JSON body`
- [ ] `DELETE request sends without body`
- [ ] `Base URL is prepended to request URL`
- [ ] `Request headers are merged with defaults`
- [ ] `Response headers are accessible`
- [ ] `Query params are serialized correctly`
- [ ] `Timeout aborts request and throws TimeoutError`
- [ ] `Manual abort signal stops request and throws AbortError`
- [ ] `Empty response body handled gracefully`
- [ ] `Text response type returns raw text`
- [ ] `Blob response type returns blob`

### 2. errors.contract.test.ts
**Purpose:** Verify error normalization (extends AppError)

**Test Cases:**
- [ ] `4xx error throws AppError with status and data`
- [ ] `5xx error throws AppError with status`
- [ ] `Network failure throws NetworkError (existing)`
- [ ] `Timeout throws TimeoutError`
- [ ] `Abort throws AbortError`
- [ ] `Invalid JSON response throws AppError`
- [ ] `Error message is preserved`
- [ ] `Error is instance of AppError and Error`

### 3. querySerializer.contract.test.ts
**Purpose:** Verify query parameter serialization matches Axios behavior

**Test Cases:**
- [ ] `Simple key-value pairs are serialized`
- [ ] `Array values become repeated keys`
- [ ] `Null and undefined values are omitted`
- [ ] `Date values become ISO strings`
- [ ] `Boolean values become string representations`
- [ ] `Object values are JSON stringified`
- [ ] `Special characters are URL encoded`
- [ ] `Empty params result in no query string`

### 4. auth.contract.test.ts
**Purpose:** Verify auth interceptor behavior

**Test Cases:**
- [ ] `Auth token is injected into Authorization header`
- [ ] `Request without token omits Authorization header`
- [ ] `401 response triggers logout`
- [ ] `ID normalization is applied to request params`
- [ ] `ID normalization is applied to request body`
- [ ] `ID normalization is applied to response data`
- [ ] `FormData body is not normalized`
- [ ] `URLSearchParams is not normalized`

### 5. upload.contract.test.ts
**Purpose:** Verify upload with progress functionality

**Test Cases:**
- [ ] `File upload without progress uses fetch`
- [ ] `File upload with progress uses XHR`
- [ ] `Progress callback receives loaded and total bytes`
- [ ] `Upload completes with response data`
- [ ] `Upload failure propagates error`
- [ ] `Upload can be aborted via signal`

### 6. integration.contract.test.ts
**Purpose:** End-to-end API flow tests

**Test Cases:**
- [ ] `Login flow: POST /auth/login`
- [ ] `Get current user: GET /auth/me`
- [ ] `List channels: GET /channels`
- [ ] `Create post: POST /channels/:id/posts`
- [ ] `Upload file with progress: POST /files`
- [ ] `Get posts with pagination params`
- [ ] `Mattermost Calls API: GET /api/v4/plugins/com.mattermost.calls/config`

## E2E Test Files

### 7. auth-flow.e2e.ts
**Purpose:** Full authentication flow

**Test Cases:**
- [ ] `User can log in with valid credentials`
- [ ] `Token is stored and used for subsequent requests`
- [ ] `401 response redirects to login`
- [ ] `Logout clears token and session state`

### 8. file-upload.e2e.ts
**Purpose:** File upload with progress

**Test Cases:**
- [ ] `User can select and upload a file`
- [ ] `Progress callback updates UI during upload`
- [ ] `Upload completes and returns file metadata`
- [ ] `Uploaded file can be attached to message`

### 9. message-send.e2e.ts
**Purpose:** Message composition and send

**Test Cases:**
- [ ] `User can type and send a message`
- [ ] `Message appears optimistically in UI`
- [ ] `Server confirmation updates message state`
- [ ] `Error shows retry option`

### 10. channel-load.e2e.ts
**Purpose:** Channel navigation and pagination

**Test Cases:**
- [ ] `Navigate to channel loads initial messages`
- [ ] `Scroll up loads older messages (pagination)`
- [ ] `Empty channel shows appropriate state`
- [ ] `Loading state shown during fetch`

## Mock Server

Tests use MSW (Mock Service Worker) or vitest fetch mocks to intercept requests.

```typescript
// Example mock setup
const mockFetch = vi.fn()
global.fetch = mockFetch
```

## Running Tests

```bash
# Run all contract tests
npm run test:unit -- src/api/http/*.contract.test.ts

# Run specific contract test
npm run test:unit -- src/api/http/HttpClient.contract.test.ts

# Run E2E tests
npm run test:e2e

# Run specific E2E test
npm run test:e2e -- auth-flow.e2e.ts
```

## Success Criteria

All contract tests must pass:
- 14 HttpClient tests
- 8 Error tests
- 8 QuerySerializer tests
- 8 Auth tests
- 6 Upload tests
- 7 Integration tests

All E2E tests must pass:
- 4 Auth flow tests
- 4 File upload tests
- 4 Message send tests
- 4 Channel load tests

**Total: 55 contract tests + 16 E2E tests = 71 tests**
