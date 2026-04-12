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

pub fn layout_minimal(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — Beerview" }
                link rel="stylesheet" href="/static/style.css";
            }
            body {
                main { (content) }
            }
        }
    }
}
