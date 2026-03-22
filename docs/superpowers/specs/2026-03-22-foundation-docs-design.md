# Foundation Docs Design

**Date:** 2026-03-22
**Status:** Approved
**Risk Tier:** Standard
**Scope:** `docs/` documentation consolidation — no production code modified

---

## Purpose

Create 7 structured foundation documents that consolidate rustchat's existing ~35 ad-hoc flat files into the format required by the LLM Development Operating Model v2.0. These docs are the primary reference material that LLM agents read when working on the codebase.

---

## Context

Rustchat already has documentation spread across ~35 flat files in `docs/` — mostly ad-hoc implementation summaries, operational guides, and VOIP/mobile implementation logs. None are structured per the operating model. The result is that agents lack reliable, navigable entrypoints for architecture, compatibility scope, testing strategy, and ownership.

The governance layer (`.governance/`, `.github/CODEOWNERS`, issue templates, branch protection) was applied in a prior phase. This phase adds the documentation layer.

---

## Scope

**7 new files** across `docs/`:

```
docs/
├── architecture/
│   └── architecture-overview.md      (new directory + file)
├── adr/
│   └── README.md                     (new directory + stub)
├── decision-notes/
│   └── README.md                     (new directory + stub)
├── agent-operating-model.md          (new file)
├── compatibility-scope.md            (new file)
├── testing-model.md                  (new file)
├── ownership-map.md                  (new file)
├── repo-current-state.md             (new file)
└── target-operating-model.md         (new file)
```

**4 flat files get redirect stubs** (their content is consolidated into new structured docs):
- `docs/architecture.md` → `docs/architecture/architecture-overview.md`
- `docs/websocket_architecture.md` → `docs/architecture/architecture-overview.md`
- `docs/MATTERMOST_CLIENTS.md` → `docs/compatibility-scope.md`
- `docs/mobile-compatibility-matrix.md` → `docs/compatibility-scope.md`

**README.md updated** with a documentation table linking all 7 structured docs.

**What does NOT change:** No existing `.governance/` or `.github/` files, no production code, no CI workflows, no existing flat files other than the 4 stubs listed above.

---

## File Specifications

### `docs/architecture/architecture-overview.md`

Consolidates: `docs/architecture.md`, `docs/websocket_architecture.md`, `docs/calls_deployment_modes.md`

Sections:
1. **System overview** — 4-service diagram (backend, frontend, push-proxy, mm-compat tool), external dependencies table (Postgres, Redis, S3, FCM/APNS, SMTP, WebRTC, OAuth providers)
2. **Backend** — Rust/Axum 0.8, top-level module boundaries (`a2a`, `api`, `auth`, `config`, `db`, `error`, `jobs`, `middleware`, `mattermost_compat`, `models`, `realtime`, `services`, `storage`, `telemetry`), WebSocket hub architecture, Mattermost compat layer position (note: channels/posts are sub-modules under `api/`, not top-level)
3. **Frontend** — Vue 3.5/TypeScript, Pinia state, router structure, E2E test layer
4. **Push proxy** — Rust microservice, FCM/APNS gateway, why separate from main backend
5. **Mattermost compatibility surface** — where the compat layer sits in the stack, what it protects
6. **Data flow** — request lifecycle (client → nginx → backend → DB/Redis), WebSocket event flow
7. **Key design decisions** — 3-5 bullets of decisions that explain the current shape (e.g., why Axum, why separate push proxy, why PostgreSQL)

### `docs/agent-operating-model.md`

Consolidates: synthesized from `AGENTS.md` (root-level canonical doc remains as-is)

This doc structures the operating model for LLM agents working in rustchat. `AGENTS.md` remains the canonical top-level agent guide; this doc provides the structured model-layer view per the operating model format.

Sections:
1. **Agent roles** — three agents: backend-agent, frontend-agent, compat-agent (mirrors `.governance/agent-contracts.yml`)
2. **Skills system** — 7 skills in `.agents/skills/` (ai-summarization-rag, mattermost-api-parity, mm-endpoint-contract-parity, mm-mobile-journey-parity, mm-websocket-calls-parity, production-readiness-gate, user-validation), how they relate to agent contracts
3. **Workflow** — plan-first → spec → plan → implementation cycle; required gates
4. **Compatibility-first rule** — when and why compat review is required
5. **Boundaries summary** — table of what each agent can/cannot do (condenses agent-contracts.yml into human-readable form)
6. **Cross-references** — links to AGENTS.md (full agent guide), `.governance/agent-contracts.yml`, `.agents/skills/`

### `docs/compatibility-scope.md`

Consolidates: `docs/MATTERMOST_CLIENTS.md`, `docs/mobile-compatibility-matrix.md`, `docs/V4_API_GAP_ANALYSIS.md`

**Important distinction for implementer:** These sources measure different scopes. `mobile-compatibility-matrix.md` covers the 41 mobile-critical endpoints (39/41 = 95.1%). `V4_API_GAP_ANALYSIS.md` covers the broader full v4 API surface (~60% coverage). The `compatibility-scope.md` doc must make this distinction explicit — do not flatten them into one number.

Sections:
1. **Compatibility commitment** — what rustchat guarantees to Mattermost clients (Mattermost v5.x+ mobile and desktop clients can connect)
2. **Current coverage** — two scopes: (a) mobile-critical: 39/41 endpoints (95.1%), table of covered vs. missing; (b) full v4 surface: ~60%, broader gap analysis
3. **Protected surface** — paths that must not regress: `backend/src/api/v4/`, `backend/src/mattermost_compat/`, `backend/compat/`
4. **Contract tests** — what the contract JSON schemas cover, how to run compat tests
5. **Gap handling** — how new gaps are tracked, the compat review process
6. **Client configuration** — how to connect Mattermost mobile/desktop clients (from MATTERMOST_CLIENTS.md)

