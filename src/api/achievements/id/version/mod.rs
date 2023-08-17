use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::api::ApiResult;

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements/{id}/version")),
    paths(put_achievement_version, delete_achievement_version),
    components(schemas(VersionUpdate))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct VersionUpdate {
    version: String,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_achievement_version)
        .service(delete_achievement_version);
}

#[utoipa::path(
    tag = "achievements/{id}/version",
    put,
    path = "/api/achievements/{id}/version",
    request_body = VersionUpdate,
    responses(
        (status = 200, description = "Updated version"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/achievements/{id}/version")]
async fn put_achievement_version(
    session: Session,
    id: web::Path<i64>,
    version_update: web::Json<VersionUpdate>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if stardb_database::get_admin_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    stardb_database::update_achievement_version(*id, &version_update.version, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "achievements/{id}/version",
    delete,
    path = "/api/achievements/{id}/version",
    responses(
        (status = 200, description = "Deleted version"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[delete("/api/achievements/{id}/version")]
async fn delete_achievement_version(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if stardb_database::get_admin_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    stardb_database::delete_achievement_version(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
