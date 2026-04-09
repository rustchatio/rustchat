# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records for RustChat.

## What is an ADR?

An ADR documents a significant architectural decision: the context, the options considered, the decision made, and the consequences.

## When to Create an ADR

Per `.governance/risk-tiers.yml`, an ADR is required for `architectural` tier changes, and for specific elevated changes involving:
- Auth or permission model changes
- API contract changes
- Storage model changes

## ADR Format

```markdown
# ADR-NNN: Title

**Date:** YYYY-MM-DD
**Status:** Proposed | Accepted | Deprecated | Superseded by ADR-NNN
**Risk tier:** architectural

## Context
[What is the problem and why does it matter?]

## Decision
[What was decided?]

## Consequences
[What are the trade-offs and implications?]
```

## ADR Index

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| *(No ADRs yet)* | | | |

## ADR Lifecycle

```
PROPOSED → ACCEPTED → (SUPERSEDED or DEPRECATED)
```

- **Don't delete old ADRs.** They capture historical context.
- When a decision changes, write a new ADR that references and supersedes the old one.

---

*See also: [Decision Notes](../archive/decision-notes/) for routine elevated changes.*
