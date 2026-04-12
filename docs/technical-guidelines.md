# Technical Guidelines — Beerview

> **Audience:** DeepSeek Coder V2-Lite (16B) implementing the beerview codebase.
> **Stack:** Rust · Axum · SQLite (SQLx) · Maud · HTMX · Vanilla JS

---

## 1. Project Structure

```
beerview/
├── Cargo.toml
├── .env                          # DATABASE_URL, SESSION_SECRET, etc.
├── migrations/
│   ├── 20260401000000_create_pubs.sql
│   ├── 20260401000001_create_users.sql
│   ├── 20260401000002_create_beers.sql
│   ├── 20260401000003_create_taps.sql
│   ├── 20260401000004_create_queue.sql
│   └── 20260401000005_create_tap_history.sql
├── src/
│   ├── main.rs                   # Entrypoint: build router, start server
│   ├── config.rs                 # Load environment, build AppState
│   ├── db.rs                     # SqlitePool setup, migration runner
│   ├── error.rs                  # AppError type, IntoResponse impl
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── handlers.rs           # login, logout handlers
│   │   ├── middleware.rs         # require_auth layer, Session extractor
│   │   └── session.rs            # Session struct, CSRF helpers
│   ├── admin/
│   │   ├── mod.rs
│   │   ├── taps.rs               # Tap management handlers
│   │   ├── queue.rs              # Queue management handlers
│   │   └── settings.rs           # Pub settings handlers
│   ├── public/
│   │   ├── mod.rs
│   │   ├── discovery.rs          # GET /v1/pubs
│   │   ├── pub_detail.rs         # GET /v1/pubs/:slug
│   │   └── tap_list.rs           # GET /v1/pubs/:slug/taps (JSON)
│   ├── beer/
│   │   ├── mod.rs
│   │   └── search.rs             # GET /beers/search?q=
│   ├── webhook.rs                # Fire outbound webhook on tap change
│   ├── models/
│   │   ├── mod.rs
│   │   ├── pub_.rs               # Pub struct (pub_ to avoid keyword clash)
│   │   ├── user.rs               # PubUser struct
│   │   ├── beer.rs               # Beer struct
│   │   ├── tap.rs                # Tap struct
│   │   ├── queue_item.rs         # QueueItem struct
│   │   └── tap_history.rs        # TapHistory struct
│   ├── templates/
│   │   ├── mod.rs
│   │   ├── layout.rs             # Base HTML layout (head, nav, footer)
│   │   ├── components.rs         # Reusable Maud components
│   │   ├── admin_taps.rs         # Tap management page/partials
│   │   ├── admin_queue.rs        # Queue management page/partials
│   │   ├── login.rs              # Login form
│   │   ├── discovery.rs          # Public discovery page
│   │   └── pub_page.rs           # Public pub detail page
│   └── embed/
│       └── widget.js             # Vanilla JS embed widget
├── static/
│   ├── style.css
│   └── htmx.min.js               # Vendored HTMX (no CDN dependency)
└── tests/
    ├── common/mod.rs             # Test helpers, test database setup
    ├── test_auth.rs
    ├── test_taps.rs
    ├── test_queue.rs
    └── test_switch.rs
```

### Naming Conventions

| Thing | Convention | Example |
|---|---|---|
| Files | `snake_case.rs` | `tap_history.rs` |
| Structs | `PascalCase` | `QueueItem` |
| Functions | `snake_case` | `switch_tap` |
| Handler functions | `verb_noun` | `show_taps`, `create_queue_item`, `switch_tap` |
| Template functions | `render_noun` | `render_tap_list`, `render_queue_card` |
| Database columns | `snake_case` | `tap_number` |
| URL paths | `kebab-case` for slugs, `snake_case` avoided | `/admin/taps`, `/v1/pubs/:slug` |
| Migration files | `YYYYMMDDHHMMSS_description.sql` | `20260401000000_create_pubs.sql` |

---

## 2. Axum Routing & Handlers

### Router Setup

All routes are registered in `main.rs`. Use nested routers to group by concern:

```rust
use axum::{Router, middleware};
use tower_http::services::ServeDir;

pub fn build_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/v1/pubs", get(public::discovery::list_pubs))
        .route("/v1/pubs/{slug}", get(public::pub_detail::show_pub))
        .route("/v1/pubs/{slug}/taps", get(public::tap_list::get_taps_json));

    let auth_routes = Router::new()
        .route("/auth/login", get(auth::handlers::show_login))
        .route("/auth/login", post(auth::handlers::do_login))
        .route("/auth/logout", post(auth::handlers::do_logout));

    let admin_routes = Router::new()
        .route("/admin/taps", get(admin::taps::show_taps))
        .route("/admin/taps/{tap_number}/switch", post(admin::taps::switch_tap))
        .route("/admin/taps/{tap_number}/empty", post(admin::taps::mark_empty))
        .route("/admin/taps/{tap_number}/quick", post(admin::taps::quick_add))
        .route("/admin/taps/{tap_number}/undo", post(admin::taps::undo_switch))
        .route("/admin/queue", get(admin::queue::show_queue))
        .route("/admin/queue", post(admin::queue::add_to_queue))
        .route("/admin/queue/{id}", delete(admin::queue::remove_from_queue))
        .route("/admin/queue/{id}/position", put(admin::queue::update_position))
        .route("/admin/settings", get(admin::settings::show_settings))
        .route("/admin/settings", put(admin::settings::update_settings))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ));

    let beer_routes = Router::new()
        .route("/beers/search", get(beer::search::search_beers));

    Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(admin_routes)
        .merge(beer_routes)
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
```

### AppState

