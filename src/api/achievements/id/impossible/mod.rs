use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements/{id}/impossible")),
    paths(put_achievement_impossible, delete_achievement_impossible)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_achievement_impossible)
        .service(delete_achievement_impossible);
}

#[utoipa::path(
    tag = "achievements/{id}/impossible",
    put,
    path = "/api/achievements/{id}/impossible",
    responses(
        (status = 200, description = "Updated impossible"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/achievements/{id}/impossible")]
async fn put_achievement_impossible(
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

    database::achievements::update_impossible_by_id(*id, true, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "achievements/{id}/impossible",
    delete,
    path = "/api/achievements/{id}/impossible",
    responses(
        (status = 200, description = "Deleted impossible"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[delete("/api/achievements/{id}/impossible")]
async fn delete_achievement_impossible(
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

    database::achievements::update_impossible_by_id(*id, false, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
