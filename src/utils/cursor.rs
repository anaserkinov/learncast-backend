use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{de::DeserializeOwned, Serialize};

pub fn encode<T: Serialize>(value: T) -> Option<String> {
    let bytes = postcard::to_allocvec(&value).ok()?;
    Some(URL_SAFE_NO_PAD.encode(bytes))
}

pub fn decode<T: DeserializeOwned>(cursor: Option<String>) -> Option<T> {
    if cursor.is_none() { return None }
    let bytes = URL_SAFE_NO_PAD.decode(cursor.unwrap()).ok()?;
    postcard::from_bytes(&bytes).ok()
}
