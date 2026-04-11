# Open Questions — Round 1

These are the decisions and ambiguities identified by the team that the product owner must resolve before the PRD can be written. Grouped by theme.

---

## 1. Beer Data & Untappd

**Q1.** What is the current status of Untappd API access — applied, pending, or not started? If access is denied or delayed, is pure manual entry acceptable for the MVP, or is a fallback database required?

**Q2.** If Untappd access is granted: are we allowed to cache their data (beer name, brewery, style, ABV, label image) permanently in our own database, or must we always fetch live? This affects both architecture and ToS compliance.

**Q3.** Is there an alternative Czech or European beer database worth considering as a primary or fallback source (e.g. a local dataset, Open Beer DB, or another source)?

---

## 2. The Switch Action

**Q4.** When an owner hits Switch on a queued beer — how is the target tap determined? Does the system auto-assign (e.g. lowest empty tap number), or does the owner select the tap? Auto-assign avoids a second interaction during service but may assign the wrong tap.

**Q5.** What happens when a keg dies and the queue is empty? Can the owner mark a tap as empty/offline? What does the public tap list and kiosk display show for an empty tap — "empty," the old beer still listed, or the tap hidden entirely?

**Q6.** What does "undo" mean precisely?
- (a) Full rollback: old beer goes back on tap, queue item is restored, social post cancelled/deleted
- (b) Quick re-switch shortcut: just makes it easy to put another beer on immediately
Option (a) is significantly more complex, especially if a social post has already been sent.

---

## 3. Social Posting

**Q7.** Is Facebook posting in scope for MVP? Facebook's Graph API now requires App Review with `pages_manage_posts` permission — a multi-week process that Meta frequently rejects. This is not a quick setup.

**Q8.** Is Instagram posting in scope for MVP? It requires the same App Review plus a linked Business account. Instagram Stories posting may not be available via API at all.

**Q9.** When a Facebook/Instagram token expires silently (after 60 days), posts stop without any visible error to the owner. Who monitors this for a free community project — and how are owners notified before it becomes a problem?

---

## 4. Authentication & Multi-User

**Q10.** Can multiple staff members switch taps at a pub, or is it always one owner account? Multi-user requires a role/permissions model; single-user is simpler but means sharing credentials.

**Q11.** How does a pub get an account? Self-registration on the site, or does Josef create accounts manually for the initial pubs? (Manual creation is fine for 3 pubs; it doesn't scale.)

---

## 5. Offline & Resilience

**Q12.** What does "offline tolerance" mean in practice for the Switch action?
- (a) **True offline**: Switch works with no internet, syncs when connectivity returns — requires a service worker or local-first architecture, significant effort on top of HTMX
- (b) **Resilient on bad wifi**: retry logic with a spinner, action is not lost if the request times out — much simpler

Option (a) contradicts the server-rendered HTMX stack. Clarification needed before the reliability requirements can be written.

---

## 6. MVP Scope

**Q13.** What ships in v1? The docs describe many outputs. Which are in scope for the first working version?
- Owner app (queue + switch) — assumed yes
- Public tap list URL per pub — assumed yes
- Kiosk/display view (full-screen, auto-refresh) — in or out?
- Homepage embed widget — in or out?
- Facebook auto-posting — in or out?
- Instagram auto-posting — in or out?
- Discovery site (browse all pubs) — in or out?

**Q14.** Is the kiosk display view a priority for the first pilot pubs? (If yes, SSE/real-time push needs to be planned from the start rather than bolted on.)

---

## 7. Data Model Decisions

**Q15.** How many taps does a pub have — is this fixed at registration or can it change? (Affects whether `tap_count` is a field on `pub` or implicitly defined by the tap records.)

**Q16.** Is price (per tap, per beer) in or out of the MVP schema? Adding it later requires a migration; it is low effort to include a nullable field now.

**Q17.** Beer deduplication: two pubs manually entering the same beer with slightly different names creates duplicates. Does this matter for v1, or is per-pub beer data acceptable (no global beer catalogue)?

---

## 8. Embed Widget

**Q18.** For the first pilot pubs: is the embed self-serve (pub owner/their web contact places the script tag themselves), or does Josef place it manually? Self-serve requires clear documentation and a default style that looks acceptable with zero customisation.

---

## 9. Adoption & Sustainability

**Q19.** What's the path from 3 pubs to the ~20-30 needed for the discovery site to be useful? Is there an active outreach plan, or does growth happen organically?

**Q20.** If a pub stops updating their tap list (holiday, illness, forgot), their "last updated" timestamp makes them look abandoned. Is there a "temporarily closed" or "on pause" state, or is stale data acceptable?

---

## 10. Webhook Architecture

**Q21.** The social posting Python service receives a webhook when a tap is switched. If that webhook call fails (Python service down, network hiccup), does the social post silently not happen? Is a simple retry outbox table (store-and-forward in SQLite) acceptable, or is something more robust needed?

---

*Generated by the beerview PRD team — Round 1. Answers from the product owner will feed directly into Round 2 (the PRD).*
