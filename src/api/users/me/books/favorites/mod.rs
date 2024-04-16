mod id;

use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/books/favorites")),
    paths(get_user_books_favorites, put_user_books_favorites)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_books_favorites)
        .service(put_user_books_favorites)
        .configure(id::configure);
}

#[utoipa::path(
    tag = "users/me/books/favorites",
    get,
    path = "/api/users/me/books/favorites",
    responses(
        (status = 200, description = "book ids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/books/favorites")]
async fn get_user_books_favorites(
    session: Session,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let favorites: Vec<_> = database::get_user_books_favorites_by_username(&username, &pool)
        .await?
        .iter()
        .map(|c| c.id)
        .collect();

    Ok(HttpResponse::Ok().json(favorites))
}

#[utoipa::path(
    tag = "users/me/books/favorites",
    put,
    path = "/api/users/me/books/favorites",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/books/favorites")]
async fn put_user_books_favorites(
    session: Session,
    ids: web::Json<Vec<i32>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let mut favorite = database::DbUserBookFavorite { username, id: 0 };

    for id in ids.0 {
        favorite.id = id;

        database::add_user_book_favorite(&favorite, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
