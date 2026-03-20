use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::{Html, IntoResponse},
    routing::get,
};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

// ---------------------------------------------------------------------------
// Shared server state
// ---------------------------------------------------------------------------

pub struct ServerState {
    /// The current rendered HTML page
    pub html: RwLock<String>,
    /// Broadcast channel — send anything to trigger a reload in all browsers
    pub reload_tx: broadcast::Sender<()>,
}

impl ServerState {
    pub fn new(html: String) -> Arc<Self> {
        let (reload_tx, _) = broadcast::channel(16);
        Arc::new(Self {
            html: RwLock::new(html),
            reload_tx,
        })
    }

    /// Replace the current HTML and notify all connected browsers to reload.
    pub async fn update(&self, html: String) {
        *self.html.write().await = html;
        let _ = self.reload_tx.send(());
    }
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// Serve the current HTML with a WebSocket reload snippet injected.
async fn serve_html(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let html = state.html.read().await.clone();
    let with_ws = inject_reload_script(&html);
    Html(with_ws)
}

/// Upgrade the connection to a WebSocket and wait for reload signals.
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: Arc<ServerState>) {
    let mut rx = state.reload_tx.subscribe();
    while let Ok(()) = rx.recv().await {
        if socket.send(Message::Text("reload".into())).await.is_err() {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Server startup
// ---------------------------------------------------------------------------

pub fn build_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/", get(serve_html))
        .route("/ws", get(ws_handler))
        .with_state(state)
}

pub async fn start(state: Arc<ServerState>, port: u16) {
    let app = build_router(state);
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {addr}: {e}");
            std::process::exit(1);
        });
    println!("Serving preview at http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn inject_reload_script(html: &str) -> String {
    let script = r#"<script>
  const ws = new WebSocket(`ws://${location.host}/ws`);
  ws.onmessage = () => location.reload();
  ws.onclose = () => console.log('[md-preview] server disconnected');
</script>"#;

    // Inject just before </body> if present, otherwise append
    if let Some(pos) = html.rfind("</body>") {
        let mut result = html.to_string();
        result.insert_str(pos, script);
        result
    } else {
        format!("{html}{script}")
    }
}
