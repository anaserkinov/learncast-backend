use crate::db::author::entity::AuthorEntity;
use crate::module::admin::author::dto::AuthorResponse;
use crate::module::common::author::mapper::to_response as author_to_response;

pub fn to_response(entity: AuthorEntity) -> AuthorResponse {
    AuthorResponse {
        author: author_to_response(entity)
    }
}