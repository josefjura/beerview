use maud::{html, Markup};

pub fn csrf_token_field(token: &str) -> Markup {
    html! {
        input type="hidden" name="csrf_token" value=(token);
    }
}

pub fn render_error_partial(message: &str) -> Markup {
    html! {
        div class="error-message" {
            p { (message) }
        }
    }
}

pub fn render_not_found() -> Markup {
    html! {
        div class="not-found" {
            h1 { "Not Found" }
            p { "The page you're looking for doesn't exist." }
            a href="/admin/taps" { "Go back to taps" }
        }
    }
}
