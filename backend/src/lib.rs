mod routes;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

pub fn app() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new().merge(routes::router()).layer(cors)
}
