# Governance Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the full GitHub control plane for rustchat — `.governance/` policy YAML files, `.github/CODEOWNERS`, PR template, and issue forms — exactly as specified in the design doc.

**Architecture:** 10 new files across `.governance/` and `.github/ISSUE_TEMPLATE/`. No existing files are modified. No production code is touched. All files are configuration/documentation only.

**Tech Stack:** YAML (GitHub issue forms, policy files), Markdown (PR template, CODEOWNERS)

**Spec:** `docs/superpowers/specs/2026-03-22-governance-layer-design.md`

---

## File Map

| File | Action | Purpose |
|---|---|---|
| `.governance/risk-tiers.yml` | Create | Three-tier risk classification |
| `.governance/agent-contracts.yml` | Create | Agent scope boundary contracts |
| `.governance/protected-paths.yml` | Create | Paths that auto-elevate risk tier |
| `.governance/pr-size-limits.yml` | Create | PR size rules with per-path exceptions |
| `.github/CODEOWNERS` | Create | Area → reviewer routing |
| `.github/pull_request_template.md` | Create | PR quality checklist |
| `.github/ISSUE_TEMPLATE/config.yml` | Create | Disable blank issues |
| `.github/ISSUE_TEMPLATE/bug.yml` | Create | Bug report form |
| `.github/ISSUE_TEMPLATE/feature.yml` | Create | Feature request form |
| `.github/ISSUE_TEMPLATE/refactor.yml` | Create | Refactor/tech-debt form |

---

## Task 1: Create `.governance/` policy files

**Files:**
- Create: `.governance/risk-tiers.yml`
- Create: `.governance/agent-contracts.yml`
- Create: `.governance/protected-paths.yml`
- Create: `.governance/pr-size-limits.yml`

- [ ] **Step 1: Create `.governance/` directory**

```bash
mkdir -p .governance
```

- [ ] **Step 2: Create `.governance/risk-tiers.yml`**

```yaml
# Risk Tiers Configuration
# Reference: docs/superpowers/specs/2026-03-22-governance-layer-design.md

tiers:
  standard:
    description: "Typos, UI polish, test additions, internal refactors, CI improvements"
    max_files: 10
    max_lines: 300
    required_reviewers: 1
    senior_review: false
    tests_required: false
    decision_note_required: false
    adr_required: false

  elevated:
    description: "Auth, permissions, API behavior, schema/migrations, compat-sensitive paths"
    max_files: 10
    max_lines: 300
    required_reviewers: 1
    senior_review: true
    tests_required: true
    decision_note_required: true
    adr_required: false

  architectural:
    description: "Architecture redesign, storage model, protocol, security model changes"
    required_reviewers: 2
    senior_review: true
    tests_required: true
    decision_note_required: true
    adr_required: true
    design_review: true
```

- [ ] **Step 3: Create `.governance/agent-contracts.yml`**

```yaml
# Agent Boundary Contracts
# Reference: docs/superpowers/specs/2026-03-22-governance-layer-design.md
#
# These define SCOPE BOUNDARIES (which paths agents may touch).
# The .agents/skills/ directory defines CAPABILITY SKILLS (what agents can do in a domain).
# These two systems are complementary: skills tell the agent HOW; contracts define WHERE.

agents:
  - name: backend-agent
    mode: bounded-write
    description: "Implements backend features and fixes"
    applicable_skills:
      - any non-compat skill
    allowed_paths:
      - backend/src/**
      - backend/tests/**
      - backend/migrations/**
      - push-proxy/**
    prohibited_paths:
      - backend/src/auth/**                # requires explicit approval
      - backend/src/api/v4/**              # requires compat-reviewer co-approval
      - backend/src/mattermost_compat/**   # requires compat-reviewer co-approval
      - backend/src/a2a/**                 # requires senior review
      - .governance/**
      - frontend/**
    max_files_changed: 10
    max_lines_changed: 300
    requires_tests: true
    requires_human_review: true
    requires_adr_for:
      - auth changes
      - permission changes
      - API contract changes

  - name: frontend-agent
    mode: bounded-write
    description: "Implements frontend features and fixes"
    applicable_skills:
      - any non-compat skill
    allowed_paths:
      - frontend/src/**
      - frontend/e2e/**
    prohibited_paths:
      - backend/**
      - .governance/**
    max_files_changed: 10
    max_lines_changed: 300
    requires_tests: false
    requires_human_review: true

  - name: compat-agent
    mode: read-only
    description: "Performs Mattermost compatibility analysis and produces design artifacts"
    applicable_skills:
      - mattermost-api-parity
      - mm-endpoint-contract-parity
      - mm-websocket-calls-parity
      - mm-mobile-journey-parity
    readable_paths:
      - backend/compat/**
      - backend/src/api/v4/**
      - backend/src/mattermost_compat/**
      - tools/mm-compat/**
    writable_paths:
      - previous-analyses/**
      - docs/superpowers/specs/**
      - docs/superpowers/plans/**
    prohibited_paths:
      - .governance/**
    cannot_approve_prs: true
    cannot_write_production_code: true
```

