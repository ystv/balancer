use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Deserializer};

pub fn deserialize_base64_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let decoded = general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)?;

    String::from_utf8(decoded).map_err(serde::de::Error::custom)
}
