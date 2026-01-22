use crate::db::user::entity::UserEntity;
use crate::module::common::auth::dto::UserResponse;

pub fn to_response(entity: UserEntity) -> UserResponse {
    UserResponse {
        id: entity.id,
        first_name: entity.first_name,
        last_name: entity.last_name,
        avatar_path: entity.avatar_path,
        email: entity.email,
        telegram_username: entity.telegram_username,
        created_at: entity.created_at,
        updated_at: entity.updated_at
    }
}