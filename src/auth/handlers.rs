use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;
use tower_sessions::Session as TowerSession;
use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::auth::session::generate_csrf_token;
use crate::config::AppState;
use crate::templates::login;

pub async fn show_login() -> impl IntoResponse {
    Html(login::render_login_page(None).into_string())
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(sqlx::FromRow)]
struct LoginUser {
    pub id: i64,
    pub pub_id: i64,
    pub password_hash: String,
    pub must_change_password: bool,
}

pub async fn do_login(
    State(state): State<AppState>,
    tower_session: TowerSession,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    const ERROR_MSG: &str = "Invalid username or password";

    let user = sqlx::query_as::<_, LoginUser>(
        "SELECT id, pub_id, password_hash, must_change_password
         FROM pub_user WHERE username = ?"
    )
    .bind(&form.username)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let user = match user {
        Some(u) => u,
        None => {
            return Html(login::render_login_page(Some(ERROR_MSG)).into_string()).into_response();
        }
    };

    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(h) => h,
        Err(_) => {
            return Html(login::render_login_page(Some(ERROR_MSG)).into_string()).into_response();
        }
    };

    let ok = Argon2::default()
        .verify_password(form.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !ok {
        return Html(login::render_login_page(Some(ERROR_MSG)).into_string()).into_response();
    }

    tower_session.insert("user_id", user.id).await.ok();
    tower_session.insert("pub_id", user.pub_id).await.ok();
    tower_session.insert("csrf_token", generate_csrf_token()).await.ok();
    tower_session.insert("must_change_password", user.must_change_password).await.ok();

    Redirect::to("/admin/taps").into_response()
}

pub async fn do_logout(
    tower_session: TowerSession,
) -> impl IntoResponse {
    let _ = tower_session.flush().await;
    Redirect::to("/auth/login").into_response()
}
