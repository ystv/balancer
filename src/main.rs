use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
mod check;
mod config;
mod consul;
mod util;
use clap::Parser;
use consul::{client::Consul, config::Config};
use std::{path::PathBuf, sync::Arc};

use crate::config::BalancerConfig;

#[derive(Clone)]
struct AppState {
    consul: Arc<Consul>,
    app_config: Arc<config::BalancerConfig>,
}

#[derive(clap::Parser)]
struct Args {
    #[clap(long = "config", short)]
    config_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let app_config = if let Some(config_file) = args.config_file {
        BalancerConfig::from_file(&config_file)
    } else {
        BalancerConfig::from_env()
    };

    let consul_config = Config {
        address: app_config.consul.agent_url.clone(),
    };

    let consul = Consul::new(consul_config, app_config.clone());

    let cluster_leader = consul.status_leader().await.unwrap();

    println!("Cluster Leader: {cluster_leader}");

    let _ = consul.register_service(false, false).await;

    let state = AppState {
        consul: Arc::new(consul),
        app_config: Arc::new(app_config),
    };

    tokio::spawn(check::start_status_checks(state.clone()));

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .route("/healthz", get(|| async { "still here boss" }))
        .route("/host", get(get_hostname))
        .route("/consul/leader", get(get_cluster_leader))
        .route("/consul/peers", get(get_peers))
        .route("/status", get(get_host_status))
        .with_state(state.clone());

    println!("> Starting HTTP server on port 3000");
    // run our app with hyper, listening globally on port 3000
    let listener = util::get_http_server(state.clone()).await;
    axum::serve(listener, app).await.unwrap();
}

async fn get_hostname(State(state): State<AppState>) -> String {
    state.app_config.hostname.clone()
}

async fn get_cluster_leader(State(state): State<AppState>) -> Result<Json<String>, StatusCode> {
    match state.consul.status_leader().await {
        Ok(leader) => Ok(Json(leader)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_peers(State(state): State<AppState>) -> Result<Json<Vec<String>>, StatusCode> {
    match state.consul.status_peers().await {
        Ok(peers) => Ok(Json(peers)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_host_status(
    State(state): State<AppState>,
) -> Result<Json<Vec<consul::client::KVResponse>>, StatusCode> {
    match state
        .consul
        .get_kv("jenkins/NixOS/hosts/temjin/status".into())
        .await
    {
        Ok(kv_res) => Ok(Json(kv_res)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
