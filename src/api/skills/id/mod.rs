use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{skills::Skill, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "skills/{id}")),
    paths(get_skill)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_skill);
}

#[utoipa::path(
    tag = "skills/{id}",
    get,
    path = "/api/skills/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Skill", body = Skill),
    )
)]
#[get("/api/skills/{id}")]
async fn get_skill(
    id: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let skill: Skill = database::get_skill_by_id(*id, language_params.lang, &pool)
        .await?
        .into();

    Ok(HttpResponse::Ok().json(skill))
}
