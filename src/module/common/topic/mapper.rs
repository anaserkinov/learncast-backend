use crate::db::topic::entity::TopicEntity;
use crate::module::common::topic::dto::{CommonTopicResponse};

pub fn to_response(entity: TopicEntity) -> CommonTopicResponse {
    CommonTopicResponse {
        id: entity.id,
        title: entity.title,
        description: entity.description,
        cover_image_path: entity.cover_image_path,
        created_at: entity.created_at,
        lesson_count: entity.lesson_count,
        total_duration: entity.total_duration
    }
}