```rust
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub session_secret: String,
}
```

### Handler Function Signatures

Every handler returns `impl IntoResponse`. Use Axum extractors as function parameters:

```rust
use axum::extract::{State, Path, Form, Query};
use axum::response::{Html, IntoResponse, Redirect};
use axum::Extension;

// State extractor — access the shared AppState (database pool, config)
async fn show_taps(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let pub_id = session.pub_id;
    let taps = sqlx::query_as!(
        Tap,
        "SELECT * FROM tap WHERE pub_id = ? ORDER BY tap_number",
        pub_id
    )
    .fetch_all(&state.db)
    .await;

    match taps {
        Ok(taps) => Html(render_taps_page(&taps, &session).into_string()).into_response(),
        Err(e) => {
            tracing::error!("Failed to load taps: {e}");
            Html(render_error_page("Could not load taps.").into_string()).into_response()
        }
    }
}

// Path extractor — captures URL segments
async fn switch_tap(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(tap_number): Path<i64>,
    Form(form): Form<SwitchForm>,
) -> impl IntoResponse {
    // tap_number comes from /admin/taps/{tap_number}/switch
    // form contains queue_item_id and csrf_token
}

// Query extractor — parses query string
#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
}

async fn search_beers(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let results = sqlx::query_as!(
        Beer,
        "SELECT * FROM beer WHERE name LIKE ? OR brewery LIKE ? LIMIT 20",
        format!("%{}%", params.q),
        format!("%{}%", params.q)
    )
    .fetch_all(&state.db)
    .await;

    match results {
        Ok(beers) => Html(render_beer_search_results(&beers).into_string()).into_response(),
        Err(e) => {
            tracing::error!("Beer search failed: {e}");
            Html(render_beer_search_results(&[]).into_string()).into_response()
        }
    }
}
```

### Response Types

```rust
// HTML response (most common — used for Maud-rendered content)
Html(render_something().into_string())

// JSON response (for the public API /v1/pubs/:slug/taps)
axum::Json(serde_json::json!({ "taps": taps_data }))

// Redirect (after login, after logout)
Redirect::to("/admin/taps")

// Status code with body
(StatusCode::NOT_FOUND, Html(render_not_found().into_string()))
```

---

## 3. SQLx & SQLite

### Connection Pool Setup

```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn create_pool(database_url: &str) -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to create database pool")
}

pub async fn run_migrations(pool: &SqlitePool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}
```

### Query Macros

Always prefer `sqlx::query!` and `sqlx::query_as!` over raw string queries. These are compile-time checked against the database schema.

```rust
// query_as! — maps rows to a struct
#[derive(sqlx::FromRow, Serialize)]
pub struct Tap {
    pub id: i64,
    pub pub_id: i64,
    pub tap_number: i64,
    pub beer_id: Option<i64>,
    pub prices: Option<String>,  // JSON string
    pub updated_at: String,
}

let taps = sqlx::query_as!(
    Tap,
    "SELECT id, pub_id, tap_number, beer_id, prices, updated_at
     FROM tap WHERE pub_id = ? ORDER BY tap_number",
    pub_id
)
.fetch_all(&state.db)
.await?;

// query! — for inserts, updates, deletes where you don't need a result struct
sqlx::query!(
    "UPDATE tap SET beer_id = ?, prices = ?, updated_at = datetime('now')
     WHERE pub_id = ? AND tap_number = ?",
    beer_id,
    prices_json,
    pub_id,
    tap_number
)
.execute(&state.db)
.await?;
```

### Migration Files

Each migration is a plain SQL file in `migrations/`. Name format: `YYYYMMDDHHMMSS_description.sql`.

```sql
-- migrations/20260401000003_create_taps.sql
CREATE TABLE IF NOT EXISTS tap (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pub_id INTEGER NOT NULL REFERENCES pub(id),
    tap_number INTEGER NOT NULL,
    beer_id INTEGER REFERENCES beer(id),
    prices TEXT,  -- JSON array: [{"size": "0.5l", "price": 72}]
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(pub_id, tap_number)
);
```

### Transaction Pattern

The Switch action is the most critical transaction. It must be atomic.

```rust
use sqlx::Acquire;

pub async fn execute_switch(
    db: &SqlitePool,
    pub_id: i64,
    tap_number: i64,
    queue_item_id: i64,
) -> Result<SwitchResult, AppError> {
    let mut tx = db.begin().await?;

    // 1. Load the queue item
    let queue_item = sqlx::query_as!(
        QueueItem,
        "SELECT id, pub_id, beer_id, prices, position, created_at
         FROM queue_item WHERE id = ? AND pub_id = ?",
        queue_item_id,
        pub_id
    )
    .fetch_optional(tx.as_mut())
    .await?
    .ok_or(AppError::NotFound("Queue item not found"))?;

    // 2. Load the current tap (to archive the old beer)
    let current_tap = sqlx::query_as!(
        Tap,
        "SELECT id, pub_id, tap_number, beer_id, prices, updated_at
         FROM tap WHERE pub_id = ? AND tap_number = ?",
        pub_id,
        tap_number
    )
    .fetch_one(tx.as_mut())
    .await?;

    // 3. Archive the old beer (if tap was not empty)
    if let Some(old_beer_id) = current_tap.beer_id {
        sqlx::query!(
            "INSERT INTO tap_history (pub_id, tap_number, beer_id, prices, tapped_at, removed_at)
             VALUES (?, ?, ?, ?, ?, datetime('now'))",
            pub_id,
            tap_number,
            old_beer_id,
            current_tap.prices,
            current_tap.updated_at
        )
        .execute(tx.as_mut())
        .await?;
    }

    // 4. Put the new beer on tap
    sqlx::query!(
        "UPDATE tap SET beer_id = ?, prices = ?, updated_at = datetime('now')
         WHERE pub_id = ? AND tap_number = ?",
        queue_item.beer_id,
        queue_item.prices,
        pub_id,
        tap_number
    )
    .execute(tx.as_mut())
    .await?;

    // 5. Remove the queue item
    sqlx::query!("DELETE FROM queue_item WHERE id = ?", queue_item_id)
        .execute(tx.as_mut())
        .await?;

    // 6. Reorder remaining queue items
    sqlx::query!(
        "UPDATE queue_item SET position = position - 1
         WHERE pub_id = ? AND position > ?",
        pub_id,
        queue_item.position
    )
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(SwitchResult {
        tap_number,
        new_beer_id: queue_item.beer_id,
        old_beer_id: current_tap.beer_id,
    })
}
```

