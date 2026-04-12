use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod admin;
mod beer;
mod config;
mod db;
mod error;
mod models;
mod public;
mod templates;
mod webhook;

use config::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "beerview=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::from_env().await;
    let app = config::build_router(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
