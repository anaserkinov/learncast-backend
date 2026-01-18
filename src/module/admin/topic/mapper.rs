use crate::db::topic::entity::TopicEntity;
use crate::module::admin::topic::dto::TopicResponse;
use crate::module::common::topic::mapper::to_response as topic_to_response;

pub fn to_response(entity: TopicEntity) -> TopicResponse {
    TopicResponse {
        topic: topic_to_response(entity)
    }
}