use std::io::BufReader;

use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "import")),
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
    achievements: Vec<i64>,
}

#[utoipa::path(
    tag = "pinned",
    post,
    path = "/api/import",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
    )
)]
#[post("/api/import")]
async fn import(
    session: Session,
    file: MultipartForm<File>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let import: Import = serde_json::from_reader(BufReader::new(&file.file.file))?;

    let mut complete = database::DbUserAchievement {
        username: username.clone(),
        id: 0,
    };

    for achievement in import.achievements {
        complete.id = achievement;

        database::add_user_achievement(&complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
