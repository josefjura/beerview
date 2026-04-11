# Open Questions — Round 1

These are the decisions and ambiguities identified by the team that the product owner must resolve before the PRD can be written. Grouped by theme.

---

## 1. Beer Data & Untappd

**Q1.** What is the current status of Untappd API access — applied, pending, or not started? If access is denied or delayed, is pure manual entry acceptable for the MVP, or is a fallback database required?

**A1.** The status is not started. Manual entry for is acceptable for our first PoC, but at least scraping some small database for ease of entry is a must for MVP. We can steer away from Untappd if it becomes a problem, but it wouldn't be user friendly to write all the information all the time. At least brewery names, or better beer names have to have some way of auto-complete. I'm accepting alternative approaches

**Q2.** If Untappd access is granted: are we allowed to cache their data (beer name, brewery, style, ABV, label image) permanently in our own database, or must we always fetch live? This affects both architecture and ToS compliance.

**A2.** From Untappd TOS: 
- If you application stores data from Untappd, you must delete your caches every 24 hours.
- You cannot use the Untappd data to build your own "beer database".
- You cannot use the Untappd API to mine, analyze or provide analytics to third parties or your self. The data from the API is meant to be displayed with attribution, not to generate reports on availability, insights in consumer behavior, etc.

**Q3.** Is there an alternative Czech or European beer database worth considering as a primary or fallback source (e.g. a local dataset, Open Beer DB, or another source)?

**A3.** There are some good sources, but without API
- http://www.atlaspiv.cz/
- https://www.pivnici.cz/seznam/piva/dle-data

Some are better, but sometimes contain attrociously small dataset
- https://openbeer.github.io/
- https://www.openbrewerydb.org/

This needs a deeper look, we need at least one viable alternative strategy to Untappd, even if a bit worse
---

## 2. The Switch Action

**Q4.** When an owner hits Switch on a queued beer — how is the target tap determined? Does the system auto-assign (e.g. lowest empty tap number), or does the owner select the tap? Auto-assign avoids a second interaction during service but may assign the wrong tap.

**A4.** The owner sees current taps, each with Switch button next to it. He hits the button, is provided with the kegs in queue and selects for which one we switch

**Q5.** What happens when a keg dies and the queue is empty? Can the owner mark a tap as empty/offline? What does the public tap list and kiosk display show for an empty tap — "empty," the old beer still listed, or the tap hidden entirely?

**A5.** Great question. There should be options to both add ad-hoc beer AND select the tap as empty. If he selects empty, the public tap doesn't list the tap at all, just a shorter list

**Q6.** What does "undo" mean precisely?
- (a) Full rollback: old beer goes back on tap, queue item is restored, social post cancelled/deleted
- (b) Quick re-switch shortcut: just makes it easy to put another beer on immediately
Option (a) is significantly more complex, especially if a social post has already been sent.

**A6.** I would say go with A, but no need to care about the social post cancellation, maybe just post another one? And maybe delay the social post by a minute, to allow clean undo?

---

## 3. Social Posting

**Q7.** Is Facebook posting in scope for MVP? Facebook's Graph API now requires App Review with `pages_manage_posts` permission — a multi-week process that Meta frequently rejects. This is not a quick setup.

**A7.** Facebook is then out of MVP

**Q8.** Is Instagram posting in scope for MVP? It requires the same App Review plus a linked Business account. Instagram Stories posting may not be available via API at all.

**A8.** Instagram is then also out of MVP

**Q9.** When a Facebook/Instagram token expires silently (after 60 days), posts stop without any visible error to the owner. Who monitors this for a free community project — and how are owners notified before it becomes a problem?

**A9.** WE will completely skip social media connection for MVP. I guess for MVP we will just allow copying of the tap list into clipboard for the owners to easily paste it into their feeds as far as social platforms go and we have to solve this problem for our full release
---

## 4. Authentication & Multi-User

