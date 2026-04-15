use std::{env, net::SocketAddr};

use stim_server::{app::build_router, state::AppState};

#[tokio::main]
async fn main() {
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let socket_addr: SocketAddr = bind_addr
        .parse()
        .expect("BIND_ADDR must be a valid socket address");

    let listener = tokio::net::TcpListener::bind(socket_addr)
        .await
        .expect("failed to bind stim-server listener");
    let app = build_router(AppState::in_memory());

    axum::serve(listener, app)
        .await
        .expect("stim-server failed while serving");
}
