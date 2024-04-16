use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "books/{id}/image2")),
    paths(put_book_image2, delete_book_image2),
    components(schemas(Image2Update))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct Image2Update {
    image2: String,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_book_image2).service(delete_book_image2);
}

#[utoipa::path(
    tag = "books/{id}/image2",
    put,
    path = "/api/books/{id}/image2",
    request_body = Image2Update,
    responses(
        (status = 200, description = "Updated image2"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/books/{id}/image2")]
async fn put_book_image2(
    session: Session,
    id: web::Path<i32>,
    image2_update: web::Json<Image2Update>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::get_admin_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::update_book_image2(*id, &image2_update.image2, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "books/{id}/image2",
    delete,
    path = "/api/books/{id}/image2",
    responses(
        (status = 200, description = "Deleted image2"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[delete("/api/books/{id}/image2")]
async fn delete_book_image2(
    session: Session,
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::get_admin_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::delete_book_image2(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
