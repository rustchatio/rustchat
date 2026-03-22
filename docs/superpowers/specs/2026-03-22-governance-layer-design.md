# Governance Layer Design

**Date:** 2026-03-22
**Status:** Approved
**Risk Tier:** Standard
**Scope:** `.governance/` policy files + `.github/` contributor controls

---

## Purpose

Apply the LLM Development Operating Model (`../000/llm-operating-model`) governance layer to rustchat. This delivers the full GitHub control plane — machine-readable policy contracts, review routing, PR quality enforcement, and structured issue intake — without modifying any production code or CI behavior.

---

## Context

The `../000/llm-development-operating-model.md` defines a reusable governance model for LLM-assisted repositories. Rustchat already has:
- Detailed `AGENTS.md` with plan-first and compatibility-first workflows
- `.agents/skills/` for domain-specific skill routing
- `docs/superpowers/` for specs and plans
- `backend/compat/` for compatibility tracking
- Active CI workflows

What is missing is the **control plane layer**: machine-readable contracts that define risk, ownership, agent scope, and contribution boundaries.

---

## Scope (Approach B — Full GitHub Control Plane)

10 new files. No existing files modified. No behavior changes.

```
rustchat/
├── .governance/
│   ├── risk-tiers.yml
│   ├── agent-contracts.yml
│   ├── protected-paths.yml
│   └── pr-size-limits.yml
└── .github/
    ├── CODEOWNERS
    ├── pull_request_template.md
    └── ISSUE_TEMPLATE/
        ├── config.yml
        ├── bug.yml
        ├── feature.yml
        └── refactor.yml
```

---

## File Specifications

### `.governance/risk-tiers.yml`

Three-tier classification system:

| Tier | Description | Files | Lines | Reviewers | Tests | ADR |
|---|---|---|---|---|---|---|
| `standard` | Typos, UI polish, test additions, internal refactors, CI improvements | ≤10 | ≤300 | 1 | Recommended | No |
| `elevated` | Auth, permissions, API behavior, schema/migrations, compat-sensitive paths | ≤10 | ≤300 | 1 senior | Mandatory | No (decision note required) |
| `architectural` | Architecture redesign, storage model, protocol, security model | — | — | 2 senior | Mandatory | Yes + design review |

### `.governance/agent-contracts.yml`

Three agents reflecting rustchat's existing skill system:

**backend-agent** (bounded-write):
- Allowed: `backend/src/**`, `backend/tests/**`, `backend/migrations/**`
- Prohibited: `backend/src/auth/**` (without explicit approval), `.governance/**`, `frontend/**`
- Limits: 10 files, 300 lines
- Requires tests: true, human review: true
- ADR required for: auth changes, permission changes, API contract changes

**frontend-agent** (bounded-write):
- Allowed: `frontend/src/**`, `frontend/e2e/**`
- Prohibited: `backend/**`, `.governance/**`
- Limits: 10 files, 300 lines
- Requires human review: true

**compat-agent** (read-only):
- Scope: Inspect `backend/compat/**`, `backend/src/api/v4/**`, produce analysis docs
- Cannot: make code changes, approve PRs

### `.governance/protected-paths.yml`

Paths that auto-elevate risk tier when changed:

| Pattern | Min Tier | Reason |
|---|---|---|
| `backend/src/auth/**` | elevated | Authentication is security-critical |
| `backend/src/api/v4/**` | elevated | Mattermost compatibility surface |
| `backend/migrations/**` | elevated | Database schema changes are irreversible |
| `backend/src/realtime/**` | elevated | WebSocket event contracts |
| `.github/CODEOWNERS` | elevated | Ownership changes affect review routing |
| `.governance/**` | architectural | Policy changes affect the entire model |
| `docs/superpowers/specs/**` | elevated | Design record integrity |

### `.governance/pr-size-limits.yml`

Default limits with exceptions:

```yaml
default:
  max_files: 10
  max_lines_changed: 300
exceptions:
  - path: "backend/tests/**"
    max_lines_changed: 500
  - path: "frontend/e2e/**"
    max_lines_changed: 500
  - path: "docs/**"
    max_lines_changed: 600
  - path: "backend/migrations/**"
    max_lines_changed: 150
    note: "Migration files must be small and focused"
```

### `.github/CODEOWNERS`

Area-based review routing (GitHub usernames are placeholders, must be replaced before enabling branch protection):

