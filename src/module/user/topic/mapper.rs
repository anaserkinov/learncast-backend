use crate::db::topic::entity::{TopicWithAuthor};
use crate::module::common::author::dto::CommonAuthorResponse;
use crate::module::user::topic::dto::TopicResponse;
use crate::module::common::topic::mapper::to_response as topic_to_response;

pub fn to_response(entity: TopicWithAuthor) -> TopicResponse {
    let topic_id = entity.topic.id;
    TopicResponse {
        topic: topic_to_response(entity.topic),
        author: CommonAuthorResponse{
            id: topic_id,
            name: entity.author_name,
            avatar_path: entity.author_avatar_path,
            created_at: entity.author_created_at,
            lesson_count: entity.author_lesson_count
        },
        completed_lesson_count: entity.completed_lesson_count.unwrap_or(0)
    }
}