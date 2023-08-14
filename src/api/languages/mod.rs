use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use strum::IntoEnumIterator;
use utoipa::{OpenApi, ToSchema};

use crate::Result;

use super::Language;

#[derive(OpenApi)]
#[openapi(
    tags((name = "languages")),
    paths(get_languages),
    components(schemas(
        LanguageName
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_languages);
}

#[derive(Serialize, ToSchema)]
struct LanguageName {
    id: Language,
    name: String,
}

#[utoipa::path(
    tag = "languages",
    get,
    path = "/api/languages",
    responses(
        (status = 200, description = "[LanguageFlag]", body = Vec<LanguageFlag>),
    )
)]
#[get("/api/languages")]
async fn get_languages() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(
        Language::iter()
            .map(|l| LanguageName {
                id: l,
                name: l.name(),
            })
            .collect::<Vec<_>>(),
    ))
}
