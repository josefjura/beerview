use axum::{middleware, Router};
use axum::routing::{delete, get, post, put};
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;
use tower_sessions::{MemoryStore, SessionManagerLayer};

use crate::auth;
use crate::admin;
use crate::beer;
use crate::public;
use crate::db;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
}

impl AppState {
    pub async fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:./beerview.db".to_string());

        let db = db::create_pool(&database_url).await;
        db::run_migrations(&db).await;

        Self { db }
    }
}

pub fn build_router(state: AppState) -> Router {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET]);

    let v1_routes = Router::new()
        .route("/v1/pubs", get(public::discovery::list_pubs))
        .route("/v1/pubs/{slug}/taps", get(public::tap_list::get_taps_json))
        .route("/v1/embed.js", get(public::tap_list::serve_embed_js))
        .layer(cors);

    let public_html_routes = Router::new()
        .route("/pubs", get(public::discovery::discovery_page))
        .route("/pubs/{slug}", get(public::pub_detail::show_pub));

    let auth_routes = Router::new()
        .route("/auth/login", get(auth::handlers::show_login))
        .route("/auth/login", post(auth::handlers::do_login))
        .route("/auth/logout", post(auth::handlers::do_logout));

    let admin_routes = Router::new()
        .route("/admin/taps", get(admin::taps::show_taps))
        .route("/admin/taps/{tap_number}/switch", post(admin::taps::switch_tap))
        .route("/admin/taps/{tap_number}/undo", post(admin::taps::undo_switch))
        .route("/admin/taps/{tap_number}/empty", post(admin::taps::mark_empty))
        .route("/admin/queue", get(admin::queue::show_queue))
        .route("/admin/queue", post(admin::queue::add_to_queue))
        .route("/admin/queue/{id}", delete(admin::queue::remove_from_queue))
        .route("/admin/queue/{id}/position", put(admin::queue::update_position))
        .route("/admin/settings", get(admin::settings::show_settings))
        .route("/admin/settings", post(admin::settings::update_settings))
        .route("/admin/change-password", get(admin::settings::show_change_password))
        .route("/admin/change-password", post(admin::settings::change_password))
        .route("/beers/search", get(beer::search::search_beers))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ));

    Router::new()
        .merge(v1_routes)
        .merge(public_html_routes)
        .merge(auth_routes)
        .merge(admin_routes)
        .nest_service("/static", ServeDir::new("static"))
        .layer(session_layer)
        .with_state(state)
}
