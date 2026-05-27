mod apps;
mod devices;
mod gpus;
mod health;
mod jobs;
mod memory;
mod network;
mod processor;
mod storage_drives;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(apps::router())
        .merge(devices::router())
        .merge(gpus::router())
        .merge(health::router())
        .merge(jobs::router())
        .merge(memory::router())
        .merge(network::router())
        .merge(processor::router())
        .merge(storage_drives::router())
}
