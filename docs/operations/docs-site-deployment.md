# Docs Site Deployment (`docs.rustchat.io`)

This guide describes how to deploy the documentation site automatically from GitHub Actions to Cloudflare Pages.

## Overview

- Source of truth: `docs/` directory in this repository
- Static site generator: VitePress
- CI/CD workflow: `.github/workflows/docs-ci-cd.yml`
- Deployment method: Cloudflare Pages Direct Upload via Wrangler in GitHub Actions

## 1. Create a Cloudflare Pages Project

In Cloudflare Dashboard:

1. Go to `Workers & Pages`
2. Create a new `Pages` project
3. Use any temporary source (the workflow deploys artifacts directly)
4. Note the project name (for example `rustchat-docs`)

## 2. Create API Token

Create a Cloudflare API token with Pages edit permissions for your account/project.

You also need your Cloudflare Account ID.

## 3. Configure GitHub Secrets

Set these repository secrets:

- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`
- `CLOUDFLARE_PAGES_PROJECT_NAME` (for example `rustchat-docs`)

## 4. Workflow Behavior

- On pull requests: runs docs link checks and builds the VitePress site.
- On push to `main`: runs the same checks, then deploys to the Cloudflare Pages project.

## 5. Configure Custom Domain

In the Pages project:

1. Open `Custom domains`
2. Add `docs.rustchat.io`
3. Complete DNS validation if required

Cloudflare manages TLS automatically once the domain is attached.

## 6. Manual Validation

```bash
cd docs
npm ci
npm run docs:check-links
npm run docs:build
```

If CI succeeds and secrets are configured, `docs.rustchat.io` updates automatically after merge to `main`.
