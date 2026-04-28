use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod db;
mod error;
mod models;
mod repositories;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let db_path = std::env::var("DATABASE_PATH")
        .unwrap_or_else(|_| "tickets.sqlite3".to_string());

    // Create parent directory if it doesn't exist (e.g. /data on first deploy)
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).expect("Failed to create database directory");
        }
    }

    // Open once to run migrations and set WAL mode (must be done before pool)
    {
        let _conn = db::open_db(&db_path).expect("Failed to open database");
    }

    let manager = r2d2_sqlite::SqliteConnectionManager::file(&db_path)
        .with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;").map_err(|e| e.into()));

    let pool = r2d2::Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("Failed to create connection pool");

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-in-production".to_string());

    if jwt_secret.len() < 32 {
        tracing::warn!("JWT_SECRET is short — use a 32+ character secret in production");
    }

    let state = Arc::new(models::AppState {
        db: pool,
        jwt_secret,
    });

    // CORS: in production, restrict allow_origin to your Vercel domain
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut app = Router::new()
        .nest("/api", routes::router(state))
        .layer(cors);

    // Optionally serve the built frontend from the same process
    if let Ok(frontend_dist) = std::env::var("FRONTEND_DIST") {
        app = app.fallback_service(tower_http::services::ServeDir::new(frontend_dist));
    }

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
