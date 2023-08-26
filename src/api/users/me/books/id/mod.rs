use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/books/{id}")),
    paths(put_user_book, delete_user_book)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_user_book).service(delete_user_book);
}

#[utoipa::path(
    tag = "users/me/books/{id}",
    put,
    path = "/api/users/me/books/{id}",
    responses(
        (status = 200, description = "Successful add of the book"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/books/{id}")]
async fn put_user_book(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let forbidden = [4082301, 4070910, 4070915, 4020203, 4070904, 4070916];

    if !forbidden.contains(&id) {
        let db_complete = database::DbUserBook { username, id };

        database::add_user_book(&db_complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/books/{id}",
    delete,
    path = "/api/users/me/books/{id}",
    responses(
        (status = 200, description = "Successful delete of the book"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/books/{id}")]
async fn delete_user_book(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let db_complete = database::DbUserBook { username, id };

    database::delete_user_book(&db_complete, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
