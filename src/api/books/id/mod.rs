mod comment;
mod image1;
mod image2;

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{
        books::{Book, LanguageParams},
        ApiResult,
    },
    database,
};

#[derive(OpenApi)]
#[openapi(tags((name = "books/{id}")), paths(get_book))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(comment::openapi());
    openapi.merge(image1::openapi());
    openapi.merge(image2::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_book)
        .configure(comment::configure)
        .configure(image1::configure)
        .configure(image2::configure);
}

#[utoipa::path(
    tag = "books/{id}",
    get,
    path = "/api/books/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Book", body = Book),
    )
)]
#[get("/api/books/{id}")]
async fn get_book(
    id: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_book = database::get_book_by_id(*id, language_params.lang, &pool).await?;

    let book = Book::from(db_book);

    Ok(HttpResponse::Ok().json(book))
}
