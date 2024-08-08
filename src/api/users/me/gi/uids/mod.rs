mod uid;

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/gi/uids")),
    paths(get_user_gi_uids)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(uid::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_gi_uids).configure(uid::configure);
}

#[utoipa::path(
    tag = "users/me/gi/uids",
    get,
    path = "/api/users/me/gi/uids",
    responses(
        (status = 200, description = "User uids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/gi/uids")]
async fn get_user_gi_uids(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let uids: Vec<_> = database::gi::connections::get_by_username(&username, &pool)
        .await?
        .iter()
        .map(|c| c.uid)
        .collect();

    Ok(HttpResponse::Ok().json(uids))
}
