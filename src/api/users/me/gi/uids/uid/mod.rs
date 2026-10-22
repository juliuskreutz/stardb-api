mod private;

use crate::api::users::SessionUser;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/gi/uids/{uid}")),
    paths(put_user_gi_uid, delete_user_gi_uid),
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(private::openapi());
    openapi
}

/// Registers this module's routes and any shared application data.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(private::configure)
        .service(put_user_gi_uid)
        .service(delete_user_gi_uid);
}

#[utoipa::path(
    tag = "users/me/gi/uids/{uid}",
    put,
    path = "/api/users/me/gi/uids/{uid}",
    responses(
        (status = 200, description = "Added uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/gi/uids/{uid}")]
/// Adds an unverified public GI connection, preserving any existing connection.
/// The profile must already satisfy database foreign-key requirements; errors propagate.
async fn put_user_gi_uid(
    SessionUser(username): SessionUser,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if database::gi::connections::get_by_uid_and_username(*uid, &username, &pool)
        .await
        .is_ok()
    {
        return Ok(HttpResponse::Ok().finish());
    }

    let connection = database::gi::connections::DbConnection {
        username,
        uid: *uid,
        verified: false,
        private: false,
    };

    database::gi::connections::set(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/gi/uids/{uid}",
    delete,
    path = "/api/users/me/gi/uids/{uid}",
    responses(
        (status = 200, description = "Deleted uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/gi/uids/{uid}")]
/// Deletes only the authenticated user's GI connection, leaving pull history intact.
async fn delete_user_gi_uid(
    SessionUser(username): SessionUser,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let connection = database::gi::connections::DbConnection {
        username,
        uid: *uid,
        verified: false,
        private: false,
    };

    database::gi::connections::delete(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
