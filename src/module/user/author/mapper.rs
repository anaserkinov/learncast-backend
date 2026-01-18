use crate::db::author::entity::AuthorEntity;
use crate::module::common::author::mapper::to_response as author_to_response;
use crate::module::user::author::dto::AuthorResponse;

pub fn to_response(entity: AuthorEntity) -> AuthorResponse {
    AuthorResponse {
        author: author_to_response(entity)
    }
}