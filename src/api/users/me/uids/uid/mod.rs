use actix_session::Session;
use actix_web::{put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/uids/{uid}")),
    paths(put_user_uid),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_user_uid);
}

#[utoipa::path(
    tag = "users/me/uids/{uid}",
    get,
    path = "/api/users/me/uids/{uid}",
    responses(
        (status = 200, description = "Added uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/uids/{uid}")]
async fn put_user_uid(
    session: Session,
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let connection = database::DbConnection {
        username,
        uid: *uid,
    };

    database::set_connection(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
