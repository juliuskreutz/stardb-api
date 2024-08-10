use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{zzz::achievements::Achievement, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(tags((name = "zzz/achievements/{id}")), paths(get_zzz_achievement))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zzz_achievement);
}

#[utoipa::path(
    tag = "zzz/achievements/{id}",
    get,
    path = "/api/zzz/achievements/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Achievement", body = Achievement),
    )
)]
#[get("/api/zzz/achievements/{id}")]
async fn get_zzz_achievement(
    session: Session,
    id: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let admin = if let Ok(Some(username)) = session.get::<String>("username") {
        database::admins::exists(&username, &pool).await?
    } else {
        false
    };

    let db_achievement =
        database::zzz::achievements::get_one_by_id(*id, language_params.lang, &pool).await?;

    if (db_achievement.impossible && db_achievement.hidden) && !admin {
        return Ok(HttpResponse::NotFound().finish());
    }

    let mut achievement = Achievement::from(db_achievement);

    if let Some(set) = achievement.set {
        achievement.related = Some(
            database::zzz::achievements::get_all_related_ids(achievement.id, set, &pool).await?,
        );
    }

    Ok(HttpResponse::Ok().json(achievement))
}
