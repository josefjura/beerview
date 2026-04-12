use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use maud::html;

#[derive(Debug)]
pub enum AppError {
    NotFound(&'static str),
    Database(sqlx::Error),
    Unauthorized,
    Conflict(String),
    Validation(String),
    BadRequest(String),
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message): (StatusCode, String) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.to_string()),
            AppError::Unauthorized => (StatusCode::FORBIDDEN, "Not authorised".to_string()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Database(e) => {
                tracing::error!("Database error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "A database error occurred".to_string())
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "An internal error occurred".to_string())
            }
        };
        let body = html! {
            div class="error-message" {
                p { (message) }
            }
        };
        (status, Html(body.into_string())).into_response()
    }
}
