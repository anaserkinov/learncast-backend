use crate::module::common::auth::dto::{TelegramClaims, TelegramData};
use anyhow::Result;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

const JWKS_URL: &str = "https://oauth.telegram.org/.well-known/jwks.json";
const TELEGRAM_ISSUER: &str = "https://oauth.telegram.org";

pub async fn verify_telegram_login(
    jwks_cache: &JwksCache,
    data: &TelegramData,
    bot_id: &str,
    nonce: Option<&str>,
) -> Result<TelegramClaims> {
    let id_token = &data.id_token.as_str();
    let header = decode_header(id_token)?;
    let kid = header.kid.ok_or(TelegramAuthError::MissingKid)?;

    let jwk = jwks_cache.get_key(&kid).await?;

    let decoding_key = jwk_to_decoding_key(&jwk)?;

    let alg = header.alg;
    let mut validation = Validation::new(alg);
    validation.set_audience(&[bot_id]);
    validation.set_issuer(&[TELEGRAM_ISSUER]);

    let token_data = decode::<TelegramClaims>(id_token, &decoding_key, &validation)?;
    let claims = token_data.claims;

    if let Some(_expected_nonce) = nonce {
        // The nonce is included as a plain claim in the Telegram ID token.
        // Add `nonce: Option<String>` to TelegramClaims and compare here.
        // For now this is left as an extension point.
    }

    Ok(claims)
}

#[derive(Debug, Error)]
pub enum TelegramAuthError {
    #[error("Failed to fetch JWKS: {0}")]
    JwksFetch(#[from] reqwest::Error),
    #[error("Key ID (kid) not found in JWKS")]
    KeyNotFound,
    #[error("Invalid JWT: {0}")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),
    #[error("Missing 'kid' header in token")]
    MissingKid,
    #[error("Unsupported key type")]
    UnsupportedKeyType,
    #[error("Lock error")]
    LockError
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

/// A minimal JWK — only the fields needed for RS256/ES256 verification.
#[derive(Debug, Deserialize, Clone)]
struct Jwk {
    kid: String,
    kty: String,   // "RSA" | "EC"
    alg: Option<String>,
    // RSA fields
    n: Option<String>,
    e: Option<String>,
    // EC fields
    crv: Option<String>,
    x: Option<String>,
    y: Option<String>,
}

pub struct JwksCache {
    http: Client,
    /// kid → Jwk
    keys: RwLock<HashMap<String, Jwk>>,
}

impl JwksCache {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            http: Client::new(),
            keys: RwLock::new(HashMap::new()),
        })
    }

    /// Refresh the local key cache from Telegram's JWKS endpoint.
    pub async fn refresh(&self) -> Result<(), TelegramAuthError> {
        let jwks: Jwks = self.http.get(JWKS_URL).send().await?.json().await?;
        let mut cache = self.keys.write()
            .map_err(|e| {TelegramAuthError::LockError})?;
        cache.clear();
        for key in jwks.keys {
            cache.insert(key.kid.clone(), key);
        }
        Ok(())
    }

    /// Look up a key by `kid`. If not found, refresh once and retry.
    async fn get_key(&self, kid: &str) -> Result<Jwk, TelegramAuthError> {
        {
            let cache = self.keys.read()
                .map_err(|e| {TelegramAuthError::LockError})?;
            if let Some(k) = cache.get(kid) {
                return Ok(k.clone());
            }
        }
        // Key unknown — could be a rotation; fetch fresh keys.
        self.refresh().await?;
        let cache = self.keys.read()
            .map_err(|e| {TelegramAuthError::LockError})?;
        cache.get(kid).cloned().ok_or(TelegramAuthError::KeyNotFound)
    }
}

/// Convert a JWK to a `jsonwebtoken::DecodingKey`.

fn jwk_to_decoding_key(jwk: &Jwk) -> Result<DecodingKey, TelegramAuthError> {
    match jwk.kty.as_str() {
        "RSA" => {
            let n = jwk.n.as_deref().ok_or(TelegramAuthError::UnsupportedKeyType)?;
            let e = jwk.e.as_deref().ok_or(TelegramAuthError::UnsupportedKeyType)?;
            Ok(DecodingKey::from_rsa_components(n, e)?)
        }
        "EC" => {
            let x = jwk.x.as_deref().ok_or(TelegramAuthError::UnsupportedKeyType)?;
            let y = jwk.y.as_deref().ok_or(TelegramAuthError::UnsupportedKeyType)?;
            let mut point = vec![0x04u8];
            point.extend_from_slice(
                &URL_SAFE_NO_PAD.decode(x).map_err(|_| TelegramAuthError::UnsupportedKeyType)?,
            );
            point.extend_from_slice(
                &URL_SAFE_NO_PAD.decode(y).map_err(|_| TelegramAuthError::UnsupportedKeyType)?,
            );
            Ok(DecodingKey::from_ec_der(&point))
        }
        _ => Err(TelegramAuthError::UnsupportedKeyType),
    }
}

/// Re-encode raw bytes as standard (non-URL-safe) base64 so that
/// `jsonwebtoken::DecodingKey::from_rsa_components` can accept them.
fn base64_encode_std(bytes: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD;
    STANDARD.encode(bytes)
}
