# Data Model (Draft)

Early-stage sketch. Subject to change.

## Entities

### `pub`
A participating craft beer pub.

| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| slug | TEXT UNIQUE | URL-friendly identifier, e.g. `u-cerneho-vola` |
| name | TEXT | Display name |
| city | TEXT | |
| address | TEXT | Optional |
| owner_id | INTEGER FK → user | |
| facebook_page_id | TEXT | For Graph API posting |
| facebook_access_token | TEXT | OAuth token (encrypted at rest) |
| instagram_account_id | TEXT | Optional |
| created_at | DATETIME | |

### `user`
Pub owner account.

| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| email | TEXT UNIQUE | |
| password_hash | TEXT | bcrypt |
| created_at | DATETIME | |

### `beer`
A beer in the local database. Populated from Untappd search or manual entry.

| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| untappd_id | INTEGER | Nullable — null if entered manually |
| name | TEXT | |
| brewery_name | TEXT | |
| style | TEXT | |
| abv | REAL | |
| label_url | TEXT | Image URL |
| country | TEXT | |
| created_at | DATETIME | |

### `tap`
Current state of a pub's tap list. One row per tap handle.

| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| pub_id | INTEGER FK → pub | |
| tap_number | INTEGER | 1-based position |
| beer_id | INTEGER FK → beer | Currently on tap |
| tapped_at | DATETIME | When this beer went on |

### `queue_item`
Beers prepared and waiting to go on tap.

| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| pub_id | INTEGER FK → pub | |
| beer_id | INTEGER FK → beer | |
| position | INTEGER | Display order in queue |
| keg_size_litres | REAL | Optional |
| notes | TEXT | e.g. "arrives Thursday" |
| added_at | DATETIME | |

### `tap_history`
Archive of all beers that have been on tap (for future stats/history page).

| Field | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| pub_id | INTEGER FK → pub | |
| beer_id | INTEGER FK → beer | |
| tap_number | INTEGER | |
| tapped_at | DATETIME | |
| removed_at | DATETIME | |

## Core operation: Switch

When an owner taps [Switch] on a queued beer:

1. Current beer on that tap → insert into `tap_history` with `removed_at = now()`
2. Queued beer → update `tap` row with new `beer_id` and `tapped_at = now()`
3. Remove item from `queue_item`
4. Emit webhook to social bot: `{ pub_id, event: "tap_changed", tap_list: [...] }`

Atomic transaction — either all steps succeed or none do.
