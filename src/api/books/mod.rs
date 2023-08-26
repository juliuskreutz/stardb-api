mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString};
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{ApiResult, LanguageParams},
    database,
};

use super::Language;

#[derive(OpenApi)]
#[openapi(
    tags((name = "books")),
    paths(get_books),
    components(schemas(
        Difficulty,
        Language,
        Book
    ))
)]
struct ApiDoc;

#[derive(Display, EnumString, Serialize, Deserialize, ToSchema)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Serialize, ToSchema)]
struct Book {
    id: i64,
    series: i32,
    series_inside: i32,
    series_name: String,
    name: String,
    percent: f64,
}

impl From<database::DbBook> for Book {
    fn from(db_book: database::DbBook) -> Self {
        Book {
            id: db_book.id,
            series: db_book.series,
            series_inside: db_book.series_inside,
            series_name: db_book.series_name.clone(),
            name: db_book.name.clone(),
            percent: db_book.percent,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_books).configure(id::configure);
}

#[utoipa::path(
    tag = "books",
    get,
    path = "/api/books",
    params(LanguageParams),
    responses(
        (status = 200, description = "[Book]", body = Vec<Book>),
    )
)]
#[get("/api/books")]
async fn get_books(
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_books = database::get_books(&language_params.lang.to_string(), &pool).await?;

    let books = db_books.into_iter().map(Book::from).collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(books))
}
