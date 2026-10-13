# FDE Rust Portfolio

A compact, production-style Rust web application and static portfolio site for a forward-deployed AI engineer and systems developer profile. The application combines a static front-end experience with a live telemetry API, a dispatch ingestion endpoint, and a WebSocket telemetry stream, all served by a single Axum HTTP server.

The project is intentionally designed as a portfolio + technical demo: it presents a defense/AI systems narrative while simultaneously demonstrating async Rust server patterns, structured logging, static file hosting, and real-time browser interaction.

## 1. Executive Summary

This repository contains:

- A Rust backend powered by Axum + Tokio
- A static portfolio interface rendered from `public/index.html`
- A JSON telemetry endpoint for operational status
- A POST endpoint for structured dispatch submission
- A WebSocket endpoint that emits synthetic SDR-style telemetry and accepts command messages
- A Dockerized deployment path for containerized runtime execution

The runtime is intentionally lightweight and operationally styled rather than layered enterprise architecture. It is a single-binary web service with a front-end presentation layer and live system simulation features.

## 2. Technology Stack

### Runtime and server
- Rust 2021 edition
- Axum 0.7
- Tokio 1.x
- Tower HTTP static file serving
- Tracing + tracing-subscriber for structured logging

### Serialization and utility
- Serde / Serde JSON
- Futures Util
- Rand

### Presentation layer
- Static HTML/CSS/JS hosted in `public/`
- Tailwind CSS CDN
- Lucide icons
- Custom JavaScript-driven radar and telemetry simulation

### Containerization
- Debian slim runtime container
- Multi-stage Docker build for small final image

## 3. Architecture Overview

The application is a single-process service that owns both application routing and static file delivery.

### Runtime composition

- `src/main.rs` contains the complete server implementation.
- `public/` contains the front-end, assets, and presentation decks.
- `Dockerfile` creates the runtime image.
- `main.py` is a placeholder sample script and is not part of the active app behavior.

### Server responsibilities

The server configures these routes:

- `GET /api/telemetry` -> returns a JSON object describing the service status
- `POST /api/dispatch` -> accepts a contact-form payload and responds with a receipt
- `GET /ws` -> upgrades to a WebSocket connection
- fallback -> serves files under `public/` using `ServeDir::new("public")`

This means the front-end is static and the backend only handles operational endpoints beyond the asset layer.

## 4. Repository Structure

```text
fde-rust-portfolio/
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── Dockerfile
├── main.py
├── public/
│   ├── Autonomous AI Recruiter Agent Architecture.pptx
│   ├── Tactical Army Counter-Drone Platform Briefing.pptx
│   ├── YOLOv8 RF Spectrogram & Satellite Signal Detector.pptx
│   ├── avatar.jpg
│   └── index.html
├── src/
│   └── main.rs
└── README.md
```

## 5. Detailed Component Analysis

### 5.1 Application bootstrap

The binary starts in `src/main.rs` with `#[tokio::main]` and initializes:

- structured logging via `tracing_subscriber`
- router setup
- socket binding using `TcpListener`
- `axum::serve(listener, app)`

The app binds to:

- address: `0.0.0.0`
- port: `PORT` environment variable or default `8080`

This makes the service easy to run in local dev or in a containerized deployment environment.

### 5.2 Telemetry endpoint

Endpoint:

```http
GET /api/telemetry
```

Response schema:

```json
{
  "status": "ACTIVE_OPERATIONAL",
  "node": "EDGE-TX-ALPHA",
  "arch": "x86_64-windows / x86_64-musl (Rust 1.80+)",
  "active_frequency": "2.412 GHz - 5.825 GHz Hopping",
  "memory_footprint": "< 14.2 MB",
  "uptime_seconds": 3600
}
```

This endpoint is a synthetic status frame intended to simulate deployment telemetry rather than a production observability system. It is the operational layer for the front-end status cards.

### 5.3 Dispatch endpoint

Endpoint:

```http
POST /api/dispatch
```

Request body:

```json
{
  "name": "Jane Recruiter",
  "email": "jane@company.com",
  "message": "We are hiring an AI engineer with Rust and systems experience."
}
```

Response:

```json
{
  "acknowledged": true,
  "receipt_id": "OP-123456"
}
```

Behavior:

- derives a random receipt ID using `rand::thread_rng()`
- logs structured textual dispatch data to stdout via tracing
- returns the response envelope as JSON

### 5.4 WebSocket endpoint

Endpoint:

```http
GET /ws
```

The server upgrades the client connection and then performs a two-part interaction model:

1. Sends a welcome message
2. Continues a background loop that emits synthetic SDR telemetry packets every 1.2 seconds
3. Reads client text commands from the socket and responds to recognized commands

Example inbound commands:

- `help`
- `status`
- `rf-suppress`
- `agent-eval`
- `yolo-stream`
- `clear`

The resulting behavior is intentionally theatrical and designed to simulate a tactical operations dashboard. The stream emits packet payloads like:

```json
{
  "type": "sdr_stream",
  "center_freq_mhz": 2412.0,
  "snr_db": "29.5",
  "target_confidence": "96.30%",
  "status": "ACTIVE_BURST"
}
```

The server keeps the socket alive in loop mode and aborts the telemetry task when the client disconnects.

## 6. Data Model and API Contracts

### Telemetry payload

```rust
struct SystemTelemetry {
    status: &'static str,
    node: &'static str,
    arch: &'static str,
    active_frequency: &'static str,
    memory_footprint: &'static str,
    uptime_seconds: u64,
}
```

### Dispatch request

```rust
struct DispatchRequest {
    name: String,
    email: String,
    message: String,
}
```

### Dispatch response

```rust
struct DispatchResponse {
    acknowledged: bool,
    receipt_id: String,
}
```

