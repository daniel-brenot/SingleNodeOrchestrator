use single_node_orchestrator_backend::{app, metrics::MetricsHub, metrics_timeseries};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind backend listener");

    tracing::info!(%addr, "starting backend");

    let metrics = MetricsHub::new();
    let local = tokio::task::LocalSet::new();
    let metrics_task_hub = metrics.clone();
    local.spawn_local(async {
        if let Err(error) = metrics_timeseries::run(metrics_task_hub).await {
            tracing::error!(%error, "metrics timeseries task stopped");
        }
    });

    local
        .run_until(async move {
            axum::serve(listener, app(metrics))
                .await
                .expect("backend server failed");
        })
        .await;
}
