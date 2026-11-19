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

impl From<database::achievement_series::DbAchievementSeries> for AchievementSeries {
    /// Convert the localized database row to the API payload.
    fn from(db_series: database::achievement_series::DbAchievementSeries) -> Self {
        Self {
            id: db_series.id,
            name: db_series.name,
        }
    }
}

/// Return the OpenAPI description for these routes, including registered child routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

/// Register this module's HTTP routes and child route configuration.
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
/// Return localized achievement-series metadata in display order.
async fn get_achievement_seriess(
    language_param: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let series: Vec<_> = database::achievement_series::get_all(language_param.lang, &pool)
        .await?
        .into_iter()
        .map(AchievementSeries::from)
        .collect();

    Ok(HttpResponse::Ok().json(series))
}
