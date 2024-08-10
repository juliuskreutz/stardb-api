use std::io::BufReader;

use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{ApiResult, File},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/import"), (name = "users/me/import-file")),
    paths(import, import_file),
    components(schemas(
        ImportData,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(import).service(import_file);
}

#[derive(Deserialize, ToSchema)]
struct ImportData {
    achievements: Option<Vec<i32>>,
}

#[utoipa::path(
    tag = "users/me/import",
    put,
    path = "/api/users/me/import",
    request_body = ImportData,
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/import")]
async fn import(
    session: Session,
    import_data: web::Json<ImportData>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if let Some(achievements) = &import_data.achievements {
        database::users_achievements_completed::delete_by_username(&username, &pool).await?;
        let mut achievement_completed =
            database::users_achievements_completed::DbUserAchievementCompleted {
                username: username.clone(),
                id: 0,
            };
        for &achievement in achievements {
            achievement_completed.id = achievement;

            database::users_achievements_completed::add(&achievement_completed, &pool).await?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/import-file",
    put,
    path = "/api/users/me/import-file",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/import-file")]
async fn import_file(
    session: Session,
    file: MultipartForm<File>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let import_data: ImportData = serde_json::from_reader(BufReader::new(&file.file.file))?;

    if let Some(achievements) = &import_data.achievements {
        database::users_achievements_completed::delete_by_username(&username, &pool).await?;
        let mut achievement_completed =
            database::users_achievements_completed::DbUserAchievementCompleted {
                username: username.clone(),
                id: 0,
            };
        for &achievement in achievements {
            achievement_completed.id = achievement;

            database::users_achievements_completed::add(&achievement_completed, &pool).await?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}
