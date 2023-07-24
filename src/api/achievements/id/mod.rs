mod comment;

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{database, Result};

use super::Achievement;

#[derive(OpenApi)]
#[openapi(paths(get_achievement))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(comment::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(comment::configure).service(get_achievement);
}

#[utoipa::path(
    get,
    path = "/api/achievements/{id}",
    responses(
        (status = 200, description = "Achievement", body = Achievement),
    ),
    tag = "Achievements"
)]
#[get("/api/achievements/{id}")]
async fn get_achievement(id: web::Path<i64>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let achievement: Achievement = database::get_achievement_by_id(*id, &pool).await?.into();

    Ok(HttpResponse::Ok().json(achievement))
}
