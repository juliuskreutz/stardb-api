mod achievements;

use actix_web::web;
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi};

use super::Region;

#[derive(OpenApi)]
#[openapi(components(schemas(Region)))]
struct ApiDoc;

#[derive(Deserialize, IntoParams)]
pub struct ScoresParams {
    pub region: Option<Region>,
    pub query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Return the OpenAPI description for these routes, including registered child routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievements::openapi());
    openapi
}

/// Register this module's HTTP routes and child route configuration.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievements::configure);
}
