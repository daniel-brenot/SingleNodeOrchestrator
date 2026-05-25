use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/system/summary", get(system_summary))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind backend listener");

    tracing::info!(%addr, "starting backend");
    axum::serve(listener, app)
        .await
        .expect("backend server failed");
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn system_summary() -> Json<SystemSummary> {
    Json(SystemSummary {
        hostname: "single-node".to_string(),
        kubernetes: KubernetesSummary {
            status: "not connected".to_string(),
            node_count: 0,
            pod_count: 0,
        },
    })
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct SystemSummary {
    hostname: String,
    kubernetes: KubernetesSummary,
}

#[derive(Serialize)]
struct KubernetesSummary {
    status: String,
    node_count: u32,
    pod_count: u32,
}
