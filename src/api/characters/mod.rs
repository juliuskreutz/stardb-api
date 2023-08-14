mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::{
    database::{self, DbCharacter},
    Result,
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
    name: String,
    element: String,
    path: String,
}

#[derive(Deserialize, IntoParams)]
struct CharacterParams {
    element: Option<String>,
    path: Option<String>,
}

impl From<DbCharacter> for Character {
    fn from(db_character: DbCharacter) -> Self {
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
    params(CharacterParams),
    responses(
        (status = 200, description = "[Character]", body = Vec<Character>),
    )
)]
#[get("/api/characters")]
async fn get_characters(
    character_params: web::Query<CharacterParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let db_characters = database::get_characters(
        character_params.element.as_deref(),
        character_params.path.as_deref(),
        &pool,
    )
    .await?;

    let characters: Vec<_> = db_characters.into_iter().map(Character::from).collect();

    Ok(HttpResponse::Ok().json(characters))
}
