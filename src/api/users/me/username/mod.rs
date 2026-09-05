use crate::api::users::SessionUser;
use actix_web::{get, web, HttpResponse, Responder};
use utoipa::OpenApi;

use crate::api::ApiResult;

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/username")),
    paths(get_username)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_username);
}

#[utoipa::path(
    tag = "users/me/username",
    get,
    path = "/api/users/me/username",
    responses(
        (status = 200, description = "Username", body = String),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/username")]
async fn get_username(SessionUser(username): SessionUser) -> ApiResult<impl Responder> {
    Ok(HttpResponse::Ok().json(username))
}
