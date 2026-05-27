use crate::metrics::{MemoryMetrics, MetricsHub};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/api/system/memory", get(memory))
        .route("/api/system/memory/ws", get(memory_stream))
}

async fn memory(
    Extension(metrics): Extension<MetricsHub>,
) -> Result<Json<MemoryMetrics>, StatusCode> {
    metrics
        .latest_memory()
        .map(Json)
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)
}

async fn memory_stream(
    ws: WebSocketUpgrade,
    Extension(metrics): Extension<MetricsHub>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| stream_memory_metrics(socket, metrics))
}

async fn stream_memory_metrics(mut socket: WebSocket, metrics: MetricsHub) {
    let mut receiver = metrics.subscribe_memory();

    let initial = receiver.borrow().clone();
    if let Some(latest) = initial {
        if send_memory_metrics(&mut socket, &latest).await.is_err() {
            return;
        }
    }

    loop {
        tokio::select! {
            changed = receiver.changed() => {
                if changed.is_err() {
                    return;
                }

                let latest = receiver.borrow().clone();
                let Some(latest) = latest else {
                    continue;
                };

                if send_memory_metrics(&mut socket, &latest).await.is_err() {
                    return;
                }
            }
            message = socket.recv() => {
                match message {
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            return;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return,
                    _ => {}
                }
            }
        }
    }
}

async fn send_memory_metrics(
    socket: &mut WebSocket,
    metrics: &MemoryMetrics,
) -> Result<(), axum::Error> {
    let payload = serde_json::to_string(metrics).map_err(axum::Error::new)?;
    socket.send(Message::from(payload)).await
}
