use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

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
    tag: String,
    name: String,
}

impl From<DbCharacter> for Character {
    fn from(db_character: DbCharacter) -> Self {
        Character {
            id: db_character.id,
            tag: db_character.tag,
            name: db_character.name,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_characters);
}

#[utoipa::path(
    tag = "characters",
    get,
    path = "/api/characters",
    responses(
        (status = 200, description = "[Character]", body = Vec<Character>),
    )
)]
#[get("/api/characters")]
async fn get_characters(pool: web::Data<PgPool>) -> Result<impl Responder> {
    let db_characters = database::get_characters(&pool).await?;

    let characters: Vec<_> = db_characters.into_iter().map(Character::from).collect();

    Ok(HttpResponse::Ok().json(characters))
}
