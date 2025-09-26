mod error;
mod routes;
mod state;

use crate::state::{AppState, SharedState};
use anyhow::{Context, Ok};
use axum::{Router, serve};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Deserialize)]
struct Settings {
    server: Server,
    database: Database,
}
#[derive(Debug, Deserialize)]
struct Server {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct Database {
    url: String,
    max_connections: u32,
}

fn load_settings() -> anyhow::Result<Settings> {
    let cfg = config::Config::builder()
        .add_source(config::File::with_name("config/base").required(false))
        .add_source(config::File::with_name("config/local").required(false))
        .add_source(config::Environment::with_prefix("APP").separator("__"))
        .build()?;
    Ok(cfg.try_deserialize::<Settings>()?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    // tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,tower_http=info,axum::rejection=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::info!("🚀 server starting...");
    let settings = load_settings().context("load settings")?;
    tracing::info!(?settings.server.host, port=%settings.server.port, "✅ settings loaded");
    // db
    let pool = PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .connect(&settings.database.url)
        .await
        .context("connect db")?;
    tracing::info!("✅ database connected");

    let state: SharedState = Arc::new(AppState { pool });

    // metrics
    let metrics = routes::metrics::build_metrics();

    // router
    let app = Router::new()
        .nest("/users", routes::users::routes())
        .nest("/metrics", routes::metrics::routes(metrics))
        .with_state(state);
    let addr = SocketAddr::new(settings.server.host.parse().unwrap(), settings.server.port);
    let listener = TcpListener::bind(addr).await?;
    serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    tracing::info!(%addr, "🚀 server listening");
    Ok(())
}