**Q10.** Can multiple staff members switch taps at a pub, or is it always one owner account? Multi-user requires a role/permissions model; single-user is simpler but means sharing credentials.

**A10.** At least for MVP, there is a pub profile and can have multiple user accounts, so users don't have to share passwords. A lot of them will create one and share it anyway, but that's not our problem

**Q11.** How does a pub get an account? Self-registration on the site, or does Josef create accounts manually for the initial pubs? (Manual creation is fine for 3 pubs; it doesn't scale.)

**A11.** Good question. For Our first test rounds, I create it manually. Later, we will need some homepage with registration process.

---

## 5. Offline & Resilience

**Q12.** What does "offline tolerance" mean in practice for the Switch action?
- (a) **True offline**: Switch works with no internet, syncs when connectivity returns — requires a service worker or local-first architecture, significant effort on top of HTMX
- (b) **Resilient on bad wifi**: retry logic with a spinner, action is not lost if the request times out — much simpler

Option (a) contradicts the server-rendered HTMX stack. Clarification needed before the reliability requirements can be written.

**A12.** Resilience is better. API is our source of truth and it needs to be contacted anyway. Giving them local servers (our infrastracture on something like Raspberry Pi and RabbitMQ for resiliency, communicating with our main server) is great idea for robustness, but completely out of scope even for first release.

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

**A13.** For MVP we want the owner app, public tap list url, some form of widget embed and discovery site

**Q14.** Is the kiosk display view a priority for the first pilot pubs? (If yes, SSE/real-time push needs to be planned from the start rather than bolted on.)

**A14.** Definitely not, it's a 'nice to have' at best 

---

## 7. Data Model Decisions

**Q15.** How many taps does a pub have — is this fixed at registration or can it change? (Affects whether `tap_count` is a field on `pub` or implicitly defined by the tap records.)

**A15.** Has to be defined per pub and has to be editable through admin app for full release. Can be staticly set during registration for MVP. But it's not that complex.

**Q16.** Is price (per tap, per beer) in or out of the MVP schema? Adding it later requires a migration; it is low effort to include a nullable field now.

**A16.** Yes, price i a must have for MVP

**Q17.** Beer deduplication: two pubs manually entering the same beer with slightly different names creates duplicates. Does this matter for v1, or is per-pub beer data acceptable (no global beer catalogue)?

**A17.** This depends on how we source our data really. I don't think pushing the pubs to reuse names is a wise strategy.

---

## 8. Embed Widget

**Q18.** For the first pilot pubs: is the embed self-serve (pub owner/their web contact places the script tag themselves), or does Josef place it manually? Self-serve requires clear documentation and a default style that looks acceptable with zero customisation.

**A18.** I don't think they would do it on their own, let's presume I'll help them. If they will be directly against it, we can create documentation and some default style in extra step

---

## 9. Adoption & Sustainability

**Q19.** What's the path from 3 pubs to the ~20-30 needed for the discovery site to be useful? Is there an active outreach plan, or does growth happen organically?

**A19.** I don't really have a plan for this. If only one pub uses it, I'll be happy. A lot of the owners know each other so maybe word of mouth helps?

**Q20.** If a pub stops updating their tap list (holiday, illness, forgot), their "last updated" timestamp makes them look abandoned. Is there a "temporarily closed" or "on pause" state, or is stale data acceptable?

**A20.** Ooof, hard question. Maybe a switch in admin app for the online tap list to go visibly "offline"? But stale data is probably still acceptable

---

## 10. Webhook Architecture

**Q21.** The social posting Python service receives a webhook when a tap is switched. If that webhook call fails (Python service down, network hiccup), does the social post silently not happen? Is a simple retry outbox table (store-and-forward in SQLite) acceptable, or is something more robust needed?

**A21.** This will be most likely postopned with the social media services. But technically, simple retry outbox table is fine for this kind of application
---

*Generated by the beerview PRD team — Round 1. Answers from the product owner will feed directly into Round 2 (the PRD).*
