use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{self, DbComplete},
    Result,
};

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
struct Achievement {
    #[serde(rename = "Done")]
    done: String,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Key")]
    key: i64,
}

#[utoipa::path(
    tag = "import",
    post,
    path = "/api/import",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in")
    )
)]
#[post("/api/import")]
async fn import(
    session: Session,
    file: MultipartForm<File>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let mut reader = csv::Reader::from_reader(&file.file.file);

    for achievement in reader.deserialize() {
        let achievement: Achievement = achievement?;

        if achievement.title.contains("OR") {
            continue;
        }

        let db_complete = DbComplete {
            username: username.clone(),
            id: achievement.key,
        };

        if achievement.done == "TRUE" {
            database::add_complete(&db_complete, &pool).await?;
        } else if achievement.done == "FALSE" {
            database::delete_complete(&db_complete, &pool).await?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}
