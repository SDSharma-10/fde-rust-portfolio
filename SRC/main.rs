use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize)]
struct SystemTelemetry {
    status: &'static str,
    node: &'static str,
    arch: &'static str,
    active_frequency: &'static str,
    memory_footprint: &'static str,
    uptime_seconds: u64,
}

#[derive(Deserialize)]
struct DispatchRequest {
    name: String,
    email: String,
    message: String,
}

#[derive(Serialize)]
struct DispatchResponse {
    acknowledged: bool,
    receipt_id: String,
}

#[tokio::main]
async fn main() {
    // Setup structured logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Route configuration
    let app = Router::new()
        .route("/api/telemetry", get(telemetry_handler))
        .route("/api/dispatch", post(dispatch_handler))
        .route("/ws", get(websocket_upgrade_handler))
        .fallback_service(ServeDir::new("public"));

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!(">>> Tactical Rust Server deployed on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind socket address");

    axum::serve(listener, app).await.expect("Server encountered a fatal error");
}

async fn telemetry_handler() -> Json<SystemTelemetry> {
    Json(SystemTelemetry {
        status: "ACTIVE_OPERATIONAL",
        node: "EDGE-TX-ALPHA",
        arch: "x86_64-windows / x86_64-musl (Rust 1.80+)",
        active_frequency: "2.412 GHz - 5.825 GHz Hopping",
        memory_footprint: "< 14.2 MB",
        uptime_seconds: 3600,
    })
}

async fn dispatch_handler(Json(payload): Json<DispatchRequest>) -> Json<DispatchResponse> {
    let receipt_id = {
        let mut rng = rand::thread_rng();
        format!("OP-{}", rng.gen_range(100000..999999))
    };

    tracing::info!(
        "[DIRECT DISPATCH] From: {} <{}> - Message: {}",
        payload.name,
        payload.email,
        payload.message
    );

    Json(DispatchResponse {
        acknowledged: true,
        receipt_id,
    })
}

async fn websocket_upgrade_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_tactical_socket)
}

async fn handle_tactical_socket(socket: WebSocket) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    // Welcome handshake
    let welcome = serde_json::json!({
        "type": "log",
        "content": "[AXUM-KERNEL] Connected to Field Deployed WebSocket Engine. Type 'help' to review operational routines."
    });

    {
        let mut tx = sender.lock().await;
        let _ = tx.send(Message::Text(welcome.to_string())).await;
    }

    // Background Task: Periodic synthetic SDR telemetry broadcast
    let telemetry_sender = Arc::clone(&sender);
    let sdr_task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1200)).await;

            // Generate random numbers within a scoped block before the await point
            let (center_freq, snr, target_prob) = {
                let mut rng = rand::thread_rng();
                let center_freq = 2412.0 + (rng.gen_range(0..10) as f32 * 5.0);
                let snr = rng.gen_range(18.5..34.2);
                let target_prob = rng.gen_range(0.85..0.99);
                (center_freq, snr, target_prob)
            };

            let sdr_packet = serde_json::json!({
                "type": "sdr_stream",
                "center_freq_mhz": center_freq,
                "snr_db": format!("{:.1}", snr),
                "target_confidence": format!("{:.2}%", target_prob * 100.0),
                "status": if snr > 28.0 { "ACTIVE_BURST" } else { "MONITORING" }
            });

            let mut tx = telemetry_sender.lock().await;
            if tx.send(Message::Text(sdr_packet.to_string())).await.is_err() {
                break;
            }
        }
    });

    // Inbound command execution loop
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        let cmd = text.trim().to_lowercase();
        let reply = match cmd.as_str() {
            "help" => "TACTICAL COMMANDS:\r\n  status      - Display running Rust engine telemetry\r\n  rf-suppress - Trigger SDR waveform synthesis loop\r\n  agent-eval  - Execute LangGraph cyclic agent state\r\n  yolo-stream - Inspect edge visual inference logs\r\n  clear       - Clear screen buffer\r\n",
            "status" => "[SYS] Rust 1.80+ | Tokio Async | RAM: ~12.8 MB | Cold-Boot: ~18ms | Sockets: Active\r\n",
            "rf-suppress" => "[SDR-ENGINE] Synthesized dynamic frequency hopping sequence across 2.4/5.8 GHz. Suppression locked.\r\n",
            "agent-eval" => "[LANGGRAPH] Ingested raw telemetry -> Multi-agent state evaluator -> Tool validation complete. 0 retries.\r\n",
            "yolo-stream" => "[PERCEPTION] YOLOv8 Spectrogram Classifier: UAS Signal Signature verified (Confidence: 97.4%).\r\n",
            "clear" => "__CLEAR__\r\n",
            _ => "Unknown command sequence. Type 'help' for available procedures.\r\n",
        };

        let mut tx = sender.lock().await;
        if tx.send(Message::Text(reply.to_string())).await.is_err() {
            break;
        }
    }

    sdr_task.abort();
}