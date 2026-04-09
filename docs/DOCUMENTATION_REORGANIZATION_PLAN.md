# Documentation Reorganization Plan

## Current State Analysis

### Root-Level Documentation Files (16 files)

| File | Type | Status | Recommendation |
|------|------|--------|----------------|
| `README.md` | Core | ✅ Current | Keep, update links |
| `AGENTS.md` | Developer | ✅ Current | Keep (referenced by workflows) |
| `CHANGELOG.md` | Core | ✅ Current | Keep |
| `CLA.md` | Legal | ✅ Static | Keep |
| `CLAUDE.md` | Developer | ⚠️ Minimal | Merge into docs/development/ |
| `CODE_OF_CONDUCT.md` | Community | ✅ Static | Keep |
| `CONTRIBUTING.md` | Developer | ✅ Current | Move to docs/development/ |
| `DESIGN.md` | Design | ⚠️ Unknown | Review before deciding |
| `IMPLEMENTATION_PLAN.md` | Archive | ✅ Completed | Move to docs/archive/ |
| `IMPLEMENTATION_STATUS.md` | Archive | ✅ Completed | Move to docs/archive/ |
| `IMPLEMENTATION_SUMMARY.md` | Archive | ✅ Completed | Move to docs/archive/ |
| `POSTCARD_MIGRATION_PLAN.md` | Archive | ✅ Completed | Move to docs/archive/ |
| `RELEASING.md` | Developer | ✅ Current | Move to docs/development/ |
| `SECURITY.md` | Core | ✅ Static | Keep |
| `SPEC.md` | Core | ✅ Current | Keep |
| `task_plan.md` | Active | ✅ Current | Keep (or move to docs/) |

### docs/ Directory Issues

**Problem 1: Flat structure with 66 markdown files**
- No clear navigation hierarchy
- Mix of user docs, dev docs, architecture, and historical reports

**Problem 2: Many completed/deprecated documents**
These completion reports should be archived:
- `docs/phase1-completion-report.md` - Phase 1 complete
- `docs/V4_API_IMPLEMENTATION_COMPLETE.md` - V4 API complete
- `docs/VOIP_PUSH_*` (4 files) - VoIP push complete
- `docs/MOBILE_RINGING_*` (2 files) - Ringing complete
- `docs/SSO_IMPLEMENTATION_SUMMARY.md` - SSO complete
- `docs/specs/MIGRATION_COMPLETE.md` - Axios→Fetch migration complete

**Problem 3: Inconsistent organization**
- `docs/architecture.md` redirects to another file
- `docs/adr/README.md` exists but no actual ADRs
- `docs/decision-notes/README.md` exists but empty
- `docs/superpowers/` contains many dated spec/plan files

---

## Proposed New Structure

```
docs/
├── README.md                          # Documentation index & navigation
├── user/                              # 👤 End-user documentation
├── admin/                             # 🔧 Administrator/operator docs
├── architecture/                      # 🏗️ System architecture
├── development/                       # 👨‍💻 Developer/contributor docs
├── operations/                        # 🚨 Operations & runbooks
├── reference/                         # 📚 Reference documentation
├── decisions/                         # 📋 Architecture Decision Records
└── archive/                           # 📦 Completed work, historical docs
```

---

## Implementation Steps

### Phase 1: Create New Structure
1. Create new directory structure
2. Create `docs/README.md` with navigation
3. Create `docs/archive/README.md`

### Phase 2: Move & Consolidate
1. Move documents to new locations
2. Update internal cross-references
3. Merge related documents

### Phase 3: Update Root Files
1. Move implementation docs to archive
2. Update `README.md` documentation links

### Phase 4: Verification
1. Check all internal links work
2. Verify no broken references

---

*Generated: 2026-04-05*
