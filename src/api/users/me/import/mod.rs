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
        ImportData,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(import);
}

#[derive(Deserialize, ToSchema)]
struct ImportData {
    hsr_achievements: Option<Vec<i32>>,
    gi_achievements: Option<Vec<i32>>,
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

    use database::achievement_lists::{apply, Game, List};
    let mut tx = pool.begin().await?;
    if let Some(ids) = &import_data.hsr_achievements {
        apply(Game::Hsr, List::Completed, &username, ids, true, &mut tx).await?;
    }
    if let Some(ids) = &import_data.gi_achievements {
        apply(Game::Gi, List::Completed, &username, ids, true, &mut tx).await?;
    }
    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
