use axum::{
    Extension, Form,
    extract::State,
    response::{Html, IntoResponse, Redirect},
    http::StatusCode,
};
use serde::Deserialize;
use tower_sessions::Session as TowerSession;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use crate::auth::session::Session;
use crate::config::AppState;
use crate::templates::layout::{layout, layout_minimal};
use crate::templates::components::csrf_token_field;

// ─── Settings ────────────────────────────────────────────────────────────────

pub async fn show_settings(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let pub_row = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT name, neighbourhood, webhook_url FROM pub WHERE id=?"
    )
    .bind(session.pub_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let (pub_name, neighbourhood, webhook_url) = pub_row
        .unwrap_or_else(|| ("".to_string(), None, None));

    let markup = render_settings_page(&session, &pub_name, neighbourhood.as_deref(), webhook_url.as_deref(), None);
    Html(markup.into_string())
}

#[derive(Deserialize)]
pub struct UpdateSettingsForm {
    pub csrf_token: String,
    pub name: String,
    pub neighbourhood: Option<String>,
    pub webhook_url: Option<String>,
}

pub async fn update_settings(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<UpdateSettingsForm>,
) -> impl IntoResponse {
    if form.csrf_token != session.csrf_token {
        return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
    }

    if form.name.trim().is_empty() {
        return Html(maud::html! {
            div class="error" { "Pub name is required." }
        }.into_string()).into_response();
    }

    if let Some(ref url) = form.webhook_url {
        if !url.is_empty() && !url.starts_with("https://") {
            return Html(maud::html! {
                div class="error" { "Webhook URL must start with https://" }
            }.into_string()).into_response();
        }
    }

    let webhook_url = form.webhook_url.as_deref().filter(|s| !s.is_empty());
    let neighbourhood = form.neighbourhood.as_deref().filter(|s| !s.is_empty());

    let result = sqlx::query(
        "UPDATE pub SET name=?, neighbourhood=?, webhook_url=? WHERE id=?"
    )
    .bind(form.name.trim())
    .bind(neighbourhood)
    .bind(webhook_url)
    .bind(session.pub_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Html(maud::html! {
            div class="success" { "Settings saved." }
        }.into_string()).into_response(),
        Err(e) => {
            tracing::error!("update_settings failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save settings").into_response()
        }
    }
}

fn render_settings_page(
    session: &Session,
    pub_name: &str,
    neighbourhood: Option<&str>,
    webhook_url: Option<&str>,
    error: Option<&str>,
) -> maud::Markup {
    let content = maud::html! {
        h1 { "Settings" }
        @if let Some(err) = error {
            div class="error" { (err) }
        }
        form method="post" action="/admin/settings"
            hx-post="/admin/settings"
            hx-target="#settings-result"
            hx-swap="innerHTML" {
            (csrf_token_field(&session.csrf_token))
            div class="form-group" {
                label for="name" { "Pub name" }
                input type="text" id="name" name="name" value=(pub_name) required;
            }
            div class="form-group" {
                label for="neighbourhood" { "Neighbourhood" }
                input type="text" id="neighbourhood" name="neighbourhood"
                    value=(neighbourhood.unwrap_or(""));
            }
            div class="form-group" {
                label for="webhook_url" { "Webhook URL (optional)" }
                input type="url" id="webhook_url" name="webhook_url"
                    value=(webhook_url.unwrap_or(""))
                    placeholder="https://your-server.example.com/hook";
            }
            button type="submit" { "Save" }
        }
        div id="settings-result" {}
        hr;
        a href="/admin/change-password" { "Change password" }
    };
    layout("Settings", content)
}

// ─── Change password ──────────────────────────────────────────────────────────

pub async fn show_change_password(
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    Html(render_change_password_page(&session, None).into_string())
}

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    pub csrf_token: String,
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    tower_session: TowerSession,
    Form(form): Form<ChangePasswordForm>,
) -> impl IntoResponse {
    if form.csrf_token != session.csrf_token {
        return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
    }

    if form.new_password != form.confirm_password {
        return Html(render_change_password_page(&session, Some("Passwords do not match.")).into_string()).into_response();
    }

    if form.new_password.len() < 8 {
        return Html(render_change_password_page(&session, Some("Password must be at least 8 characters.")).into_string()).into_response();
    }

    // Load current hash
    let user = sqlx::query_as::<_, (String, bool)>(
        "SELECT password_hash, must_change_password FROM pub_user WHERE id=?"
    )
    .bind(session.user_id)
    .fetch_optional(&state.db)
    .await;

    let (current_hash, must_change) = match user {
        Ok(Some(row)) => row,
        _ => return (StatusCode::INTERNAL_SERVER_ERROR, "User not found").into_response(),
    };

    // If not a forced change, verify current password
    if !must_change && !form.current_password.is_empty() {
        let parsed = PasswordHash::new(&current_hash).expect("stored hash is valid");
        if Argon2::default().verify_password(form.current_password.as_bytes(), &parsed).is_err() {
            return Html(render_change_password_page(&session, Some("Current password is incorrect.")).into_string()).into_response();
        }
    }

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let new_hash = match Argon2::default().hash_password(form.new_password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(e) => {
            tracing::error!("Failed to hash password: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response();
        }
    };

    if let Err(e) = sqlx::query(
        "UPDATE pub_user SET password_hash=?, must_change_password=false WHERE id=?"
    )
    .bind(&new_hash)
    .bind(session.user_id)
    .execute(&state.db)
    .await {
        tracing::error!("Failed to update password: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update password").into_response();
    }

    // Update session
    let _ = tower_session.insert("must_change_password", false).await;

    Redirect::to("/admin/taps").into_response()
}

fn render_change_password_page(session: &Session, error: Option<&str>) -> maud::Markup {
    let content = maud::html! {
        h1 { "Change Password" }
        @if session.must_change_password {
            div class="warning" { "You must set a new password before continuing." }
        }
        @if let Some(err) = error {
            div class="error" { (err) }
        }
        form method="post" action="/admin/change-password" {
            (csrf_token_field(&session.csrf_token))
            @if !session.must_change_password {
                div class="form-group" {
                    label for="current_password" { "Current password" }
                    input type="password" id="current_password" name="current_password" required;
                }
            } @else {
                input type="hidden" name="current_password" value="";
            }
            div class="form-group" {
                label for="new_password" { "New password" }
                input type="password" id="new_password" name="new_password" required;
            }
            div class="form-group" {
                label for="confirm_password" { "Confirm new password" }
                input type="password" id="confirm_password" name="confirm_password" required;
            }
            button type="submit" { "Change Password" }
        }
    };
    layout_minimal("Change Password", content)
}
