use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::api::ApiResult;

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements/{id}/video")),
    paths(put_achievement_video, delete_achievement_video),
    components(schemas(VideoUpdate))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct VideoUpdate {
    video: String,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_achievement_video)
        .service(delete_achievement_video);
}

#[utoipa::path(
    tag = "achievements/{id}/video",
    put,
    path = "/api/achievements/{id}/video",
    request_body = VideoUpdate,
    responses(
        (status = 200, description = "Updated video"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/achievements/{id}/video")]
async fn put_achievement_video(
    session: Session,
    id: web::Path<i64>,
    video_update: web::Json<VideoUpdate>,
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

    stardb_database::update_achievement_video(*id, &video_update.video, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "achievements/{id}/video",
    delete,
    path = "/api/achievements/{id}/video",
    responses(
        (status = 200, description = "Deleted video"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[delete("/api/achievements/{id}/video")]
async fn delete_achievement_video(
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

    stardb_database::delete_achievement_video(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