### Error Handling from DB Calls

Never `.unwrap()` database results in handlers. Always use `?` with a proper error type or match explicitly:

```rust
// With the ? operator (requires AppError to implement From<sqlx::Error>)
let tap = sqlx::query_as!(Tap, "SELECT ... WHERE id = ?", id)
    .fetch_one(&state.db)
    .await?;

// With explicit match when you need custom handling
match sqlx::query_as!(Tap, "SELECT ... WHERE id = ?", id)
    .fetch_optional(&state.db)
    .await
{
    Ok(Some(tap)) => { /* use tap */ }
    Ok(None) => return Html(render_not_found().into_string()).into_response(),
    Err(e) => {
        tracing::error!("DB error loading tap: {e}");
        return Html(render_error_page("Database error").into_string()).into_response();
    }
}
```

---

## 4. Maud Templates

### The `html!` Macro — Syntax Reference

Maud uses a macro-based syntax to produce HTML. It does not use angle brackets for elements.

```rust
use maud::{html, Markup, DOCTYPE};

// Elements: element name followed by braces
html! { div { "Hello" } }
// Produces: <div>Hello</div>

// Attributes: inside the element before the content braces
html! { div class="card" id="tap-1" { "Content" } }
// Produces: <div class="card" id="tap-1">Content</div>

// Boolean attributes
html! { button disabled[true] { "Can't click" } }
// Produces: <button disabled>Can't click</button>

// Self-closing elements: use semicolon instead of braces
html! { input type="hidden" name="csrf_token" value=(token); }
// Produces: <input type="hidden" name="csrf_token" value="...">

// Dynamic values: wrap in parentheses
html! { span { (beer.name) } }

// Conditionals
html! {
    @if let Some(style) = &beer.style {
        span class="style" { (style) }
    } @else {
        span class="style unknown" { "Unknown style" }
    }
}

// Loops
html! {
    ul {
        @for tap in taps {
            li { (tap.tap_number) ": " (tap.beer_name) }
        }
    }
}
```

### Component Functions

Every reusable template piece is a plain function returning `Markup`:

```rust
use maud::{html, Markup};

pub fn render_tap_card(tap: &Tap, beer: Option<&Beer>) -> Markup {
    html! {
        div class="tap-card" id={"tap-" (tap.tap_number)} {
            h3 { "Tap " (tap.tap_number) }
            @if let Some(beer) = beer {
                p class="beer-name" { (beer.name) }
                p class="brewery" { (beer.brewery) }
                @if let Some(style) = &beer.style {
                    span class="style" { (style) }
                }
                @if let Some(abv) = beer.abv {
                    span class="abv" { (format!("{:.1}%", abv)) }
                }
            } @else {
                p class="empty" { "Empty" }
            }
        }
    }
}
```

### Layout Pattern

A layout function wraps page content with the full HTML shell:

```rust
use maud::{html, Markup, DOCTYPE};

pub fn layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — Beerview" }
                link rel="stylesheet" href="/static/style.css";
                script src="/static/htmx.min.js" {}
            }
            body {
                nav class="navbar" {
                    a href="/admin/taps" { "Taps" }
                    a href="/admin/queue" { "Queue" }
                    a href="/admin/settings" { "Settings" }
                    form method="post" action="/auth/logout" {
                        button type="submit" { "Logout" }
                    }
                }
                main { (content) }
            }
        }
    }
}
```

Assembling a full page:

```rust
pub fn render_taps_page(taps: &[(Tap, Option<Beer>)], session: &Session) -> Markup {
    let content = html! {
        h1 { "Your Taps" }
        div id="tap-list" {
            @for (tap, beer) in taps {
                (render_tap_card(tap, beer.as_ref()))
            }
        }
    };
    layout("Taps", content)
}
```

### Passing Data

Always borrow data into template functions. Never clone structs just to pass them to a template.

```rust
// CORRECT — borrow
pub fn render_tap_card(tap: &Tap, beer: Option<&Beer>) -> Markup { ... }

// WRONG — unnecessary clone forces caller to give up ownership
pub fn render_tap_card(tap: Tap, beer: Option<Beer>) -> Markup { ... }
```

### Complete Example: Tap List Component

