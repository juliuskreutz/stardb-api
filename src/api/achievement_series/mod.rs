mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

use super::LanguageParams;

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievement-series")),
    paths(get_achievement_seriess),
    components(schemas(
        AchievementSeries
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct AchievementSeries {
    id: i32,
    name: String,
}

impl From<database::DbAchievementSeries> for AchievementSeries {
    fn from(db_series: database::DbAchievementSeries) -> Self {
        Self {
            id: db_series.id,
            name: db_series.name,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_achievement_seriess)
        .configure(id::configure);
}

#[utoipa::path(
    tag = "achievement-series",
    get,
    path = "/api/achievement-series",
    params(LanguageParams),
    responses(
        (status = 200, description = "[AchievementSeries]", body = Vec<AchievementSeries>),
    )
)]
#[get("/api/achievement-series")]
async fn get_achievement_seriess(
    language_param: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let series: Vec<_> = database::get_series(&language_param.lang.to_string(), &pool)
        .await?
        .into_iter()
        .map(AchievementSeries::from)
        .collect();

    Ok(HttpResponse::Ok().json(series))
}
