use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::api::{scores::achievements::ScoreAchievement, ApiResult};

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/achievements/{uid}")),
    paths(get_score_achievement, put_score_achievement)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

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
async fn get_score_achievement(
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let score: ScoreAchievement = stardb_database::get_score_achievement_by_uid(*uid, &pool)
        .await?
        .into();

    Ok(HttpResponse::Ok().json(score))
}

#[utoipa::path(
    tag = "scores/achievements/{uid}",
    put,
    path = "/api/scores/achievements/{uid}",
    responses(
        (status = 200, description = "ScoreAchievement", body = ScoreAchievement),
    )
)]
#[put("/api/scores/achievements/{uid}")]
async fn put_score_achievement(
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    reqwest::Client::new()
        .put(format!("http://localhost:8000/api/mihomo/{uid}"))
        .send()
        .await?;

    let score: ScoreAchievement = stardb_database::get_score_achievement_by_uid(*uid, &pool)
        .await?
        .into();

    Ok(HttpResponse::Ok().json(score))
}
