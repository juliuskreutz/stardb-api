mod private;

use actix_session::Session;
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

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(private::openapi());
    openapi
}

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
async fn put_user_gi_uid(
    session: Session,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

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
async fn delete_user_gi_uid(
    session: Session,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let connection = database::gi::connections::DbConnection {
        username,
        uid: *uid,
        verified: false,
        private: false,
    };

    database::gi::connections::delete(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