```
# Backend (Rust API server)
/backend/src/           @<backend-owner>
/backend/migrations/    @<backend-owner>
/push-proxy/            @<backend-owner>

# Frontend (Vue 3 SPA)
/frontend/              @<frontend-owner>

# Mattermost Compatibility (always requires elevated reviewer)
/backend/compat/        @<backend-owner> @<compat-reviewer>
/backend/src/api/v4/    @<backend-owner> @<compat-reviewer>

# CI / Infrastructure
/.github/               @<infra-owner>
/docker/                @<infra-owner>
/scripts/               @<infra-owner>

# Governance (project owner only)
/.governance/           @<project-owner>

# Docs
/docs/                  @<project-owner>
/AGENTS.md              @<project-owner>
```

### `.github/pull_request_template.md`

Fields:
1. **Problem** — what issue does this solve?
2. **Area** — checkbox: `[ ] backend  [ ] frontend  [ ] compat  [ ] infra  [ ] docs`
3. **Public behavior change** — `[ ] yes  [ ] no` — if yes, describe
4. **Risk tier** — `[ ] standard  [ ] elevated  [ ] architectural`
5. **Tests** — `[ ] added  [ ] updated  [ ] not applicable` — explain if not applicable
6. **Docs / ADR** — `[ ] updated  [ ] ADR created  [ ] not needed`
7. **If elevated or architectural** — decision note (3–5 lines explaining what was decided and why)
8. **Agent-generated** — `[ ] yes (Generated-by: <agent-name>)  [ ] no`

### `.github/ISSUE_TEMPLATE/config.yml`

- Disables blank issue creation
- Links to GitHub Discussions for questions/support

### `.github/ISSUE_TEMPLATE/bug.yml`

Fields:
- Title (text)
- Area (dropdown: backend / frontend / compat / push-proxy / infra)
- Current behavior (textarea)
- Expected behavior (textarea)
- Steps to reproduce (textarea)
- Compatibility impact (dropdown: yes – affects Mattermost clients / no / unknown)
- Risk tier suggestion (dropdown: standard / elevated)

Auto-labels: `type/bug`

### `.github/ISSUE_TEMPLATE/feature.yml`

Fields:
- Title (text)
- Area (dropdown)
- Problem statement (textarea)
- Proposed behavior (textarea)
- Compatibility impact (dropdown)
- Acceptance criteria (textarea)
- ADR needed (dropdown: yes / no / unsure)

Auto-labels: `type/feature`

### `.github/ISSUE_TEMPLATE/refactor.yml`

Fields:
- Title (text)
- Area (dropdown)
- What is wrong today (textarea)
- Proposed improvement (textarea)
- Blast radius — what could break (textarea)
- Risk tier (dropdown: standard / elevated / architectural)

Auto-labels: `type/refactor`

---

## Label Taxonomy

All issue forms auto-apply labels. The full taxonomy (labels must be created in GitHub):

**Area labels** (one required):
- `area/backend`
- `area/frontend`
- `area/compat`
- `area/infra`
- `area/docs`

**Risk labels** (one required):
- `risk/standard`
- `risk/elevated`
- `risk/architectural`

**Type labels** (one required, set by issue form):
- `type/bug`
- `type/feature`
- `type/refactor`
- `type/test`

**Special labels**:
- `agent-generated`
- `needs-adr`
- `good-first-issue`

---

## What This Does NOT Do

- Does not modify any CI workflows (no enforcement automation — that is Approach C)
- Does not restructure `docs/`
- Does not move any code
- Does not change any production behavior
- CODEOWNERS will not auto-enforce until branch protection is configured (done separately)

---

## Implementation Order

1. `.governance/` directory and 4 YAML files (risk-tiers, agent-contracts, protected-paths, pr-size-limits)
2. `.github/CODEOWNERS` (with placeholder usernames)
3. `.github/pull_request_template.md`
4. `.github/ISSUE_TEMPLATE/config.yml`
5. `.github/ISSUE_TEMPLATE/bug.yml`, `feature.yml`, `refactor.yml`

Single commit, single PR. Risk tier: **standard** (governance/docs only, no behavior change).

---

## Success Criteria

- [ ] `.governance/` directory exists with all 4 policy files populated for rustchat
- [ ] `.github/CODEOWNERS` maps all major areas
- [ ] PR template covers all required fields from the operating model
- [ ] All 3 issue forms work and auto-apply correct labels
- [ ] No existing CI workflows are broken
- [ ] No production code is modified
