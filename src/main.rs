use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
mod check;
mod config;
mod consul;
mod util;
use clap::Parser;
use consul::{client::Consul, config::Config};
use std::{path::PathBuf, sync::Arc};

use crate::{check::is_eligible, config::BalancerConfig, util::json_status_response};

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
    dotenvy::dotenv().ok();

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
        .route("/status", get(get_eligibility))
        .with_state(state.clone());

    println!("> Starting HTTP server on port 3000");
    // run our app with hyper, listening globally on port 3000
    let listener = util::get_http_server(state.clone()).await;
    axum::serve(listener, app).await.unwrap();
}

async fn get_hostname(State(state): State<AppState>) -> String {
    state.app_config.hostname.clone()
}

async fn get_eligibility(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let client = reqwest::Client::new();

    let is_eligible = match is_eligible(&state, &client).await {
        Ok(eligibility) => eligibility,
        Err(_) => false,
    };

    match is_eligible {
        true => json_status_response(StatusCode::OK),
        false => json_status_response(StatusCode::IM_A_TEAPOT),
    }
}
