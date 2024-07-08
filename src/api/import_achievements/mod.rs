use std::io::{BufRead, BufReader};

use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "import-achievements")),
    paths(import_achievements),
    components(schemas(
        File,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(import_achievements);
}

#[derive(MultipartForm, ToSchema)]
struct File {
    #[schema(value_type = String, format = Binary)]
    file: TempFile,
}

#[derive(Deserialize)]
struct Achievement {
    #[serde(rename = "Ver")]
    version: String,
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
    path = "/api/import-achievements",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
        (status = 403, description = "Not an admin")
    )
)]
#[post("/api/import-achievements")]
async fn import_achievements(
    session: Session,
    file: MultipartForm<File>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::admins::get_one_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let lines = BufReader::new(&file.file.file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    let mut reader = csv::Reader::from_reader(lines.as_bytes());
    for achievement in reader.deserialize() {
        let achievement: Achievement = achievement?;

        database::achievements::update_version_by_id(achievement.key, &achievement.version, &pool)
            .await?;

        if let Some(difficulty) = &achievement.difficulty {
            database::achievements::update_difficulty_by_id(
                achievement.key,
                &difficulty.to_lowercase(),
                &pool,
            )
            .await?
        } else {
            database::achievements::delete_difficulty_by_id(achievement.key, &pool).await?;
        }

        if let Some(comment) = &achievement.comment {
            database::achievements::update_comment_by_id(achievement.key, comment, &pool).await?
        } else {
            database::achievements::delete_comment_by_id(achievement.key, &pool).await?;
        }

        if let Some(reference) = &achievement.reference {
            database::achievements::update_reference_by_id(achievement.key, reference, &pool)
                .await?
        } else {
            database::achievements::delete_reference_by_id(achievement.key, &pool).await?;
        }

        if let Some(video) = &achievement.video {
            database::achievements::update_video_by_id(achievement.key, video, &pool).await?
        } else {
            database::achievements::delete_video_by_id(achievement.key, &pool).await?;
        }

        database::achievements::update_gacha_by_id(
            achievement.key,
            achievement.gacha.is_some(),
            &pool,
        )
        .await?;

        database::achievements::update_impossible_by_id(
            achievement.key,
            achievement.impossible.is_some(),
            &pool,
        )
        .await?;
    }

    Ok(HttpResponse::Ok().finish())
}
