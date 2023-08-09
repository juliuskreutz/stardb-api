use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::characters::Character, database, Result};

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
    responses(
        (status = 200, description = "Character", body = Character),
    )
)]
#[get("/api/characters/{id}")]
async fn get_character(id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let character: Character = database::get_character_by_id(*id, &pool).await?.into();

    Ok(HttpResponse::Ok().json(character))
}
