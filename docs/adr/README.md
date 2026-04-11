# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records for rustchat.

An ADR documents a significant architectural decision: the context, the options considered, the decision made, and the consequences.

## When to create an ADR

Per `.governance/risk-tiers.yml`, an ADR is required for `architectural` tier changes, and for specific elevated changes involving:
- Auth or permission model changes
- API contract changes
- Storage model changes

## Format

```
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

## Index

- [ADR: Frontend Supply-Chain Security Model](./ADR-frontend-supply-chain-security.md) - npm normalization, hardened CI installs, dependency governance, and override/patch policy for the Vue frontend.
