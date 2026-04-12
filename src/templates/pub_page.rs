use maud::{html, Markup};

pub fn render_pub_page_placeholder(slug: &str) -> Markup {
    html! {
        h1 { "Pub: " (slug) }
        p { "TODO: implement pub detail" }
    }
}
