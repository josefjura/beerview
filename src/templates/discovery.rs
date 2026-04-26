use maud::{html, Markup};
use crate::templates::layout::layout_minimal;

pub fn render_discovery_page(pubs: &[(String, String, Option<String>)]) -> Markup {
    let content = html! {
        h1 { "Craft Beer Pubs on Beerview" }
        @if pubs.is_empty() {
            p { "No pubs listed yet." }
        } @else {
            ul class="pub-list" {
                @for (slug, name, neighbourhood) in pubs {
                    li class="pub-item" {
                        a href=(format!("/pubs/{}", slug)) { (name) }
                        @if let Some(ref n) = neighbourhood {
                            span class="neighbourhood" { " · " (n) }
                        }
                    }
                }
            }
        }
    };
    layout_minimal("Pubs on Beerview", content)
}
