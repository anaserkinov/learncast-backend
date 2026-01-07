use time::OffsetDateTime;
use crate::db::lesson::entity::{LessonProgressEntity, LessonWithAuthorTopic};
use crate::module::common::enums::UserProgressStatus;
use crate::module::user::lesson::dto::{LessonProgressResponse, LessonResponse};
use crate::module::common::lesson::mapper::to_response as lesson_to_response;

pub fn to_response(entity: LessonWithAuthorTopic) -> LessonResponse {
    LessonResponse {
        lesson: lesson_to_response(&entity),
        is_favourite: entity.is_favourite,
        lesson_progress: if entity.user_id.is_some() {
            Some(LessonProgressResponse {
                user_id: entity.user_id.unwrap(),
                author_id: entity.lesson.author_id,
                lesson_id: entity.lesson.id,
                started_at: entity.started_at.unwrap_or(OffsetDateTime::now_utc()),
                last_position_ms: entity.last_position_ms.unwrap(),
                status: entity.status.unwrap_or(UserProgressStatus::InProgress),
                completed_at: entity.completed_at
            })
        } else { None }
    }
}

pub fn progress_to_response(entity: LessonProgressEntity) -> LessonProgressResponse {
    LessonProgressResponse{
        user_id: entity.user_id,
        author_id: entity.author_id,
        lesson_id: entity.lesson_id,
        started_at: entity.started_at,
        last_position_ms: entity.last_position_ms,
        status: entity.status,
        completed_at: entity.completed_at
    }
}