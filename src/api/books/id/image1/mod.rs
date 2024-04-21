use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "books/{id}/image1")),
    paths(put_book_image1, delete_book_image1),
    components(schemas(Image1Update))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct Image1Update {
    image1: String,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_book_image1).service(delete_book_image1);
}

#[utoipa::path(
    tag = "books/{id}/image1",
    put,
    path = "/api/books/{id}/image1",
    request_body = Image1Update,
    responses(
        (status = 200, description = "Updated image1"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/books/{id}/image1")]
async fn put_book_image1(
    session: Session,
    id: web::Path<i32>,
    image1_update: web::Json<Image1Update>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::admins::get_one_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::update_book_image1(*id, &image1_update.image1, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "books/{id}/image1",
    delete,
    path = "/api/books/{id}/image1",
    responses(
        (status = 200, description = "Deleted image1"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[delete("/api/books/{id}/image1")]
async fn delete_book_image1(
    session: Session,
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::admins::get_one_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::delete_book_image1(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