```rust
use maud::{html, Markup};

#[derive(sqlx::FromRow)]
pub struct TapWithBeer {
    pub tap_number: i64,
    pub beer_name: Option<String>,
    pub brewery: Option<String>,
    pub style: Option<String>,
    pub abv: Option<f64>,
    pub prices: Option<String>,
}

pub fn render_pub_tap_list(pub_name: &str, taps: &[TapWithBeer]) -> Markup {
    html! {
        div class="pub-tap-list" {
            h2 { (pub_name) }
            @if taps.is_empty() {
                p { "No beers currently on tap." }
            } @else {
                table class="tap-table" {
                    thead {
                        tr {
                            th { "#" }
                            th { "Beer" }
                            th { "Brewery" }
                            th { "Style" }
                            th { "ABV" }
                            th { "Price" }
                        }
                    }
                    tbody {
                        @for tap in taps {
                            @if tap.beer_name.is_some() {
                                tr {
                                    td { (tap.tap_number) }
                                    td { (tap.beer_name.as_deref().unwrap_or("")) }
                                    td { (tap.brewery.as_deref().unwrap_or("")) }
                                    td { (tap.style.as_deref().unwrap_or("")) }
                                    td {
                                        @if let Some(abv) = tap.abv {
                                            (format!("{:.1}%", abv))
                                        }
                                    }
                                    td { (format_prices(tap.prices.as_deref())) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_prices(prices_json: Option<&str>) -> String {
    let Some(json) = prices_json else { return String::new() };
    let Ok(prices) = serde_json::from_str::<Vec<PriceEntry>>(json) else { return String::new() };
    prices
        .iter()
        .map(|p| format!("{} / {} Kč", p.size, p.price))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(serde::Deserialize)]
struct PriceEntry {
    size: String,
    price: i64,
}
```

---

## 5. HTMX Patterns

**This is the most critical section. Read every line carefully.**

### What HTMX Does

HTMX makes HTTP requests from HTML elements and swaps the server's HTML response into the page. There is no client-side templating or JSON parsing in HTMX handlers. The server returns ready-to-render HTML fragments.

The cycle:
1. User clicks a button (or triggers an event)
2. HTMX sends an HTTP request (GET or POST) to the server
3. The server returns an HTML fragment (NOT a full page, NOT JSON)
4. HTMX swaps that fragment into a target element on the page

### hx-get vs hx-post

- `hx-get` — for reading data: loading a list, fetching search results, refreshing a section
- `hx-post` — for actions that change data: switching a tap, adding to queue, deleting

Rule: if the action modifies server state, use `hx-post` (or `hx-delete`, `hx-put`). If it only reads, use `hx-get`.

### hx-target

Tells HTMX which DOM element to replace with the response.

```html
<!-- Replace by ID -->
hx-target="#tap-list"

<!-- Replace the element itself -->
hx-target="this"

<!-- Replace closest ancestor matching a selector -->
hx-target="closest .tap-card"
```

If `hx-target` is omitted, HTMX replaces the element that triggered the request.

### hx-swap Strategies

| Strategy | What it does | Use case |
|---|---|---|
| `innerHTML` (default) | Replaces the inner content of the target | Refreshing a list inside a container div |
| `outerHTML` | Replaces the entire target element | Replacing a tap card with an updated version |
| `beforeend` | Appends inside the target | Adding a new queue item to the end of a list |
| `afterend` | Inserts after the target | Adding a sibling element |
| `delete` | Removes the target element | Removing a queue item after deletion |
| `none` | Does nothing with the response body | When you only need out-of-band swaps |

### hx-trigger

Default triggers by element type:
- `<button>`, `<a>` → `click`
- `<input>`, `<textarea>`, `<select>` → `change`
- `<form>` → `submit`

Custom triggers:
```html
<!-- Trigger on keyup with 300ms debounce (for search inputs) -->
hx-trigger="keyup changed delay:300ms"

<!-- Trigger on a custom event fired elsewhere on the page -->
hx-trigger="tap-switched from:body"
```

### Forms with HTMX — CRITICAL PATTERN

**ALWAYS wrap `hx-post` and `hx-delete` buttons inside a `<form>` element. ALWAYS include the CSRF token as a hidden input inside the form.**

HTMX sends form data from the enclosing `<form>` element automatically. If there is no `<form>`, no form data is sent — including the CSRF token.

#### ✅ CORRECT Pattern

```rust
pub fn render_switch_button(tap_number: i64, queue_item_id: i64, csrf_token: &str) -> Markup {
    html! {
        form hx-post={"/admin/taps/" (tap_number) "/switch"}
             hx-target={"#tap-" (tap_number)}
             hx-swap="outerHTML"
        {
            input type="hidden" name="csrf_token" value=(csrf_token);
            input type="hidden" name="queue_item_id" value=(queue_item_id);
            button type="submit" class="btn-switch" { "Switch" }
        }
    }
}
```

#### ❌ DO NOT DO THIS — hx-post on button without form wrapper

```rust
// WRONG: hx-post on a button WITHOUT a form wrapper.
// HTMX will NOT send csrf_token or queue_item_id.
// The server receives an empty POST body and returns 422 or 403.
pub fn render_switch_button_BROKEN(tap_number: i64, queue_item_id: i64, csrf_token: &str) -> Markup {
    html! {
        div {
            input type="hidden" name="csrf_token" value=(csrf_token);
            input type="hidden" name="queue_item_id" value=(queue_item_id);
            // hx-post is on the button — WRONG
            button hx-post={"/admin/taps/" (tap_number) "/switch"}
                   hx-target={"#tap-" (tap_number)}
                   hx-swap="outerHTML"
            {
                "Switch"
            }
        }
    }
}
```

**Why it breaks:** When `hx-post` is on the `<button>` with no enclosing `<form>`, HTMX only collects inputs that are children of the triggering element. A `<button>` has no input children, so the POST body is empty. The `csrf_token` and `queue_item_id` inputs are siblings of the button, not children — HTMX never sees them.

**Fix:** Always put `hx-post` on the `<form>` element. The button should be `type="submit"`.

#### ❌ DO NOT DO THIS — missing CSRF token

