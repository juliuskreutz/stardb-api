use std::io::{BufRead, BufReader};

use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{ApiResult, File},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "import-zzz-achievements")),
    paths(import_zzz_achievements)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(import_zzz_achievements);
}

#[derive(Deserialize)]
struct Achievement {
    #[serde(rename = "Ver")]
    version: Option<String>,
    #[serde(rename = "Key")]
    key: i32,
    #[serde(rename = "Meow Comments")]
    comment: Option<String>,
    #[serde(rename = "Pop Cultural References")]
    reference: Option<String>,
    #[serde(rename = "Difficulty")]
    difficulty: Option<String>,
    #[serde(rename = "Video")]
    video: Option<String>,
    #[serde(rename = "Character Locked")]
    gacha: Option<String>,
    #[serde(rename = "Time Gated")]
    timegated: Option<String>,
    #[serde(rename = "Missable")]
    missable: Option<String>,
    #[serde(rename = "Forbidden")]
    impossible: Option<String>,
}

#[utoipa::path(
    tag = "pinned",
    post,
    path = "/api/import-zzz-achievements",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
        (status = 403, description = "Not an admin")
    )
)]
#[post("/api/import-zzz-achievements")]
async fn import_zzz_achievements(
    session: Session,
    file: MultipartForm<File>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !database::admins::exists(&username, &pool).await? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let lines = BufReader::new(&file.file.file)
        .lines()
        .map_while(Result::ok)
        .collect::<Vec<_>>()
        .join("\n");

    let mut reader = csv::Reader::from_reader(lines.as_bytes());
    for achievement in reader.deserialize() {
        let achievement: Achievement = achievement?;

        database::zzz::achievements::update_version_by_id(
            achievement.key,
            achievement.version.as_deref(),
            &pool,
        )
        .await?;

        database::zzz::achievements::update_difficulty_by_id(
            achievement.key,
            achievement.difficulty.map(|d| d.to_lowercase()).as_deref(),
            &pool,
        )
        .await?;

        database::zzz::achievements::update_comment_by_id(
            achievement.key,
            achievement.comment.as_deref(),
            &pool,
        )
        .await?;

        database::zzz::achievements::update_reference_by_id(
            achievement.key,
            achievement.reference.as_deref(),
            &pool,
        )
        .await?;

        database::zzz::achievements::update_video_by_id(
            achievement.key,
            achievement.video.as_deref(),
            &pool,
        )
        .await?;

        database::zzz::achievements::update_gacha_by_id(
            achievement.key,
            achievement.gacha.is_some(),
            &pool,
        )
        .await?;

        database::zzz::achievements::update_impossible_by_id(
            achievement.key,
            achievement.impossible.is_some(),
            &pool,
        )
        .await?;

        database::zzz::achievements::update_timegated_by_id(
            achievement.key,
            achievement.timegated.as_deref(),
            &pool,
        )
        .await?;

        database::zzz::achievements::update_missable_by_id(
            achievement.key,
            achievement.missable.is_some(),
            &pool,
        )
        .await?;
    }

    Ok(HttpResponse::Ok().finish())
}
