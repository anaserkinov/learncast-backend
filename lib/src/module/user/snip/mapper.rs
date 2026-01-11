use crate::db::snip::entity::SnipEntityWithLesson;
use crate::module::common::author::dto::CommonAuthorResponse;
use crate::module::common::base::FileResponse;
use crate::module::common::lesson::dto::CommonLessonResponse;
use crate::module::common::topic::dto::CommonTopicResponse;
use crate::module::user::snip::dto::SnipResponse;

pub fn to_response(entity: SnipEntityWithLesson, user_snip_count: Option<i64>) -> SnipResponse {
    SnipResponse {
        id: entity.snip.id,
        client_snip_id: entity.snip.client_snip_id,
        user_id: entity.snip.user_id,
        lesson: CommonLessonResponse {
            id: entity.snip.lesson_id,
            title: entity.lesson_title,
            description: entity.lesson_description,
            cover_image_path: entity.lesson_cover_image_path,
            author: CommonAuthorResponse {
                id: entity.snip.author_id,
                name: entity.author_name,
                avatar_path: entity.author_avatar_path,
                created_at: entity.author_created_at,
                lesson_count: entity.author_lesson_count
            },
            topic: if entity.topic_title.is_some() {
                Some(
                    CommonTopicResponse {
                        id: entity.snip.topic_id.unwrap(),
                        title: entity.topic_title.unwrap(),
                        description: entity.topic_description,
                        cover_image_path: entity.topic_cover_image_path,
                        created_at: entity.topic_created_at.unwrap(),
                        lesson_count: entity.topic_lesson_count.unwrap(),
                        total_duration: entity.topic_total_duration.unwrap()
                    }
                )
            } else { None },
            audio: FileResponse {
                path: entity.lesson_audio_path,
                size: entity.lesson_file_size,
                duration: entity.lesson_duration
            },
            listen_count: entity.lesson_listen_count,
            snip_count: entity.lesson_snip_count,
            created_at: entity.lesson_created_at
        },
        start_ms: entity.snip.start_ms,
        end_ms: entity.snip.end_ms,
        note_text: entity.snip.note_text,
        created_at: entity.snip.created_at,
        user_snip_count
    }
}