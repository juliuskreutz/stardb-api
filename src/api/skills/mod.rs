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
    tags((name = "skills")),
    paths(get_skills),
    components(schemas(
        Skill
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct Skill {
    id: i32,
    name: String,
}

impl From<database::DbSkill> for Skill {
    fn from(db_skill: database::DbSkill) -> Self {
        Skill {
            id: db_skill.id,
            name: db_skill.name,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_skills).configure(id::configure);
}

#[utoipa::path(
    tag = "skills",
    get,
    path = "/api/skills",
    params(LanguageParams),
    responses(
        (status = 200, description = "[Skill]", body = Vec<Skill>),
    )
)]
#[get("/api/skills")]
async fn get_skills(
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_skills = database::get_skills(&language_params.lang.to_string(), &pool).await?;

    let skills: Vec<_> = db_skills.into_iter().map(Skill::from).collect();

    Ok(HttpResponse::Ok().json(skills))
}
