use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{characters::Character, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "characters/{id}")),
    paths(get_character)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_character);
}

#[utoipa::path(
    tag = "characters/{id}",
    get,
    path = "/api/characters/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Character", body = Character),
    )
)]
#[get("/api/characters/{id}")]
async fn get_character(
    id: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let character: Character =
        database::get_character_by_id(*id, language_params.lang, &pool)
            .await?
            .into();

    Ok(HttpResponse::Ok().json(character))
}
