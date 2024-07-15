use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/zzz/uids/{uid}")),
    paths(put_user_zzz_uid, delete_user_zzz_uid),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_user_zzz_uid).service(delete_user_zzz_uid);
}

#[utoipa::path(
    tag = "users/me/zzz/uids/{uid}",
    put,
    path = "/api/users/me/zzz/uids/{uid}",
    responses(
        (status = 200, description = "Added uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/zzz/uids/{uid}")]
async fn put_user_zzz_uid(
    session: Session,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let connection = database::zzz::connections::DbConnection {
        username,
        uid: *uid,
        verified: false,
        private: false,
    };

    database::zzz::uids::set(&database::zzz::uids::DbUid { uid: *uid }, &pool).await?;
    database::zzz::connections::set(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/zzz/uids/{uid}",
    delete,
    path = "/api/users/me/zzz/uids/{uid}",
    responses(
        (status = 200, description = "Deleted uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/zzz/uids/{uid}")]
async fn delete_user_zzz_uid(
    session: Session,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let connection = database::zzz::connections::DbConnection {
        username,
        uid: *uid,
        verified: false,
        private: false,
    };

    database::zzz::connections::delete(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
