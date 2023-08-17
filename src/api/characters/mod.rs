mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::api::{ApiResult, LanguageParams};

#[derive(OpenApi)]
#[openapi(
    tags((name = "characters")),
    paths(get_characters),
    components(schemas(
        Character
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct Character {
    id: i32,
    name: String,
    element: String,
    path: String,
}

impl From<stardb_database::DbCharacter> for Character {
    fn from(db_character: stardb_database::DbCharacter) -> Self {
        Character {
            id: db_character.id,
            name: db_character.name,
            element: db_character.element,
            path: db_character.path,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_characters).configure(id::configure);
}

#[utoipa::path(
    tag = "characters",
    get,
    path = "/api/characters",
    params(LanguageParams),
    responses(
        (status = 200, description = "[Character]", body = Vec<Character>),
    )
)]
#[get("/api/characters")]
async fn get_characters(
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_characters =
        stardb_database::get_characters(&language_params.lang.to_string(), &pool).await?;

    let characters: Vec<_> = db_characters.into_iter().map(Character::from).collect();

    Ok(HttpResponse::Ok().json(characters))
}
