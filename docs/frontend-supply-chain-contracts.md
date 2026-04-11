# Frontend Supply-Chain Contracts

This document captures the contract-style verification for frontend supply-chain hardening.

## Contract A: Lockfile Enforcement

Given:

1. `frontend/package.json` changes
2. `frontend/package-lock.json` is not updated to match

Expected:

1. frontend governance checks fail before merge

Enforced by:

1. `npm ci`
2. `frontend/scripts/check-dependency-policy.mjs`

## Contract B: Script Blocking

Given:

1. a dependency defines lifecycle scripts

Expected:

1. frontend CI does not execute them during install
2. any required follow-up setup runs as an explicit trusted step

Enforced by:

1. `npm ci --ignore-scripts` in frontend CI
2. explicit Playwright browser installation step after install

## Contract C: Dependency Review

Given:

1. a pull request changes frontend dependencies

Expected:

1. dependency deltas are visible in PR checks
2. vulnerable dependency additions are blocked at configured severity
3. direct dependencies without approved metadata are blocked

Enforced by:

1. GitHub dependency review
2. `frontend/scripts/check-dependency-policy.mjs`
3. `npm audit --audit-level=high`

## Contract D: Override Path

Given:

1. a vulnerable transitive dependency can be fixed by version forcing

Expected:

1. the safer version can be pinned centrally without unrelated code changes

Enforced by:

1. `frontend/package.json` `overrides`
2. override metadata in `frontend/dependency-policy.json`

## Contract E: Patch Path

Given:

1. a vulnerable or broken transitive dependency cannot be fixed by override alone

Expected:

1. a repo-managed patch can be applied centrally after install

Enforced by:

1. `frontend/dependency-patches.json`
2. `frontend/patches/`
3. `frontend/scripts/apply-dependency-patches.mjs`

## Contract F: Build Integrity

Given:

1. frontend dependencies are installed via the hardened CI flow

Expected:

1. unit tests still pass
2. the frontend still builds

Enforced by:

1. frontend CI unit-test step
2. frontend CI build step
3. frontend Docker build path using `npm ci --ignore-scripts`

## Contract G: HTTP Boundary Integrity

Given:

1. theme preference HTTP flows are moved behind the internal client boundary

Expected:

1. auth headers remain centralized
2. JSON handling remains centralized
3. 401/error handling stays consistent with the rest of the app

Enforced by:

1. using the internal API client for theme preference requests
2. existing HTTP client contract coverage in `frontend/src/api/http/__tests__/`
