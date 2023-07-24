mod comment;
mod difficulty;
mod reference;
mod version;

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::achievements::Achievement, database, Result};

#[derive(OpenApi)]
#[openapi(paths(get_achievement))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(version::openapi());
    openapi.merge(comment::openapi());
    openapi.merge(reference::openapi());
    openapi.merge(difficulty::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(version::configure)
        .configure(comment::configure)
        .configure(reference::configure)
        .configure(difficulty::configure)
        .service(get_achievement);
}

#[utoipa::path(
    tag = "achievements",
    get,
    path = "/api/achievements/{id}",
    responses(
        (status = 200, description = "Achievement", body = Achievement),
    )
)]
#[get("/api/achievements/{id}")]
async fn get_achievement(id: web::Path<i64>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let achievement: Achievement = database::get_achievement_by_id(*id, &pool).await?.into();

    Ok(HttpResponse::Ok().json(achievement))
}
