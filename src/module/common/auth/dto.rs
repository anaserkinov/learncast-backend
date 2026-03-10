use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, ToSchema)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum LoginRequest {
    Telegram { data: TelegramData },
    Google { data: GoogleData }
}

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct TelegramData {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub auth_date: Option<OffsetDateTime>,
    pub hash: String
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct GoogleData {
    pub id_token: String
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub avatar_path: Option<String>,
    pub email: Option<String>,
    pub telegram_username: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Credentials {
    pub refresh_token: String,
    pub access_token: String
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub credentials: Credentials
}