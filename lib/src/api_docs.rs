use utoipa::{Modify, OpenApi};
use utoipa::openapi::security::HttpAuthScheme;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::module::user::auth::controller::signin,
        crate::module::user::auth::controller::refresh_token,
        crate::module::user::auth::controller::logout,

        crate::module::common::file::controller::download_file,

        crate::module::user::author::controller::page_author,
        crate::module::user::author::controller::deleted_authors,

        crate::module::user::topic::controller::page_topic,
        crate::module::user::topic::controller::deleted_topics,

        crate::module::user::lesson::controller::page_lesson,
        crate::module::user::lesson::controller::deleted_lessons,
        crate::module::user::lesson::controller::increase_listen_count,
        crate::module::user::lesson::controller::update_lesson_progress,
        crate::module::user::lesson::controller::set_favourite,
        crate::module::user::lesson::controller::remove_favourite,

        crate::module::user::snip::controller::create_snip,
        crate::module::user::snip::controller::update_snip,
        crate::module::user::snip::controller::delete_snip,
        crate::module::user::snip::controller::page_snip,
        crate::module::user::snip::controller::deleted_snips,
        crate::module::user::snip::controller::count_snip
    ),
    components(
        schemas(
            crate::module::common::enums::UserProgressStatus,
            crate::module::common::paging::QueryOrder,
            crate::module::common::topic::dto::QuerySort,
            crate::module::common::lesson::dto::QuerySort,
            crate::module::user::snip::dto::QuerySort
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct UserApiDoc;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::module::admin::auth::controller::signin,
        crate::module::admin::auth::controller::get_me,
        crate::module::admin::auth::controller::refresh_token,
        crate::module::admin::auth::controller::logout,

        crate::module::common::file::controller::upload,
        crate::module::common::file::controller::upload_url,
        crate::module::common::file::controller::download_file,

        crate::module::admin::author::controller::create_author,
        crate::module::admin::author::controller::update_author,
        crate::module::admin::author::controller::get_author,
        crate::module::admin::author::controller::delete_author,
        crate::module::admin::author::controller::page_author,

        crate::module::admin::topic::controller::create_topic,
        crate::module::admin::topic::controller::update_topic,
        crate::module::admin::topic::controller::get_topic,
        crate::module::admin::topic::controller::delete_topic,
        crate::module::admin::topic::controller::page_topic,

        crate::module::admin::lesson::controller::create_lesson,
        crate::module::admin::lesson::controller::update_lesson,
        crate::module::admin::lesson::controller::get_lesson,
        crate::module::admin::lesson::controller::delete_lesson,
        crate::module::admin::lesson::controller::page_lesson,
    ),
    components(
        schemas(
            crate::module::common::enums::UserProgressStatus,
            crate::module::common::paging::QueryOrder,
            crate::module::common::topic::dto::QuerySort,
            crate::module::common::lesson::dto::QuerySort
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct AdminApiDoc;

pub struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let existing = openapi.components.take().unwrap();
        openapi.components = Some(
            utoipa::openapi::ComponentsBuilder::from(existing)
                .security_scheme(
                    "bearerAuth",
                    utoipa::openapi::security::SecurityScheme::Http(
                        utoipa::openapi::security::HttpBuilder::new()
                            .scheme(HttpAuthScheme::Bearer)
                            .bearer_format("JWT")
                            .build(),
                    ),
                )
                .build(),
        );
    }
}