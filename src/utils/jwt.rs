use serde::{Deserialize, Serialize};
use anyhow::Result;
use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation, Algorithm};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, Duration};
use crate::utils::CONFIG;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

pub fn generate(
    user_id: i64,
    role: &str
) -> Result<(String, String)> {
    let now = OffsetDateTime::now_utc();

    let refresh_token = encode(
        &Header::default(),
        &Claims {
            sub: user_id,
            role: role.to_string(),
            iat: now.unix_timestamp(),
            exp: (now + Duration::days(7)).unix_timestamp(),
        },
        &EncodingKey::from_secret(CONFIG.jwt_secret_refresh.as_bytes()),
    )?;

    let access_token = encode(
        &Header::default(),
        &Claims {
            sub: user_id,
            role: role.to_string(),
            iat: now.unix_timestamp(),
            exp: (now + Duration::minutes(15)).unix_timestamp(),
        },
        &EncodingKey::from_secret(CONFIG.jwt_secret_access.as_bytes()),
    )?;

    Ok((refresh_token, access_token))
}

pub fn validate_access_token(token: &str) -> Result<Claims> {
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CONFIG.jwt_secret_access.as_bytes()),
        &Validation::new(Algorithm::HS256)
    )?;

    Ok(decoded.claims)
}

pub fn validate_refresh_token(token: &str) -> Result<Claims> {
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CONFIG.jwt_secret_refresh.as_bytes()),
        &Validation::new(Algorithm::HS256)
    )?;

    Ok(decoded.claims)
}

pub fn hash_token(token: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(token.as_bytes());
    hex::encode(hash.finalize())
}