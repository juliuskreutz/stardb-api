mod comment;
mod difficulty;
mod gacha;
mod reference;
mod version;

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::achievements::Achievement, database, Result};

#[derive(OpenApi)]
#[openapi(tags((name = "achievements/{id}")), paths(get_achievement))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(version::openapi());
    openapi.merge(comment::openapi());
    openapi.merge(reference::openapi());
    openapi.merge(difficulty::openapi());
    openapi.merge(gacha::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(version::configure)
        .configure(comment::configure)
        .configure(reference::configure)
        .configure(difficulty::configure)
        .configure(gacha::configure)
        .service(get_achievement);
}

#[utoipa::path(
    tag = "achievements/{id}",
    get,
    path = "/api/achievements/{id}",
    responses(
        (status = 200, description = "Achievement", body = Achievement),
    )
)]
#[get("/api/achievements/{id}")]
async fn get_achievement(id: web::Path<i64>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let db_achievement = database::get_achievement_by_id(*id, &pool).await?;
    let set = db_achievement.set;

    let mut achievement = Achievement::from(db_achievement);

    if let Some(set) = set {
        achievement.related = Some(database::get_related(achievement.id, set, &pool).await?);
    };

    Ok(HttpResponse::Ok().json(achievement))
}
