use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{scores::achievements::ScoreAchievement, ApiResult, LanguageParams},
    database, mihomo,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/achievements/{uid}")),
    paths(get_score_achievement, put_score_achievement)
)]
struct ApiDoc;

/// Return the OpenAPI description for these routes, including registered child routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Register this module's HTTP routes and child route configuration.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_score_achievement)
        .service(put_score_achievement);
}

#[utoipa::path(
    tag = "scores/achievements/{uid}",
    get,
    path = "/api/scores/achievements/{uid}",
    responses(
        (status = 200, description = "ScoreAchievement", body = ScoreAchievement),
    )
)]
#[get("/api/scores/achievements/{uid}")]
/// Return the UID's current ranked score, or 404 when no joined score exists.
async fn get_score_achievement(
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Some(score) = database::achievement_scores::get_by_uid(*uid, &pool).await? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let score: ScoreAchievement = score.into();

    Ok(HttpResponse::Ok().json(score))
}

#[utoipa::path(
    tag = "scores/achievements/{uid}",
    put,
    path = "/api/scores/achievements/{uid}",
    params(LanguageParams),
    responses(
        (status = 200, description = "ScoreAchievement", body = ScoreAchievement),
    )
)]
#[put("/api/scores/achievements/{uid}")]
/// Attempt a mihomo refresh before reading the ranked score; return 404 if it remains unavailable.
async fn put_score_achievement(
    uid: web::Path<i32>,
    language_param: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let _ = mihomo::update_and_get(*uid, language_param.lang, &pool).await?;

    let Some(score) = database::achievement_scores::get_by_uid(*uid, &pool).await? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let score: ScoreAchievement = score.into();

    Ok(HttpResponse::Ok().json(score))
}
