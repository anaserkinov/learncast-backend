use crate::module::common::auth::dto::TelegramData;
use anyhow::Result;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub fn verify_telegram_login(
    data: &TelegramData,
    bot_token: &str
) -> Result<()> {

    let json_string = serde_json::to_string(data)?;
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
    
    Ok(())
}
