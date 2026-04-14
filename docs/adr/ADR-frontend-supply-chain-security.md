# ADR: Frontend Supply-Chain Security Model

**Date:** 2026-04-11
**Status:** Accepted
**Risk tier:** architectural

## Context

RustChat's frontend is a Vue 3 SPA with a standard JavaScript dependency graph and build pipeline. The current baseline already has some good properties:

1. The active frontend CI path uses `npm ci` and a committed `frontend/package-lock.json`.
2. The app already ships an internal fetch-based HTTP client in [frontend/src/api/http/HttpClient.ts](../../frontend/src/api/http/HttpClient.ts).
3. The current branch is already removing the stale `frontend/pnpm-lock.yaml`, signaling package-manager normalization toward npm.

But several supply-chain gaps remain:

1. CI still allows package lifecycle scripts during dependency installation.
2. PRs do not have automated dependency review or direct-dependency policy enforcement.
3. The frontend Docker build still falls back to unlocked `npm install`.
4. Vulnerability scanning and machine-readable dependency inventory are not enforced in frontend CI.
5. Remaining raw `fetch()` call sites in theme preference code bypass the internal HTTP boundary.
6. Some direct dependencies appear unused and still expand the review and remediation surface.

This ADR establishes a security model that reduces attack surface without rewriting the Vue stack or adding dependency-management tools just to manage dependencies.

## Decision

### 1. Package manager choice: npm

RustChat will standardize the frontend on **npm** with **one committed lockfile**: `frontend/package-lock.json`.

Reasons:

1. npm is already the active package manager in frontend CI, contributor docs, and Docker build flow.
2. The current branch is already converging away from `pnpm-lock.yaml`.
3. npm provides the features required for this hardening effort:
   - deterministic `npm ci`
   - `overrides` for transitive version control
   - built-in audit support
   - machine-readable dependency tree generation
4. Staying on npm avoids introducing a package-manager migration during a security hardening task whose goal is reduced surprise and smaller operational burden.

### 2. Lockfile enforcement approach

Frontend installs in CI and release-oriented build paths will use **lockfile-only installs**:

1. CI uses `npm ci --ignore-scripts`.
2. The frontend Docker build uses `npm ci --ignore-scripts`.
3. `frontend/package-lock.json` is the single source of truth for resolved frontend dependencies.
4. Additional lockfiles such as `pnpm-lock.yaml` and `yarn.lock` are forbidden in the frontend workspace.
5. A repo-owned policy script validates that `package.json` and `package-lock.json` stay synchronized.

### 3. CI install policy

Frontend CI will enforce a **hardened install pipeline**:

1. install from lockfile only
2. block lifecycle scripts by default
3. apply only trusted, repo-owned dependency patches after install if configured
4. run dependency policy validation
5. run vulnerability scanning
6. run unit tests and build only after the hardened dependency phase passes

### 4. Lifecycle script policy

Lifecycle scripts are **blocked in CI by default** and remain **allowed locally**.

Decision:

1. CI-only script blocking is the default and enforced path.
2. Local development keeps standard npm behavior to avoid surprising contributors and to preserve normal package workflows.
3. If a tool needs post-install behavior in CI, that behavior must be invoked as an explicit trusted step after install, not as an implicit dependency lifecycle hook.

Example:

1. Playwright browser binaries are installed via explicit `npx playwright install chromium` in CI after dependency installation, not through dependency lifecycle execution.

### 5. Dependency review tooling

Frontend dependency changes are governed by **three layers**:

1. **GitHub dependency review** on pull requests for package and lockfile deltas.
2. **Repo-owned dependency policy validation** against a machine-readable allowlist in `frontend/dependency-policy.json`.
3. **`npm audit` high-severity gate** in frontend CI.

This combination makes dependency changes:

1. visible in PRs
2. attributable to an approved category and rationale
3. blocked when high-severity vulnerabilities are introduced

### 6. Override and patch mechanism

RustChat will use a **two-step transitive remediation model**:

1. **`package.json` `overrides` first**
   - lowest-friction path
   - works well when a safe transitive version can be forced centrally
2. **repo-managed patch manifest second**
   - used when version forcing is insufficient
   - patches are stored in `frontend/patches/`
   - patch metadata is stored in `frontend/dependency-patches.json`
   - patches are applied by a trusted repo script after install

This avoids adding a new package like `patch-package` just to patch dependencies.

