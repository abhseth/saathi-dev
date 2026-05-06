use axum::{middleware, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Performance budget middleware — logs request duration and warns if budgets are breached.
async fn perf_budget_middleware(
    req: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let start = std::time::Instant::now();
    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    let response = next.run(req).await;

    let elapsed = start.elapsed().as_millis() as u64;
    let budget = if path.starts_with("/api/analytics") {
        500 // P2: Analytics dashboards
    } else if path.starts_with("/api/tickets")
        || path.starts_with("/api/comments")
        || path.starts_with("/api/schools")
    {
        if method == "GET" {
            150 // P1: List views
        } else {
            50 // P0: Simple CRUD
        }
    } else if method == "GET" {
        150
    } else {
        50 // P0: Simple CRUD
    };

    if elapsed > budget {
        tracing::warn!(
            method = %method,
            path = %path,
            elapsed_ms = elapsed,
            budget_ms = budget,
            "Performance budget breached"
        );
    } else {
        tracing::info!(
            method = %method,
            path = %path,
            elapsed_ms = elapsed,
            budget_ms = budget,
            "Request completed within budget"
        );
    }

    response
}

mod alerts;
mod analytics;
mod auth;
mod bulk_ops;
mod db;
mod digests;
mod error;
mod escalation;
mod models;
mod policies;
mod repo;
use repo as repositories;
mod routes;
mod substitution_engine;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "tickets.sqlite3".to_string());

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

    let manager = r2d2_sqlite::SqliteConnectionManager::file(&db_path).with_init(|c| {
        c.execute_batch("PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")
            .map_err(|e| e.into())
    });

    let pool = r2d2::Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("Failed to create connection pool");

    let jwt_secret =
        std::env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set");

    if jwt_secret.len() < 32 {
        panic!("JWT_SECRET must be at least 32 characters");
    }

    let state = Arc::new(models::AppState {
        db: pool,
        jwt_secret,
    });

    // Background SLA breach scanner — runs every 60s, decoupled from hot write path
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                interval.tick().await;
                let start = std::time::Instant::now();
                match state.db.get() {
                    Ok(conn) => {
                        match repositories::refresh_escalations(&*conn) {
                            Ok(count) => {
                                let elapsed = start.elapsed().as_millis();
                                tracing::info!(
                                    "SLA scan completed: {count} ticket(s) updated in {elapsed}ms"
                                );
                                if elapsed > 1000 {
                                    tracing::warn!("SLA scan took {elapsed}ms (>1s); query optimisation needed");
                                }
                            }
                            Err(e) => {
                                tracing::error!("SLA scan failed: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("SLA scan: failed to acquire DB connection: {e}");
                    }
                }
            }
        });
    }

    // CORS: restrict to known origin in production; allow localhost for dev
    let cors = CorsLayer::new()
        .allow_origin(
            std::env::var("CORS_ORIGIN")
                .ok()
                .and_then(|s| s.parse::<axum::http::HeaderValue>().ok())
                .unwrap_or_else(|| "http://localhost:5173".parse().unwrap()),
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let mut app = Router::new()
        .merge(routes::health_router(state.clone()))
        .nest("/api", routes::router(state))
        .layer(middleware::from_fn(perf_budget_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

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
