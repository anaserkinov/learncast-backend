use crate::db::snip::entity::SnipEntityWithLesson;
use crate::module::user::snip::dto::SnipResponse;
use crate::module::common::lesson::mapper::to_response as lesson_to_response;

pub fn to_response(entity: SnipEntityWithLesson) -> SnipResponse {
    panic!()
    // SnipResponse {
    //     id: entity.snip.id,
    //     client_snip_id: entity.snip.client_snip_id,
    //     user_id: entity.snip.user_id,
    //     lesson: lesson_to_response(&entity.lesson),
    //     start_ms: entity.snip.start_ms,
    //     end_ms: entity.snip.end_ms,
    //     note_text: entity.snip.note_text,
    //     created_at: entity.snip.created_at
    // }
}