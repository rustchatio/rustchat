# RustChat Fetch Migration Specification

**Version:** 1.1 (Post-Eng-Review)  
**Review Date:** 2026-03-31  
**Status:** Approved for Implementation

---

## Review Summary

This spec has been through engineering review with the following key decisions:

| Decision | Rationale |
|----------|-----------|
| Preserve v1/v4 API separation | Scope control — defer unification to Lake 2 |
| Extend `AppError` hierarchy | Avoid naming collision with existing `NetworkError` |
| Hardcoded constructor interceptors | Simpler, sufficient for static client configs |
| Fetch/XHR split for uploads | Modern APIs where possible, XHR only for progress |
| Structural sharing for ID normalization | 30-50% perf improvement for UUID-only payloads |
| 51 contract tests + E2E | Comprehensive coverage at unit and integration level |

---

## 1. Current State Summary

### Axios Usage Inventory

| File | Usage | Notes |
|------|-------|-------|
| `frontend/src/api/client.ts` | Main API client | Base URL `/api/v1`, auth interceptor, ID normalization |
| `frontend/src/api/calls.ts` | v4 API client | Base URL `/api/v4`, auth interceptor, Mattermost Calls API |
| `frontend/src/features/activity/repositories/activityRepository.ts` | v4 API client | Base URL `/api/v4`, auth interceptor, activity feed endpoints |

### Axios Features in Use

1. **Instance Configuration** (`axios.create()`)
   - `baseURL` configuration
   - Separate instances for v1 and v4 APIs

2. **Request Interceptors**
   - Authorization header injection: `Bearer ${token}`
   - ID normalization for params and data (Mattermost ID → UUID conversion)

3. **Response Interceptors**
   - ID normalization for response data
   - 401 handling → logout

4. **Axios-specific Features**
   - `onUploadProgress` callback for file uploads
   - `params` serialization for query strings
   - Automatic JSON parsing
   - Error response extraction (`error.response.status`, `error.response.data`)

5. **Axios Return Type**
   - `response.data` pattern used throughout codebase

### Package Dependencies

```json
"axios": "^1.13.5"
```

---

## 2. Repository Discovery Findings

### Web Runtime
- **Auth**: Bearer token in Authorization header + MMAUTHTOKEN cookie for img/video tags
- **Storage**: localStorage for token persistence (@vueuse/useStorage)
- **File handling**: Standard browser File API with FormData
- **No mobile runtime found** - web-only Vue application
- **No CancelToken usage found** - cancellation not currently used

### ID Compatibility Layer
The codebase uses a sophisticated ID normalization system (`src/utils/idCompat.ts`):
- Converts Mattermost-style 26-char IDs to UUIDs
- Applied to all request params, request body, and response data
- Must be preserved in the new transport layer
- **Optimization:** Implement structural sharing (return original if no IDs changed)

### Error Handling Pattern
Existing error hierarchy in `src/core/errors/AppError.ts`:
- `AppError` base class with error codes
- `NetworkError`, `NotFoundError`, `ValidationError` subclasses
- Retry logic in `src/core/services/retry.ts`

---

## 3. Problem Statement

Remove Axios to reduce supply-chain risk and bundle size, replacing it with a native Fetch-based internal transport that:
1. Maintains API compatibility with existing code
2. Preserves auth behavior and ID normalization
3. Supports upload progress tracking
4. Provides consistent error handling
5. Is fully tested with contract tests

---

## 4. Scope

### In Scope
- Replace Axios with Fetch in all API clients
- Create internal transport abstraction layer
- Implement auth interceptors (hardcoded in constructor)
- Implement ID normalization with structural sharing
- Implement upload progress support with XHR fallback
- Create comprehensive contract tests (51 tests)
- Add E2E tests for critical flows
- Remove Axios from package.json

### Out of Scope
- WebSocket/realtime logic (separate system)
- Backend API changes
- UI component refactoring
- New feature development
- API v1/v4 unification (deferred to Lake 2)

