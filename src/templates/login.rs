use maud::{html, Markup};
use crate::templates::layout::layout_minimal;

pub fn render_login_page(error: Option<&str>) -> Markup {
    let content = html! {
        div class="login-container" {
            h1 { "Beerview" }
            form method="post" action="/auth/login" {
                @if let Some(err) = error {
                    div class="error" { (err) }
                }
                div class="form-group" {
                    label for="username" { "Username" }
                    input type="text" id="username" name="username" required;
                }
                div class="form-group" {
                    label for="password" { "Password" }
                    input type="password" id="password" name="password" required;
                }
                button type="submit" { "Log in" }
            }
        }
    };
    layout_minimal("Login", content)
}
