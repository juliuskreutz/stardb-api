use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{book_series_worlds::BookSeriesWorld, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(tags((name = "book-series-worlds/{id}")), paths(get_book_series_world))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_book_series_world);
}

#[utoipa::path(
    tag = "book-series-worlds/{id}",
    get,
    path = "/api/book-series-worlds/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "BookSeriesWorld", body = BookSeriesWorld),
    )
)]
#[get("/api/book-series-worlds/{id}")]
async fn get_book_series_world(
    id: web::Path<i32>,
    language_param: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let series = BookSeriesWorld::from(
        database::get_book_series_world_by_id(*id, language_param.lang, &pool).await?,
    );

    Ok(HttpResponse::Ok().json(series))
}