```rust
// WRONG: form exists but no CSRF token. Server will reject with 403.
pub fn render_delete_button_BROKEN(queue_item_id: i64) -> Markup {
    html! {
        form hx-delete={"/admin/queue/" (queue_item_id)}
             hx-target={"#queue-item-" (queue_item_id)}
             hx-swap="delete"
        {
            button type="submit" { "Remove" }
        }
    }
}
```

#### ✅ CORRECT — delete with CSRF

```rust
pub fn render_delete_button(queue_item_id: i64, csrf_token: &str) -> Markup {
    html! {
        form hx-delete={"/admin/queue/" (queue_item_id)}
             hx-target={"#queue-item-" (queue_item_id)}
             hx-swap="delete"
        {
            input type="hidden" name="csrf_token" value=(csrf_token);
            button type="submit" { "Remove" }
        }
    }
}
```

### Returning Partial HTML from Handlers

When a request comes from HTMX, return only the HTML fragment — NOT a full page. HTMX inserts the fragment into the target element.

Detect HTMX requests by checking for the `HX-Request` header:

```rust
use axum::http::HeaderMap;

async fn show_taps(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let taps = load_taps(&state.db, session.pub_id).await?;
    let is_htmx = headers.get("HX-Request").is_some();

    if is_htmx {
        // Return just the tap list fragment
        Html(render_tap_list_partial(&taps, &session).into_string()).into_response()
    } else {
        // Return full page with layout
        Html(render_taps_page(&taps, &session).into_string()).into_response()
    }
}
```

### Out-of-Band Swaps (hx-swap-oob)

When one action needs to update multiple parts of the page, use out-of-band (OOB) swaps. The server response contains the primary content PLUS additional elements marked with `hx-swap-oob="true"`. HTMX matches them to existing elements by `id` and swaps them in place.

```rust
pub fn render_switch_response(
    updated_tap: &Tap,
    beer: Option<&Beer>,
    remaining_queue: &[QueueItemWithBeer],
    session: &Session,
) -> Markup {
    html! {
        // Primary content: updated tap card (replaces hx-target)
        (render_tap_card_with_actions(updated_tap, beer, session))

        // Out-of-band: also update the queue list
        div id="queue-list" hx-swap-oob="true" {
            @for item in remaining_queue {
                (render_queue_item(item, &session.csrf_token))
            }
        }
    }
}
```

The element with `hx-swap-oob="true"` must have an `id` that matches an existing element on the page. HTMX silently does nothing if no matching element is found.

### COMPLETE WORKED EXAMPLE: Switching a Tap

**Step 1: The Maud template — tap card with Switch buttons**

```rust
// src/templates/admin_taps.rs

pub fn render_tap_card_with_actions(
    tap: &Tap,
    beer: Option<&Beer>,
    queue: &[QueueItemWithBeer],
    session: &Session,
) -> Markup {
    html! {
        div class="tap-card" id={"tap-" (tap.tap_number)} {
            div class="tap-header" {
                h3 { "Tap " (tap.tap_number) }
            }
            div class="tap-body" {
                @if let Some(beer) = beer {
                    p class="beer-name" { (beer.name) }
                    p class="brewery" { (beer.brewery) }
                    @if let Some(abv) = beer.abv {
                        span class="abv" { (format!("{:.1}%", abv)) }
                    }
                    (render_prices(tap.prices.as_deref()))
                } @else {
                    p class="empty-tap" { "Empty" }
                }
            }
            div class="tap-actions" {
                // One Switch form per queued beer
                @if !queue.is_empty() {
                    div class="switch-options" {
                        p { "Switch to:" }
                        @for item in queue {
                            form hx-post={"/admin/taps/" (tap.tap_number) "/switch"}
                                 hx-target={"#tap-" (tap.tap_number)}
                                 hx-swap="outerHTML"
                            {
                                input type="hidden" name="csrf_token" value=(session.csrf_token);
                                input type="hidden" name="queue_item_id" value=(item.id);
                                button type="submit" class="btn-switch" {
                                    (item.beer_name) " (" (item.brewery) ")"
                                }
                            }
                        }
                    }
                }
                // Mark empty button
                @if beer.is_some() {
                    form hx-post={"/admin/taps/" (tap.tap_number) "/empty"}
                         hx-target={"#tap-" (tap.tap_number)}
                         hx-swap="outerHTML"
                    {
                        input type="hidden" name="csrf_token" value=(session.csrf_token);
                        button type="submit" class="btn-empty" { "Mark Empty" }
                    }
                }
            }
        }
    }
}
```

**Step 2: The Axum handler**

```rust
// src/admin/taps.rs

#[derive(Deserialize)]
pub struct SwitchForm {
    pub csrf_token: String,
    pub queue_item_id: i64,
}

pub async fn switch_tap(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(tap_number): Path<i64>,
    Form(form): Form<SwitchForm>,
) -> impl IntoResponse {
    // 1. Validate CSRF
    if form.csrf_token != session.csrf_token {
        return (StatusCode::FORBIDDEN, Html("Forbidden".to_string())).into_response();
    }

    // 2. Execute atomic transaction
    match execute_switch(&state.db, session.pub_id, tap_number, form.queue_item_id).await {
        Ok(_) => {
            // 3. Fire webhook in background (non-blocking, best-effort)
            let db = state.db.clone();
            let pub_id = session.pub_id;
            tokio::spawn(async move {
                if let Err(e) = fire_webhook(&db, pub_id).await {
                    tracing::warn!("Webhook failed for pub {pub_id}: {e}");
                }
            });

            // 4. Load updated data for the response
            match (
                load_tap_with_beer(&state.db, session.pub_id, tap_number).await,
                load_queue_with_beers(&state.db, session.pub_id).await,
            ) {
                (Ok((tap, beer)), Ok(queue)) => {
                    // 5. Return updated tap card + OOB queue update
                    Html(render_switch_response(&tap, beer.as_ref(), &queue, &session)
                        .into_string())
                    .into_response()
                }
                _ => {
                    tracing::error!("Failed to load data after switch");
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Html("Switch succeeded but reload failed. Please refresh.".to_string()))
                    .into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Switch failed for tap {tap_number}: {e}");
            Html(render_switch_error(tap_number).into_string()).into_response()
        }
    }
}
```

