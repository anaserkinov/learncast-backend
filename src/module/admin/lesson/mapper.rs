use crate::db::lesson::entity::{LessonWithAuthorTopic};
use crate::module::admin::lesson::dto::LessonResponse;
use crate::module::common::lesson::mapper::to_response as lesson_to_response;

pub fn to_response(entity: LessonWithAuthorTopic) -> LessonResponse {
    LessonResponse {
        lesson: lesson_to_response(&entity)
    }
}