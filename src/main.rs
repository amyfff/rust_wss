use axum::{routing::get, Router};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::net::{SocketAddr, TcpListener}; // Gunakan TcpListener dari std
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod db;
mod error;
mod handlers;
mod models;
mod rate_limiter;
mod routes;
mod ws;

#[derive(Clone)]
pub struct AppState {
    db_pool: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_axum_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_pool = db::init_db_pool().await?;

    let app_state = Arc::new(AppState { db_pool });

    let governor_layer = rate_limiter::create_governor_layer();

    let app = Router::new()
        .merge(routes::create_router(app_state.clone()))
        .route("/health", get(health_check))
        .layer(
            tower::ServiceBuilder::new()
                .layer(governor_layer)
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        );

    let addr_str =
        std::env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    let addr: SocketAddr = addr_str.parse()?;

    // Cara baru untuk menjalankan server di Axum 0.7+
    let listener = TcpListener::bind(addr)?;
    tracing::info!("🚀 Server listening on {}", listener.local_addr()?);
    axum::serve(
        tokio::net::TcpListener::from_std(listener)?,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}