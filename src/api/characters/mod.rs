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
    tag: String,
    name: String,
}

impl<T: AsRef<DbCharacter>> From<T> for Character {
    fn from(value: T) -> Self {
        let db_character = value.as_ref();

        Character {
            tag: db_character.tag.clone(),
            name: db_character.name.clone(),
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

    let characters: Vec<_> = db_characters.iter().map(Character::from).collect();

    Ok(HttpResponse::Ok().json(characters))
}