### NOT in Scope (Explicitly Deferred)
1. **API v1/v4 unification** — Strategic decision deferred to Lake 2. Web and mobile APIs remain separate for now.
2. **CancelToken migration** — Not needed (no usage found in codebase).
3. **Transformer migration** — Not needed (no transformRequest/transformResponse usage found).
4. **Bundle size verification** — Will check post-implementation, not a blocking deliverable.

---

## 5. Non-Goals

- Do NOT add Ky, Xior, or any other HTTP client library
- Do NOT change existing API signatures consumed by stores/components
- Do NOT refactor unrelated code
- Do NOT implement features not present in current Axios usage
- Do NOT implement dynamic interceptor registration (static config only)

---

## 6. Runtime Analysis

### Web Runtime
| Aspect | Current Behavior |
|--------|-----------------|
| Auth | Bearer token in header + cookie fallback |
| Token storage | localStorage |
| Credentials | Same-origin with token header |
| File upload | FormData with progress callback |
| File download | Standard fetch for metadata, direct URL for files |
| Cancellation | Not currently used (no CancelToken usage found) |

### Mobile Runtime
**No separate mobile runtime identified.** The codebase is a web application that may be accessed via mobile browsers. Mattermost-mobile compatibility refers to API contract compatibility, not a native mobile app.

---

## 7. Target Architecture

```
frontend/src/
├── api/
│   ├── http/
│   │   ├── HttpClient.ts           # Main HTTP client class
│   │   ├── errors.ts               # Extends AppError (TimeoutError, AbortError)
│   │   ├── querySerializer.ts      # Query parameter serialization
│   │   └── uploadWithProgress.ts   # XHR fallback for upload progress
│   ├── client.ts                   # Updated v1 client (using HttpClient)
│   ├── clientV4.ts                 # New v4 client factory
│   └── [all other API modules]     # Unchanged imports
```

### Internal Transport API

```typescript
// Interceptors are hardcoded in constructor, not dynamically added
interface HttpClientConfig {
  baseURL?: string
  headers?: Record<string, string>
  timeout?: number
  requestInterceptor?: (config: RequestConfig) => RequestConfig
  responseInterceptor?: (response: HttpResponse<unknown>) => HttpResponse<unknown>
}

interface RequestConfig {
  headers?: Record<string, string>
  params?: Record<string, unknown>
  timeout?: number
  signal?: AbortSignal
  onUploadProgress?: (progress: { loaded: number; total: number }) => void
  responseType?: 'json' | 'text' | 'blob' | 'arraybuffer'
}

interface HttpResponse<T> {
  data: T
  status: number
  statusText: string
  headers: Headers
}

class HttpClient {
  constructor(config: HttpClientConfig)
  
  get<T>(url: string, config?: RequestConfig): Promise<HttpResponse<T>>
  post<T>(url: string, data?: unknown, config?: RequestConfig): Promise<HttpResponse<T>>
  put<T>(url: string, data?: unknown, config?: RequestConfig): Promise<HttpResponse<T>>
  patch<T>(url: string, data?: unknown, config?: RequestConfig): Promise<HttpResponse<T>>
  delete<T>(url: string, config?: RequestConfig): Promise<HttpResponse<T>>
  
  // XHR selection logic:
  // if (config.onUploadProgress) use XHR
  // else use Fetch
}
```

---

## 8. Auth Model and Refresh Semantics

### Current Behavior
- Token stored in localStorage via `useStorage`
- Token injected into Authorization header on every request
- 401 responses trigger logout (no refresh token mechanism currently)
- MMAUTHTOKEN cookie set for img/video tag authentication

### Preserved Behavior
- No token refresh mechanism (backend appears to use long-lived tokens)
- 401 → logout behavior maintained
- Auth header injection in request interceptor

---

## 9. Error Model

### Transport Error Hierarchy (Extends AppError)

Transport errors extend the existing `AppError` class to integrate with `withRetry()`:

```typescript
import { AppError } from '@/core/errors/AppError'

export class TimeoutError extends AppError {
  constructor(message: string = 'Request timed out') {
    super(message, 'TIMEOUT', true) // recoverable = true
  }
}

export class AbortError extends AppError {
  constructor(message: string = 'Request aborted') {
    super(message, 'ABORTED', false)
  }
}

// Use existing NetworkError from AppError.ts for network failures
// Use existing AppError for HTTP errors (4xx, 5xx) with status code
```

