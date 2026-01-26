use serde::Serialize;

#[derive(Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AgentServiceRegister {
    pub name: String,
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<ServiceCheck>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceCheck {
    pub name: String,
    // #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    // pub id: Option<String>,
    pub http: String,
    pub interval: String,
}
