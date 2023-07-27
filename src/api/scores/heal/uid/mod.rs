use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::schemas::{HealUpdate, ScoreHeal},
    database::{self, DbScoreHeal},
    Result,
};

#[derive(OpenApi)]
#[openapi(tags((name = "scores/heal/{uid}")), paths(get_score_heal, put_score_heal))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_score_heal).service(put_score_heal);
}

#[utoipa::path(
    tag = "scores/heal/{uid}",
    get,
    path = "/api/scores/heal/{uid}",
    responses(
        (status = 200, description = "[ScoreHeal]", body = Vec<ScoreHeal>),
    )
)]
#[get("/api/scores/heal/{uid}")]
async fn get_score_heal(uid: web::Path<i64>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let score: ScoreHeal = database::get_score_heal_by_uid(*uid, &pool).await?.into();

    Ok(HttpResponse::Ok().json(score))
}

#[utoipa::path(
    tag = "scores/heal/{uid}",
    put,
    path = "/api/scores/heal/{uid}",
    request_body = HealUpdate,
    responses(
        (status = 200, description = "ScoreHeal updated"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/scores/heal/{uid}")]
async fn put_score_heal(
    session: Session,
    uid: web::Path<i64>,
    heal_update: web::Json<HealUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let uid = *uid;
    let heal = heal_update.heal;
    let video = heal_update.video.clone();

    let db_set_score_heal = DbScoreHeal {
        uid,
        heal,
        video,
        ..Default::default()
    };

    database::set_score_heal(&db_set_score_heal, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
