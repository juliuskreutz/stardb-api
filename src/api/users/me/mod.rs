mod achievements;
mod email;
mod password;
mod uids;
mod verifications;

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::api::ApiResult;

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me")),
    paths(get_me),
    components(schemas(
        User,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievements::openapi());
    openapi.merge(email::openapi());
    openapi.merge(password::openapi());
    openapi.merge(uids::openapi());
    openapi.merge(verifications::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me)
        .configure(achievements::configure)
        .configure(email::configure)
        .configure(password::configure)
        .configure(uids::configure)
        .configure(verifications::configure);
}

#[derive(Serialize, ToSchema)]
pub struct User {
    username: String,
    admin: bool,
}

#[utoipa::path(
    tag = "users/me",
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "User", body = User),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me")]
async fn get_me(session: Session) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let user = User { username, admin };

    Ok(HttpResponse::Ok().json(user))
}