The service uses `serde` deriving macros to ensure clean JSON conversion between Rust and HTTP payloads.

## 7. Front-End Interaction Model

The static site at `public/index.html` is a single-page portfolio with multiple sections:

- Hero / profile section
- Project exhibits
- Presentation cards
- Radar / simulation section
- Contact form

### Static file hosting

`ServeDir::new("public")` serves the root directory, which means:

- `/` loads the HTML front-end
- assets such as `avatar.jpg` are served automatically
- generated presentation content remains accessible under the same document root

This architecture enables a clean separation between the deployed portfolio experience and the server-side operational layer.

## 8. Build and Run

### Local development

From repo root:

```bash
cargo run
```

This will compile the Rust binary and start the HTTP server on port `8080` by default.

### Custom port

```bash
PORT=9090 cargo run
```

### Release build

```bash
cargo build --release
./target/release/fde-rust-portfolio
```

## 9. Docker Deployment

The repository includes a multi-stage Docker build to keep the final runtime image minimal.

### Build image

```bash
docker build -t fde-rust-portfolio .
```

### Run container

```bash
docker run --rm -p 8080:8080 fde-rust-portfolio
```

### Docker notes

The `Dockerfile` does the following:

1. Uses `rust:1.80-slim` as the build image
2. Copies dependency metadata and creates a stub crate entry for build cache efficiency
3. Builds in release mode
4. Removes generated dependency artifacts from prior build steps to reduce image size
5. Copies the final binary and `PUBLIC` directory to a Debian slim runtime image
6. Exposes port `8080`

This pattern is a pragmatic optimization for small, single-binary web services.

## 10. Cargo Dependencies

The application is intentionally dependency-light and focused. Relevant dependencies in `Cargo.toml`:

```toml
[dependencies]
axum = { version = "0.7", features = ["ws"] }
tokio = { version = "1.37", features = ["full"] }
tower-http = { version = "0.5", features = ["fs", "cors", "trace"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures-util = "0.3"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
rand = "0.8"
```

This dependency set is sufficient for:

- async HTTP server lifecycle
- file serving
- WebSocket support
- structured JSON processing
- event logging
- ephemeral receipt generation

## 11. Operational Behavior and Observability

### Logging

The application initializes tracing with environment-level filtering and default output to stdout:

```rust
tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
    .with(tracing_subscriber::fmt::layer())
    .init();
```

This provides operational visibility for:

- service startup
- request handling
- dispatch events
- WebSocket activity

### Runtime health semantics

The server reflects a synthetic deployment state rather than a formal health-check system. Health is effectively inferred from:

- successful socket bind
- successful route matching
- successful JSON responses from the telemetry and dispatch handlers

## 12. Security and Production Considerations

This is a portfolio/demo application, but there are a few important production-minded notes:

- `POST /api/dispatch` currently accepts unvalidated data beyond basic JSON shape checks.
- The application does not implement TLS termination or reverse proxy hardening.
- There is no persistence layer for messages or receipts.
- The generated receipt IDs are random but not cryptographically signed or persisted.
- The telemetry data is synthetic and should not be interpreted as real system instrumentation.

For production deployment, likely additions would include:

- validation and request schema enforcement
- rate limiting
- structured error handling
- database-backed persistence
- observability stack (Prometheus/OpenTelemetry)
- TLS and authentication for external deployment

## 13. Example Commands

### Health check via telemetry

```bash
curl http://localhost:8080/api/telemetry
```

### Dispatch submission

```bash
curl -X POST http://localhost:8080/api/dispatch \
  -H "Content-Type: application/json" \
  -d '{"name":"Jane","email":"jane@example.com","message":"Interested in AI systems roles."}'
```

### WebSocket quick connect

```bash
websocat ws://localhost:8080/ws
```

Then send commands such as:

```text
help
status
agent-eval
```

## 14. Notable Implementation Notes

### Single-binary design

The code is intentionally self-contained. There is no separate service layer, no separate front-end build system, and no out-of-process worker. This keeps the runtime composition simple and suitable for static demo hosting or lightweight internal deployments.

### Thematic code styling

The code deliberately uses a technical, tactical, and fictional operations aesthetic. This is visible in:

- route names like `dispatch_handler`
- telemetry labels such as `EDGE-TX-ALPHA`
- WebSocket labels like `AXUM-KERNEL`
- command strings such as `rf-suppress` and `agent-eval`

This is not a limitation of the architecture; it is a deliberate front-end narrative designed for a portfolio website and technical storytelling experience.

## 15. Important Note on `main.py`

The file `main.py` is a minimal sample Python script and is not required for the Rust application runtime. It does not participate in the server logic or front-end serving. It appears to be a placeholder artifact rather than an active deployment component.

## 16. Suggested Future Enhancements

Potential upgrades for the project:

- migrate the static portfolio to a real frontend framework
- add persistent contact storage and admin view
- add health check and metrics endpoints
- integrate real telemetry ingestion with system sensors or logs
- add authenticated admin routes
- package with CI/CD for linting, tests, and Docker publishing

## 17. Summary

This repository is a compact, high-impact Rust service that serves as both a personal portfolio experience and a lightweight demonstration of Axum, Tokio, static file hosting, JSON APIs, and WebSocket-driven simulation. It strikes a balance between polished front-end presentation and clean backend implementation, making it suitable for showcasing technical capability in AI systems, browser-based interfaces, and asynchronous Rust service design.

## 18. License

No explicit license file is present in the repository. If this project is intended for public reuse or external distribution, a license should be added before distribution or open-source publication.

---

For local execution, use:

```bash
cargo run
```

Then open:

```text
http://localhost:8080/
```

This repo is ready for local development and container deployment with minimal operational overhead.
