pub mod config;
pub mod database;
pub mod metrics;
pub mod metrics_timeseries;
mod routes;

use crate::metrics::MetricsHub;
use axum::extract::Extension;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

#[macro_use]
extern crate lazy_static;

pub fn app(metrics: MetricsHub) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(routes::router())
        .layer(Extension(metrics))
        .layer(cors)
}
