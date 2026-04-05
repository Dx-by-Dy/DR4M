use axum::{Router, routing::get};
use dr4m_server::clients::Clients;
use dr4m_server::handlers::ws_handler;

#[tokio::main]
async fn main() {
    let clients = Clients::default();
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(clients);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
