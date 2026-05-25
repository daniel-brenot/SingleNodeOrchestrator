use axum::{routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router {
    Router::new().route("/api/system/summary", get(system_summary))
}

async fn system_summary() -> Json<SystemSummaryResponse> {
    Json(SystemSummaryResponse {
        hostname: "single-node".to_string(),
        kubernetes: KubernetesSummary {
            status: "not connected".to_string(),
            node_count: 0,
            pod_count: 0,
        },
    })
}

#[derive(Serialize)]
struct SystemSummaryResponse {
    hostname: String,
    kubernetes: KubernetesSummary,
}

#[derive(Serialize)]
struct KubernetesSummary {
    status: String,
    node_count: u32,
    pod_count: u32,
}
