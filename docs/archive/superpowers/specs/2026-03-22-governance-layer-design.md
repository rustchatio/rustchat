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
- `.agents/skills/` for domain-specific skill routing (8 skills: mattermost-api-parity, mm-endpoint-contract-parity, mm-websocket-calls-parity, mm-mobile-journey-parity, production-readiness-gate, user-validation, ai-summarization-rag, workflows)
- `docs/superpowers/` for specs and plans
- `backend/compat/` for compatibility contracts and tests
- `backend/src/mattermost_compat/` for compatibility utilities
- `tools/mm-compat/` for compatibility analysis tooling
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

Three role-level agents that sit above the skills system. The `.agents/skills/` directory defines *capability skills* (what an agent can do in a specific domain); these contracts define *scope boundaries* (which paths an agent may touch and under what conditions). The two systems are complementary: a skill tells the agent how to do the work; the contract tells it where it is allowed to do it.

**backend-agent** (bounded-write):
- Role: implements backend features and fixes
- Applicable skills: any non-compat skill
- Allowed: `backend/src/**`, `backend/tests/**`, `backend/migrations/**`, `push-proxy/**`
- Prohibited: `backend/src/auth/**` (without explicit approval), `backend/src/api/v4/**` (without compat-reviewer co-approval), `backend/src/mattermost_compat/**` (without compat-reviewer co-approval), `backend/src/a2a/**` (without senior review), `.governance/**`, `frontend/**`
- Limits: 10 files, 300 lines
- Requires tests: true, human review: true
- ADR required for: auth changes, permission changes, API contract changes

**frontend-agent** (bounded-write):
- Role: implements frontend features and fixes
- Applicable skills: any non-compat skill
- Allowed: `frontend/src/**`, `frontend/e2e/**`
- Prohibited: `backend/**`, `.governance/**`
- Limits: 10 files, 300 lines
- Requires human review: true

**compat-agent** (read-only with bounded analysis output):
- Role: performs Mattermost compatibility analysis and produces design artifacts
- Applicable skills: mattermost-api-parity, mm-endpoint-contract-parity, mm-websocket-calls-parity, mm-mobile-journey-parity
- Can read: `backend/compat/**`, `backend/src/api/v4/**`, `backend/src/mattermost_compat/**`, `tools/mm-compat/**`
- Can write (analysis output only): `previous-analyses/**`, `docs/superpowers/specs/**`, `docs/superpowers/plans/**`
- Cannot: write production code, approve PRs, modify `.governance/**`

### `.governance/protected-paths.yml`

Paths that auto-elevate risk tier when changed:

| Pattern | Min Tier | Reason |
|---|---|---|
| `backend/src/auth/**` | elevated | Authentication is security-critical |
| `backend/src/api/v4/**` | elevated | Mattermost HTTP compatibility surface |
| `backend/src/mattermost_compat/**` | elevated | Mattermost compatibility utilities |
| `backend/src/realtime/**` | elevated | WebSocket event contracts |
| `backend/migrations/**` | elevated | Database schema changes are irreversible |
| `backend/compat/**` | elevated | Compatibility contracts and golden tests |
| `tools/mm-compat/**` | elevated | Compatibility analysis tooling |
| `backend/src/a2a/**` | elevated | Agent-to-agent communication layer |
| `.github/CODEOWNERS` | elevated | Ownership changes affect review routing |
| `.governance/**` | architectural | Policy changes affect the entire model |
| `docs/superpowers/specs/**` | elevated | Design record integrity |

### `.governance/pr-size-limits.yml`

Default limits with per-path exceptions. Exceptions are **additive constraints** on top of the risk-tier limits — a path subject to both must satisfy the stricter rule.

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
    note: "Migrations are irreversible — keep them small and focused; split into multiple PRs if needed"
```

### `.github/CODEOWNERS`

Area-based review routing (GitHub usernames are placeholders — must be replaced with real active GitHub handles before enabling branch protection enforcement; resolve from the repository's maintainer list or team settings):

```
# Backend (Rust API server + push proxy)
/backend/src/                    @<backend-owner>
/backend/src/a2a/                @<backend-owner>
/backend/migrations/             @<backend-owner>
/push-proxy/                     @<backend-owner>

# Frontend (Vue 3 SPA)
/frontend/                       @<frontend-owner>

# Mattermost Compatibility (always requires compat reviewer co-approval)
/backend/compat/                 @<backend-owner> @<compat-reviewer>
/backend/src/api/v4/             @<backend-owner> @<compat-reviewer>
/backend/src/mattermost_compat/  @<backend-owner> @<compat-reviewer>
/tools/mm-compat/                @<backend-owner> @<compat-reviewer>

# CI / Infrastructure
/.github/                        @<infra-owner>
/docker/                         @<infra-owner>
/scripts/                        @<infra-owner>

# Governance (project owner only)
/.governance/                    @<project-owner>

# Docs and agent guidelines
/docs/                           @<project-owner>
/AGENTS.md                       @<project-owner>
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
8. **Agent-generated** — `[ ] yes  [ ] no` — if yes: `Generated-by: <agent-name>`, `Skill used: <skill-name>`

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

Note: `area/*` label is applied during triage, not at submission (GitHub issue forms do not support conditional label assignment based on dropdown value without automation).

### `.github/ISSUE_TEMPLATE/feature.yml`

Fields:
- Title (text)
- Area (dropdown: backend / frontend / compat / push-proxy / infra)
- Problem statement (textarea)
- Proposed behavior (textarea)
- Compatibility impact (dropdown: yes – affects Mattermost clients / no / unknown)
- Acceptance criteria (textarea)
- ADR needed (dropdown: yes / no / unsure)

Auto-labels: `type/feature`

### `.github/ISSUE_TEMPLATE/refactor.yml`

Fields:
- Title (text)
- Area (dropdown: backend / frontend / compat / push-proxy / infra)
- What is wrong today (textarea)
- Proposed improvement (textarea)
- Blast radius — what could break (textarea)
- Risk tier (dropdown: standard / elevated / architectural)

Auto-labels: `type/refactor`

---

## Label Taxonomy

All issue forms auto-apply a `type/*` label at submission. `area/*` and `risk/*` labels are applied during triage (requires a maintainer or CI automation after submission). Labels must be created in the GitHub repository before the forms are used.

**Area labels** (applied during triage):
- `area/backend`
- `area/frontend`
- `area/compat`
- `area/infra`
- `area/docs`

**Risk labels** (applied during triage):
- `risk/standard`
- `risk/elevated`
- `risk/architectural`

**Type labels** (auto-applied by issue form at submission):
- `type/bug`
- `type/feature`
- `type/refactor`

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
- [ ] `.github/CODEOWNERS` maps all major areas including compat paths
- [ ] PR template covers all required fields from the operating model, including skill attribution
- [ ] All 3 issue forms render correctly in GitHub and auto-apply type labels
- [ ] All GitHub Actions workflows that existed before this PR continue to pass
- [ ] No production code is modified
