use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{book_series::BookSeries, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(tags((name = "book-series/{id}")), paths(get_book_series))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_book_series);
}

#[utoipa::path(
    tag = "book-series/{id}",
    get,
    path = "/api/book-series/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "BookSeries", body = BookSeries),
    )
)]
#[get("/api/book-series/{id}")]
async fn get_book_series(
    id: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let series =
        BookSeries::from(database::get_book_series_by_id(*id, language_params.lang, &pool).await?);

    Ok(HttpResponse::Ok().json(series))
}
