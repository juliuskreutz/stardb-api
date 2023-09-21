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
    key: i64,
    #[serde(rename = "Meow Comments")]
    comment: Option<String>,
    #[serde(rename = "Pop Cultural References")]
    reference: Option<String>,
    #[serde(rename = "Difficulty")]
    difficulty: Option<String>,
    #[serde(rename = "Video")]
    video: Option<String>,
    #[serde(rename = "Gacha Required")]
    gacha: Option<String>,
    #[serde(rename = "Can't Get Yet")]
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

    if database::get_admin_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let lines = BufReader::new(&file.file.file)
        .lines()
        .flatten()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    let mut reader = csv::Reader::from_reader(lines.as_bytes());
    for achievement in reader.deserialize() {
        let achievement: Achievement = achievement?;

        let mut ids = vec![achievement.key];

        let db_achievement = database::get_achievement_by_id(ids[0], "en", true, &pool).await?;

        if let Some(set) = db_achievement.set {
            for related in database::get_related(ids[0], set, &pool).await? {
                ids.push(related);
            }
        }

        for id in ids {
            database::update_achievement_version(id, &achievement.version, &pool).await?;

            if let Some(difficulty) = &achievement.difficulty {
                database::update_achievement_difficulty(id, &difficulty.to_lowercase(), &pool)
                    .await?
            } else {
                database::delete_achievement_difficulty(id, &pool).await?;
            }

            if let Some(comment) = &achievement.comment {
                database::update_achievement_comment(id, comment, &pool).await?
            } else {
                database::delete_achievement_comment(id, &pool).await?;
            }

            if let Some(reference) = &achievement.reference {
                database::update_achievement_reference(id, reference, &pool).await?
            } else {
                database::delete_achievement_reference(id, &pool).await?;
            }

            if let Some(video) = &achievement.video {
                database::update_achievement_video(id, video, &pool).await?
            } else {
                database::delete_achievement_video(id, &pool).await?;
            }

            database::update_achievement_gacha(id, achievement.gacha.is_some(), &pool).await?;

            database::update_achievement_impossible(id, achievement.impossible.is_some(), &pool)
                .await?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}
