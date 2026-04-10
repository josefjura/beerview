# Architecture

## Guiding principle

Right tool for each layer. The system is split along a clean boundary: Rust handles the core data and owner UI; a small Python service handles social media posting where the ecosystem is stronger.

## Services

```
┌─────────────────────────────────┐     ┌──────────────────────────┐
│         Core App (Rust)         │     │   Social Bot (Python)    │
│                                 │     │                          │
│  Axum + Maud + HTMX + SQLite   │────▶│  Facebook Graph API      │
│                                 │     │  Instagram Basic Display │
│  - Owner UI (mobile-first)      │     │  Image generation        │
│  - Public tap list pages        │     │  (Pillow / HTML→PNG)     │
│  - Embeddable widget JS         │     │                          │
│  - Internal webhook emitter     │     │  Triggered by webhook    │
└─────────────────────────────────┘     └──────────────────────────┘
         │
         │ serves
         ▼
┌─────────────────┐
│   SQLite DB     │
│                 │
│  pubs           │
│  beers          │
│  taps (current) │
│  queue          │
│  archive        │
└─────────────────┘
```

## Core app (Rust)

**Stack:** Axum · Maud · HTMX · SQLx · SQLite

Chosen because:
- Same stack as existing projects — no new learning curve
- SQLite is sufficient for the scale (tens of pubs, hundreds of beers)
- Server-rendered HTML + HTMX covers the owner UI well
- Fast, low resource usage, easy to self-host

**Responsibilities:**
- Authentication (pub owner login)
- Beer queue management
- Tap switching logic
- Public tap list pages (one per pub, cacheable)
- Embeddable widget endpoint
- Untappd API proxy (beer search)
- Webhook emit to social bot on tap changes

## Social bot (Python)

**Stack:** Python · httpx · Pillow · APScheduler (or simple cron)

Chosen because:
- Facebook/Instagram API ecosystem is far richer in Python
- Image generation with Pillow is straightforward
- Small, isolated service with a single responsibility
- Can be deployed independently (or as a sidecar)

**Responsibilities:**
- Receive webhook from core app on tap change events
- Generate tap list image (Pillow or headless browser screenshot)
- Post to Facebook Page via Graph API
- Post to Instagram (optional)
- Scheduled digest posts (e.g. daily at 15:00)

**Communication with core app:**
- Core app POSTs a JSON webhook: `{ "pub_id": "...", "event": "tap_changed", "tap_list": [...] }`
- Internal only — not exposed publicly
- Simple shared secret for auth

## Deployment

Target: single small VPS (or fly.io / Railway for ease).

- Core app: single Rust binary
- Social bot: Python process or Docker container
- SQLite: file on disk (simple, no separate DB server needed)
- Nginx: reverse proxy, TLS termination

## External dependencies

| Service | Purpose | Risk |
|---|---|---|
| Untappd API | Beer search & metadata | Medium — free but requires approval, can be revoked |
| Facebook Graph API | Page posting | Medium — OAuth per pub, policy changes possible |
| Instagram API | Feed/Stories posting | Medium — linked to Facebook, same risks |

All external data is cached locally. Loss of Untappd access only affects search for *new* beers — existing beer data is unaffected.
