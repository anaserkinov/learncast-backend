use crate::db::author::entity::AuthorEntity;
use crate::module::common::author::dto::CommonAuthorResponse;

pub fn to_response(entity: AuthorEntity) -> CommonAuthorResponse {
    CommonAuthorResponse {
        id: entity.id,
        name: entity.name,
        avatar_path: entity.avatar_path,
        created_at: entity.created_at,
        lesson_count: entity.lesson_count
    }
}