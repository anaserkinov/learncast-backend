use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_progress_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserProgressStatus {
    NotStarted,
    InProgress,
    Completed
}