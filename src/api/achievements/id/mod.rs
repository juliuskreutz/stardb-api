use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{
        achievements::{Achievement, LanguageParams},
        ApiResult,
    },
    database,
};

#[derive(OpenApi)]
#[openapi(tags((name = "achievements/{id}")), paths(get_achievement))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_achievement);
}

#[utoipa::path(
    tag = "achievements/{id}",
    get,
    path = "/api/achievements/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Achievement", body = Achievement),
        (status = 404, description = "Achievement not found"),
    )
)]
#[get("/api/achievements/{id}")]
async fn get_achievement(
    id: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Some(db_achievement) =
        database::achievements::get_one_by_id(*id, language_params.lang, &pool).await?
    else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let mut achievement = Achievement::from(db_achievement);

    if let Some(set) = achievement.set {
        achievement.related =
            Some(database::achievements::get_all_related_ids(achievement.id, set, &pool).await?);
    }

    Ok(HttpResponse::Ok().json(achievement))
}