### `docs/testing-model.md`

Source material: CI workflow files, `backend/tests/`, `backend/compat/tests/`, `frontend/e2e/`

Sections:
1. **Test layers** — table: unit tests (none formal), integration tests (backend/tests/), contract tests (backend/compat/tests/), E2E (Playwright)
2. **Backend integration tests** — ~170 tests, require live Postgres/Redis/S3, how to run locally
3. **Compat contract tests** — what they cover, how to run, the golden test concept
4. **Frontend E2E** — Playwright snapshot tests, how to run, how to update snapshots
5. **CI gates** — which tests run in which workflows (backend-ci.yml, compat.yml, frontend-ci.yml)
6. **Test requirements by risk tier** — standard: recommended; elevated: mandatory; architectural: mandatory + design review

### `docs/ownership-map.md`

Source material: `.github/CODEOWNERS`, `.governance/agent-contracts.yml`

Sections:
1. **Area map** — table: path pattern → human owner(s) → agent allowed → risk tier
2. **Dual-owner areas** — compat and governance paths that require @zoorpha co-review
3. **Catchall** — @senolcolak owns everything not explicitly listed
4. **Agent boundaries** — which agent is authorized per area (cross-ref to agent-operating-model.md)
5. **How to update ownership** — CODEOWNERS change is elevated tier, requires 2 reviewers

### `docs/repo-current-state.md`

Sections:
1. **Version** — current: v0.3.5, semver, how to check
2. **Services running** — 4 services, ports, dependencies
3. **Compatibility status** — 39/41 endpoints (95.1%), last updated
4. **Known gaps** — top 5 known missing/incomplete features
5. **Active work streams** — (kept brief, will drift — link to GitHub issues for live state)
6. **Quick start** — how to run the full stack locally (links to `running_environment.md` (stays as-is — operational content, not stubbed))

### `docs/target-operating-model.md`

Sections:
1. **Goal** — what rustchat is trying to become (Mattermost-compatible open-source chat server)
2. **Operating model applied** — governance layer ✅, foundation docs ✅ (this pass), CI enforcement (Approach C, deferred), incident readiness (deferred)
3. **Compatibility target** — 41/41 Mattermost mobile endpoints, desktop client support
4. **Agent workflow target** — all agents work from specs + plans, no ad-hoc changes to protected paths
5. **Deferred items** — what's explicitly out of scope until later (Approach C CI enforcement, migration-roadmap, release-model, dependency-policy)

### `docs/adr/README.md` and `docs/decision-notes/README.md`

Stubs that explain the purpose of each directory and the format for new entries. No content yet — they are placeholders that make the directories tracked by git and explain the convention.

---

## Redirect Stubs

Each superseded flat file gets replaced with a one-line stub:

```markdown
> **Moved:** This document has been consolidated into [`docs/architecture/architecture-overview.md`](architecture/architecture-overview.md).
```

Files getting stubs:
- `docs/architecture.md`
- `docs/websocket_architecture.md`
- `docs/MATTERMOST_CLIENTS.md`
- `docs/mobile-compatibility-matrix.md`

`docs/running_environment.md` does NOT get a stub — it contains Docker/env setup details that are operational, not architectural. It stays as-is but is referenced from `docs/repo-current-state.md`.

---

## README Update

Add a **Documentation** table after the quick start section:

| Document | Purpose |
|---|---|
| [Architecture Overview](docs/architecture/architecture-overview.md) | System components, data flow, key decisions |
| [Agent Operating Model](docs/agent-operating-model.md) | LLM agent roles, skills, workflow, boundaries |
| [Compatibility Scope](docs/compatibility-scope.md) | Mattermost client compatibility commitments and coverage |
| [Testing Model](docs/testing-model.md) | Test layers, CI gates, requirements by risk tier |
| [Ownership Map](docs/ownership-map.md) | Code ownership, review routing, agent boundaries |
| [Repo Current State](docs/repo-current-state.md) | Current version, services, known gaps |
| [Target Operating Model](docs/target-operating-model.md) | Goals, applied model, deferred items |

---

## Implementation Order

1. Create `docs/architecture/` directory, write `architecture-overview.md`
2. Write `docs/agent-operating-model.md`
3. Write `docs/compatibility-scope.md`
4. Write `docs/testing-model.md`
5. Write `docs/ownership-map.md`
6. Write `docs/repo-current-state.md`
7. Write `docs/target-operating-model.md`
8. Create `docs/adr/README.md` and `docs/decision-notes/README.md` stubs
9. Write redirect stubs in the 4 superseded flat files
10. Update README.md with documentation table
11. Commit all changes in one commit

---

## What This Does NOT Do

- Does not modify any production code
- Does not modify CI workflows
- Does not create migration-roadmap, release-model, dependency-policy (deferred)
- Does not implement Approach C (CI enforcement)
- Does not delete any existing flat files — only adds redirect stubs

---

## Success Criteria

- [ ] All 7 structured foundation docs exist with content
- [ ] `docs/architecture/`, `docs/adr/`, `docs/decision-notes/` directories exist
- [ ] 4 flat files have redirect stubs
- [ ] README.md has documentation table
- [ ] No CI workflows broken
- [ ] No production code modified