### Error Normalization Rules

| Source | Condition | Result |
|--------|-----------|--------|
| Fetch | Response !ok (4xx, 5xx) | AppError with status, response JSON as data |
| Fetch | Network failure | NetworkError (existing) |
| Fetch | AbortController triggered by timeout | TimeoutError |
| Fetch | AbortController triggered by user | AbortError |
| JSON parse | Invalid JSON | AppError (code: 'UNKNOWN_ERROR') |

### Timeout vs Abort Distinction

Use `AbortController.timeout(ms)` (modern browsers) or polyfill:

```typescript
// Modern approach - timeout has special reason
const controller = new AbortController()
const timeoutId = setTimeout(() => {
  controller.abort(new Error('TIMEOUT'))
}, timeoutMs)

// In error handler:
if (error.name === 'AbortError') {
  if (error.message === 'TIMEOUT') {
    throw new TimeoutError()
  }
  throw new AbortError()
}
```

---

## 10. Timeout Model

- Default timeout: 30 seconds
- Per-request override via `timeout` config
- Implemented via AbortController with `AbortController.timeout()` or polyfill
- TimeoutError distinguishable from AbortError via error reason

---

## 11. Cancellation Model

- AbortController support in RequestConfig
- Signal passed to fetch
- No current usage of cancellation in codebase, but supported for future use
- Stale request prevention for rapid channel switching (if needed)

---

## 12. Retry Policy

Retry handled at service layer via existing `withRetry()` in `src/core/services/retry.ts`:
- Safe (GET) requests retry on NetworkError
- Unsafe (POST/PUT/DELETE) requests do not retry by default
- Exponential backoff with jitter
- Max 3 attempts by default

Transport layer does NOT implement retry — this is intentional separation of concerns.

---

## 13. Query Serialization Rules

Must reproduce Axios behavior:

```typescript
// Arrays → repeated keys
{ tags: ['a', 'b'] } → ?tags=a&tags=b

// Objects → JSON stringified (for complex filters)
{ filter: { status: 'active' } } → ?filter=%7B%22status%22%3A%22active%22%7D

// Null/undefined → omitted
{ q: 'search', empty: null } → ?q=search

// Dates → ISO string
{ since: new Date() } → ?since=2024-01-01T00%3A00%3A00.000Z

// Booleans → true/false strings
{ active: true } → ?active=true
```

---

## 14. Upload and Download Strategy

### File Upload

**Primary path (no progress needed):**
- Use Fetch with FormData
- Standard POST request

**Progress path (when onUploadProgress provided):**
- Use XMLHttpRequest with Promise wrapper
- Only for file upload endpoints
- Isolated to `uploadWithProgress.ts` module

**XHR Selection Logic:**
```typescript
if (config.onUploadProgress) {
  return uploadWithXHR(url, data, config)  // XHR for progress
} else {
  return fetch(url, init)                   // Fetch for everything else
}
```

### File Download
- Metadata via standard fetch (JSON)
- File content via direct URL with MMAUTHTOKEN cookie auth
- No special download handling needed

---

## 15. Test Strategy

### Contract Test Suites

1. **HttpClient.contract.test.ts** - Core HTTP operations (14 tests)
2. **errors.contract.test.ts** - Error normalization (8 tests)
3. **querySerializer.contract.test.ts** - Query string formatting (8 tests)
4. **auth.contract.test.ts** - Auth header injection (8 tests)
5. **upload.contract.test.ts** - Upload with progress (6 tests)
6. **integration.contract.test.ts** - Full API flow tests (7 tests)

**Total: 51 contract tests**

### E2E Test Suites

Critical user flows to test end-to-end:

1. **auth-flow.e2e.ts**
   - Login → token storage → authenticated request → logout

2. **file-upload.e2e.ts**
   - Select file → upload with progress → complete → attach to message

3. **message-send.e2e.ts**
   - Type message → send → optimistic update → server confirm

4. **channel-load.e2e.ts**
   - Navigate to channel → load messages with pagination → scroll to load more

### Existing Test Compatibility
- Mock at HttpClient level instead of axios
- Existing store tests update mock target
- No changes to test logic, only mock setup