### 7. Update automation policy

Frontend dependency updates will be **PR-based and reviewable**:

1. Dependabot manages frontend npm update PRs.
2. Minor and patch updates may be grouped for review efficiency.
3. Major version updates are not auto-grouped for routine rollout and are never auto-merged by policy.
4. Security updates remain distinct from ordinary feature/tooling upgrades and must still pass the same CI gates.

### 8. HTTP layer decision: internal fetch wrapper over third-party client

RustChat will keep the existing **internal fetch-based HTTP client** and migrate remaining raw preference-related `fetch()` calls behind internal APIs where practical.

Decision:

1. Do not add or reintroduce a third-party HTTP client.
2. Reuse the existing internal client and centralized error/auth handling.
3. Keep external package access behind internal application boundaries where reasonable.

## Alternatives Considered

### npm vs pnpm

**npm**

Pros:

1. already active in CI, Docker, and contributor docs
2. no migration cost during hardening
3. sufficient support for lockfile enforcement, overrides, audit, and inventory generation

Cons:

1. weaker store isolation than pnpm
2. historically more permissive defaults unless CI is hardened explicitly

**pnpm**

Pros:

1. excellent disk deduplication and stricter node_modules layout
2. strong support for patching and overrides

Cons:

1. would require a package-manager migration during a security hardening task
2. would increase contributor surprise because the repo already runs npm in CI
3. would re-open workflow normalization work that the current branch is already resolving in the opposite direction

Decision:

1. Choose **npm** because contributor clarity and low migration risk matter more here than pnpm's structural advantages.

### Internal fetch wrapper vs third-party HTTP client

**Internal fetch wrapper**

Pros:

1. smaller attack surface
2. no additional runtime dependency
3. already exists in the repo
4. enough functionality for auth headers, JSON handling, cancellation, retries, and upload progress

Cons:

1. maintenance burden stays in-repo
2. some edge behavior must be tested explicitly

**Third-party HTTP client**

Pros:

1. rich feature set out of the box
2. familiar abstractions for many contributors

Cons:

1. adds another supply-chain dependency to solve a supply-chain problem
2. duplicates capabilities already present in the repo
3. increases transitive exposure and upgrade burden

Decision:

1. Keep the **internal fetch wrapper** and tighten remaining call sites around it.

### Strict CI-only script blocking vs global script blocking

**CI-only blocking**

Pros:

1. hardens the untrusted environment where code runs automatically
2. avoids surprising local development workflows
3. preserves normal package installation ergonomics for contributors

Cons:

1. local installs can still execute dependency scripts

**Global blocking**

Pros:

1. strongest default everywhere

Cons:

1. higher contributor surprise
2. more friction for common tooling workflows
3. likely to create ad hoc bypasses instead of clean explicit exceptions

Decision:

1. Use **CI-only script blocking by default** and require explicit trusted follow-up steps where needed.

### Override-only vs override + patch workflow

**Override-only**

Pros:

1. simple and low-maintenance
2. uses built-in npm support

Cons:

1. cannot help when the vulnerable or broken code must be edited directly
2. blocks emergency response when no safe upstream release exists

**Override + patch workflow**

Pros:

1. supports emergency remediation without waiting on upstream
2. still keeps overrides as the default first-line response
3. can be implemented with repo-owned scripts instead of new helper packages

Cons:

1. additional documentation and process
2. patched dependencies require retirement discipline

Decision:

1. Use **override + patch**, with overrides first and patches as the documented fallback.

## Consequences

### Positive

1. Frontend installs become deterministic and non-scripted in CI.
2. Dependency changes become reviewable, attributable, and easier to audit.
3. Transitive vulnerability response becomes faster because overrides and patches are both available.
4. The app avoids adding a new HTTP client or dependency-management helper package.
5. Contributor guidance becomes clearer because the frontend package-manager story is singular.

### Negative / Trade-offs

1. CI may surface latent assumptions in packages that previously relied on lifecycle scripts.
2. Policy metadata adds light maintenance overhead whenever a direct dependency is added or removed.
3. Repo-managed patches are operationally more manual than using a dedicated patching package.

### Follow-up constraints

1. New frontend dependencies must be added to `frontend/dependency-policy.json` with an approved category and reason.
2. Every override and patch must include a removal condition.
3. Frontend release/build paths must continue to use the same hardened npm trust model instead of drifting back to unlocked installs.
