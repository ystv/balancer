use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct BalancerConfig {
    pub hostname: String,
    pub external_url: String,
    pub consul: ConfigConsul,
    pub http: ConfigHttp,
}

#[derive(Deserialize, Clone)]
pub struct ConfigConsul {
    pub agent_url: String,
    pub kv_prefix: String,
    pub service_name: String,
    pub service_address: String,
}

#[derive(Deserialize, Clone)]
pub struct ConfigHttp {
    pub address: String,
    pub port: u16,
    pub reverse_proxy: Option<String>,
}

impl BalancerConfig {
    pub fn from_file(path: &PathBuf) -> Self {
        let app_config_string =
            std::fs::read_to_string(path).expect(&format!("failed to read config file: {path:?}"));

        let app_config: BalancerConfig =
            toml::from_str(&app_config_string).expect("failed to parse config file");

        return app_config;
    }

    pub fn from_env() -> Self {
        Self {
            hostname: get_env_option("BALANCER_HOSTNAME"),
            external_url: get_env_option("BALANCER_EXTERNAL_URL"),
            consul: ConfigConsul::from_env(),
            http: ConfigHttp::from_env(),
        }
    }
}

impl ConfigConsul {
    pub fn from_env() -> Self {
        Self {
            agent_url: get_env_option("BALANCER_CONSUL_AGENT_URL"),
            kv_prefix: get_env_option("BALANCER_CONSUL_KV_PREFIX"),
            service_name: get_env_option("BALANCER_CONSUL_SERVICE_NAME"),
            service_address: get_env_option("BALANCER_CONSUL_SERVICE_ADDRESS"),
        }
    }
}

impl ConfigHttp {
    pub fn from_env() -> Self {
        Self {
            address: get_env_option("BALANCER_HTTP_ADDRESS"),
            port: str::parse::<u16>(&get_env_option("BALANCER_HTTP_PORT"))
                .expect("BALANCER_HTTP_PORT not a number"),
            reverse_proxy: Some(std::env::var("BALANCER_HTTP_REVERSE_PROXY").unwrap()),
        }
    }
}

pub fn get_env_option(key: &'static str) -> String {
    std::env::var("BALANCER_HTTP_ADDRESS").expect(&format!("env {key} not set"))
}
