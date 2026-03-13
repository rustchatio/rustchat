# Rustchat Product Hunt Launch Documentation

Last updated: 2026-03-12

This package assumes your request meant "launch at Product Hunt."

## 1) Required Product Hunt Submission Fields

Use this as the final pre-submit checklist.

- [ ] Product name: `Rustchat`
- [ ] Tagline (target <= 60 chars)
- [ ] Description (keep <= 260 chars to stay within stricter UI/help limits)
- [ ] Product URL (public landing page, no short links)
- [ ] Thumbnail/logo (square image)
- [ ] Gallery media (at least 2 images, optional video)
- [ ] Makers added from personal Product Hunt profiles
- [ ] Topics selected (up to 3 relevant topics)
- [ ] First maker comment drafted (strongly recommended for launch quality)

## 2) Launch Copy (Ready To Paste)

### Product name

`Rustchat`

### Tagline (primary)

`Self-hosted team chat with Rust backend + MM v4 API`

### Tagline alternatives

- `Own your team chat: Rust backend, Vue client, MM v4 compatible`
- `Open-source collaboration stack with Rust performance`
- `Self-hosted collaboration with Mattermost-compatible APIs`

### Description (primary, 206 chars)

`Open-source team collaboration you can self-host: Rust backend, Vue app, Mattermost-compatible API v4, realtime chat, channels, and calls surfaces. Keep data in your infra and migrate clients progressively.`

### Description alternatives

- `Rustchat is a self-hosted collaboration platform with a Rust backend and Vue web app. It supports native APIs plus Mattermost-compatible v4 routes, so teams can adopt it gradually without giving up data ownership.`
- `Run your own team chat stack with Rustchat: channels, posts, realtime events, and compatibility-focused API v4 support for migration-friendly self-hosted deployments.`

## 3) First Maker Comment (Draft)

Paste as first comment and adjust names/links before launch:

```md
Hi Product Hunt, we are the makers of Rustchat.

Rustchat is an open-source, self-hosted collaboration platform built with a Rust backend and a Vue frontend. We built it for teams that want full infrastructure control and a migration path via Mattermost-compatible API v4 surfaces.

What is live today:
- Core collaboration flows (teams/channels/posts/realtime)
- Native API plus compatibility-focused `/api/v4` routes
- Self-host-first deployment model (Postgres, Redis, S3-compatible storage)

What we want feedback on:
1. Which migration blockers matter most for your team?
2. Which admin/security controls are required before broader adoption?
3. Which clients/integrations should we prioritize next?

Thanks for checking us out. We will be in the comments all day.
```

## 4) Required Supporting Docs To Link

Before launch, ensure these are accessible from your site/repo:

- [ ] Public product overview: `README.md`
- [ ] Setup/install instructions: `README.md` quick start
- [ ] Security policy: `SECURITY.md`
- [ ] License: `LICENSE`
- [ ] Changelog or release notes: `CHANGELOG.md`
- [ ] Contact/support route (email, issues page, or community link)

## 5) Media Brief (For Design/Marketing)

Prepare these assets before scheduling:

- [ ] Square logo/thumbnail with clear contrast
- [ ] 2-6 gallery images showing: login, channels, messaging, admin/security, compatibility story
- [ ] Optional short demo video (30-90s) with captions
- [ ] One architecture visual showing Rust backend + self-hosted infra control

## 6) Launch Day Runbook

- [ ] T-24h: freeze launch copy, links, and media
- [ ] T-12h: dry-run first comment + team replies
- [ ] T-1h: verify product URL, uptime, signup/onboarding path
- [ ] Launch: post, publish first maker comment immediately
- [ ] +1h to +12h: reply to every meaningful comment quickly
- [ ] End of day: summarize feedback into roadmap issues

## 7) Fast Q&A Response Bank

Use these in comments:

- **How is this different from Mattermost?**  
  Rustchat is a Rust-first implementation with self-hosting control and compatibility-focused API v4 surfaces for gradual migration.

- **Is this production-ready?**  
  It is in active development/pre-release; we recommend validating with your own deployment gates before production rollout.

- **Can I self-host everything?**  
  Yes, core deployment is designed around self-hosted Postgres, Redis, and S3-compatible object storage.

## 8) Owner Fill-In Section (Complete Before Publish)

- Launch owner: `@________`
- Product Hunt maker profiles: `@________`, `@________`
- Launch date: `YYYY-MM-DD`
- Final launch URL: `https://www.producthunt.com/posts/________`
- Support contact shown publicly: `________`

## 9) Source References (Submission Rules)

- Product Hunt Launch Guide: <https://help.producthunt.com/en/articles/11976731-launch-guide>
- Product Launch Checklist: <https://help.producthunt.com/en/articles/11976729-product-launch-checklist>
- Pricing and Discounts: <https://help.producthunt.com/en/articles/11976726-pricing-and-discounts>
- Add Team Members as Makers: <https://help.producthunt.com/en/articles/13036869-inviting-your-team-members-as-makers>
- Startup Toolkit: <https://startup.producthunt.com/toolkit>
