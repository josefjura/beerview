use axum::{
    Extension,
    Form,
    extract::State,
    response::IntoResponse,
    http::StatusCode,
};
use serde::Deserialize;
use tower_sessions::Session as TowerSession;

use crate::auth::session::Session;
use crate::config::AppState;

pub async fn show_settings(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: show settings page")
}

#[derive(Deserialize)]
pub struct UpdateSettingsForm {
    pub csrf_token: String,
    pub name: String,
    pub neighbourhood: Option<String>,
    pub webhook_url: Option<String>,
}

pub async fn update_settings(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
    Form(_form): Form<UpdateSettingsForm>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: update settings")
}

pub async fn show_change_password(
    Extension(_session): Extension<Session>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: show change password page")
}

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    pub csrf_token: String,
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

pub async fn change_password(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
    _tower_session: TowerSession,
    Form(_form): Form<ChangePasswordForm>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: change password")
}
