use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{achievements::Difficulty, ApiResult},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements/{id}/difficulty")),
    paths(put_achievement_difficulty, delete_achievement_difficulty),
    components(schemas(DifficultyUpdate))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct DifficultyUpdate {
    difficulty: Difficulty,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_achievement_difficulty)
        .service(delete_achievement_difficulty);
}

#[utoipa::path(
    tag = "achievements/{id}/difficulty",
    put,
    path = "/api/achievements/{id}/difficulty",
    request_body = DifficultyUpdate,
    responses(
        (status = 200, description = "Updated difficulty"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/achievements/{id}/difficulty")]
async fn put_achievement_difficulty(
    session: Session,
    id: web::Path<i32>,
    difficulty_update: web::Json<DifficultyUpdate>,
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

    database::achievements::update_difficulty_by_id(
        *id,
        &difficulty_update.difficulty.to_string(),
        &pool,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "achievements/{id}/difficulty",
    delete,
    path = "/api/achievements/{id}/difficulty",
    responses(
        (status = 200, description = "Deleted difficulty"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[delete("/api/achievements/{id}/difficulty")]
async fn delete_achievement_difficulty(
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

    database::achievements::delete_difficulty_by_id(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
