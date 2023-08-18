mod login;
mod logout;
mod request_token;

use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(login::openapi());
    openapi.merge(logout::openapi());
    openapi.merge(request_token::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(login::configure)
        .configure(logout::configure)
        .configure(request_token::configure);
}
