use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{achievement_series::AchievementSeries, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(tags((name = "achievement-series/{id}")), paths(get_achievement_series))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_achievement_series);
}

#[utoipa::path(
    tag = "achievement-series/{id}",
    get,
    path = "/api/achievement-series/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "AchievementSeries", body = AchievementSeries),
    )
)]
#[get("/api/achievement-series/{id}")]
async fn get_achievement_series(
    id: web::Path<i32>,
    language_param: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let series = AchievementSeries::from(
        database::get_achievement_series_by_id(*id, &language_param.lang.to_string(), &pool)
            .await?,
    );

    Ok(HttpResponse::Ok().json(series))
}
