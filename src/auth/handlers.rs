use axum::{
    Extension,
    Form,
    extract::State,
    response::{Html, IntoResponse, Redirect},
    http::StatusCode,
};
use serde::Deserialize;
use tower_sessions::Session as TowerSession;

use crate::auth::session::{Session, generate_csrf_token};
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

pub async fn do_login(
    State(_state): State<AppState>,
    tower_session: TowerSession,
    Form(_form): Form<LoginForm>,
) -> impl IntoResponse {
    // TODO: implement login — verify credentials against DB, store session
    // Stub: always redirect to login with error
    let _ = tower_session;
    let _ = generate_csrf_token;
    (StatusCode::NOT_IMPLEMENTED, "TODO: implement login").into_response()
}

pub async fn do_logout(
    tower_session: TowerSession,
) -> impl IntoResponse {
    let _ = tower_session.flush().await;
    Redirect::to("/auth/login").into_response()
}