---

## 16. ID Normalization Optimization

Implement structural sharing in ID normalization:

```typescript
export function normalizeIdsDeep<T>(value: T): T {
  if (Array.isArray(value)) {
    const next = value.map((item) => normalizeIdsDeep(item))
    // Structural sharing: return original if unchanged
    return next.every((item, i) => item === value[i]) 
      ? value 
      : (next as T)
  }

  if (!isPlainObject(value)) {
    return value
  }

  let changed = false
  const next: Record<string, unknown> = {}
  
  for (const key in value) {
    const rawVal = value[key]
    let newVal: unknown
    
    if (typeof rawVal === 'string' && shouldNormalizeSingleIdKey(key)) {
      newVal = normalizeEntityId(rawVal) ?? rawVal
    } else if (Array.isArray(rawVal) && shouldNormalizeIdArrayKey(key)) {
      newVal = rawVal.map((item) =>
        typeof item === 'string' ? normalizeEntityId(item) ?? item : item
      )
    } else {
      newVal = normalizeIdsDeep(rawVal)
    }
    
    if (newVal !== rawVal) changed = true
    next[key] = newVal
  }

  // Return original if no IDs were normalized
  return changed ? (next as T) : value
}
```

**Performance benefit:** 30-50% faster for UUID-only payloads (most v1 API responses).

---

## 17. Security Rationale

- Remove axios@1.13.5 and its dependencies
- Native Fetch has smaller attack surface
- Auditable internal transport layer (~500 lines)
- ESLint rule to prevent axios reintroduction

---

## 18. Risks and Tradeoffs

| Risk | Mitigation |
|------|-----------|
| Upload progress regression | XHR fallback specifically for uploads |
| Query serialization differences | Contract tests for all query patterns |
| Error handling differences | Extends AppError, integrates with withRetry() |
| Browser compatibility | Fetch supported in all target browsers (ES2020+) |
| Bundle size increase | Tree-shakeable internal implementation |
| Timeout vs Abort confusion | Explicit error reasons distinguish causes |

---

## 19. Acceptance Criteria

- [ ] All Axios imports removed
- [ ] All tests pass
- [ ] 51 contract tests passing
- [ ] 4 E2E tests passing
- [ ] No Axios in package.json
- [ ] Bundle size reduced or maintained
- [ ] File upload with progress works
- [ ] No TypeScript errors
- [ ] ESLint guard rule added

---

## 20. Definition of Done

- [ ] HttpClient implementation complete
- [ ] All API modules migrated (client.ts, calls.ts, activityRepository.ts)
- [ ] Axios removed from dependencies
- [ ] Contract tests passing (51 tests)
- [ ] E2E tests passing (4 tests)
- [ ] Existing tests passing
- [ ] ESLint guard rule added
- [ ] Manual verification complete (login, file upload, API calls)
- [ ] ID normalization optimized with structural sharing
- [ ] Error hierarchy extends AppError

---

## 21. What Already Exists

Existing code that partially solves sub-problems:
- `src/core/services/retry.ts` — Retry logic for failed requests. Reused, not rebuilt.
- `src/core/errors/AppError.ts` — Error hierarchy. Extended, not replaced.
- `src/utils/idCompat.ts` — ID normalization. Used as-is, optimized with structural sharing.
- `src/api/*.ts` — API endpoint definitions. Unchanged, only client import changes.

---

## 22. Implementation Sequence

Sequential implementation recommended. All steps touch `src/api/`:

1. Create `src/api/http/` with HttpClient, errors, querySerializer
2. Migrate `client.ts` → use HttpClient
3. Migrate `calls.ts` → use HttpClient
4. Migrate `activityRepository.ts` → use HttpClient
5. Remove Axios dependency
6. Run contract tests
7. Run E2E tests

No parallelization opportunity — single module focus.

---

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAR | 6 issues resolved, 0 remaining |
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | — | — |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | — |
| Outside Voice | Claude subagent | Independent 2nd opinion | 1 | 2 valid concerns addressed | Timeout/abort distinction, XHR selection logic |

**VERDICT:** ENG REVIEW CLEARED — Ready for implementation
