use crate::db::lesson::entity::LessonWithAuthorTopic;
use crate::module::common::author::dto::CommonAuthorResponse;
use crate::module::common::base::FileResponse;
use crate::module::common::lesson::dto::CommonLessonResponse;
use crate::module::common::topic::dto::CommonTopicResponse;

pub fn to_response(entity: &LessonWithAuthorTopic) -> CommonLessonResponse {
    CommonLessonResponse {
        id: entity.lesson.id,
        title: entity.lesson.title.to_owned(),
        description: entity.lesson.description.to_owned(),
        cover_image_path: entity.lesson.cover_image_path.to_owned(),
        author: CommonAuthorResponse {
            id: entity.lesson.author_id,
            name: entity.author_name.to_owned(),
            avatar_path: entity.author_avatar_path.to_owned(),
            created_at: entity.author_created_at,
            lesson_count: entity.author_lesson_count
        },
        topic: if entity.topic_title.is_some() {
            Some(
                CommonTopicResponse {
                    id: entity.lesson.topic_id.unwrap(),
                    title: entity.topic_title.to_owned().unwrap(),
                    description: entity.topic_description.to_owned(),
                    cover_image_path: entity.topic_cover_image_path.to_owned(),
                    created_at: entity.topic_created_at.unwrap(),
                    lesson_count: entity.topic_lesson_count.unwrap(),
                    total_duration: entity.topic_total_duration.unwrap()
                }
            )
        } else { None },
        audio: FileResponse {
            path: entity.lesson.audio_path.to_owned(),
            size: entity.lesson.file_size,
            duration: entity.lesson.duration
        },
        listen_count: entity.lesson.listen_count,
        snip_count: entity.lesson.snip_count,
        created_at: entity.lesson.created_at
    }
}