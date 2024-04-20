use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

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
    id: web::Path<i32>,
    version_update: web::Json<VersionUpdate>,
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

    database::achievements::update_version_by_id(*id, &version_update.version, &pool).await?;

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
    id: web::Path<i32>,
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

    database::achievements::delete_version_by_id(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
