use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/books/completed/{id}")),
    paths(put_user_book_completed, delete_user_book_completed)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_user_book_completed)
        .service(delete_user_book_completed);
}

#[utoipa::path(
    tag = "users/me/books/completed/{id}",
    put,
    path = "/api/users/me/books/completed/{id}",
    responses(
        (status = 200, description = "Successful add of the book"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/books/completed/{id}")]
async fn put_user_book_completed(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;
    let db_complete = database::DbUserBookCompleted { username, id };
    database::add_user_book_completed(&db_complete, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/books/completed/{id}",
    delete,
    path = "/api/users/me/books/completed/{id}",
    responses(
        (status = 200, description = "Successful delete of the book"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/books/completed/{id}")]
async fn delete_user_book_completed(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let db_complete = database::DbUserBookCompleted { username, id };

    database::delete_user_book_completed(&db_complete, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
