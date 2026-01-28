use axum::Json;
use hyper::StatusCode;
use serde_json::json;

use crate::AppState;

pub async fn get_http_server(state: &AppState) -> tokio::net::TcpListener {
    let listen_address = get_listen_host(state);
    return tokio::net::TcpListener::bind(listen_address).await.unwrap();
}

pub fn get_listen_host(state: &AppState) -> String {
    let listen_address = &state.app_config.http.address;
    let port = &state.app_config.http.port;
    format!("{listen_address}:{port}")
}

// pub fn get_consul_service_address(config: Arc<BalancerConfig>) -> String {
//     let listen_address = &config.consul.service_address;
//     let port = &config.http.port;
//     format!("{listen_address}:{port}")
// }

pub fn json_status_response(code: StatusCode) -> (StatusCode, Json<serde_json::Value>) {
    (
        code,
        Json(json!({
            "ok": code.is_success(),
            "code": code.as_u16(),
            "msg": code.canonical_reason()
        })),
    )
}
