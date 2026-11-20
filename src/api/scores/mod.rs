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
    /// Rows to return, from 1 to 1000; 100 when omitted. Values outside that range are rejected with 400.
    #[param(minimum = 1, maximum = 1000, default = 100)]
    pub limit: Option<i64>,
    /// Rows to skip; 0 when omitted. Negative values are rejected with 400.
    #[param(minimum = 0, default = 0)]
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
