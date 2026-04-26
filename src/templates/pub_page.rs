use maud::{html, Markup};
use crate::templates::layout::layout_minimal;

pub fn render_pub_page(
    slug: &str,
    name: &str,
    neighbourhood: Option<&str>,
    taps: &[(i64, Option<String>, Option<String>, Option<String>)],
) -> Markup {
    let active_taps: Vec<_> = taps.iter().filter(|(_, beer_name, _, _)| beer_name.is_some()).collect();

    let content = html! {
        h1 { (name) }
        @if let Some(n) = neighbourhood {
            p class="neighbourhood" { (n) }
        }

        h2 { "Currently on tap" }
        @if active_taps.is_empty() {
            p { "No beers currently on tap." }
        } @else {
            ul class="tap-list" {
                @for (tap_number, beer_name, brewery, prices) in &active_taps {
                    li {
                        strong { (beer_name.as_deref().unwrap_or("")) }
                        @if let Some(ref b) = brewery { " — " (b) }
                        @if let Some(ref p) = prices {
                            span class="prices" { " · " (p) }
                        }
                    }
                }
            }
        }

        hr;
        h3 { "Embed this tap list on your website" }
        p { "Add this snippet to your website HTML:" }
        pre { code { (format!(
            r#"<script src="https://YOUR-BEERVIEW-HOST/v1/embed.js" data-pub="{}"></script>"#,
            slug
        )) } }
    };
    layout_minimal(name, content)
}
