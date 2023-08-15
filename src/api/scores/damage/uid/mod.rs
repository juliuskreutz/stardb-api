use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{
        scores::damage::{DamageParams, ScoreDamage},
        ApiResult,
    },
    database::{self, DbScoreDamage},
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/damage/{uid}")),
    paths(get_score_damage, put_score_damage),
    components(schemas(
        DamageUpdate
    ))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
pub struct DamageUpdate {
    pub character: i32,
    pub support: bool,
    pub damage: i32,
    pub video: String,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_score_damage).service(put_score_damage);
}

#[utoipa::path(
    tag = "scores/damage/{uid}",
    get,
    path = "/api/scores/damage/{uid}",
    params (
        DamageParams
    ),
    responses(
        (status = 200, description = "[ScoreDamage]", body = Vec<ScoreDamage>),
    )
)]
#[get("/api/scores/damage/{uid}")]
async fn get_score_damage(
    uid: web::Path<i64>,
    damage_params: web::Query<DamageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let score: ScoreDamage = database::get_score_damage_by_uid(
        *uid,
        damage_params.character,
        damage_params.support,
        &pool,
    )
    .await?
    .into();

    Ok(HttpResponse::Ok().json(score))
}

#[utoipa::path(
    tag = "scores/damage/{uid}",
    put,
    path = "/api/scores/damage/{uid}",
    request_body = DamageUpdate,
    responses(
        (status = 200, description = "ScoreDamage updated"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/scores/damage/{uid}")]
async fn put_score_damage(
    session: Session,
    uid: web::Path<i64>,
    damage_update: web::Json<DamageUpdate>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::get_admin_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let uid = *uid;
    let character = damage_update.character;
    let support = damage_update.support;
    let damage = damage_update.damage;
    let video = damage_update.video.clone();

    let db_set_score_damage = DbScoreDamage {
        uid,
        character,
        support,
        damage,
        video,
        ..Default::default()
    };

    database::set_score_damage(&db_set_score_damage, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
