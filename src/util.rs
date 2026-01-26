use std::sync::Arc;

use crate::{config::BalancerConfig, AppState};

pub async fn get_http_server(state: AppState) -> tokio::net::TcpListener {
    let listen_address = get_listen_host(state.app_config);
    return tokio::net::TcpListener::bind(listen_address).await.unwrap();
}

pub fn get_listen_host(config: Arc<BalancerConfig>) -> String {
    let listen_address = &config.http.address;
    let port = &config.http.port;
    format!("{listen_address}:{port}")
}
