mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

use super::LanguageParams;

#[derive(OpenApi)]
#[openapi(
    tags((name = "series")),
    paths(get_seriess),
    components(schemas(
        Series
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct Series {
    id: i32,
    name: String,
}

impl From<database::DbSeries> for Series {
    fn from(db_series: database::DbSeries) -> Self {
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
    cfg.service(get_seriess).configure(id::configure);
}

#[utoipa::path(
    tag = "series",
    get,
    path = "/api/series",
    params(LanguageParams),
    responses(
        (status = 200, description = "[Series]", body = Vec<Series>),
    )
)]
#[get("/api/series")]
async fn get_seriess(
    language_param: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let series: Vec<_> = database::get_series(&language_param.lang.to_string(), &pool)
        .await?
        .into_iter()
        .map(Series::from)
        .collect();

    Ok(HttpResponse::Ok().json(series))
}
