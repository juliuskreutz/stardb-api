use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/books/favorites/{id}")),
    paths(put_user_book_favorites, delete_user_book_favorites)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_user_book_favorites)
        .service(delete_user_book_favorites);
}

#[utoipa::path(
    tag = "users/me/books/favorites/{id}",
    put,
    path = "/api/users/me/books/favorites/{id}",
    responses(
        (status = 200, description = "Successful add of the book"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/books/favorites/{id}")]
async fn put_user_book_favorites(
    session: Session,
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;
    let favorite = database::DbUserBookFavorite { username, id };
    database::add_user_book_favorite(&favorite, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/books/favorites/{id}",
    delete,
    path = "/api/users/me/books/favorites/{id}",
    responses(
        (status = 200, description = "Successful delete of the book"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/books/favorites/{id}")]
async fn delete_user_book_favorites(
    session: Session,
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let favorite = database::DbUserBookFavorite { username, id };

    database::delete_user_book_favorite(&favorite, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
