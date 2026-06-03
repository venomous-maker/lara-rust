mod app;
mod bootstrap;
mod config;
mod database {
    pub mod migrations;
    pub mod seeders;
}
mod routes;

use std::net::SocketAddr;
use axum::Router;
use tower_http::trace::TraceLayer;

use bootstrap::app::AppState;
use database::migrations::all_migrations;
use routes::{api::api_routes, web::web_routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load `.env` before anything reads configuration.
    lara_core::env::load();

    tracing_subscriber::fmt()
        .with_env_filter(lara_core::env::env_or("RUST_LOG", "info,example=debug"))
        .init();

    // Boot the application: DB + mail facades, then all service providers.
    let state = AppState::boot().await?;

    // Run migrations on startup (dev convenience).
    tracing::info!("running migrations...");
    all_migrations().run().await.map_err(|e| anyhow::anyhow!("migration: {}", e))?;

    // Assemble the HTTP kernel: web routes + /api, shared state, tracing.
    let router = Router::new()
        .merge(web_routes())
        .nest("/api", api_routes(state.clone()))
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = lara_core::env::env_or("APP_ADDR", format!("0.0.0.0:{}", state.config.port))
        .parse()?;

    tracing::info!("🦀 {} listening on http://{}", state.config.name, addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
