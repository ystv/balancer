use hyper::http::StatusCode;
use reqwest::{Method, RequestBuilder, Result};
use serde::{Deserialize, Serialize};

use crate::config::BalancerConfig;

use super::config::Config;
use super::service::{AgentServiceRegister, ServiceCheck};

pub struct Consul {
    config: Config,
    app_config: BalancerConfig,
    http_client: reqwest::Client,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct KVResponse {
    #[serde(alias = "Key")]
    pub key: String,
    #[serde(
        alias = "Value",
        deserialize_with = "super::util::deserialize_base64_string"
    )]
    pub value: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ServiceResponse {
    #[serde(alias = "Tags")]
    pub tags: Vec<String>,
}

impl Consul {
    pub fn new(config: Config, app_config: BalancerConfig) -> Self {
        let http_client = reqwest::Client::new();
        Self {
            config,
            app_config,
            http_client,
        }
    }

    fn make_request(&self, method: reqwest::Method, path: &str) -> RequestBuilder {
        let address = &self.config.address;
        return self.http_client.request(method, format!("{address}{path}"));
    }

    pub async fn status_leader(&self) -> Result<String> {
        let res = self
            .make_request(Method::GET, "/v1/status/leader")
            .send()
            .await?
            .json::<String>()
            .await?;
        Ok(res)
    }

    pub async fn get_kv(&self, path: String) -> Result<Vec<KVResponse>> {
        let res = self
            .make_request(Method::GET, &*format!("/v1/kv/{path}"))
            .send()
            .await?
            .json::<Vec<KVResponse>>()
            .await?;
        Ok(res)
    }

    pub async fn put_kv(&self, path: String, value: String) -> Result<bool> {
        let res = self
            .make_request(Method::PUT, &*format!("/v1/kv/{path}"))
            .body(value)
            .send()
            .await?
            .json::<bool>()
            .await?;
        Ok(res)
    }

    pub async fn get_self(&self) -> Result<ServiceResponse> {
        let hostname = &self.app_config.hostname;
        let service_name = &self.app_config.consul.service_name;

        let res = self
            .make_request(
                Method::GET,
                &*format!("/v1/agent/service/{service_name}/{hostname}"),
            )
            .send()
            .await?
            .json::<ServiceResponse>()
            .await?;
        Ok(res)
    }

    pub async fn register_service(&self, active: bool, eligible: bool) -> Result<StatusCode> {
        let hostname = &self.app_config.hostname;
        let service_name = &self.app_config.consul.service_name;

        let mut tags: Vec<String> = Vec::from(["live".into()]);

        if active {
            tags.push("active".into());
        } else {
            tags.push("backup".into());
        }

        if eligible {
            tags.push("eligible".into());
        } else {
            tags.push("ineligible".into());
        }

        let service_address = &self.app_config.consul.service_address;
        let service_port = &self.app_config.http.port;

        let http_check = ServiceCheck {
            http: format!("http://{service_address}:{service_port}/healthz").into(),
            interval: "10s".into(),
            name: "Check It's Still There".into(),
        };

        let service = AgentServiceRegister {
            name: service_name.into(),
            id: Some(format!("{service_name}/{hostname}").into()),
            address: Some((&self.app_config.consul.service_address).into()),
            port: Some(self.app_config.http.port),
            tags,
            checks: Vec::from([http_check]),
        };

        let res = self
            .make_request(Method::PUT, "/v1/agent/service/register")
            .json(&service)
            .send()
            .await?
            .status();
        Ok(res)
    }
}
