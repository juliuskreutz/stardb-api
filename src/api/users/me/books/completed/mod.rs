mod id;

use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/books/completed")),
    paths(get_user_books_completed, put_user_books_completed)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_books_completed)
        .service(put_user_books_completed)
        .configure(id::configure);
}

#[utoipa::path(
    tag = "users/me/books/completed",
    get,
    path = "/api/users/me/books/completed",
    responses(
        (status = 200, description = "book ids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/books/completed")]
async fn get_user_books_completed(
    session: Session,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let completed: Vec<_> = database::get_user_books_completed_by_username(&username, &pool)
        .await?
        .iter()
        .map(|c| c.id)
        .collect();

    Ok(HttpResponse::Ok().json(completed))
}

#[utoipa::path(
    tag = "users/me/books/completed",
    put,
    path = "/api/users/me/books/completed",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/books/completed")]
async fn put_user_books_completed(
    session: Session,
    ids: web::Json<Vec<i32>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let mut complete = database::DbUserBookCompleted { username, id: 0 };

    for id in ids.0 {
        complete.id = id;

        database::add_user_book_completed(&complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
