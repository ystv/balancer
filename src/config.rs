use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct BalancerConfig {
    pub hostname: String,
    pub consul: ConfigConsul,
    pub http: ConfigHttp,
}

#[derive(Deserialize, Clone)]
pub struct ConfigConsul {
    pub agent_url: String,
    pub kv_prefix: String,
    pub service_name: String,
}

#[derive(Deserialize, Clone)]
pub struct ConfigHttp {
    pub address: String,
    pub port: u16,
}

pub fn parse_config() -> BalancerConfig {
    let app_config_string = std::fs::read_to_string("config.toml").unwrap();

    let app_config: BalancerConfig = toml::from_str(&app_config_string).unwrap();

    return app_config;
}
