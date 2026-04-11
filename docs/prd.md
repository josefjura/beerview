# Product Requirements Document — Beerview PoC

**Version:** 1.0
**Status:** Draft
**Date:** 2026-04-11

---

## 1. Purpose & Framing

Beerview is a tap list management tool for small, independent craft beer pubs that currently manage their tap lists on physical chalkboards. The goal is to eliminate manual work for pub owners while making current tap information available to beer hunters online.

### PoC vs. Full Product

What is described in this document is a **Proof of Concept (PoC)** — a working demo intended to be shown to three specific Prague craft beer pubs. It is scoped to demonstrate core value with minimum build effort. A full product will only be developed if the PoC generates genuine interest from pub owners.

**PoC target:** 3 craft beer pubs in Prague frequented by the product owner. Accounts created manually by the developer.

---

## 2. Target Users

### Pub Owner / Staff
Small, independent craft beer pub operators who:
- Manage their tap list on a physical chalkboard
- Are not using Untappd for Business or any digital tap list tool
- Post to Facebook manually (if at all)
- Have no developer on staff
- Will not tolerate friction — any bug or confusing moment during service risks permanent abandonment

### Beer Hunter (Consumer)
Craft beer enthusiasts who:
- Visit multiple pubs regularly and want to plan where to go
- Currently have no reliable way to know what's on tap before arriving
- Use Untappd personally to track what they've tasted

---

## 3. Core Principles

1. **Prep during quiet, execute during busy** — the queue exists to separate thinking from tapping
2. **The Switch action is sacred** — must work instantly, never fail visibly during service
3. **Outputs are automatic** — once set up, the owner never touches them
4. **Mobile-first** — all owner interactions happen on a phone behind the bar
5. **Feather light** — every added feature must justify its complexity against the risk of owner abandonment

---

## 4. PoC Scope

### In scope
- Owner app: pub profile, tap management, queue, switch
- Public tap list URL (per pub)
- Embeddable widget (placed manually by developer for PoC pubs)
- Read-only discovery site (list of participating pubs with their current taps)
- Basic beer search with manual entry fallback

### Out of scope for PoC
- Social media posting (Facebook, Instagram) — deferred to full release
- Kiosk / wall display view — deferred
- Self-serve pub registration — accounts created manually for PoC
- Staff roles and permissions — all pub users have equal rights
- Beer deduplication / global beer catalogue

---

## 5. User Flows

### 5.1 Owner: Add beer to queue (prep time)

1. Owner opens beerview on their phone
2. Taps **Add to Queue**
3. Searches for a beer by name or brewery
   - System queries Untappd API (if available) or local reference database
   - Results show: name, brewery, style, ABV
4. Owner selects a beer from results
5. Owner optionally adds price(s) as size/price pairs (e.g. 0.5l / 72 Kč, 0.3l / 49 Kč)
6. Beer is added to the queue
7. Owner can reorder the queue by drag or up/down buttons

### 5.2 Owner: Switch a tap (service time)

1. Owner opens beerview — sees current tap view (all taps, each showing current beer or "empty")
2. Taps **Switch** next to the tap whose keg has run out
3. System shows the current queue
4. Owner taps the queued beer they want to put on
5. System atomically:
   - Archives the old beer (moves to tap history)
   - Removes the selected beer from the queue
   - Sets the new beer as active on that tap
6. Confirmation appears immediately (optimistic UI)
7. Public tap list and embed update instantly
8. **Undo** button is available for 30 seconds — tapping it fully reverses the operation (old beer restored to tap, switched beer restored to queue)

### 5.3 Owner: Handle empty tap with no queue

When a keg dies and no beer is queued, the owner has two options:

**Option A — Mark tap as empty:**
- Tap **Mark Empty** next to the tap
- Tap disappears from the public tap list and embed (not shown as "empty", just absent)
- Owner can switch it on again later

**Option B — Quick add:**
- Tap **Quick Add** next to the tap
- Enter just beer name and brewery (minimal form, no other fields required)
- Beer appears on tap immediately

**Option C — Full add:**
- Tap **Full Add** next to the tap
- Full beer creation form (name, brewery, style, ABV, prices)
- Beer appears on tap immediately

