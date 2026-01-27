use tokio::time::{interval, Duration};

#[derive(PartialEq)]
enum OverrideStatus {
    Eligible,
    Ineligible,
    Unknown,
}

impl std::fmt::Display for OverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            OverrideStatus::Eligible => write!(f, "Eligible"),
            OverrideStatus::Ineligible => write!(f, "Ineligible"),
            OverrideStatus::Unknown => write!(f, "Unknown"),
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
    let is_eligible = is_eligible(&state).await;
    let is_active_service = is_active_service(&state).await;
    let is_active_host = is_active_host(&state, client).await;

    if !(is_active_service && is_active_host) {
        println!("Status doesn't match is_active: {is_active_host} but is_active_service: {is_active_service}");

        println!("Changing service state");
        state
            .consul
            .register_service(!is_active_service && is_active_host, is_eligible)
            .await
            .expect("failed to change service state");
    }
}

pub async fn is_active_service(state: &super::AppState) -> bool {
    let service_tags = state.consul.get_self().await.unwrap();
    service_tags.tags.contains(&"active".to_string())
}

pub async fn is_active_host(state: &super::AppState, client: &reqwest::Client) -> bool {
    let external_url = &state.app_config.external_url;
    let hostname = &state.app_config.hostname;

    let external_host_res = client
        .get(format!("{external_url}/host"))
        .send()
        .await
        .expect("failed to query external_url")
        .text()
        .await
        .expect("failed to get host from external_url");

    let external_host = external_host_res.trim();

    return external_host == hostname;
}

pub async fn is_eligible(state: &super::AppState) -> bool {
    let hostname = &state.app_config.hostname;
    let kv_prefix = &state.app_config.consul.kv_prefix;

    let mut kv_override: OverrideStatus = match state
        .consul
        .get_kv(format!("{kv_prefix}/{hostname}/eligible").into())
        .await
    {
        Ok(v) => (|| {
            if v.is_empty() {
                return OverrideStatus::Unknown;
            } else {
                let status = v.first().unwrap();
                if status.value == "FALSE".to_string() {
                    return OverrideStatus::Ineligible;
                }
            }
            return OverrideStatus::Eligible;
        })(),
        Err(_) => OverrideStatus::Unknown,
    };

    if kv_override == OverrideStatus::Unknown {
        println!("No KV value found for override, setting...");
        let kv_override_set = state
            .consul
            .put_kv(
                format!("{kv_prefix}/{hostname}/eligible").into(),
                "TRUE".to_string(),
            )
            .await
            .unwrap();

        if kv_override_set {
            println!("Set KV value for override successfully!");
            kv_override = OverrideStatus::Eligible
        } else {
            println!("Failed to set eligibility KV in consul")
        }
    }

    return kv_override == OverrideStatus::Eligible;
}
