mod health;
mod system_summary;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(health::router())
        .merge(system_summary::router())
}