**Step 3: What happens in the browser**

1. Owner clicks "Switch to: Raptor IPA" inside tap card `#tap-3`
2. HTMX sends `POST /admin/taps/3/switch` with `csrf_token=...&queue_item_id=7`
3. Server executes atomic transaction, returns HTML:
   - Primary: updated `<div id="tap-3">` showing Raptor IPA
   - OOB: updated `<div id="queue-list">` with one fewer item
4. HTMX replaces `#tap-3` with the new tap card (outerHTML swap)
5. HTMX finds `#queue-list` with `hx-swap-oob="true"` and replaces the queue

The owner sees the tap card update and the queue shrink — no page reload.

### HTMX Anti-Patterns Summary

| Anti-Pattern | Problem | Fix |
|---|---|---|
| `hx-post` on `<button>` without `<form>` wrapper | Empty POST body — no CSRF, no data | Put `hx-post` on `<form>`, button is `type="submit"` |
| Missing CSRF `<input>` in form | Server returns 403 | Always include `input type="hidden" name="csrf_token"` |
| Returning full HTML page from HTMX handler | Page nested inside target element | Return fragment only; check `HX-Request` header |
| Returning JSON from HTMX handler | Raw JSON rendered as text | Always return HTML from HTMX endpoints |
| `innerHTML` when you need `outerHTML` | Old wrapper stays, content duplicated | Use `outerHTML` when replacing the element itself |
| No `id` on swappable elements | `hx-target="#tap-3"` finds nothing | Every swappable element needs a stable unique `id` |
| OOB element `id` doesn't match DOM | OOB swap silently does nothing | Ensure `id` exactly matches existing element |

---

## 6. Authentication & Sessions

### Session Middleware Setup

```rust
use tower_sessions::{SessionManagerLayer, MemoryStore};
use tower_sessions::cookie::SameSite;

pub fn session_layer() -> SessionManagerLayer<MemoryStore> {
    let store = MemoryStore::default();
    SessionManagerLayer::new(store)
        .with_secure(true)
        .with_same_site(SameSite::Lax)
        .with_max_age(time::Duration::days(30))  // Long-lived for pub owners
}
```

### Require Auth Middleware

```rust
// src/auth/middleware.rs

use axum::{
    extract::{State, Request},
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use tower_sessions::Session as TowerSession;

pub async fn require_auth(
    session: TowerSession,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    let user_id: Option<i64> = session.get("user_id").await.ok().flatten();
    let pub_id: Option<i64> = session.get("pub_id").await.ok().flatten();
    let csrf_token: Option<String> = session.get("csrf_token").await.ok().flatten();

    match (user_id, pub_id, csrf_token) {
        (Some(user_id), Some(pub_id), Some(csrf_token)) => {
            request.extensions_mut().insert(Session { user_id, pub_id, csrf_token });
            next.run(request).await.into_response()
        }
        _ => Redirect::to("/auth/login").into_response(),
    }
}
```

### Session Struct

```rust
// src/auth/session.rs

#[derive(Clone)]
pub struct Session {
    pub user_id: i64,
    pub pub_id: i64,
    pub csrf_token: String,
}
```

### Accessing Session in Handlers

```rust
async fn show_taps(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,  // injected by require_auth middleware
) -> impl IntoResponse {
    let taps = load_taps(&state.db, session.pub_id).await;
    // ...
}
```

### CSRF Token

Generated at login, stored in the session, validated on every state-changing request.

**Generating at login:**
```rust
use rand::Rng;

pub fn generate_csrf_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}
```

**Validating in handlers:**
```rust
if form.csrf_token != session.csrf_token {
    return (StatusCode::FORBIDDEN, Html("Invalid CSRF token".to_string())).into_response();
}
```

**Helper for templates:**
```rust
pub fn csrf_token_field(token: &str) -> Markup {
    html! {
        input type="hidden" name="csrf_token" value=(token);
    }
}
```

**Using in a form:**
```rust
form hx-post="/admin/queue" hx-target="#queue-list" hx-swap="beforeend" {
    (csrf_token_field(&session.csrf_token))
    input type="text" name="beer_name" placeholder="Beer name";
    button type="submit" { "Add to Queue" }
}
```

### Password Change

Every user account must support password change. The first login after manual account creation must prompt for a new password.

```rust
// In the pub_user table: add must_change_password BOOLEAN NOT NULL DEFAULT false
// Set to true when developer creates the account manually

// In the require_auth middleware: after validating the session,
// check must_change_password and redirect to /admin/change-password if true
if session.must_change_password {
    if !request.uri().path().starts_with("/admin/change-password") {
        return Redirect::to("/admin/change-password").into_response();
    }
}
```

---

## 7. Error Handling

### Never Panic in Handlers