- [ ] **Step 4: Create `.governance/protected-paths.yml`**

```yaml
# Protected Paths
# Paths that auto-elevate minimum risk tier when changed.
# Reference: docs/superpowers/specs/2026-03-22-governance-layer-design.md

paths:
  - pattern: "backend/src/auth/**"
    min_tier: elevated
    reason: "Authentication is security-critical"

  - pattern: "backend/src/api/v4/**"
    min_tier: elevated
    reason: "Mattermost HTTP compatibility surface"

  - pattern: "backend/src/mattermost_compat/**"
    min_tier: elevated
    reason: "Mattermost compatibility utilities"

  - pattern: "backend/src/realtime/**"
    min_tier: elevated
    reason: "WebSocket event contracts"

  - pattern: "backend/migrations/**"
    min_tier: elevated
    reason: "Database schema changes are irreversible"

  - pattern: "backend/compat/**"
    min_tier: elevated
    reason: "Compatibility contracts and golden tests"

  - pattern: "tools/mm-compat/**"
    min_tier: elevated
    reason: "Compatibility analysis tooling"

  - pattern: "backend/src/a2a/**"
    min_tier: elevated
    reason: "Agent-to-agent communication layer"

  - pattern: ".github/CODEOWNERS"
    min_tier: elevated
    reason: "Ownership changes affect review routing"

  - pattern: ".governance/**"
    min_tier: architectural
    reason: "Policy changes affect the entire development model"

  - pattern: "docs/superpowers/specs/**"
    min_tier: elevated
    reason: "Design record integrity"
```

- [ ] **Step 5: Create `.governance/pr-size-limits.yml`**