### 5.4 Beer hunter: Browse discovery site

1. Visits beerview discovery site
2. Sees a list of participating pubs with:
   - Pub name
   - Neighbourhood / city
   - Number of beers currently on tap
   - Last updated timestamp
3. Taps a pub to see their full current tap list
4. No account required, no interaction — read only

---

## 6. Feature Requirements

### 6.1 Tap Management

| Requirement | Detail |
|---|---|
| Tap count | Set per pub at account creation; editable by any pub user |
| Tap states | Active (beer on tap), Empty (no beer, hidden from public) |
| Switch action | Atomic; old beer archived, queued beer activated |
| Undo window | 30 seconds; full rollback |
| Concurrent switches | Last write wins; no error shown to user who "loses" |

### 6.2 Queue

| Requirement | Detail |
|---|---|
| Queue items | Linked to a beer record; includes price pairs |
| Ordering | Manual drag/reorder |
| Ad-hoc tap | Quick add (name + brewery) or full add (all fields) |

### 6.3 Beer Data

| Requirement | Detail |
|---|---|
| Primary source | Untappd API (if access granted) |
| Fallback source | Local reference database seeded from atlaspiv.cz or similar |
| Local cache | Beers added or confirmed by any pub are stored locally |
| Manual entry | Always available as fallback |
| Fields | Name, brewery, style, ABV, optional label image URL |

### 6.4 Price Format

Prices are stored as an ordered array of size/price pairs:
```json
[
  { "size": "0.5l", "price": 72 },
  { "size": "0.3l", "price": 49 }
]
```
Currency is CZK. Sizes are free-form strings. Array may be empty (no price shown).

### 6.5 Public Tap List

| Requirement | Detail |
|---|---|
| URL | `beerview.app/pub/:slug` |
| Contents | All active taps; empty taps not shown |
| Per tap | Beer name, brewery, style, ABV, prices, tap number |
| Freshness | Updates immediately on Switch |
| Last updated | Displayed prominently |
| "Offline" state | Owner can set pub to offline; page shows "tap list temporarily unavailable" |

### 6.6 Embed Widget

| Requirement | Detail |
|---|---|
| Integration | Single `<script>` tag with `data-pub` attribute |
| Rendering | Vanilla JS; injects a styled div into the page |
| Default style | Readable with no customisation; matches a neutral pub aesthetic |
| CORS | Public tap list API allows requests from any origin |
| Update | Fetches fresh data on page load; no real-time push for PoC |
| Versioning | `embed.js` always latest; backwards compatible |
| PoC deployment | Developer places manually on pilot pub websites |

### 6.7 Discovery Site

| Requirement | Detail |
|---|---|
| Contents | List of all participating pubs |
| Per pub card | Name, neighbourhood, tap count, last updated timestamp |
| Interaction | Click through to full tap list; no accounts, no favourites |
| Scope | Read-only; no self-serve registration |
| Stale pubs | "Offline" pubs shown with their offline status |

### 6.8 Authentication

| Requirement | Detail |
|---|---|
| Auth method | Session-based (cookie); fits HTMX server-rendered model |
| Account creation | Manual by developer for PoC |
| Multi-user | Multiple user accounts per pub; all have equal rights |
| Session expiry | Must not expire during service hours; long-lived sessions |

### 6.9 Resilience

| Requirement | Detail |
|---|---|
| Offline tolerance | Resilient on bad wifi: retry with spinner, action not lost on timeout |
| True offline | Out of scope |
| Switch failure | If server unreachable, show clear retry prompt (never silent fail) |
| External APIs | If Untappd/beer DB unavailable, fall back to manual entry silently |

---

## 7. Data Model (PoC)

### `pub`
| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| slug | TEXT UNIQUE | URL-safe identifier |
| name | TEXT | |
| neighbourhood | TEXT | e.g. "Žižkov" |
| tap_count | INTEGER | Set at creation |
| is_offline | BOOLEAN | Owner-controlled; hides tap list |
| created_at | DATETIME | |
| updated_at | DATETIME | |

