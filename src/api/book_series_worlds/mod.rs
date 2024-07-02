mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

use super::LanguageParams;

#[derive(OpenApi)]
#[openapi(
    tags((name = "book-series-worlds")),
    paths(get_book_series_worlds),
    components(schemas(
        BookSeriesWorld
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct BookSeriesWorld {
    id: i32,
    name: String,
}

impl From<database::DbBookSeriesWorld> for BookSeriesWorld {
    fn from(db_series: database::DbBookSeriesWorld) -> Self {
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
    cfg.service(get_book_series_worlds).configure(id::configure);
}

#[utoipa::path(
    tag = "book-series-worlds",
    get,
    path = "/api/book-series-worlds",
    params(LanguageParams),
    responses(
        (status = 200, description = "[BookSeriesWorld]", body = Vec<BookSeriesWorld>),
    )
)]
#[get("/api/book-series-worlds")]
async fn get_book_series_worlds(
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let series: Vec<_> = database::get_book_series_worlds(language_params.lang, &pool)
        .await?
        .into_iter()
        .map(BookSeriesWorld::from)
        .collect();

    Ok(HttpResponse::Ok().json(series))
}
