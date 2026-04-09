# Target Operating Model

**Last updated:** 2026-03-22
**Reference:** LLM Development Operating Model v2.0

---

## 1. Goal

rustchat aims to be a **production-ready, self-hosted team collaboration server** that:

1. Is **fully Mattermost-compatible** for mobile and desktop clients (target: 41/41; currently 39/41 — see Section 3)
2. Is **safe to develop with LLM agents** — every area has clear ownership, risk classification, and scope boundaries
3. Has **high test confidence** — elevated and architectural changes require mandatory test coverage
4. Supports **horizontal scaling** — stateless API servers, Redis pub/sub, connection pooling

---

## 2. Operating Model Applied

The LLM Development Operating Model v2.0 is being applied to rustchat in phases:

| Phase | Description | Status |
|---|---|---|
| Governance layer | `.governance/` policy files, CODEOWNERS, PR template, issue forms, branch protection, GitHub labels | ✅ Complete (2026-03-22) |
| Foundation docs | 7 structured docs: architecture, agent model, compat scope, testing, ownership, repo state, target model | ✅ Complete (2026-03-22) |
| CI enforcement (Approach C) | Automated PR policy checks, agent boundary enforcement, label validation | 🔲 Deferred |
| Incident readiness | Revert policy, hotfix process, post-mortem template, dependency vulnerability SLA | 🔲 Deferred |
| Remaining foundation docs | `migration-roadmap.md`, `release-model.md`, `dependency-policy.md` | 🔲 Deferred |

---

## 3. Compatibility Target

**Current:** 39/41 mobile-critical endpoints (95.1%)

**Target:** 41/41 mobile endpoints + verified desktop client support

**Gap to close:**
- `POST /api/v4/emoji` — custom emoji upload (Phase 2)
- `POST /api/v4/posts/search` — advanced search (Phase 2)

---

## 4. Agent Workflow Target

All agent-driven development follows:
- Spec → plan → subagent execution → two-stage review (spec compliance + code quality) → PR
- No direct commits to protected paths without prior spec
- compat-agent produces analysis artifacts only; production code written by backend-agent with compat-reviewer co-approval

This is enforced by convention today (`.governance/agent-contracts.yml`). Approach C will add automated CI enforcement.

---

## 5. Deferred Items

The following are explicitly out of scope until a future phase:

| Item | Reason deferred |
|---|---|
| Approach C (CI enforcement workflows) | Requires stable policy files first; enforcement added after contracts prove out |
| `docs/migration-roadmap.md` | Requires Phase 2 scope to be defined first |
| `docs/release-model.md` | Current release process is documented in `bump-version.sh`; formal doc deferred |
| `docs/dependency-policy.md` | No Dependabot yet; doc deferred until automation is in place |
| Incident readiness docs | Low priority until production deployment |
