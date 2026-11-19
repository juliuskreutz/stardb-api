use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/auth/logout")),
    paths(logout)
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
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
/// Purges the current session and returns success even when no user was logged in.
async fn logout(session: Session) -> impl Responder {
    session.purge();

    HttpResponse::Ok().finish()
}
