use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::{process::Command, time::Duration};
use tokio::time::sleep;

const GPU_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const MIB_PER_GIB: f32 = 1024.0;

pub fn router() -> Router {
    Router::new()
        .route("/api/system/gpus", get(gpus))
        .route("/api/system/gpus/ws", get(gpu_stream))
}

async fn gpus() -> Json<GpusResponse> {
    Json(read_gpu_metrics().await)
}

async fn gpu_stream(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(stream_gpu_metrics)
}

async fn stream_gpu_metrics(mut socket: WebSocket) {
    loop {
        let Ok(payload) = serde_json::to_string(&read_gpu_metrics().await) else {
            break;
        };

        if socket.send(Message::from(payload)).await.is_err() {
            break;
        }

        let wait_for_next_refresh = sleep(GPU_REFRESH_INTERVAL);
        tokio::pin!(wait_for_next_refresh);

        loop {
            tokio::select! {
                _ = &mut wait_for_next_refresh => break,
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
}

async fn read_gpu_metrics() -> GpusResponse {
    query_nvidia_smi()
}

fn query_nvidia_smi() -> GpusResponse {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,name,utilization.gpu,memory.used,memory.total,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output();

    let Ok(output) = output else {
        return GpusResponse { gpus: Vec::new() };
    };

    if !output.status.success() {
        return GpusResponse { gpus: Vec::new() };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    GpusResponse {
        gpus: stdout.lines().filter_map(parse_nvidia_smi_gpu).collect(),
    }
}

fn parse_nvidia_smi_gpu(line: &str) -> Option<GpuResponse> {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();

    if fields.len() != 6 {
        return None;
    }

    let index = fields[0];
    let name = fields[1];
    let usage_percent = parse_metric(fields[2])?;
    let memory_used_mib = parse_metric(fields[3])?;
    let memory_total_mib = parse_metric(fields[4])?;
    let temperature_celsius = parse_metric(fields[5])?;

    Some(GpuResponse {
        id: format!("gpu{index}"),
        name: name.to_string(),
        usage_percent: clamp_percent(usage_percent),
        memory_used_gib: memory_used_mib / MIB_PER_GIB,
        memory_total_gib: memory_total_mib / MIB_PER_GIB,
        temperature_celsius,
    })
}

fn parse_metric(value: &str) -> Option<f32> {
    value.parse::<f32>().ok()
}

fn clamp_percent(value: f32) -> f32 {
    value.clamp(0.0, 100.0)
}

#[derive(Serialize)]
struct GpusResponse {
    gpus: Vec<GpuResponse>,
}

#[derive(Serialize)]
struct GpuResponse {
    id: String,
    name: String,
    usage_percent: f32,
    memory_used_gib: f32,
    memory_total_gib: f32,
    temperature_celsius: f32,
}
