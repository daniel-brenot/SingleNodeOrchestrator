use axum::{routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router {
    Router::new().route("/api/health", get(health))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}
