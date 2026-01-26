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

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                check_status(state.clone()).await;
            }
        }
    }
}

pub async fn check_status(state: super::AppState) {
    let service_tags = state.consul.get_self().await.unwrap();

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

    let is_primary = service_tags.tags.contains(&"primary".to_string());

    if is_primary {
        println!("In Primary")
    } else {
        println!("In Backup, current status {kv_override}")
    }
}
