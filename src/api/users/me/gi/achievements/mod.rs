mod completed;
mod favorites;

use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

/// Return the OpenAPI description for these routes, including registered child routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(completed::openapi());
    openapi.merge(favorites::openapi());
    openapi
}

/// Register this module's HTTP routes and child route configuration.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(completed::configure)
        .configure(favorites::configure);
}
