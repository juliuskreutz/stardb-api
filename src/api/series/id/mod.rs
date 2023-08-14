use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{series::Series, LanguageParams},
    database, Result,
};

#[derive(OpenApi)]
#[openapi(tags((name = "series/{id}")), paths(get_series))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_series);
}

#[utoipa::path(
    tag = "series/{id}",
    get,
    path = "/api/series/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Series", body = Series),
    )
)]
#[get("/api/series/{id}")]
async fn get_series(
    id: web::Path<i32>,
    language_param: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let series = Series::from(
        database::get_series_by_id(*id, &language_param.lang.to_string(), &pool).await?,
    );

    Ok(HttpResponse::Ok().json(series))
}
