use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements/{id}/reference")),
    paths(put_achievement_reference, delete_achievement_reference),
    components(schemas(ReferenceUpdate))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct ReferenceUpdate {
    reference: String,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_achievement_reference)
        .service(delete_achievement_reference);
}

#[utoipa::path(
    tag = "achievements/{id}/reference",
    put,
    path = "/api/achievements/{id}/reference",
    request_body = ReferenceUpdate,
    responses(
        (status = 200, description = "Updated reference"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/achievements/{id}/reference")]
async fn put_achievement_reference(
    session: Session,
    id: web::Path<i32>,
    reference_update: web::Json<ReferenceUpdate>,
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

    database::update_achievement_reference(*id, &reference_update.reference, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "achievements/{id}/reference",
    delete,
    path = "/api/achievements/{id}/reference",
    responses(
        (status = 200, description = "Deleted reference"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[delete("/api/achievements/{id}/reference")]
async fn delete_achievement_reference(
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

    database::delete_achievement_reference(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
