# Design System — RustChat

## Product Context
- **What this is:** RustChat is a self-hosted team collaboration product with channels, threads, calls, settings, admin tooling, and Mattermost-compatible surfaces.
- **Who it's for:** Operators, engineering teams, security-conscious organizations, and self-hosters who want team chat that feels familiar but trustworthy.
- **Space/industry:** Team collaboration, work chat, self-hosted communication software.
- **Project type:** Authenticated web app with admin/dashboard surfaces, settings, and collaboration workflows.

## Current State Analysis
- The product already has the right structural model. Sidebar, channel canvas, composer, and settings all map to user expectations from Slack and Mattermost.
- The current weakness is not information architecture. It is visual intent. The shell has been readable but too generic, with too little differentiation between brand, context, and utility chrome.
- The best direction is **not** "copy Slack" or "copy Mattermost". It is: Slack ease, Mattermost trust.
- RustChat should feel calm, operational, and fast. Warm enough to feel human, strict enough to feel dependable.

## Aesthetic Direction
- **Direction:** Focused Warm Utility
- **Decoration level:** Intentional
- **Mood:** Calm, competent, trustworthy collaboration software. Not playful startup chat, not heavy enterprise bunker. The product should feel like a tool people can live in all day without fatigue.
- **Reference sites:** [Slack](https://slack.com/), [Mattermost](https://mattermost.com/)

## Typography
- **Display/Hero:** IBM Plex Sans Semibold
  Why: strong, technical, credible, and less interchangeable than Inter. Good for product headers and high-signal moments.
- **Body:** IBM Plex Sans Regular
  Why: very readable at app sizes, holds up well in dense collaboration UI, and fits a self-hosted/dev-adjacent product better than trendier startup sans families.
- **UI/Labels:** IBM Plex Sans Medium
  Why: gives controls enough authority without making the shell shout.
- **Data/Tables:** IBM Plex Mono
  Why: perfect for code, timestamps, narrow metadata, counts, and operational/admin views. Use sparingly.
- **Code:** IBM Plex Mono
- **Loading:** self-host or use a single hosted family strategy. Avoid font sprawl.
- **Scale:**
  - `xs`: 12px
  - `sm`: 14px
  - `base`: 15px
  - `md`: 16px
  - `lg`: 18px
  - `xl`: 21px
  - `2xl`: 28px
  - `3xl`: 34px

## Color
- **Approach:** Balanced, restrained, warm-neutral collaboration system
- **Primary:** `#B45309`
  Usage: primary CTA, selected shell state, active navigation, strong emphasis
- **Primary Hover:** `#92400E`
- **Primary Foreground:** `#FFFAF2`
- **Secondary:** `#0F766E`
  Usage: secondary emphasis, structured highlights, non-destructive interactive accents
- **Accent:** `#14B8A6`
  Usage: lightweight freshness, badges, subtle highlights, charts, support states
- **Neutrals:**
  - `#FFFDF8` app surface
  - `#F5F3EF` app background
  - `#F2EEE7` muted surface
  - `#E4DDD2` default border
  - `#CFC4B5` strong border
  - `#9B9287` subtle text
  - `#6B645C` muted text
  - `#44403C` secondary text
  - `#1C1917` primary text
- **Semantic:**
  - success `#16A34A`
  - warning `#F59E0B`
  - error `#DC2626`
  - info `#0F766E`
- **Dark mode:** do not invert mechanically. Keep the same warm personality, reduce saturation slightly, protect text contrast first, and keep brand accents bright enough to remain legible without becoming neon.

## Spacing
- **Base unit:** 4px
- **Density:** Comfortable by default, compact where scanning speed matters
- **Scale:** `2xs(2) xs(4) sm(8) md(16) lg(24) xl(32) 2xl(48) 3xl(64)`
- **Rules:**
  - Keep shell controls on the 44px touch target floor where interaction matters
  - Use denser spacing inside channel lists and metadata rows than in settings/forms
  - Empty states and major settings headers should breathe more than channel chrome

## Layout
- **Approach:** Grid-disciplined
- **Grid:** collaboration shell first, not editorial. Structure wins.
- **Max content width:** `1120px` for message canvas, with room to grow to `1200px` on large displays
- **Border radius:**
  - `sm`: 6px
  - `md`: 8px
  - `lg`: 12px
  - `xl`: 16px
  - `pill/full`: 9999px
- **Shell hierarchy rules:**
  - Team rail is a compact identity rail, not a second sidebar
  - Channel sidebar should optimize scanning and unread state first
  - Global header should de-emphasize search slightly and strengthen product/current-context recognition
  - Channel header should communicate channel identity, topic, and actions in that order
  - Empty center states must feel intentional, never abandoned

## Motion
- **Approach:** Minimal-functional
- **Easing:** enter `ease-out`, exit `ease-in`, move `ease-in-out`
- **Duration:** micro `50-100ms`, short `150-250ms`, medium `250-400ms`, long `400-700ms`
- **Rules:**
  - Motion should explain state changes, not decorate the interface
  - Use animation for dropdowns, drawers, toasts, and composer affordances
  - Avoid animated gradients, floating cards, or “alive” chrome

## Component Principles
- **Sidebar:** fast to scan, obvious selection state, clear unread hierarchy, muted chrome
- **Message Canvas:** content-first, low noise, clear date boundaries, better intentional empty states
- **Composer:** high confidence, one obvious send action, secondary tools visually subordinate
- **Settings:** readable forms, token-driven surfaces, consistent action placement, less gray Tailwind drift
- **Admin:** more monochrome and structured than the collaboration shell, but still within the same token system

## Anti-Patterns
- Do not reintroduce default indigo SaaS styling as the main visual voice.
- Do not let utility chrome overpower brand and current channel context.
- Do not use hardcoded `gray-*`, `dark:*`, or ad hoc color classes on theme-sensitive surfaces.
- Do not use more than one “hero” accent in the same viewport.
- Do not make empty states look like loading bugs.

## Rollout Priorities
1. **Shell coherence**
   - Sidebar, global header, channel header, message empty/loading states
2. **Design token enforcement**
   - remove local gray overrides and use shared tokens everywhere
3. **Theme simplification**
   - keep a small set of polished first-class themes, treat the rest as advanced
4. **Settings and admin normalization**
   - align modals, settings cards, and admin forms to the same system
5. **Marketing or external surfaces later**
   - only after the product shell is unquestionably solid

## Decisions Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-28 | Adopted Focused Warm Utility as the core product direction | RustChat needs to feel clearer and more ownable than generic SaaS while staying familiar for collaboration users |
| 2026-03-28 | Set IBM Plex Sans as the primary product typeface | It is more distinctive and credible for this product than Inter while staying highly readable |
| 2026-03-28 | Kept warm neutrals with rust and teal accents | This separates RustChat from the overused indigo startup look and supports long-session readability |
| 2026-03-28 | Chose minimal-functional motion | Collaboration UI should feel fast and dependable, not ornamental |

