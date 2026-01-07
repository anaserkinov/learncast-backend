use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};
use base64::Engine;
use serde_json::Value;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
pub struct TelegramAuthData {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: i64,
}

pub fn verify_telegram_login(data: &str, bot_token: &str) -> Result<TelegramAuthData> {
    eprint!("{}: ", data);
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|_| anyhow::anyhow!("Base64 decode failed"))?;

    let json_string = String::from_utf8(decoded)?;
    let value: Value = serde_json::from_str(&json_string)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64;

    let auth_date = value["auth_date"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Missing auth_date"))?;

    if now - auth_date > 15 {
        return Err(anyhow::anyhow!("Telegram auth data expired"));
    }

    let provided_hash = value["hash"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing hash"))?;

    let mut params: Vec<(String, String)> = value.as_object()
        .ok_or_else(|| anyhow::anyhow!("Invalid JSON format"))?
        .iter()
        .filter(|(k, _)| k.as_str() != "hash")
        .map(|(k, v)| {
            (k.clone(), v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string()))
        })
        .collect();
    params.sort_by(|a, b| a.0.cmp(&b.0));

    let data_check_string = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    let secret_key = Sha256::digest(bot_token.as_bytes());

    let mut mac = HmacSha256::new_from_slice(&secret_key)
        .expect("HMAC can take key of any size");

    mac.update(data_check_string.as_bytes());

    let calculated_hash = hex::encode(mac.finalize().into_bytes());

    if calculated_hash != provided_hash {
        return Err(anyhow::anyhow!("Invalid Telegram auth hash"));
    }


    let auth_data: TelegramAuthData = serde_json::from_str(&json_string)?;

    Ok(auth_data)
}