### `pub_user`
| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| pub_id | INTEGER FK | |
| email | TEXT UNIQUE | |
| password_hash | TEXT | |
| created_at | DATETIME | |

### `beer`
| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| name | TEXT | |
| brewery | TEXT | |
| style | TEXT | nullable |
| abv | REAL | nullable |
| label_image_url | TEXT | nullable |
| untappd_id | INTEGER | nullable; for dedup if using Untappd |
| created_at | DATETIME | |

### `tap`
| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| pub_id | INTEGER FK | |
| tap_number | INTEGER | 1-based |
| beer_id | INTEGER FK | nullable; null = empty |
| prices | TEXT | JSON array of {size, price} pairs |
| updated_at | DATETIME | |

### `queue_item`
| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| pub_id | INTEGER FK | |
| beer_id | INTEGER FK | |
| prices | TEXT | JSON array of {size, price} pairs |
| position | INTEGER | ordering |
| created_at | DATETIME | |

### `tap_history`
| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| pub_id | INTEGER FK | |
| tap_number | INTEGER | |
| beer_id | INTEGER FK | |
| prices | TEXT | JSON snapshot |
| tapped_at | DATETIME | |
| removed_at | DATETIME | |

---

## 8. API Surface (PoC)

### Public (no auth)
```
GET  /pubs                          — discovery site: list all non-offline pubs
GET  /pubs/:slug                    — pub detail + current taps
GET  /pubs/:slug/taps               — current tap list (JSON for embed widget)
```

### Owner (session auth)
```
# Tap management
GET  /admin/taps                    — current taps view
POST /admin/taps/:tap_number/switch — switch: body contains queue_item_id
POST /admin/taps/:tap_number/empty  — mark tap as empty
POST /admin/taps/:tap_number/quick  — quick add (name + brewery)
POST /admin/taps/:tap_number/undo   — undo last switch (within 30s)

# Queue management
GET  /admin/queue                   — current queue
POST /admin/queue                   — add beer to queue (with prices)
DELETE /admin/queue/:id             — remove from queue
PUT  /admin/queue/:id/position      — reorder

# Beer search
GET  /beers/search?q=               — search local db + Untappd fallback

# Pub settings
GET  /admin/settings                — pub profile
PUT  /admin/settings                — update tap count, offline status, etc.
```

### Auth
```
POST /auth/login
POST /auth/logout
```

---

## 9. Tech Stack

| Layer | Technology |
|---|---|
| Backend API + auth | Rust + Axum |
| Database | SQLite (SQLx) |
| Owner UI | Server-rendered HTML, Maud, HTMX |
| Public tap list | Same backend; cacheable |
| Embed widget | Vanilla JavaScript |
| Discovery site | Same backend |
| Social posting | Python microservice — **deferred, not built for PoC** |
| Beer reference DB | Scraped local dataset (atlaspiv.cz or similar) |

---

## 10. Success Criteria for PoC

The PoC is considered successful if:

1. At least one of the three target pubs actively uses it to manage their tap list for one month
2. The Switch action works without visible errors during service hours
3. The public tap list is accurate within seconds of a switch
4. The pub owner reports it requires less effort than their current process (chalkboard / Facebook post)

---

## 11. Deferred to Full Release

| Feature | Notes |
|---|---|
| Facebook auto-posting | Requires Meta App Review (weeks); deferred |
| Instagram auto-posting | Same; Stories API may not exist |
| Social clipboard copy | Simple workaround for PoC — owner copies formatted tap list |
| Self-serve pub registration | Manual for PoC; needs homepage + onboarding for release |
| Staff roles & permissions | Flat multi-user for PoC |
| Kiosk / wall display view | Nice to have; TV + browser URL is the approach when ready |
| Real-time push (SSE/WebSocket) | Polling on page load sufficient for PoC embed |
| Tap count editing in admin | Static for PoC; editable in full release |
| Global beer catalogue / dedup | Per-pub beer data acceptable for PoC |
| Untappd check-in links | Vision feature; post-PoC |
| Geolocation on discovery site | Post-PoC |
| User favourites | Post-PoC |

---

*End of PoC PRD v1.0*
