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
    tags((name = "index")),
    paths(get_index),
    components(schemas(
        Index
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_index);
}

#[derive(Serialize, ToSchema)]
struct Index {
    characters: Vec<CharacterEntry>,
    light_cones: Vec<LightConeEntry>,
}

#[derive(Serialize, ToSchema)]
struct LightConeEntry {
    id: i32,
    name: String,
}

#[derive(Serialize, ToSchema)]
struct CharacterEntry {
    id: i64,
    name: String,
    skills: Vec<SkillEntry>,
}

#[derive(Serialize, ToSchema)]
struct SkillEntry {
    id: i32,
    name: String,
}

#[utoipa::path(
    tag = "index",
    get,
    path = "/api/index",
    params(LanguageParams),
    responses(
        (status = 200, description = "Index", body = Index),
    )
)]
#[get("/api/index")]
async fn get_index(
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let language = language_params.lang.to_string();

    let characters = database::get_characters(&language, &pool).await?;

    let mut character_entries = Vec::new();

    for character in characters {
        let skills = database::get_skills_by_character(character.id, &language, &pool).await?;

        let mut skill_entries = Vec::new();

        for skill in skills {
            skill_entries.push(SkillEntry {
                id: skill.id,
                name: skill.name,
            });
        }

        character_entries.push(CharacterEntry {
            id: character.id as i64,
            name: character.name,
            skills: skill_entries,
        });
    }

    let light_cones = database::get_light_cones(&language, &pool).await?;

    let light_cone_entries = light_cones
        .into_iter()
        .map(|light_cone| LightConeEntry {
            id: light_cone.id,
            name: light_cone.name,
        })
        .collect();

    let index = Index {
        characters: character_entries,
        light_cones: light_cone_entries,
    };

    Ok(HttpResponse::Ok().json(index))
}