```rust
// WRONG — panics on None, crashes the entire handler task
let tap = load_tap(&state.db, tap_number).await.unwrap();

// CORRECT — propagate with ?
let tap = load_tap(&state.db, tap_number).await?;

// CORRECT — handle explicitly
match load_tap(&state.db, tap_number).await {
    Ok(Some(tap)) => tap,
    Ok(None) => return Html(render_not_found().into_string()).into_response(),
    Err(e) => {
        tracing::error!("Failed to load tap {tap_number}: {e}");
        return Html(render_error_partial("Could not load tap").into_string()).into_response();
    }
}
```

### AppError Type

```rust
// src/error.rs

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

pub enum AppError {
    NotFound(&'static str),
    Database(sqlx::Error),
    Unauthorized,
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            AppError::Unauthorized => (StatusCode::FORBIDDEN, "Not authorised"),
            AppError::Database(e) => {
                tracing::error!("Database error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "A database error occurred")
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "An internal error occurred")
            }
        };
        (status, Html(render_error_partial(message).into_string())).into_response()
    }
}
```

### Error as HTML Partial (for HTMX handlers)

```rust
pub fn render_error_partial(message: &str) -> Markup {
    html! {
        div class="error-message" {
            p { (message) }
            button hx-get="/admin/taps" hx-target="#tap-list" hx-swap="innerHTML" {
                "Retry"
            }
        }
    }
}
```

### Error as Full Page (for direct browser navigation)

```rust
pub fn render_error_page(message: &str) -> Markup {
    let content = html! {
        div class="error-page" {
            h1 { "Something went wrong" }
            p { (message) }
            a href="/admin/taps" { "Go back to taps" }
        }
    };
    layout("Error", content)
}
```

### Logging

```rust
// Errors with context fields
tracing::error!(pub_id = session.pub_id, tap_number, "Switch failed: {e}");

// Important operations
tracing::info!(pub_id = session.pub_id, tap_number, beer_id, "Tap switched successfully");

// Non-critical warnings
tracing::warn!(pub_id, "Webhook delivery failed (best-effort, not retrying)");
```

---

## 8. API Versioning

All public-facing routes use a `/v1/` prefix. Internal admin and auth routes are unversioned.

```rust
pub fn build_router(state: AppState) -> Router {
    // Public API — versioned, CORS enabled
    let v1_routes = Router::new()
        .route("/v1/pubs", get(public::discovery::list_pubs))
        .route("/v1/pubs/{slug}", get(public::pub_detail::show_pub))
        .route("/v1/pubs/{slug}/taps", get(public::tap_list::get_taps_json))
        .route("/v1/embed.js", get(serve_embed_js))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods([Method::GET]));

    // Internal admin — NOT versioned
    let admin_routes = Router::new()
        .route("/admin/taps", get(admin::taps::show_taps))
        // ...
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Auth — NOT versioned
    let auth_routes = Router::new()
        .route("/auth/login", get(auth::handlers::show_login))
        .route("/auth/login", post(auth::handlers::do_login))
        .route("/auth/logout", post(auth::handlers::do_logout));

    Router::new()
        .merge(v1_routes)
        .merge(admin_routes)
        .merge(auth_routes)
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
```

When v2 is needed: add a separate `v2_routes` block. Both run in parallel until consumers migrate.

---

## 9. Vanilla JS Embed Widget

### Embed Tag

```html
<script src="https://beerview.app/v1/embed.js" data-pub="u-cerneho-vola"></script>
```

### Widget Implementation

```javascript
// src/embed/widget.js
(function () {
    "use strict";

    var script = document.currentScript;
    var pubSlug = script.getAttribute("data-pub");
    if (!pubSlug) {
        console.error("Beerview widget: missing data-pub attribute");
        return;
    }

    var apiBase = script.src.replace(/\/v1\/embed\.js.*$/, "");
    var apiUrl = apiBase + "/v1/pubs/" + encodeURIComponent(pubSlug) + "/taps";

    var container = document.createElement("div");
    container.className = "beerview-widget";
    container.innerHTML = "<p>Loading tap list...</p>";
    script.parentNode.insertBefore(container, script.nextSibling);

    fetch(apiUrl)
        .then(function (response) {
            if (!response.ok) throw new Error("HTTP " + response.status);
            return response.json();
        })
        .then(function (data) {
            container.innerHTML = renderTapList(data.taps, data.pub_name);
        })
        .catch(function (err) {
            console.error("Beerview widget error:", err);
            container.innerHTML = "<p>Could not load tap list.</p>";
        });

    function renderTapList(taps, pubName) {
        if (!taps || taps.length === 0) return "<p>No beers currently on tap.</p>";
        var html = "<div class=\"beerview-taps\">";
        html += "<h3>" + escapeHtml(pubName) + " — On Tap</h3>";
        html += "<table><thead><tr><th>#</th><th>Beer</th><th>Brewery</th><th>Style</th><th>ABV</th><th>Price</th></tr></thead><tbody>";
        for (var i = 0; i < taps.length; i++) {
            var t = taps[i];
            html += "<tr>";
            html += "<td>" + t.tap_number + "</td>";
            html += "<td>" + escapeHtml(t.beer.name) + "</td>";
            html += "<td>" + escapeHtml(t.beer.brewery) + "</td>";
            html += "<td>" + escapeHtml(t.beer.style || "") + "</td>";
            html += "<td>" + (t.beer.abv ? t.beer.abv.toFixed(1) + "%" : "") + "</td>";
            html += "<td>" + formatPrices(t.prices) + "</td>";
            html += "</tr>";
        }
        html += "</tbody></table></div>";
        return html;
    }

    function formatPrices(prices) {
        if (!prices || prices.length === 0) return "";
        return prices.map(function (p) {
            return escapeHtml(p.size) + "&nbsp;/&nbsp;" + p.price + "&nbsp;Kč";
        }).join(", ");
    }

    function escapeHtml(str) {
        if (!str) return "";
        var div = document.createElement("div");
        div.appendChild(document.createTextNode(str));
        return div.innerHTML;
    }
})();
```

