use std::io::BufReader;

use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::Session;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/import")),
    paths(import),
    components(schemas(
        File,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(import);
}

#[derive(MultipartForm, ToSchema)]
struct File {
    #[schema(value_type = String, format = Binary)]
    file: TempFile,
}

#[derive(Deserialize)]
struct Import {
    achievements: Vec<i32>,
}

#[utoipa::path(
    tag = "users/me/import",
    put,
    path = "/api/users/me/import",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/import")]
async fn import(
    session: Session,
    file: MultipartForm<File>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let import: Import = serde_json::from_reader(BufReader::new(&file.file.file))?;

    database::delete_user_achievements_completed(&username, &pool).await?;

    let mut complete = database::DbUserAchievementCompleted {
        username: username.clone(),
        id: 0,
    };

    for achievement in import.achievements {
        complete.id = achievement;

        database::add_user_achievement_completed(&complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
