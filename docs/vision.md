# Vision & Future Thinking

Brainstorming and longer-term ideas. Not commitments — just things worth remembering.

## Two audiences, one flywheel

Beerview serves two distinct groups whose needs reinforce each other:

**Pub owners** adopt it because it eliminates manual work — no more editing websites, no more typing Facebook posts. The bar for adoption is: "a regular offered to set it up for free."

**Beer hunters** use it to plan where to go. A single screen showing current tap lists for all their favourite pubs, updated in real time.

The flywheel: hunters check beerview before going out → owners who aren't on it lose visibility → owners have an incentive to join and stay active → more pubs attract more hunters.

The **last-updated timestamp** is the key trust signal. "Updated 4 minutes ago" means the list is real. "Updated 6 days ago" means ignore it. Transparency about freshness nudges owners to keep their lists current or risk being discounted.

## The discovery site

A dedicated website beyond the per-pub embed — a place for beer hunters to browse all participating pubs.

**Core view:**
- List of all pubs on beerview
- Each card shows: pub name, city/neighbourhood, current tap count, last updated timestamp
- Click through to full live tap list

**Personal favourites:**
- Save your regular pubs
- One screen showing all their current taps before your evening walk

**Later (once enough pubs exist):**
- Filter by neighbourhood / walking distance
- "Open now" filter
- Geolocation — pubs near me with active tap lists

## The real-time pitch

The origin story: sitting in Obývák (tvujpivnibar.cz) watching two beers get switched off the tap. The website still shows the old list. It'll be corrected on Monday. The pitch to the owner is personal and concrete — no sales, just a regular offering something useful for free.

## Reference pub: Obývák (tvujpivnibar.cz)

- Menu page lists beers on tap as static hardcoded text (name, °, ABV, brewery, price)
- No timestamp, no dynamic updates
- Site built on traditional CMS + jQuery — a vanilla JS `<script>` embed would work with zero friction
- Natural insertion point: replace the static beer section on the Menu page with the beerview widget
- The owner would never need to touch that page again

## Potential later integrations

- **Untappd check-in link** per beer — since many craft beer drinkers already use Untappd, linking each tap list entry to the Untappd beer page adds value for hunters without extra work for owners
- **Price display** — optional field in the data model, useful for the discovery site
- **Opening hours** — passive signal for the "open now" filter, doesn't need to be real-time
- **Instagram Stories** — auto-generated tap list graphic, potentially more reach than Facebook for the craft beer demographic in Prague

## Fridge / bottle inventory — deferred

The idea of tracking bottle and can inventory was considered. Decision: **not in scope for now.**

Reasons:
- Tap management is a single binary action at a predictable moment (keg switch). Fridge inventory is ongoing stock management — a fundamentally different behaviour ask from the owner
- Adding it risks losing the "feather light" quality that makes the tap list adoptable
- The moment it becomes stock management software, the lazy-owner-friendly pitch breaks down

**If revisited later:** a low-fidelity "we stock these" list (presence only, no quantities, no real-time accuracy expectation) could work as an optional addition without threatening the core. But taps come first and stay the sharp, reliable, always-current feature.