### CORS Configuration

Only `/v1/*` routes need CORS. Admin and auth routes must NOT have `Access-Control-Allow-Origin: *`.

```rust
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([Method::GET])
    .allow_headers(Any);

// Apply only to v1_routes, NOT to admin_routes or auth_routes
let v1_routes = Router::new()
    .route("/v1/pubs/{slug}/taps", get(public::tap_list::get_taps_json))
    // ...
    .layer(cors);
```

### JSON Response for Widget

```rust
#[derive(Serialize)]
pub struct TapListResponse {
    pub pub_name: String,
    pub pub_slug: String,
    pub taps: Vec<TapEntry>,
    pub updated_at: String,
    pub schema_version: &'static str,  // Always "1" for v1 endpoint
}

#[derive(Serialize)]
pub struct TapEntry {
    pub tap_number: i64,
    pub beer: BeerInfo,
    pub prices: Vec<PriceEntry>,
}

#[derive(Serialize)]
pub struct BeerInfo {
    pub name: String,
    pub brewery: String,
    pub style: Option<String>,
    pub abv: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct PriceEntry {
    pub size: String,
    pub price: i64,
}
```

---

## 10. Testing

### Unit Tests

Test pure business logic that doesn't touch the database:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_prices_empty() {
        assert_eq!(format_prices(None), "");
        assert_eq!(format_prices(Some("[]")), "");
    }

    #[test]
    fn test_format_prices_single() {
        let json = r#"[{"size": "0.5l", "price": 72}]"#;
        assert_eq!(format_prices(Some(json)), "0.5l / 72 Kč");
    }

    #[test]
    fn test_format_prices_multiple() {
        let json = r#"[{"size": "0.5l", "price": 72}, {"size": "0.3l", "price": 49}]"#;
        assert_eq!(format_prices(Some(json)), "0.5l / 72 Kč, 0.3l / 49 Kč");
    }

    #[test]
    fn test_generate_csrf_token_is_unique() {
        let t1 = generate_csrf_token();
        let t2 = generate_csrf_token();
        assert_ne!(t1, t2);
        assert_eq!(t1.len(), 64);  // 32 bytes → 64 hex chars
    }
}
```

### Integration Tests — Axum Handlers

Use in-memory SQLite with migrations applied:

```rust
// tests/common/mod.rs
use sqlx::SqlitePool;

pub async fn test_pool() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

pub async fn seed_switch_test_data(pool: &SqlitePool) {
    sqlx::query!(
        "INSERT INTO pub (id, slug, name, neighbourhood, tap_count, is_offline)
         VALUES (1, 'test-pub', 'Test Pub', 'Žižkov', 4, false)"
    ).execute(pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO beer (id, name, brewery, style, abv) VALUES (1, 'Raptor IPA', 'Matuška', 'IPA', 6.5)"
    ).execute(pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO beer (id, name, brewery, style, abv) VALUES (2, 'Polotmavé 13', 'Únětický', 'Amber', 5.4)"
    ).execute(pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO tap (pub_id, tap_number, beer_id, prices) VALUES (1, 1, 1, '[{\"size\":\"0.5l\",\"price\":79}]')"
    ).execute(pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO queue_item (id, pub_id, beer_id, prices, position) VALUES (1, 1, 2, '[{\"size\":\"0.5l\",\"price\":65}]', 1)"
    ).execute(pool).await.unwrap();
}
```

```rust
// tests/test_switch.rs

#[tokio::test]
async fn test_switch_tap_atomicity() {
    let pool = test_pool().await;
    seed_switch_test_data(&pool).await;

    let result = execute_switch(&pool, 1, 1, 1).await;
    assert!(result.is_ok());

    // Tap now has beer_id = 2
    let tap = sqlx::query!("SELECT beer_id FROM tap WHERE pub_id = 1 AND tap_number = 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(tap.beer_id, Some(2));

    // Queue is empty
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM queue_item WHERE pub_id = 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0);

    // History was created
    let history = sqlx::query!("SELECT beer_id FROM tap_history WHERE pub_id = 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(history.beer_id, 1);  // Old beer (Raptor IPA) is now in history
}

#[tokio::test]
async fn test_switch_tap_invalid_queue_item_returns_error() {
    let pool = test_pool().await;
    seed_switch_test_data(&pool).await;

    let result = execute_switch(&pool, 1, 1, 999).await;  // queue_item_id 999 doesn't exist
    assert!(result.is_err());

    // Tap must be unchanged — transaction was rolled back
    let tap = sqlx::query!("SELECT beer_id FROM tap WHERE pub_id = 1 AND tap_number = 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(tap.beer_id, Some(1));  // Still the original beer
}
```

### What NOT to Test

- **HTMX interactions** — DOM updates are browser behaviour. Verify with E2E tests (Playwright), not Rust tests.
- **Exact Maud HTML output** — testing exact HTML strings is brittle and breaks on whitespace changes. Test data flow and status codes instead.
- **Migration SQL** — tested implicitly by every integration test that calls `sqlx::migrate!`.

**Focus tests on:**
1. The Switch transaction — atomicity and rollback on failure
2. Auth middleware — rejects unauthenticated requests, passes session data correctly
3. CSRF validation — rejects mismatched tokens
4. Public API JSON schema — correct structure for embed widget consumers
5. Edge cases — empty queue switch, marking already-empty tap, concurrent switches
