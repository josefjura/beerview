use maud::{html, Markup};
use crate::auth::session::Session;
use crate::admin::taps::TapView;
use crate::templates::layout::layout;
use crate::templates::components::csrf_token_field;

pub fn render_taps_page(session: &Session, taps: &[TapView]) -> Markup {
    let content = html! {
        h1 { "Your Taps" }
        div id="tap-list" {
            @for tap in taps {
                (render_tap_row(session, tap))
            }
        }
    };
    layout("Taps", content)
}

pub fn render_tap_row(session: &Session, tap: &TapView) -> Markup {
    html! {
        div id=(format!("tap-{}", tap.tap_number)) class="tap-row" {
            span class="tap-number" { "Tap " (tap.tap_number) }
            @if let Some(ref name) = tap.beer_name {
                span class="beer-name" { (name) }
                @if let Some(ref brewery) = tap.beer_brewery {
                    span class="brewery" { " — " (brewery) }
                }
            } @else {
                span class="empty" { "Empty" }
            }

            // Switch button — requires a queue_item_id (placeholder for now)
            form method="post" action=(format!("/admin/taps/{}/switch", tap.tap_number)) {
                (csrf_token_field(&session.csrf_token))
                // queue_item_id will be a select populated from the queue
                // For now render a placeholder input
                input type="number" name="queue_item_id" placeholder="Queue item ID" required;
                button type="submit"
                    hx-post=(format!("/admin/taps/{}/switch", tap.tap_number))
                    hx-target=(format!("#tap-{}", tap.tap_number))
                    hx-swap="outerHTML"
                    hx-include="closest form" {
                    "Switch"
                }
            }

            // Mark Empty button
            form method="post" action=(format!("/admin/taps/{}/empty", tap.tap_number)) {
                (csrf_token_field(&session.csrf_token))
                button type="submit"
                    hx-post=(format!("/admin/taps/{}/empty", tap.tap_number))
                    hx-target=(format!("#tap-{}", tap.tap_number))
                    hx-swap="outerHTML"
                    hx-include="closest form" {
                    "Mark Empty"
                }
            }

            // Undo button — only if within window
            @if tap.can_undo {
                form method="post" action=(format!("/admin/taps/{}/undo", tap.tap_number)) {
                    (csrf_token_field(&session.csrf_token))
                    button type="submit"
                        hx-post=(format!("/admin/taps/{}/undo", tap.tap_number))
                        hx-target=(format!("#tap-{}", tap.tap_number))
                        hx-swap="outerHTML"
                        hx-include="closest form" {
                        "Undo"
                    }
                }
            }
        }
    }
}
