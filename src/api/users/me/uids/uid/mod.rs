mod private;

use crate::api::users::SessionUser;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, mihomo};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/uids/{uid}")),
    paths(put_user_uid, delete_user_uid),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(private::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(private::configure)
        .service(put_user_uid)
        .service(delete_user_uid);
}

#[utoipa::path(
    tag = "users/me/uids/{uid}",
    put,
    path = "/api/users/me/uids/{uid}",
    responses(
        (status = 200, description = "Added uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/uids/{uid}")]
async fn put_user_uid(
    SessionUser(username): SessionUser,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    if !(100000000..1000000000).contains(&uid) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    if database::connections::get_by_uid_and_username(uid, &username, &pool)
        .await
        .is_ok()
    {
        return Ok(HttpResponse::Ok().finish());
    }

    let connection = database::connections::DbConnection {
        username,
        uid,
        verified: false,
        private: false,
    };

    mihomo::ensure_row(uid, &pool).await?;

    database::connections::set(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/uids/{uid}",
    delete,
    path = "/api/users/me/uids/{uid}",
    responses(
        (status = 200, description = "Deleted uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/uids/{uid}")]
async fn delete_user_uid(
    SessionUser(username): SessionUser,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let connection = database::connections::DbConnection {
        username,
        uid: *uid,
        verified: false,
        private: false,
    };

    database::connections::delete(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
