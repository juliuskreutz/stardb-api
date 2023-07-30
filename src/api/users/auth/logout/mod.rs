use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/auth/logout")),
    paths(logout)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(logout);
}

#[utoipa::path(
    tag = "users/auth/logout",
    post,
    path = "/api/users/auth/logout",
    responses(
        (status = 200, description = "Successfull logout. The session id is deleted"),
    )
)]
#[post("/api/users/auth/logout")]
async fn logout(session: Session) -> impl Responder {
    session.purge();

    HttpResponse::Ok().finish()
}
