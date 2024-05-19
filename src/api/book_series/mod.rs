mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

use super::LanguageParams;

#[derive(OpenApi)]
#[openapi(
    tags((name = "book-series")),
    paths(get_book_seriess),
    components(schemas(
        BookSeries
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct BookSeries {
    id: i32,
    world: i32,
    bookshelf: bool,
    world_name: String,
    name: String,
}

impl From<database::DbBookSeries> for BookSeries {
    fn from(db_series: database::DbBookSeries) -> Self {
        Self {
            id: db_series.id,
            world: db_series.world,
            bookshelf: db_series.bookshelf,
            world_name: db_series.world_name,
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
    cfg.service(get_book_seriess).configure(id::configure);
}

#[utoipa::path(
    tag = "book-series",
    get,
    path = "/api/book-series",
    params(LanguageParams),
    responses(
        (status = 200, description = "[BookSeries]", body = Vec<BookSeries>),
    )
)]
#[get("/api/book-series")]
async fn get_book_seriess(
    language_param: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let series: Vec<_> = database::get_book_series(language_param.lang, &pool)
        .await?
        .into_iter()
        .map(BookSeries::from)
        .collect();

    Ok(HttpResponse::Ok().json(series))
}
