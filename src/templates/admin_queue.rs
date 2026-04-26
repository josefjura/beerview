use maud::{html, Markup};
use crate::auth::session::Session;
use crate::admin::queue::QueueItemView;
use crate::templates::layout::layout;
use crate::templates::components::csrf_token_field;

pub fn render_queue_page(session: &Session, items: &[QueueItemView]) -> Markup {
    let content = html! {
        h1 { "Queue" }
        div class="add-beer-form" {
            h2 { "Add Beer to Queue" }
            input type="text" id="beer-search" placeholder="Search existing beers..."
                hx-get="/beers/search"
                hx-trigger="input changed delay:300ms"
                hx-target="#beer-search-results";
            div id="beer-search-results" {}

            form id="add-queue-form" method="post" action="/admin/queue"
                hx-post="/admin/queue"
                hx-target="#queue-list"
                hx-swap="outerHTML" {
                (csrf_token_field(&session.csrf_token))
                input type="hidden" id="selected-beer-id" name="beer_id";

                details {
                    summary { "Add new beer manually" }
                    div class="new-beer-fields" {
                        input type="text" name="beer_name" placeholder="Beer name";
                        input type="text" name="beer_brewery" placeholder="Brewery";
                        input type="text" name="beer_style" placeholder="Style (optional)";
                        input type="number" name="beer_abv" placeholder="ABV % (optional)" step="0.1" min="0" max="20";
                    }
                }
                input type="text" name="prices" placeholder=(r#"[{"size":"0.5l","price":72}]"#);
                button type="submit" { "Add to Queue" }
            }
        }
        (render_queue_list(session, items))
    };
    layout("Queue", content)
}

pub fn render_queue_list(session: &Session, items: &[QueueItemView]) -> Markup {
    html! {
        div id="queue-list" {
            @if items.is_empty() {
                p { "Queue is empty." }
            } @else {
                ol {
                    @for item in items {
                        li id=(format!("queue-item-{}", item.id)) {
                            strong { (item.beer_name) }
                            " — " (item.beer_brewery)
                            @if let Some(ref p) = item.prices { span class="prices" { " " (p) } }
                            " "
                            form method="post" action=(format!("/admin/queue/{}/position", item.id)) style="display:inline" {
                                (csrf_token_field(&session.csrf_token))
                                input type="hidden" name="position" value=(item.position - 1);
                                button type="submit"
                                    hx-put=(format!("/admin/queue/{}/position", item.id))
                                    hx-target="#queue-list" hx-swap="outerHTML" hx-include="closest form"
                                    disabled[item.position == 1] { "↑" }
                            }
                            form method="post" action=(format!("/admin/queue/{}/position", item.id)) style="display:inline" {
                                (csrf_token_field(&session.csrf_token))
                                input type="hidden" name="position" value=(item.position + 1);
                                button type="submit"
                                    hx-put=(format!("/admin/queue/{}/position", item.id))
                                    hx-target="#queue-list" hx-swap="outerHTML" hx-include="closest form" { "↓" }
                            }
                            form method="post" action=(format!("/admin/queue/{}", item.id)) style="display:inline" {
                                (csrf_token_field(&session.csrf_token))
                                button type="submit"
                                    hx-delete=(format!("/admin/queue/{}", item.id))
                                    hx-target="#queue-list" hx-swap="outerHTML" hx-include="closest form" { "✕" }
                            }
                        }
                    }
                }
            }
        }
    }
}
