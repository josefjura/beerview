use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use tower_sessions::Session as TowerSession;

use crate::auth::session::Session;

pub async fn require_auth(
    tower_session: TowerSession,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    let user_id: Option<i64> = tower_session.get("user_id").await.ok().flatten();
    let pub_id: Option<i64> = tower_session.get("pub_id").await.ok().flatten();
    let csrf_token: Option<String> = tower_session.get("csrf_token").await.ok().flatten();
    let must_change_password: bool = tower_session
        .get("must_change_password")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    match (user_id, pub_id, csrf_token) {
        (Some(user_id), Some(pub_id), Some(csrf_token)) => {
            let session = Session {
                user_id,
                pub_id,
                csrf_token,
                must_change_password,
            };

            // Force password change if required
            if must_change_password
                && !request.uri().path().starts_with("/admin/change-password")
                && !request.uri().path().starts_with("/auth/logout")
            {
                return Redirect::to("/admin/change-password").into_response();
            }

            request.extensions_mut().insert(session);
            next.run(request).await.into_response()
        }
        _ => Redirect::to("/auth/login").into_response(),
    }
}
