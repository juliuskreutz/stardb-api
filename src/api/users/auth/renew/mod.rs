use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use utoipa::OpenApi;

use crate::api::ApiResult;

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/auth/renew")),
    paths(post_renew)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_renew);
}

#[utoipa::path(
    tag = "users/auth/renew",
    post,
    path = "/api/users/auth/renew",
    responses(
        (status = 200, description = "Successfully renewed", body = String),
        (status = 400, description = "Not logged in"),
    )
)]
#[post("/api/users/auth/renew")]
async fn post_renew(session: Session) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    session.renew();

    Ok(HttpResponse::Ok().json(username))
}
