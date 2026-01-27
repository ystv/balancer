use reqwest::{Client, Result};
use tokio::time::{interval, Duration};

use crate::consul::client::ServiceResponse;

#[derive(PartialEq)]
enum EligibleStatus {
    Eligible,
    Ineligible,
    Unknown,
}

impl From<EligibleStatus> for bool {
    fn from(value: EligibleStatus) -> bool {
        matches!(value, EligibleStatus::Eligible)
    }
}

impl From<bool> for EligibleStatus {
    fn from(value: bool) -> Self {
        match value {
            true => EligibleStatus::Eligible,
            false => EligibleStatus::Ineligible,
        }
    }
}

impl std::fmt::Display for EligibleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            EligibleStatus::Eligible => write!(f, "Eligible"),
            EligibleStatus::Ineligible => write!(f, "Ineligible"),
            EligibleStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

pub async fn start_status_checks(state: super::AppState) {
    let mut ticker = interval(Duration::from_secs(5));
    let http_client = reqwest::Client::new();

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                check_status(state.clone(), &http_client).await;
            }
        }
    }
}

pub async fn check_status(state: super::AppState, client: &reqwest::Client) {
    let service_response = get_service_tags(&state).await;

    let is_active_service = is_active_service(&service_response).await;
    let is_active_host = is_active_host(&state, client).await;
    let is_eligible_service = is_eligible_service(&service_response).await;
    let is_eligible_host = is_eligible(&state, client).await.unwrap_or(false);

    if (is_active_service != is_active_host) || (is_eligible_service != is_eligible_host) {
        println!("Status doesn't match: is_active: {is_active_host} but is_active_service: {is_active_service}");
        println!("  - is_active:   {is_active_host} but is_active_service:   {is_active_service}");
        println!(
            "  - is_eligible: {is_eligible_host} but is_eligible_service: {is_eligible_service}"
        );

        println!("Changing service state");
        match state
            .consul
            .register_service(is_active_host, is_eligible_host)
            .await
        {
            Result::Ok(r) => println!("Success!"),
            _ => println!("Failed"),
        }
    }
}

pub async fn get_service_tags(state: &super::AppState) -> ServiceResponse {
    state
        .consul
        .get_self()
        .await
        .unwrap_or(ServiceResponse { tags: Vec::new() })
}

pub async fn is_active_service(service_response: &ServiceResponse) -> bool {
    service_response.tags.contains(&"active".to_string())
}

pub async fn is_eligible_service(service_response: &ServiceResponse) -> bool {
    service_response.tags.contains(&"eligible".to_string())
}

pub async fn is_active_host(state: &super::AppState, client: &reqwest::Client) -> bool {
    let external_url = &state.app_config.external_url;
    let hostname = &state.app_config.hostname;

    let external_host_res = match client.get(format!("{external_url}/host")).send().await {
        Result::Ok(r) => Some(r).unwrap(),
        _ => return false,
    };

    let external_host_hostname = external_host_res.text().await.unwrap_or("".into());

    let external_host = external_host_hostname.trim();

    return external_host == hostname;
}

pub async fn is_eligible(state: &super::AppState, client: &Client) -> Result<bool> {
    let kv_override_status: EligibleStatus = get_override_status(state)
        .await
        .unwrap_or(EligibleStatus::Ineligible);

    let reverse_proxy_status: EligibleStatus = get_reverse_proxy_status(state, client)
        .await
        .unwrap_or(EligibleStatus::Ineligible);

    Ok(kv_override_status.into() && reverse_proxy_status.into())
}

async fn get_override_status(state: &super::AppState) -> Result<EligibleStatus> {
    let hostname = &state.app_config.hostname;
    let kv_prefix = &state.app_config.consul.kv_prefix;

    let mut kv_override: EligibleStatus = match state
        .consul
        .get_kv(format!("{kv_prefix}/{hostname}/eligible").into())
        .await
    {
        Ok(v) => (|| {
            if v.is_empty() {
                return EligibleStatus::Unknown;
            } else {
                let status = v.first().unwrap();
                if status.value == "FALSE".to_string() {
                    return EligibleStatus::Ineligible;
                }
            }
            return EligibleStatus::Eligible;
        })(),
        Err(_) => EligibleStatus::Unknown,
    };

    if kv_override == EligibleStatus::Unknown {
        println!("No KV value found for override, setting...");
        let kv_override_set = state
            .consul
            .put_kv(
                format!("{kv_prefix}/{hostname}/eligible").into(),
                "TRUE".to_string(),
            )
            .await
            .unwrap_or(false);

        if kv_override_set {
            println!("Set KV value for override successfully!");
            kv_override = EligibleStatus::Eligible
        } else {
            println!("Failed to set eligibility KV in consul")
        }
    }

    Ok(kv_override)
}

async fn get_reverse_proxy_status(
    state: &super::AppState,
    client: &Client,
) -> Result<EligibleStatus> {
    let hostname = &state.app_config.hostname;
    let rp_addr = &state.app_config.http.reverse_proxy;

    // Return OK if addr not set
    let rp_addr = match rp_addr {
        Some(addr) => Some(addr).unwrap(),
        None => return Ok(EligibleStatus::Eligible),
    };

    let rp_status_res = match client.get(format!("{rp_addr}/host")).send().await {
        Result::Ok(r) => Some(r).unwrap(),
        _ => return Ok(EligibleStatus::Unknown),
    };

    let rp_status_host = rp_status_res.text().await.unwrap_or("".into());

    Ok((&rp_status_host == hostname).into())
}
