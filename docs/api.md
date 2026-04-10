# API Design

API-first approach: the core logic is built and tested as a REST API before any UI or integrations are added. The owner UI, social bot, and homepage embed widget are all consumers of this API.

## Auth

JWT-based. Stateless, works cleanly for future mobile clients and the social bot.

```
POST /auth/login
POST /auth/logout
```

## Public endpoints (no auth)

Available to anyone — used by the homepage embed widget and anyone hitting the public tap list URL.

```
GET /pubs/:slug/taps          Current tap list for a pub
GET /pubs/:slug/taps/embed    Lightweight JSON for the embeddable widget
```

## Owner endpoints (JWT required)

### Queue management

```
POST   /pubs/:slug/queue                Add a beer to the queue
DELETE /pubs/:slug/queue/:id            Remove a beer from the queue
PUT    /pubs/:slug/queue/:id/position   Reorder queue items
```

### Tap management

```
POST   /pubs/:slug/queue/:id/switch     Core action: move queued beer onto a tap
DELETE /pubs/:slug/taps/:tap_number     Clear a tap (keg empty, nothing queued yet)
```

`switch` is the heart of the system. It atomically:
1. Archives the current beer on that tap (writes to `tap_history`)
2. Moves the queued beer onto the tap
3. Removes the item from the queue
4. Emits a webhook to the social bot

### History

```
GET /pubs/:slug/history     Archive of all past taps
```

## Beer search

```
GET  /beers/search?q=       Search local DB first, fall back to Untappd API
POST /beers                 Manual beer entry (if not found anywhere)
```

## Summary

~12 endpoints total. Small surface, each endpoint has a single clear responsibility.

## Design decisions

- **JWT over sessions** — stateless auth works for API consumers (social bot, future mobile app) without cookie dependency
- **`:slug` not `:id` in public routes** — human-readable URLs for the public tap list pages (e.g. `/pubs/u-cerneho-vola/taps`)
- **switch as a dedicated endpoint** — not a PATCH on the tap, because it's a meaningful domain event with side effects (archive, webhook), not just a field update