```yaml
# PR Size Limits
# Reference: docs/superpowers/specs/2026-03-22-governance-layer-design.md
#
# Exceptions are additive constraints on top of risk-tier limits.
# A path subject to both must satisfy the stricter rule.

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

- [ ] **Step 6: Verify all four YAML files parse cleanly**

Run:
```bash
for f in .governance/*.yml; do
  python3 -c "import yaml, sys; yaml.safe_load(open('$f'))" && echo "OK: $f" || echo "FAIL: $f"
done
```

Expected: `OK:` for all four files. If any FAIL, fix the YAML syntax error before continuing.

- [ ] **Step 7: Commit**

```bash
git add .governance/
git commit -m "chore: add governance policy files (risk-tiers, agent-contracts, protected-paths, pr-size-limits)"
```

---

## Task 2: Create `.github/CODEOWNERS` and PR template

**Files:**
- Create: `.github/CODEOWNERS`
- Create: `.github/pull_request_template.md`

- [ ] **Step 1: Create `.github/CODEOWNERS`**

```
# CODEOWNERS
# Reference: docs/superpowers/specs/2026-03-22-governance-layer-design.md
#
# IMPORTANT: Replace all @<placeholder> values with real GitHub usernames before
# enabling branch protection enforcement. Resolve from the repository maintainer list.

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

- [ ] **Step 2: Create `.github/pull_request_template.md`**

```markdown
## What does this PR do?

<!-- Describe the problem this solves or the feature it adds. -->

## Area affected

- [ ] backend
- [ ] frontend
- [ ] compat
- [ ] infra
- [ ] docs

## Does this change public behavior or API contracts?

- [ ] Yes — describe: <!-- brief description -->
- [ ] No

## Risk tier

- [ ] standard — typos, UI polish, test additions, internal refactors
- [ ] elevated — auth, permissions, API behavior, schema, compat paths
- [ ] architectural — architecture redesign, storage, protocol, security model

## Tests

- [ ] Added
- [ ] Updated
- [ ] Not applicable — reason: <!-- explain -->

## Docs / ADR

- [ ] Updated
- [ ] ADR created — link: <!-- link -->
- [ ] Not needed

## Decision note (required for elevated and architectural changes)

<!-- 3–5 lines: what was decided and why. -->

## Agent-generated

- [ ] Yes — `Generated-by: <agent-name>`, `Skill used: <skill-name>`
- [ ] No
```

- [ ] **Step 3: Verify CODEOWNERS does not reference non-existent paths**

Run:
```bash
# Check that paths listed in CODEOWNERS exist in the repo
grep '^/' .github/CODEOWNERS | awk '{print $1}' | sed 's|^/||' | sed 's|/\*\*||' | sed 's|/$||' | while read p; do
  if [ -e "$p" ]; then echo "OK: $p"; else echo "MISSING: $p"; fi
done
```

Expected: `OK:` for all paths. Note: `push-proxy/` and `tools/mm-compat/` may show as MISSING if not present in the current branch — that is acceptable for placeholder CODEOWNERS. `backend/src/a2a/`, `backend/src/mattermost_compat/`, `backend/compat/`, `tools/` should all exist.

- [ ] **Step 4: Commit**

```bash
git add .github/CODEOWNERS .github/pull_request_template.md
git commit -m "chore: add CODEOWNERS and PR template"
```

---

## Task 3: Create `.github/ISSUE_TEMPLATE/` forms

**Files:**
- Create: `.github/ISSUE_TEMPLATE/config.yml`
- Create: `.github/ISSUE_TEMPLATE/bug.yml`
- Create: `.github/ISSUE_TEMPLATE/feature.yml`
- Create: `.github/ISSUE_TEMPLATE/refactor.yml`

- [ ] **Step 1: Create `.github/ISSUE_TEMPLATE/` directory**

```bash
mkdir -p .github/ISSUE_TEMPLATE
```

- [ ] **Step 2: Create `.github/ISSUE_TEMPLATE/config.yml`**

```yaml
blank_issues_enabled: false
contact_links:
  - name: Questions & Support
    url: https://github.com/rustchatio/rustchat/discussions
    about: Ask questions and get help in GitHub Discussions
```

- [ ] **Step 3: Create `.github/ISSUE_TEMPLATE/bug.yml`**

```yaml
name: Bug Report
description: Report a bug or unexpected behavior
labels: ["type/bug"]
body:
  - type: dropdown
    id: area
    attributes:
      label: Area
      description: Which part of the system is affected?
      options:
        - backend
        - frontend
        - compat
        - push-proxy
        - infra
    validations:
      required: true

  - type: textarea
    id: current-behavior
    attributes:
      label: Current behavior
      description: What is happening?
    validations:
      required: true

  - type: textarea
    id: expected-behavior
    attributes:
      label: Expected behavior
      description: What should happen instead?
    validations:
      required: true

  - type: textarea
    id: steps
    attributes:
      label: Steps to reproduce
      placeholder: |
        1. Go to...
        2. Click...
        3. See error...
    validations:
      required: false

  - type: dropdown
    id: compat-impact
    attributes:
      label: Compatibility impact
      description: Does this affect Mattermost client compatibility?
      options:
        - "No"
        - "Yes — affects Mattermost clients"
        - Unknown
    validations:
      required: true

  - type: dropdown
    id: risk-tier
    attributes:
      label: Risk tier suggestion
      options:
        - standard
        - elevated
    validations:
      required: true
```

- [ ] **Step 4: Create `.github/ISSUE_TEMPLATE/feature.yml`**

```yaml
name: Feature Request
description: Propose a new feature or improvement
labels: ["type/feature"]
body:
  - type: dropdown
    id: area
    attributes:
      label: Area
      options:
        - backend
        - frontend
        - compat
        - push-proxy
        - infra
    validations:
      required: true

  - type: textarea
    id: problem
    attributes:
      label: Problem statement
      description: What problem does this feature solve?
    validations:
      required: true

  - type: textarea
    id: proposed-behavior
    attributes:
      label: Proposed behavior
      description: What should the feature do?
    validations:
      required: true

  - type: dropdown
    id: compat-impact
    attributes:
      label: Compatibility impact
      options:
        - "No"
        - "Yes — affects Mattermost clients"
        - Unknown
    validations:
      required: true

  - type: textarea
    id: acceptance-criteria
    attributes:
      label: Acceptance criteria
      placeholder: |
        - [ ] ...
        - [ ] ...
    validations:
      required: false

  - type: dropdown
    id: adr-needed
    attributes:
      label: ADR needed?
      options:
        - "No"
        - "Yes"
        - Unsure
    validations:
      required: true
```

- [ ] **Step 5: Create `.github/ISSUE_TEMPLATE/refactor.yml`**

```yaml
name: Refactor / Tech Debt
description: Propose a code improvement or refactor
labels: ["type/refactor"]
body:
  - type: dropdown
    id: area
    attributes:
      label: Area
      options:
        - backend
        - frontend
        - compat
        - push-proxy
        - infra
    validations:
      required: true

  - type: textarea
    id: current-state
    attributes:
      label: What is wrong today?
      description: Describe the current problem or tech debt.
    validations:
      required: true

  - type: textarea
    id: proposed-improvement
    attributes:
      label: Proposed improvement
    validations:
      required: true

  - type: textarea
    id: blast-radius
    attributes:
      label: Blast radius
      description: What could break if this goes wrong?
    validations:
      required: false

  - type: dropdown
    id: risk-tier
    attributes:
      label: Risk tier
      options:
        - standard
        - elevated
        - architectural
    validations:
      required: true
```

- [ ] **Step 6: Verify all four ISSUE_TEMPLATE YAML files parse cleanly**

Run:
```bash
for f in .github/ISSUE_TEMPLATE/*.yml; do
  python3 -c "import yaml, sys; yaml.safe_load(open('$f'))" && echo "OK: $f" || echo "FAIL: $f"
done
```

Expected: `OK:` for all four files. Fix any YAML errors before continuing.

- [ ] **Step 7: Commit**

```bash
git add .github/ISSUE_TEMPLATE/
git commit -m "chore: add issue templates (bug, feature, refactor)"
```

---

## Task 4: Post-implementation verification

- [ ] **Step 1: Verify all 10 files exist**

Run:
```bash
echo "=== .governance/ ===" && ls -1 .governance/
echo "=== .github/ ===" && ls -1 .github/CODEOWNERS .github/pull_request_template.md
echo "=== .github/ISSUE_TEMPLATE/ ===" && ls -1 .github/ISSUE_TEMPLATE/
```

Expected output:
```
=== .governance/ ===
agent-contracts.yml
pr-size-limits.yml
protected-paths.yml
risk-tiers.yml
=== .github/ ===
.github/CODEOWNERS
.github/pull_request_template.md
=== .github/ISSUE_TEMPLATE/ ===
bug.yml
config.yml
feature.yml
refactor.yml
```

- [ ] **Step 2: Verify no existing CI workflows were broken**

Run:
```bash
git diff HEAD~3 --name-only -- .github/workflows/
```

Expected: empty output (no workflow files were touched).

- [ ] **Step 3: Verify no production code was modified**

Run:
```bash
git diff HEAD~3 --name-only -- backend/ frontend/ push-proxy/
```

Expected: empty output.

- [ ] **Step 4: Final commit log check**

Run:
```bash
git log --oneline -4
```

Expected: three governance commits visible:
```
<hash> governance: add issue templates (bug, feature, refactor)
<hash> governance: add CODEOWNERS and PR template
<hash> governance: add policy files (risk-tiers, agent-contracts, protected-paths, pr-size-limits)
```

---

## Post-implementation notes

**After this PR is merged:**

1. **Replace CODEOWNERS placeholders** — edit `.github/CODEOWNERS` and replace all `@<placeholder>` entries with real GitHub usernames. Do this before enabling branch protection enforcement.

2. **Create GitHub labels** — the issue forms reference these labels which must be created in the GitHub repository settings:
   - Type: `type/bug`, `type/feature`, `type/refactor`
   - Area: `area/backend`, `area/frontend`, `area/compat`, `area/infra`, `area/docs`
   - Risk: `risk/standard`, `risk/elevated`, `risk/architectural`
   - Special: `agent-generated`, `needs-adr`, `good-first-issue`

3. **Enable branch protection** (optional, Approach C prerequisite) — once CODEOWNERS has real usernames, enable "Require review from Code Owners" in GitHub branch protection settings for `main`.
