use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/uids/{uid}/private")),
    paths(put_user_uid_private, delete_user_uid_private),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_user_uid_private)
        .service(delete_user_uid_private);
}

#[utoipa::path(
    tag = "users/me/uids/{uid}/private",
    put,
    path = "/api/users/me/uids/{uid}/private",
    responses(
        (status = 200, description = "Added uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/uids/{uid}/private")]
async fn put_user_uid_private(
    session: Session,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    database::update_connection_private_by_uid_and_username(*uid, &username, true, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/uids/{uid}/private",
    delete,
    path = "/api/users/me/uids/{uid}/private",
    responses(
        (status = 200, description = "Deleted uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/uids/{uid}/private")]
async fn delete_user_uid_private(
    session: Session,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    database::update_connection_private_by_uid_and_username(*uid, &username, false, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
