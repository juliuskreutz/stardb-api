mod comment;
mod difficulty;
mod gacha;
mod impossible;
mod reference;
mod version;
mod video;

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{
        achievements::{Achievement, LanguageParams},
        ApiResult,
    },
    database,
};

#[derive(OpenApi)]
#[openapi(tags((name = "achievements/{id}")), paths(get_achievement))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(comment::openapi());
    openapi.merge(difficulty::openapi());
    openapi.merge(gacha::openapi());
    openapi.merge(impossible::openapi());
    openapi.merge(reference::openapi());
    openapi.merge(version::openapi());
    openapi.merge(video::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(comment::configure)
        .configure(difficulty::configure)
        .configure(gacha::configure)
        .configure(impossible::configure)
        .configure(reference::configure)
        .configure(version::configure)
        .configure(video::configure)
        .service(get_achievement);
}

#[utoipa::path(
    tag = "achievements/{id}",
    get,
    path = "/api/achievements/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Achievement", body = Achievement),
    )
)]
#[get("/api/achievements/{id}")]
async fn get_achievement(
    session: Session,
    id: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let admin = if let Ok(Some(username)) = session.get::<String>("username") {
        database::get_admin_by_username(&username, &pool)
            .await
            .is_ok()
    } else {
        false
    };

    let db_achievement =
        database::get_achievement_by_id(*id, &language_params.lang.to_string(), &pool).await?;

    if (db_achievement.impossible && db_achievement.hidden) && !admin {
        return Ok(HttpResponse::NotFound().finish());
    }

    let mut achievement = Achievement::from(db_achievement);

    if let Some(set) = achievement.set {
        achievement.related = Some(database::get_related(achievement.id, set, &pool).await?);
    }

    Ok(HttpResponse::Ok().json(achievement))
}
