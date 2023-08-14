use actix_web::{get, web, HttpResponse, Responder};
use strum::IntoEnumIterator;
use utoipa::OpenApi;

use crate::Result;

use super::Language;

#[derive(OpenApi)]
#[openapi(
    tags((name = "languages")),
    paths(get_languages)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_languages);
}

#[utoipa::path(
    tag = "languages",
    get,
    path = "/api/languages",
    responses(
        (status = 200, description = "[Language]", body = Vec<Language>),
    )
)]
#[get("/api/languages")]
async fn get_languages() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(
        Language::iter()
            .map(|l| {
                serde_json::json!({
                    l.to_string(): l.get_flag()
                })
            })
            .collect::<Vec<_>>(),
    ))
}
