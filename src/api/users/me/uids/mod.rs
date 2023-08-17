use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::api::ApiResult;

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/uids")),
    paths(get_user_uids)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_uids);
}

#[utoipa::path(
    tag = "users/me/uids",
    get,
    path = "/api/users/me/uids",
    responses(
        (status = 200, description = "User uids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/uids")]
async fn get_user_uids(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let uids: Vec<_> = stardb_database::get_connections_by_username(&username, &pool)
        .await?
        .iter()
        .map(|c| c.uid)
        .collect();

    Ok(HttpResponse::Ok().json(uids))
}
