# Concept & Product Design

## Target audience

Small, independent craft beer pubs that:
- Keep their tap list on a physical chalkboard
- Are not on Untappd for Business (and never will be)
- Post to Facebook manually, if at all
- Have no developer on staff
- Are run by people who love beer, not admin work

Prototype target: ~3 specialist craft beer pubs in Prague.

## Core workflow

### Owner side

```
Quiet time (morning / afternoon prep)
  → Search for an upcoming beer by name
  → Beerview pulls name, brewery, style, ABV from Untappd / manual entry
  → Beer goes into the Queue

Service time (busy, behind the bar)
  → Keg runs out
  → Owner taps [Switch] next to the queued beer
  → Old beer moves to Archive, queued beer appears On Tap
  → Everything publishes automatically — no further action needed
```

### Output side (automatic, zero ongoing effort)

| Channel | What happens |
|---|---|
| Public tap list URL | Updates instantly |
| Homepage embed | `<script>` tag placed once by their web person, stays current forever |
| Facebook page | Bot posts a formatted tap list update |
| Instagram | Optional — auto-post to feed or Stories |

## Key design principles

1. **Prep during quiet, execute during busy** — the queue concept exists entirely to separate the thinking from the tapping
2. **One tap to switch** — no forms, no confirmation dialogs, no typing during service
3. **Outputs are automatic** — once configured, the owner never touches them again
4. **Mobile-first** — everything is done on a phone behind the bar

## Beer data

Primary source: **Untappd public API** (beer search + lookup for name, brewery, style, ABV, label image).

Fallback: manual entry. All entered beers are stored locally so the database grows over time and reduces dependency on Untappd.

Untappd API requires registration for a client ID/secret. Free for non-commercial use. Since this is a community project, approval is expected.

## Social posting

Facebook and Instagram posting requires the pub owner to do a **one-time OAuth authorisation** — they log in with Facebook and grant beerview permission to post to their page. After that, posts are fully automatic.

Post format (draft idea):
- Header: pub name + date/time
- List of current beers: name, brewery, style, ABV
- Optional: link to public tap list URL
- Consistent visual style so followers recognise it instantly

Post trigger options:
- Immediately on Switch
- Scheduled (e.g. every day at 15:00 if there are changes)
- Manual "Post now" button

## Homepage embed

A small JavaScript snippet that fetches the current tap list from beerview and renders it inline on the pub's existing website. The pub owner gives the snippet to whoever manages their site — one time, never touched again.

```html
<!-- Example embed -->
<script src="https://beerview.app/embed.js" data-pub="u-cerneho-vola"></script>
```

Renders a styled, auto-updating tap list. Style can be customised via CSS variables or a theme parameter.
