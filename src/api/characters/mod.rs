mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{ApiResult, LanguageParams},
    database,
};

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
    rarity: i32,
    name: String,
    path: String,
    element: String,
    path_id: String,
    element_id: String,
}

impl From<database::DbCharacter> for Character {
    fn from(db_character: database::DbCharacter) -> Self {
        Character {
            id: db_character.id,
            rarity: db_character.rarity,
            name: db_character.name,
            path: db_character.path,
            element: db_character.element,
            path_id: db_character.path_id,
            element_id: db_character.element_id,
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
    let db_characters = database::get_characters(language_params.lang, &pool).await?;

    let characters: Vec<_> = db_characters.into_iter().map(Character::from).collect();

    Ok(HttpResponse::Ok().json(characters))
}
